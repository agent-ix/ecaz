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

    #[pg_test]
    fn test_ec_spire_customscan_remote_backend_termination_rejoin_sql() {
        let _env_lock = env_var_test_lock();
        let loopback_conninfo = current_pg_test_loopback_conninfo();
        let terminate_conninfo = format!(
            "{loopback_conninfo} options='-c search_path=ec_spire_customscan_remote_restart,public'"
        );
        let _conninfo_secret = ScopedEnvVar::set(
            "EC_SPIRE_REMOTE_CONNINFO_SPIRE_REMOTE_CUSTOMSCAN_REMOTE_RESTART",
            &terminate_conninfo,
        );
        let mut loopback_client = postgres::Client::connect(&loopback_conninfo, postgres::NoTls)
            .expect("loopback client connection should succeed");

        loopback_client
            .batch_execute(&format!(
                "DROP TABLE IF EXISTS ec_spire_customscan_remote_restart_remote_sql; \
                 DROP SCHEMA IF EXISTS ec_spire_customscan_remote_restart CASCADE; \
                 CREATE TABLE ec_spire_customscan_remote_restart_remote_sql \
                     (id bigint primary key, title text not null, embedding ecvector); \
                 INSERT INTO ec_spire_customscan_remote_restart_remote_sql (id, title, embedding) VALUES \
                     (101, 'remote restart alpha', encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
                     (102, 'remote restart beta', encode_to_ecvector(ARRAY[0.5, 0.5], 4, 42)); \
                 CREATE INDEX ec_spire_customscan_remote_restart_remote_idx \
                     ON ec_spire_customscan_remote_restart_remote_sql USING ec_spire \
                     (embedding ecvector_spire_ip_ops) \
                     WITH (nlists = 2, nprobe = 2, storage_format = 'rabitq'); \
                 CREATE SCHEMA ec_spire_customscan_remote_restart; \
                 CREATE FUNCTION ec_spire_customscan_remote_restart.ec_spire_remote_search(\
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
                     FROM pg_sleep(0.05), pg_terminate_backend(pg_backend_pid()) \
                 $function$",
                extension_version = env!("CARGO_PKG_VERSION")
            ))
            .expect("loopback remote-restart fixture should be created");
        let remote_identity_hex = loopback_remote_index_identity_hex(
            &mut loopback_client,
            "ec_spire_customscan_remote_restart_remote_idx",
        );

        Spi::run(
            "CREATE TABLE ec_spire_customscan_remote_restart_coord_sql \
             (id bigint primary key, title text not null, embedding ecvector)",
        )
        .expect("coordinator remote-restart table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_customscan_remote_restart_coord_sql (id, title, embedding) VALUES \
             (1, 'coordinator restart alpha', encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, 'coordinator restart beta', encode_to_ecvector(ARRAY[0.5, 0.5], 4, 42))",
        )
        .expect("coordinator remote-restart insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_customscan_remote_restart_coord_idx \
             ON ec_spire_customscan_remote_restart_coord_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) \
             WITH (nlists = 2, nprobe = 2, storage_format = 'rabitq')",
        )
        .expect("coordinator remote-restart index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_spire_customscan_remote_restart_coord_idx'::regclass::oid",
        )
        .expect("coordinator remote-restart index oid query should succeed")
        .expect("coordinator remote-restart index oid should exist");
        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_customscan_remote_restart_coord_idx'::regclass)",
        )
        .expect("coordinator remote-restart active epoch query should succeed")
        .expect("coordinator remote-restart active epoch should exist");
        let coord_leaf_pids = Spi::get_one::<Vec<i64>>(
            "SELECT array_agg(leaf_pid ORDER BY leaf_pid) FROM \
             ec_spire_index_leaf_snapshot('ec_spire_customscan_remote_restart_coord_idx'::regclass)",
        )
        .expect("coordinator remote-restart leaf pid query should succeed")
        .expect("coordinator remote-restart leaf pids should exist");

        unsafe {
            for pid in &coord_leaf_pids {
                am::debug_spire_rewrite_placement_node(index_oid, *pid as u64, 2);
            }
        }
        let register_result = Spi::get_one::<bool>(&format!(
            "SELECT ec_spire_register_remote_node_descriptor(\
                     '{}'::oid, 2, 109, 'spire/remote/customscan/remote_restart', \
                     decode('{remote_identity_hex}', 'hex'), \
                     'ec_spire_customscan_remote_restart_remote_idx', 'active', {active_epoch}, \
                     {active_epoch}, '{}', 'none')",
            u32::from(index_oid),
            env!("CARGO_PKG_VERSION")
        ))
        .expect("terminating remote descriptor registration should succeed")
        .expect("terminating remote descriptor registration result should exist");
        assert!(register_result);

        let mut query_client = postgres::Client::connect(&loopback_conninfo, postgres::NoTls)
            .expect("query client connection should succeed");
        query_client
            .batch_execute(
                "SET enable_seqscan = off; \
                 SET enable_indexscan = off; \
                 SET ec_spire.remote_search_consistency_mode = 'strict'",
            )
            .expect("strict CustomScan GUCs should be set");
        let plan = query_client
            .query(
                "EXPLAIN (COSTS OFF) \
                 SELECT id, title \
                   FROM ec_spire_customscan_remote_restart_coord_sql \
                  ORDER BY embedding <#> ARRAY[1.0, 0.0]::real[], id \
                  LIMIT 2",
                &[],
            )
            .expect("remote-restart CustomScan explain should succeed")
            .into_iter()
            .map(|row| {
                row.try_get::<_, String>(0)
                    .expect("remote-restart explain row should decode")
            })
            .collect::<Vec<_>>()
            .join("\n");
        assert!(
            plan.contains("Custom Scan (EcSpireDistributedScan)"),
            "expected remote-restart fixture to use EcSpireDistributedScan:\n{plan}"
        );

        let strict_error = query_client
            .query(
                "SELECT id, title \
                   FROM ec_spire_customscan_remote_restart_coord_sql \
                  ORDER BY embedding <#> ARRAY[1.0, 0.0]::real[], id \
                  LIMIT 2",
                &[],
            )
            .expect_err("strict CustomScan should fail closed on remote backend termination");
        let strict_message = strict_error.to_string();
        assert!(
            strict_message.contains("remote_backend_terminated")
                || strict_message.contains("remote_candidate_receive_failed"),
            "{strict_message}"
        );

        query_client
            .batch_execute("SET ec_spire.remote_search_consistency_mode = 'degraded'")
            .expect("degraded CustomScan GUC should be set");
        let degraded_rows = query_client
            .query(
                "SELECT id, title \
                   FROM ec_spire_customscan_remote_restart_coord_sql \
                  ORDER BY embedding <#> ARRAY[1.0, 0.0]::real[], id \
                  LIMIT 2",
                &[],
            )
            .expect("degraded CustomScan should skip terminated remote dispatch");
        assert!(degraded_rows.is_empty());

        Spi::run("SET LOCAL ec_spire.remote_search_consistency_mode = 'degraded'")
            .expect("degraded summary GUC should be set");
        let degraded_skip_count = Spi::get_one::<i64>(
            "SELECT degraded_skipped_dispatch_count \
               FROM ec_spire_remote_search_production_scan_handoff_summary(\
                    'ec_spire_customscan_remote_restart_coord_idx'::regclass, \
                    ARRAY[1.0, 0.0]::real[], 2)",
        )
        .expect("degraded skip count query should succeed")
        .expect("degraded skip count should exist");
        let degraded_skip_category = Spi::get_one::<String>(
            "SELECT first_degraded_skip_category \
               FROM ec_spire_remote_search_production_scan_handoff_summary(\
                    'ec_spire_customscan_remote_restart_coord_idx'::regclass, \
                    ARRAY[1.0, 0.0]::real[], 2)",
        )
        .expect("degraded skip category query should succeed")
        .expect("degraded skip category should exist");
        assert_eq!(degraded_skip_count, 1);
        assert_eq!(degraded_skip_category, "remote_backend_terminated");

        std::env::set_var(
            "EC_SPIRE_REMOTE_CONNINFO_SPIRE_REMOTE_CUSTOMSCAN_REMOTE_RESTART",
            &loopback_conninfo,
        );
        let rejoin_register_result = Spi::get_one::<bool>(&format!(
            "SELECT ec_spire_register_remote_node_descriptor(\
                     '{}'::oid, 2, 110, 'spire/remote/customscan/remote_restart', \
                     decode('{remote_identity_hex}', 'hex'), \
                     'ec_spire_customscan_remote_restart_remote_idx', 'active', {active_epoch}, \
                     {active_epoch}, '{}', 'none')",
            u32::from(index_oid),
            env!("CARGO_PKG_VERSION")
        ))
        .expect("rejoined remote descriptor registration should succeed")
        .expect("rejoined remote descriptor registration result should exist");
        assert!(rejoin_register_result);

        query_client
            .batch_execute("SET ec_spire.remote_search_consistency_mode = 'strict'")
            .expect("strict CustomScan GUC should be restored");
        let rejoined_rows = query_client
            .query(
                "SELECT id, title \
                   FROM ec_spire_customscan_remote_restart_coord_sql \
                  ORDER BY embedding <#> ARRAY[1.0, 0.0]::real[], id \
                  LIMIT 2",
                &[],
            )
            .expect("strict CustomScan should succeed after remote descriptor rejoins");
        let rejoined_ids = rejoined_rows
            .into_iter()
            .map(|row| {
                row.try_get::<_, i64>(0)
                    .expect("rejoined CustomScan id should decode")
            })
            .collect::<Vec<_>>();
        assert_eq!(rejoined_ids, vec![101, 102]);
    }

    #[pg_test]
    fn test_ec_spire_customscan_coord_drop_waits_for_scan_sql() {
        let _env_lock = env_var_test_lock();
        let loopback_conninfo = current_pg_test_loopback_conninfo();
        let sleep_conninfo = format!(
            "{loopback_conninfo} application_name=ec_spire_coord_drop_sleep \
             options='-c search_path=ec_spire_customscan_coord_drop,public'"
        );
        let _conninfo_secret = ScopedEnvVar::set(
            "EC_SPIRE_REMOTE_CONNINFO_SPIRE_REMOTE_CUSTOMSCAN_COORD_DROP",
            &sleep_conninfo,
        );
        let mut loopback_client = postgres::Client::connect(&loopback_conninfo, postgres::NoTls)
            .expect("loopback client connection should succeed");

        loopback_client
            .batch_execute(
                "DROP TABLE IF EXISTS ec_spire_customscan_coord_drop_remote_sql; \
                 DROP SCHEMA IF EXISTS ec_spire_customscan_coord_drop CASCADE; \
                 CREATE TABLE ec_spire_customscan_coord_drop_remote_sql \
                     (id bigint primary key, title text not null, embedding ecvector); \
                 INSERT INTO ec_spire_customscan_coord_drop_remote_sql (id, title, embedding) VALUES \
                     (101, 'remote drop alpha', encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
                     (102, 'remote drop beta', encode_to_ecvector(ARRAY[0.5, 0.5], 4, 42)); \
                 CREATE INDEX ec_spire_customscan_coord_drop_remote_idx \
                     ON ec_spire_customscan_coord_drop_remote_sql USING ec_spire \
                     (embedding ecvector_spire_ip_ops) \
                     WITH (nlists = 2, nprobe = 2, storage_format = 'rabitq'); \
                 CREATE SCHEMA ec_spire_customscan_coord_drop; \
                 CREATE FUNCTION ec_spire_customscan_coord_drop.ec_spire_remote_search(\
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
                     SELECT rs.* \
                     FROM pg_sleep(0.30), \
                          public.ec_spire_remote_search($1, $2, $3, $4, $5, $6) AS rs \
                 $function$",
            )
            .expect("loopback coordinator-drop remote fixture should be created");
        let remote_identity_hex = loopback_remote_index_identity_hex(
            &mut loopback_client,
            "ec_spire_customscan_coord_drop_remote_idx",
        );

        Spi::run(
            "CREATE TABLE ec_spire_customscan_coord_drop_coord_sql \
             (id bigint primary key, title text not null, embedding ecvector)",
        )
        .expect("coordinator-drop table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_customscan_coord_drop_coord_sql (id, title, embedding) VALUES \
             (1, 'coordinator drop alpha', encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, 'coordinator drop beta', encode_to_ecvector(ARRAY[0.5, 0.5], 4, 42))",
        )
        .expect("coordinator-drop insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_customscan_coord_drop_coord_idx \
             ON ec_spire_customscan_coord_drop_coord_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) \
             WITH (nlists = 2, nprobe = 2, storage_format = 'rabitq')",
        )
        .expect("coordinator-drop index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_spire_customscan_coord_drop_coord_idx'::regclass::oid",
        )
        .expect("coordinator-drop index oid query should succeed")
        .expect("coordinator-drop index oid should exist");
        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_customscan_coord_drop_coord_idx'::regclass)",
        )
        .expect("coordinator-drop active epoch query should succeed")
        .expect("coordinator-drop active epoch should exist");
        let coord_leaf_pids = Spi::get_one::<Vec<i64>>(
            "SELECT array_agg(leaf_pid ORDER BY leaf_pid) FROM \
             ec_spire_index_leaf_snapshot('ec_spire_customscan_coord_drop_coord_idx'::regclass)",
        )
        .expect("coordinator-drop leaf pid query should succeed")
        .expect("coordinator-drop leaf pids should exist");

        unsafe {
            for pid in &coord_leaf_pids {
                am::debug_spire_rewrite_placement_node(index_oid, *pid as u64, 2);
            }
        }
        let register_result = Spi::get_one::<bool>(&format!(
            "SELECT ec_spire_register_remote_node_descriptor(\
                     '{}'::oid, 2, 111, 'spire/remote/customscan/coord_drop', \
                     decode('{remote_identity_hex}', 'hex'), \
                     'ec_spire_customscan_coord_drop_remote_idx', 'active', {active_epoch}, \
                     {active_epoch}, '{}', 'none')",
            u32::from(index_oid),
            env!("CARGO_PKG_VERSION")
        ))
        .expect("coordinator-drop remote descriptor registration should succeed")
        .expect("coordinator-drop remote descriptor registration result should exist");
        assert!(register_result);

        let scan_conninfo = loopback_conninfo.clone();
        let scan_handle = std::thread::spawn(move || -> Result<Vec<i64>, String> {
            let mut scan_client =
                postgres::Client::connect(&scan_conninfo, postgres::NoTls)
                    .map_err(|error| error.to_string())?;
            scan_client
                .batch_execute("SET enable_seqscan = off; SET enable_indexscan = off")
                .map_err(|error| error.to_string())?;
            let rows = scan_client
                .query(
                    "SELECT id \
                       FROM ec_spire_customscan_coord_drop_coord_sql \
                      ORDER BY embedding <#> ARRAY[1.0, 0.0]::real[], id \
                      LIMIT 2",
                    &[],
                )
                .map_err(|error| error.to_string())?;
            rows.into_iter()
                .map(|row| row.try_get::<_, i64>(0).map_err(|error| error.to_string()))
                .collect::<Result<Vec<_>, _>>()
        });

        let wait_started = std::time::Instant::now();
        loop {
            let sleepers = loopback_client
                .query_one(
                    "SELECT count(*)::bigint \
                       FROM pg_stat_activity \
                      WHERE application_name = 'ec_spire_coord_drop_sleep' \
                        AND wait_event = 'PgSleep'",
                    &[],
                )
                .expect("remote sleep activity query should succeed")
                .try_get::<_, i64>(0)
                .expect("remote sleep activity count should decode");
            if sleepers > 0 {
                break;
            }
            assert!(
                wait_started.elapsed() < std::time::Duration::from_secs(2),
                "timed out waiting for long-running CustomScan remote receive"
            );
            std::thread::sleep(std::time::Duration::from_millis(25));
        }

        let mut drop_client = postgres::Client::connect(&loopback_conninfo, postgres::NoTls)
            .expect("drop client connection should succeed");
        drop_client
            .batch_execute("SET lock_timeout = '100ms'; SET statement_timeout = '2s'")
            .expect("drop client timeout GUCs should be set");
        let drop_error = drop_client
            .batch_execute("DROP INDEX ec_spire_customscan_coord_drop_coord_idx")
            .expect_err("coordinator DROP INDEX should wait behind the active CustomScan lock");
        let drop_message = drop_error.to_string();
        assert!(
            drop_message.contains("lock timeout")
                || drop_message.contains("canceling statement due to lock timeout"),
            "{drop_message}"
        );
        let index_still_present = drop_client
            .query_one(
                "SELECT to_regclass('ec_spire_customscan_coord_drop_coord_idx') IS NOT NULL",
                &[],
            )
            .expect("post-timeout index presence query should succeed")
            .try_get::<_, bool>(0)
            .expect("post-timeout index presence should decode");
        assert!(index_still_present);

        let scan_ids = scan_handle
            .join()
            .expect("scan thread should not panic")
            .expect("active CustomScan should complete after blocked DROP INDEX times out");
        assert_eq!(scan_ids, vec![101, 102]);

        let prepared_prefix = format!("ec_spire_insert_{}_%", u32::from(index_oid));
        let prepared_count = loopback_client
            .query_one(
                "SELECT count(*)::bigint FROM pg_prepared_xacts WHERE gid LIKE $1",
                &[&prepared_prefix],
            )
            .expect("coordinator-drop prepared xact cleanup query should succeed")
            .try_get::<_, i64>(0)
            .expect("coordinator-drop prepared xact cleanup count should decode");
        assert_eq!(prepared_count, 0);
    }
