    #[pg_test]
    fn test_ec_spire_plan_coordinator_insert_sql() {
        Spi::run(
            "CREATE TABLE ec_spire_insert_plan_sql (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_insert_plan_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_insert_plan_sql_idx ON ec_spire_insert_plan_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("ec_spire index creation should succeed");
        let index_oid =
            Spi::get_one::<pg_sys::Oid>("SELECT 'ec_spire_insert_plan_sql_idx'::regclass::oid")
                .expect("index oid query should succeed")
                .expect("index oid should exist");
        let expected_centroid_id = Spi::get_one::<i64>(
            "SELECT child_pid \
               FROM ec_spire_index_routing_centroid_snapshot(\
                    'ec_spire_insert_plan_sql_idx'::regclass) r \
               CROSS JOIN LATERAL ( \
                    SELECT sum(q.value * c.value)::real AS score \
                      FROM unnest(ARRAY[1.0, 0.0]::real[]) WITH ORDINALITY q(value, ord) \
                      JOIN unnest(r.centroid) WITH ORDINALITY c(value, ord) USING (ord) \
               ) scored \
              WHERE parent_kind = 'root' AND child_kind = 'leaf' \
              ORDER BY scored.score DESC, centroid_index, child_pid \
              LIMIT 1",
        )
        .expect("expected centroid query should succeed")
        .expect("expected centroid should exist");
        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_insert_plan_sql_idx'::regclass)",
        )
        .expect("active epoch query should succeed")
        .expect("active epoch should exist");

        unsafe {
            am::debug_spire_rewrite_placement_node(index_oid, expected_centroid_id as u64, 7)
        };

        let plan_row = Spi::get_one::<String>(
            "SELECT index_oid::text || ':' || encode(pk_value, 'hex') || ':' || \
                    node_id::text || ':' || centroid_id::text || ':' || \
                    served_epoch::text || ':' || encode(source_identity, 'hex') \
               FROM ec_spire_plan_coordinator_insert(\
                    'ec_spire_insert_plan_sql_idx'::regclass, \
                    decode('010203', 'hex'), \
                    ARRAY[1.0, 0.0]::real[], \
                    decode('000102030405060708090a0b0c0d0e0f', 'hex'))",
        )
        .expect("coordinator insert plan query should succeed")
        .expect("coordinator insert plan should exist");

        assert_eq!(
            plan_row,
            format!(
                "{}:010203:7:{expected_centroid_id}:{active_epoch}:000102030405060708090a0b0c0d0e0f",
                u32::from(index_oid)
            )
        );
    }
    #[pg_test]
    #[should_panic(expected = "pk_value must not be empty")]
    fn test_ec_spire_plan_coordinator_insert_rejects_empty_pk_sql() {
        Spi::run(
            "CREATE TABLE ec_spire_insert_plan_empty_pk_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_insert_plan_empty_pk_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_insert_plan_empty_pk_idx \
             ON ec_spire_insert_plan_empty_pk_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops)",
        )
        .expect("ec_spire index creation should succeed");
        Spi::run(
            "SELECT * FROM ec_spire_plan_coordinator_insert(\
                 'ec_spire_insert_plan_empty_pk_idx'::regclass, \
                 ''::bytea, \
                 ARRAY[1.0, 0.0]::real[], \
                 decode('000102030405060708090a0b0c0d0e0f', 'hex'))",
        )
        .expect("coordinator insert planning should reject empty pk");
    }

    #[pg_test]
    #[should_panic(expected = "source_identity must be exactly 16 bytes")]
    fn test_ec_spire_plan_coordinator_insert_rejects_bad_identity_sql() {
        Spi::run(
            "CREATE TABLE ec_spire_insert_plan_bad_identity_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_insert_plan_bad_identity_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_insert_plan_bad_identity_idx \
             ON ec_spire_insert_plan_bad_identity_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops)",
        )
        .expect("ec_spire index creation should succeed");
        Spi::run(
            "SELECT * FROM ec_spire_plan_coordinator_insert(\
                 'ec_spire_insert_plan_bad_identity_idx'::regclass, \
                 decode('01', 'hex'), \
                 ARRAY[1.0, 0.0]::real[], \
                 decode('0001', 'hex'))",
        )
        .expect("coordinator insert planning should reject bad source identity");
    }

    #[pg_test]
    fn test_ec_spire_plan_coordinator_insert_dispatch_ready_sql() {
        let _env_lock = env_var_test_lock();
        let _conninfo_secret = ScopedEnvVar::set(
            "EC_SPIRE_REMOTE_CONNINFO_SPIRE_REMOTE_INSERT_DISPATCH_READY",
            "host=127.0.0.1 port=1 dbname=postgres",
        );
        Spi::run(
            "CREATE TABLE ec_spire_insert_dispatch_ready_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_insert_dispatch_ready_idx \
             ON ec_spire_insert_dispatch_ready_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops)",
        )
        .expect("ec_spire index creation should succeed");
        Spi::run(
            "SELECT ec_spire_register_remote_node_descriptor(\
                 'ec_spire_insert_dispatch_ready_idx'::regclass, \
                 7, 11, 'spire/remote/insert_dispatch_ready', \
                 decode('aabb', 'hex'), 'remote_insert_ready_idx', \
                 'active', 9, 1, '0.1.1', '')",
        )
        .expect("remote descriptor registration should succeed");

        let dispatch_row = Spi::get_one::<String>(
            "SELECT status || ':' || dispatch_action || ':' || next_step || ':' || \
                    dispatch_transport || ':' || transaction_protocol || ':' || \
                    conninfo_provider_lookup_key || ':' || remote_index_regclass || ':' || \
                    descriptor_generation::text || ':' || \
                    remote_index_identity_bytes::text \
               FROM ec_spire_plan_coordinator_insert_dispatch(\
                    'ec_spire_insert_dispatch_ready_idx'::regclass, 7, 5)",
        )
        .expect("coordinator insert dispatch plan query should succeed")
        .expect("coordinator insert dispatch plan should exist");

        assert_eq!(
            dispatch_row,
            "ready:open_remote_transaction_send_insert_prepare_xact:remote_insert_prepare_transaction:libpq:remote_prepare_local_placement_commit_remote_prepared:EC_SPIRE_REMOTE_CONNINFO_SPIRE_REMOTE_INSERT_DISPATCH_READY:remote_insert_ready_idx:11:2"
        );
    }

    #[pg_test]
    fn test_ec_spire_insert_dispatch_missing_descriptor_sql() {
        Spi::run(
            "CREATE TABLE ec_spire_insert_dispatch_missing_desc_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_insert_dispatch_missing_desc_idx \
             ON ec_spire_insert_dispatch_missing_desc_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops)",
        )
        .expect("ec_spire index creation should succeed");

        let dispatch_row = Spi::get_one::<String>(
            "SELECT status || ':' || dispatch_action || ':' || next_step || ':' || \
                    conninfo_secret_name || ':' || remote_index_regclass \
               FROM ec_spire_plan_coordinator_insert_dispatch(\
                    'ec_spire_insert_dispatch_missing_desc_idx'::regclass, 8, 5)",
        )
        .expect("coordinator insert dispatch plan query should succeed")
        .expect("coordinator insert dispatch plan should exist");

        assert_eq!(
            dispatch_row,
            "requires_remote_node_descriptor:blocked:remote_node_descriptor:none:none"
        );
    }

    #[pg_test]
    fn test_ec_spire_insert_dispatch_missing_secret_sql() {
        let _env_lock = env_var_test_lock();
        let _missing_secret = ScopedEnvVar {
            key: "EC_SPIRE_REMOTE_CONNINFO_SPIRE_REMOTE_INSERT_DISPATCH_MISSING_SECRET",
            previous: std::env::var_os(
                "EC_SPIRE_REMOTE_CONNINFO_SPIRE_REMOTE_INSERT_DISPATCH_MISSING_SECRET",
            ),
        };
        std::env::remove_var(
            "EC_SPIRE_REMOTE_CONNINFO_SPIRE_REMOTE_INSERT_DISPATCH_MISSING_SECRET",
        );
        Spi::run(
            "CREATE TABLE ec_spire_insert_dispatch_missing_secret_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_insert_dispatch_missing_secret_idx \
             ON ec_spire_insert_dispatch_missing_secret_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops)",
        )
        .expect("ec_spire index creation should succeed");
        Spi::run(
            "SELECT ec_spire_register_remote_node_descriptor(\
                 'ec_spire_insert_dispatch_missing_secret_idx'::regclass, \
                 9, 12, 'spire/remote/insert_dispatch_missing_secret', \
                 decode('ccdd', 'hex'), 'remote_insert_missing_secret_idx', \
                 'active', 9, 1, '0.1.1', '')",
        )
        .expect("remote descriptor registration should succeed");

        let dispatch_row = Spi::get_one::<String>(
            "SELECT status || ':' || dispatch_action || ':' || next_step || ':' || \
                    conninfo_provider_lookup_key || ':' || remote_index_regclass \
               FROM ec_spire_plan_coordinator_insert_dispatch(\
                    'ec_spire_insert_dispatch_missing_secret_idx'::regclass, 9, 5)",
        )
        .expect("coordinator insert dispatch plan query should succeed")
        .expect("coordinator insert dispatch plan should exist");

        assert_eq!(
            dispatch_row,
            "requires_conninfo_secret_resolution:blocked:conninfo_secret_resolution:EC_SPIRE_REMOTE_CONNINFO_SPIRE_REMOTE_INSERT_DISPATCH_MISSING_SECRET:remote_insert_missing_secret_idx"
        );
    }

    #[pg_test]
    fn test_ec_spire_plan_coordinator_insert_dispatch_stale_epoch_sql() {
        let _env_lock = env_var_test_lock();
        let _conninfo_secret = ScopedEnvVar::set(
            "EC_SPIRE_REMOTE_CONNINFO_SPIRE_REMOTE_INSERT_DISPATCH_STALE",
            "host=127.0.0.1 port=1 dbname=postgres",
        );
        Spi::run(
            "CREATE TABLE ec_spire_insert_dispatch_stale_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_insert_dispatch_stale_idx \
             ON ec_spire_insert_dispatch_stale_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops)",
        )
        .expect("ec_spire index creation should succeed");
        Spi::run(
            "SELECT ec_spire_register_remote_node_descriptor(\
                 'ec_spire_insert_dispatch_stale_idx'::regclass, \
                 10, 13, 'spire/remote/insert_dispatch_stale', \
                 decode('eeff', 'hex'), 'remote_insert_stale_idx', \
                 'active', 4, 1, '0.1.1', '')",
        )
        .expect("remote descriptor registration should succeed");

        let dispatch_row = Spi::get_one::<String>(
            "SELECT status || ':' || dispatch_action || ':' || next_step \
               FROM ec_spire_plan_coordinator_insert_dispatch(\
                    'ec_spire_insert_dispatch_stale_idx'::regclass, 10, 5)",
        )
        .expect("coordinator insert dispatch plan query should succeed")
        .expect("coordinator insert dispatch plan should exist");

        assert_eq!(dispatch_row, "stale_epoch:blocked:remote_epoch_window");
    }

    #[pg_test]
    fn test_ec_spire_insert_remote_prepare_stages_placement_sql() {
        let _env_lock = env_var_test_lock();
        let loopback_conninfo = current_pg_test_loopback_conninfo();
        let _conninfo_secret = ScopedEnvVar::set(
            "EC_SPIRE_REMOTE_CONNINFO_SPIRE_REMOTE_INSERT_PREPARE",
            &loopback_conninfo,
        );
        let mut loopback_client = postgres::Client::connect(&loopback_conninfo, postgres::NoTls)
            .expect("loopback client connection should succeed");
        loopback_client
            .batch_execute(
                "DROP TABLE IF EXISTS ec_spire_insert_prepare_remote_sql; \
                 CREATE TABLE ec_spire_insert_prepare_remote_sql \
                     (id bigint primary key, title text not null, embedding ecvector, \
                      source_identity bytea not null); \
                 CREATE INDEX ec_spire_insert_prepare_remote_idx \
                     ON ec_spire_insert_prepare_remote_sql USING ec_spire \
                     (embedding ecvector_spire_ip_ops);",
            )
            .expect("loopback remote INSERT target should be created");

        Spi::run(
            "CREATE TABLE ec_spire_insert_prepare_coord_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("coordinator table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_insert_prepare_coord_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42))",
        )
        .expect("coordinator seed insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_insert_prepare_coord_idx \
             ON ec_spire_insert_prepare_coord_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops)",
        )
        .expect("coordinator ec_spire index creation should succeed");
        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_spire_insert_prepare_coord_idx'::regclass::oid",
        )
        .expect("index oid query should succeed")
        .expect("index oid should exist");
        Spi::run(
            "SELECT ec_spire_register_remote_node_descriptor(\
                 'ec_spire_insert_prepare_coord_idx'::regclass, \
                 11, 14, 'spire/remote/insert_prepare', \
                 decode('1234', 'hex'), 'ec_spire_insert_prepare_remote_idx', \
                 'active', 9, 1, '0.1.1', '')",
        )
        .expect("remote descriptor registration should succeed");

        let result = test_prepare_coordinator_insert_remote_sql(
            index_oid,
            vec![0x11],
            11,
            7,
            5,
            hex::decode("000102030405060708090a0b0c0d0e0f").expect("source identity hex"),
            "INSERT INTO ec_spire_insert_prepare_remote_sql \
                 (id, title, embedding, source_identity) VALUES \
             (101, 'prepared alpha', encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42), \
              decode('000102030405060708090a0b0c0d0e0f', 'hex'))",
        );

        // The shared remote 2PC helper still uses the historical insert gid
        // prefix for prepared DELETE transactions.
        let prepared_count = loopback_client
            .query_one(
                "SELECT count(*)::bigint FROM pg_prepared_xacts WHERE gid = $1",
                &[&result.prepared_gid],
            )
            .expect("prepared xact query should succeed")
            .try_get::<_, i64>(0)
            .expect("prepared xact count should decode");
        let remote_visible_count = loopback_client
            .query_one(
                "SELECT count(*)::bigint FROM ec_spire_insert_prepare_remote_sql WHERE id = 101",
                &[],
            )
            .expect("remote visibility query should succeed")
            .try_get::<_, i64>(0)
            .expect("remote visibility count should decode");
        let placement_count = Spi::get_one::<i64>(
            "SELECT count(*) FROM ec_spire_placement \
              WHERE index_oid = 'ec_spire_insert_prepare_coord_idx'::regclass \
                AND pk_value = decode('11', 'hex') \
                AND node_id = 11 \
                AND centroid_id = 7 \
                AND served_epoch = 5 \
                AND source_identity = decode('000102030405060708090a0b0c0d0e0f', 'hex')",
        )
        .expect("placement query should succeed")
        .expect("placement count should exist");

        assert_eq!(result.status, "remote_insert_prepared");
        assert_eq!(result.next_step, "local_placement_directory_write");
        assert_stable_spire_prepared_gid(&result.prepared_gid, index_oid, 11, 5);
        assert_eq!(prepared_count, 1);
        assert_eq!(
            remote_visible_count, 0,
            "prepared remote INSERT should not be visible before transaction resolution"
        );
        assert_eq!(placement_count, 1);
    }

    #[pg_test]
    fn test_ec_spire_insert_prepare_local_cancel_rolls_back() {
        let _env_lock = env_var_test_lock();
        let loopback_conninfo = current_pg_test_loopback_conninfo();
        let _conninfo_secret = ScopedEnvVar::set(
            "EC_SPIRE_REMOTE_CONNINFO_SPIRE_REMOTE_INSERT_PREPARE_CANCEL",
            &loopback_conninfo,
        );
        let mut loopback_client = postgres::Client::connect(&loopback_conninfo, postgres::NoTls)
            .expect("loopback client connection should succeed");
        loopback_client
            .batch_execute(
                "DROP TABLE IF EXISTS ec_spire_insert_prepare_cancel_remote_sql; \
                 CREATE TABLE ec_spire_insert_prepare_cancel_remote_sql \
                     (id bigint primary key, title text not null, embedding ecvector, \
                      source_identity bytea not null); \
                 CREATE INDEX ec_spire_insert_prepare_cancel_remote_idx \
                     ON ec_spire_insert_prepare_cancel_remote_sql USING ec_spire \
                     (embedding ecvector_spire_ip_ops);",
            )
            .expect("loopback remote INSERT target should be created");

        Spi::run(
            "CREATE TABLE ec_spire_insert_prepare_cancel_coord_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("coordinator table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_insert_prepare_cancel_coord_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42))",
        )
        .expect("coordinator seed insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_insert_prepare_cancel_coord_idx \
             ON ec_spire_insert_prepare_cancel_coord_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops)",
        )
        .expect("coordinator ec_spire index creation should succeed");
        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_spire_insert_prepare_cancel_coord_idx'::regclass::oid",
        )
        .expect("index oid query should succeed")
        .expect("index oid should exist");
        Spi::run(
            "SELECT ec_spire_register_remote_node_descriptor(\
                 'ec_spire_insert_prepare_cancel_coord_idx'::regclass, \
                 21, 24, 'spire/remote/insert_prepare_cancel', \
                 decode('1234', 'hex'), 'ec_spire_insert_prepare_cancel_remote_idx', \
                 'active', 9, 1, '0.1.1', '')",
        )
        .expect("remote descriptor registration should succeed");

        let local_backend_pid = Spi::get_one::<i32>("SELECT pg_backend_pid()")
            .expect("local backend pid query should succeed")
            .expect("local backend pid should exist");
        let remote_sql = format!(
            "SELECT pg_cancel_backend({local_backend_pid}); \
             SELECT pg_sleep(0.30); \
             INSERT INTO ec_spire_insert_prepare_cancel_remote_sql \
                 (id, title, embedding, source_identity) VALUES \
             (401, 'cancelled prepare', encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42), \
              decode('303132333435363738393a3b3c3d3e3f', 'hex'))"
        );
        let index_relation = unsafe {
            open_valid_ec_spire_index(
                index_oid,
                "test_ec_spire_insert_remote_prepare_local_cancel",
            )
        };
        let result = unsafe {
            am::spire_coordinator_insert_prepare_remote_sql(index_relation, 21, 5, &remote_sql)
        };
        unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
        unsafe { ScopedPgQueryCancelFlags::clear_pending_for_test() };

        let error = result.expect_err("local cancel should abort remote insert prepare");
        let prepared_gid_prefix = format!("ec_spire_insert_{}_21_5_%", u32::from(index_oid));
        let prepared_count = loopback_client
            .query_one(
                "SELECT count(*)::bigint FROM pg_prepared_xacts WHERE gid LIKE $1",
                &[&prepared_gid_prefix],
            )
            .expect("prepared xact query should succeed")
            .try_get::<_, i64>(0)
            .expect("prepared xact count should decode");
        let remote_visible_count = loopback_client
            .query_one(
                "SELECT count(*)::bigint \
                   FROM ec_spire_insert_prepare_cancel_remote_sql \
                  WHERE id = 401",
                &[],
            )
            .expect("remote visibility query should succeed")
            .try_get::<_, i64>(0)
            .expect("remote visibility count should decode");

        assert!(error.contains("local_query_cancelled"), "{error}");
        assert_eq!(
            prepared_count, 0,
            "local cancel should not leave an orphaned remote prepared xact"
        );
        assert_eq!(
            remote_visible_count, 0,
            "local cancel should roll back the remote INSERT transaction"
        );
    }

    #[pg_test]
    fn test_ec_spire_insert_remote_prepare_tuple_payload_endpoint_sql() {
        let _env_lock = env_var_test_lock();
        let loopback_conninfo = current_pg_test_loopback_conninfo();
        let _conninfo_secret = ScopedEnvVar::set(
            "EC_SPIRE_REMOTE_CONNINFO_SPIRE_REMOTE_INSERT_PREPARE_PAYLOAD",
            &loopback_conninfo,
        );
        let mut loopback_client = postgres::Client::connect(&loopback_conninfo, postgres::NoTls)
            .expect("loopback client connection should succeed");
        loopback_client
            .batch_execute(
                "DROP TABLE IF EXISTS ec_spire_insert_prepare_payload_remote_sql; \
                 CREATE TABLE ec_spire_insert_prepare_payload_remote_sql \
                     (id bigint primary key, title text not null, embedding ecvector); \
                 CREATE INDEX ec_spire_insert_prepare_payload_remote_idx \
                     ON ec_spire_insert_prepare_payload_remote_sql USING ec_spire \
                     (embedding ecvector_spire_ip_ops);",
            )
            .expect("loopback remote tuple-payload target should be created");

        Spi::run(
            "CREATE TABLE ec_spire_insert_prepare_payload_coord_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("coordinator table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_insert_prepare_payload_coord_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42))",
        )
        .expect("coordinator seed insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_insert_prepare_payload_coord_idx \
             ON ec_spire_insert_prepare_payload_coord_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops)",
        )
        .expect("coordinator ec_spire index creation should succeed");
        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_spire_insert_prepare_payload_coord_idx'::regclass::oid",
        )
        .expect("index oid query should succeed")
        .expect("index oid should exist");
        Spi::run(
            "SELECT ec_spire_register_remote_node_descriptor(\
                 'ec_spire_insert_prepare_payload_coord_idx'::regclass, \
                 12, 15, 'spire/remote/insert_prepare_payload', \
                 decode('5678', 'hex'), 'ec_spire_insert_prepare_payload_remote_idx', \
                 'active', 9, 1, '0.1.1', '')",
        )
        .expect("remote descriptor registration should succeed");
        let row_payload_json = Spi::get_one::<String>(
            "SELECT jsonb_build_object(\
                 'id', 202, \
                 'title', 'prepared tuple payload', \
                 'embedding', encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)::text)::text",
        )
        .expect("row payload json query should succeed")
        .expect("row payload json should exist");

        let result = test_prepare_coordinator_insert_remote_tuple_payload(
            index_oid,
            vec![0x12],
            12,
            8,
            5,
            hex::decode("101112131415161718191a1b1c1d1e1f").expect("source identity hex"),
            &row_payload_json,
            vec!["id".to_owned(), "title".to_owned(), "embedding".to_owned()],
        );

        let prepared_count = loopback_client
            .query_one(
                "SELECT count(*)::bigint FROM pg_prepared_xacts WHERE gid = $1",
                &[&result.prepared_gid],
            )
            .expect("prepared xact query should succeed")
            .try_get::<_, i64>(0)
            .expect("prepared xact count should decode");
        let remote_visible_count = loopback_client
            .query_one(
                "SELECT count(*)::bigint \
                   FROM ec_spire_insert_prepare_payload_remote_sql \
                  WHERE id = 202",
                &[],
            )
            .expect("remote visibility query should succeed")
            .try_get::<_, i64>(0)
            .expect("remote visibility count should decode");
        let placement_count = Spi::get_one::<i64>(
            "SELECT count(*) FROM ec_spire_placement \
              WHERE index_oid = 'ec_spire_insert_prepare_payload_coord_idx'::regclass \
                AND pk_value = decode('12', 'hex') \
                AND node_id = 12 \
                AND centroid_id = 8 \
                AND served_epoch = 5 \
                AND source_identity = decode('101112131415161718191a1b1c1d1e1f', 'hex')",
        )
        .expect("placement query should succeed")
        .expect("placement count should exist");

        assert_eq!(result.status, "remote_insert_prepared");
        assert_eq!(result.next_step, "local_placement_directory_write");
        assert_stable_spire_prepared_gid(&result.prepared_gid, index_oid, 12, 5);
        assert_eq!(prepared_count, 1);
        assert_eq!(
            remote_visible_count, 0,
            "prepared remote tuple-payload INSERT should not be visible before transaction resolution"
        );
        assert_eq!(placement_count, 1);
    }

    #[pg_test]
    fn test_ec_spire_prepare_coordinator_insert_tuple_payload_sql() {
        let _env_lock = env_var_test_lock();
        let loopback_conninfo = current_pg_test_loopback_conninfo();
        let _conninfo_secret = ScopedEnvVar::set(
            "EC_SPIRE_REMOTE_CONNINFO_SPIRE_REMOTE_COORDINATOR_INSERT_PAYLOAD_SQL",
            &loopback_conninfo,
        );
        let mut loopback_client = postgres::Client::connect(&loopback_conninfo, postgres::NoTls)
            .expect("loopback client connection should succeed");
        loopback_client
            .batch_execute(
                "DROP TABLE IF EXISTS ec_spire_coord_insert_payload_remote_sql; \
                 CREATE TABLE ec_spire_coord_insert_payload_remote_sql \
                     (id bigint primary key, title text not null, embedding ecvector); \
                 CREATE INDEX ec_spire_coord_insert_payload_remote_idx \
                     ON ec_spire_coord_insert_payload_remote_sql USING ec_spire \
                     (embedding ecvector_spire_ip_ops);",
            )
            .expect("loopback remote tuple-payload target should be created");

        Spi::run(
            "CREATE TABLE ec_spire_coord_insert_payload_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("coordinator table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_coord_insert_payload_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[0.8, 0.2], 4, 42)), \
             (3, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42)), \
             (4, encode_to_ecvector(ARRAY[-0.8, 0.2], 4, 42))",
        )
        .expect("coordinator seed insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_coord_insert_payload_idx \
             ON ec_spire_coord_insert_payload_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("coordinator ec_spire index creation should succeed");
        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_spire_coord_insert_payload_idx'::regclass::oid",
        )
        .expect("index oid query should succeed")
        .expect("index oid should exist");
        let expected_centroid_id = Spi::get_one::<i64>(
            "SELECT child_pid \
               FROM ec_spire_index_routing_centroid_snapshot(\
                    'ec_spire_coord_insert_payload_idx'::regclass) r \
               CROSS JOIN LATERAL ( \
                    SELECT sum(q.value * c.value)::real AS score \
                      FROM unnest(ARRAY[1.0, 0.0]::real[]) WITH ORDINALITY q(value, ord) \
                      JOIN unnest(r.centroid) WITH ORDINALITY c(value, ord) USING (ord) \
               ) scored \
              WHERE parent_kind = 'root' AND child_kind = 'leaf' \
              ORDER BY scored.score DESC, centroid_index, child_pid \
              LIMIT 1",
        )
        .expect("expected centroid query should succeed")
        .expect("expected centroid should exist");
        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_coord_insert_payload_idx'::regclass)",
        )
        .expect("active epoch query should succeed")
        .expect("active epoch should exist");
        unsafe {
            am::debug_spire_rewrite_placement_node(index_oid, expected_centroid_id as u64, 13)
        };
        Spi::run(
            "SELECT ec_spire_register_remote_node_descriptor(\
                 'ec_spire_coord_insert_payload_idx'::regclass, \
                 13, 16, 'spire/remote/coordinator_insert_payload_sql', \
                 decode('9abc', 'hex'), 'ec_spire_coord_insert_payload_remote_idx', \
                 'active', 9, 1, '0.1.1', '')",
        )
        .expect("remote descriptor registration should succeed");

        let result_row = Spi::get_one::<String>(
            "WITH result AS ( \
                 SELECT * FROM ec_spire_prepare_coordinator_insert_tuple_payload(\
                     'ec_spire_coord_insert_payload_idx'::regclass, \
                     decode('13', 'hex'), \
                     ARRAY[1.0, 0.0]::real[], \
                     decode('202122232425262728292a2b2c2d2e2f', 'hex'), \
                     jsonb_build_object(\
                         'id', 303, \
                         'title', 'prepared coordinator payload', \
                         'embedding', encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)::text), \
                     ARRAY['id', 'title', 'embedding']::text[]) \
             ) \
             SELECT status || '|' || next_step || '|' || node_id::text || '|' || \
                    centroid_id::text || '|' || served_epoch::text || '|' || \
                    placement_staged::text || '|' || remote_prepared::text || '|' || \
                    prepared_gid \
               FROM result",
        )
        .expect("coordinator insert tuple-payload helper query should succeed")
        .expect("coordinator insert tuple-payload helper should return a row");
        let parts = result_row.split('|').collect::<Vec<_>>();
        assert_eq!(parts.len(), 8);
        let prepared_gid = parts[7].to_owned();

        let prepared_count = loopback_client
            .query_one(
                "SELECT count(*)::bigint FROM pg_prepared_xacts WHERE gid = $1",
                &[&prepared_gid],
            )
            .expect("prepared xact query should succeed")
            .try_get::<_, i64>(0)
            .expect("prepared xact count should decode");
        let remote_visible_count = loopback_client
            .query_one(
                "SELECT count(*)::bigint \
                   FROM ec_spire_coord_insert_payload_remote_sql \
                  WHERE id = 303",
                &[],
            )
            .expect("remote visibility query should succeed")
            .try_get::<_, i64>(0)
            .expect("remote visibility count should decode");
        let placement_query = format!(
            "SELECT count(*) FROM ec_spire_placement \
              WHERE index_oid = 'ec_spire_coord_insert_payload_idx'::regclass \
                AND pk_value = decode('13', 'hex') \
                AND node_id = 13 \
                AND centroid_id = {expected_centroid_id} \
                AND served_epoch = {active_epoch} \
                AND source_identity = decode('202122232425262728292a2b2c2d2e2f', 'hex')"
        );
        let placement_count = Spi::get_one::<i64>(&placement_query)
            .expect("placement query should succeed")
            .expect("placement count should exist");
        let descriptor_row = Spi::get_one::<String>(
            "SELECT descriptor_generation::text || '|' || \
                    (last_served_epoch >= 1)::text || '|' || \
                    (min_retained_epoch = last_served_epoch)::text || '|' || \
                    (octet_length(remote_index_identity) > 0)::text \
               FROM ec_spire_remote_node_descriptor \
              WHERE coordinator_index_oid = 'ec_spire_coord_insert_payload_idx'::regclass \
                AND node_id = 13",
        )
        .expect("descriptor refresh query should succeed")
        .expect("descriptor refresh row should exist");

        assert_eq!(parts[0], "remote_insert_prepared_pending_local_commit");
        assert_eq!(parts[1], "await_local_commit");
        assert_eq!(parts[2], "13");
        assert_eq!(parts[3], expected_centroid_id.to_string());
        assert_eq!(parts[4], active_epoch.to_string());
        assert_eq!(parts[5], "true");
        assert_eq!(parts[6], "true");
        assert_stable_spire_prepared_gid(&prepared_gid, index_oid, 13, active_epoch);
        assert_eq!(prepared_count, 1);
        assert_eq!(
            remote_visible_count, 0,
            "prepared coordinator helper INSERT should not be visible before transaction resolution"
        );
        assert_eq!(placement_count, 1);
        assert_eq!(descriptor_row, "17|true|true|true");
    }

    #[pg_test]
    fn test_ec_spire_enable_coordinator_insert_trigger_sql() {
        let _env_lock = env_var_test_lock();
        let loopback_conninfo = current_pg_test_loopback_conninfo();
        let _conninfo_secret = ScopedEnvVar::set(
            "EC_SPIRE_REMOTE_CONNINFO_SPIRE_REMOTE_COORDINATOR_INSERT_TRIGGER_SQL",
            &loopback_conninfo,
        );
        let mut loopback_client = postgres::Client::connect(&loopback_conninfo, postgres::NoTls)
            .expect("loopback client connection should succeed");
        loopback_client
            .batch_execute(
                "DROP TABLE IF EXISTS ec_spire_coord_insert_trigger_remote_sql; \
                 CREATE TABLE ec_spire_coord_insert_trigger_remote_sql \
                     (id bigint primary key, title text not null, embedding ecvector, \
                      source_identity bytea not null); \
                 CREATE INDEX ec_spire_coord_insert_trigger_remote_idx \
                     ON ec_spire_coord_insert_trigger_remote_sql USING ec_spire \
                     (embedding ecvector_spire_ip_ops);",
            )
            .expect("loopback remote trigger target should be created");

        Spi::run(
            "CREATE TABLE ec_spire_coord_insert_trigger_sql \
             (id bigint primary key, title text not null, embedding ecvector, \
              source_identity bytea not null)",
        )
        .expect("coordinator trigger table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_coord_insert_trigger_sql \
                 (id, title, embedding, source_identity) VALUES \
             (1, 'coordinator positive', encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42), \
                 decode('000102030405060708090a0b0c0d0e0f', 'hex')), \
             (2, 'coordinator negative', encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42), \
                 decode('101112131415161718191a1b1c1d1e1f', 'hex'))",
        )
        .expect("coordinator seed insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_coord_insert_trigger_idx \
             ON ec_spire_coord_insert_trigger_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops)",
        )
        .expect("coordinator trigger ec_spire index creation should succeed");
        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_spire_coord_insert_trigger_idx'::regclass::oid",
        )
        .expect("index oid query should succeed")
        .expect("index oid should exist");
        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_coord_insert_trigger_idx'::regclass)",
        )
        .expect("active epoch query should succeed")
        .expect("active epoch should exist");
        let selected_pid = Spi::get_one::<i64>(
            "SELECT child_pid \
               FROM ec_spire_index_routing_centroid_snapshot(\
                    'ec_spire_coord_insert_trigger_idx'::regclass) r \
               CROSS JOIN LATERAL ( \
                    SELECT sum(q.value * c.value)::real AS score \
                      FROM unnest(ARRAY[1.0, 0.0]::real[]) WITH ORDINALITY q(value, ord) \
                      JOIN unnest(r.centroid) WITH ORDINALITY c(value, ord) USING (ord) \
               ) scored \
              WHERE parent_kind = 'root' AND child_kind = 'leaf' \
              ORDER BY scored.score DESC, centroid_index, child_pid \
              LIMIT 1",
        )
        .expect("selected pid query should succeed")
        .expect("selected pid should exist");
        unsafe { am::debug_spire_rewrite_placement_node(index_oid, selected_pid as u64, 14) };
        let remote_identity_hex = Spi::get_one::<String>(
            "SELECT profile_fingerprint \
               FROM ec_spire_remote_search_endpoint_identity(\
                    'ec_spire_coord_insert_trigger_remote_idx'::regclass::oid)",
        )
        .expect("remote identity query should succeed")
        .expect("remote identity should exist");
        Spi::run(&format!(
            "SELECT ec_spire_register_remote_node_descriptor(\
                 'ec_spire_coord_insert_trigger_idx'::regclass, \
                 14, 17, 'spire/remote/coordinator_insert_trigger_sql', \
                 decode('{remote_identity_hex}', 'hex'), \
                 'ec_spire_coord_insert_trigger_remote_idx', \
                 'active', {active_epoch}, {active_epoch}, '0.1.1', '')"
        ))
        .expect("remote descriptor registration should succeed");
        Spi::run(
            "SELECT ec_spire_enable_coordinator_insert(\
                 'ec_spire_coord_insert_trigger_sql'::regclass, \
                 'ec_spire_coord_insert_trigger_idx'::regclass, \
                 'id', 'embedding', 'source_identity')",
        )
        .expect("coordinator insert trigger enable should succeed");
        let installed_trigger_count = Spi::get_one::<i64>(
            "SELECT count(*)::bigint \
               FROM pg_trigger \
              WHERE tgrelid = 'ec_spire_coord_insert_trigger_sql'::regclass \
                AND tgname IN (\
                    'ec_spire_coordinator_insert_forward', \
                    'ec_spire_coordinator_insert_flush') \
                AND NOT tgisinternal",
        )
        .expect("coordinator insert trigger count query should succeed")
        .expect("coordinator insert trigger count should exist");

        Spi::run(
            "INSERT INTO ec_spire_coord_insert_trigger_sql \
                 (id, title, embedding, source_identity) VALUES \
             (404, 'trigger routed coordinator payload', \
                 encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42), \
                 decode('303132333435363738393a3b3c3d3e3f', 'hex'))",
        )
        .expect("coordinator insert trigger should forward row");

        let coordinator_row_count = Spi::get_one::<i64>(
            "SELECT count(*) FROM ec_spire_coord_insert_trigger_sql WHERE id = 404",
        )
        .expect("coordinator row count query should succeed")
        .expect("coordinator row count should exist");
        let queued_after_statement_count = Spi::get_one::<i64>(
            "SELECT CASE \
                    WHEN to_regclass('pg_temp.ec_spire_coordinator_insert_tuple_payload_queue') IS NULL \
                    THEN 0 \
                    ELSE (SELECT count(*)::bigint FROM ec_spire_coordinator_insert_tuple_payload_queue) \
                    END",
        )
        .expect("coordinator insert queue count query should succeed")
        .expect("coordinator insert queue count should exist");
        let placement_query = format!(
            "SELECT count(*) FROM ec_spire_placement \
              WHERE index_oid = 'ec_spire_coord_insert_trigger_idx'::regclass \
                AND pk_value = int8send(404::bigint)::bytea \
                AND node_id = 14 \
                AND centroid_id = {selected_pid} \
                AND served_epoch = {active_epoch} \
                AND source_identity = decode('303132333435363738393a3b3c3d3e3f', 'hex')"
        );
        let placement_count = Spi::get_one::<i64>(&placement_query)
            .expect("placement query should succeed")
            .expect("placement count should exist");
        let remote_visible_count = loopback_client
            .query_one(
                "SELECT count(*)::bigint \
                   FROM ec_spire_coord_insert_trigger_remote_sql \
                  WHERE id = 404",
                &[],
            )
            .expect("remote visibility query should succeed")
            .try_get::<_, i64>(0)
            .expect("remote visibility count should decode");
        let prepared_count = loopback_client
            .query_one(
                "SELECT count(*)::bigint \
                   FROM pg_prepared_xacts \
                  WHERE gid LIKE 'ec_spire_insert_%'",
                &[],
            )
            .expect("prepared xact query should succeed")
            .try_get::<_, i64>(0)
            .expect("prepared xact count should decode");
        let descriptor_row = Spi::get_one::<String>(
            "SELECT descriptor_generation::text || '|' || \
                    (last_served_epoch >= 1)::text || '|' || \
                    (min_retained_epoch = last_served_epoch)::text || '|' || \
                    (octet_length(remote_index_identity) > 0)::text \
               FROM ec_spire_remote_node_descriptor \
              WHERE coordinator_index_oid = 'ec_spire_coord_insert_trigger_idx'::regclass \
                AND node_id = 14",
        )
        .expect("trigger descriptor refresh query should succeed")
        .expect("trigger descriptor refresh row should exist");

        assert_eq!(
            coordinator_row_count, 0,
            "BEFORE trigger should suppress the coordinator heap row"
        );
        assert_eq!(installed_trigger_count, 2);
        assert_eq!(queued_after_statement_count, 0);
        assert_eq!(placement_count, 1);
        assert_eq!(
            remote_visible_count, 0,
            "remote INSERT should remain prepared until local transaction commit"
        );
        assert_eq!(prepared_count, 1);
        assert_eq!(descriptor_row, "18|true|true|true");
    }

    struct InsertDescriptorRaceResult {
        rows: u64,
        sqlstate: Option<String>,
        message: String,
        detail: Option<String>,
    }

    fn spire_insert_prepared_count(
        client: &mut postgres::Client,
        index_oid: u32,
        node_id: i32,
    ) -> i64 {
        // The prefix pins coordinator-owned numeric identifiers before the
        // served_epoch/top_xid suffix, so the wildcard cannot match
        // user-supplied GID text across descriptors.
        client
            .query_one(
                "SELECT count(*)::bigint \
                   FROM pg_prepared_xacts \
                  WHERE gid LIKE $1",
                &[&format!("ec_spire_insert_{}_{}_%", index_oid, node_id)],
            )
            .expect("SPIRE prepared xact count query should succeed")
            .try_get::<_, i64>(0)
            .expect("SPIRE prepared xact count should decode")
    }

    #[pg_test]
    fn test_ec_spire_insert_descriptor_race_sql() {
        let _env_lock = env_var_test_lock();
        const SECRET_KEY: &str =
            "EC_SPIRE_REMOTE_CONNINFO_SPIRE_REMOTE_COORDINATOR_INSERT_DESCRIPTOR_RACE";
        const SECRET_NAME: &str = "spire/remote/coordinator_insert_descriptor_race";
        let loopback_conninfo = current_pg_test_loopback_conninfo();
        let _conninfo_secret = ScopedEnvVar::set(SECRET_KEY, &loopback_conninfo);
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
                 DECLARE idx oid := to_regclass('ec_spire_coord_insert_descriptor_race_idx'); \
                 BEGIN \
                     IF idx IS NOT NULL THEN \
                         DELETE FROM ec_spire_placement WHERE index_oid = idx; \
                         DELETE FROM ec_spire_remote_node_descriptor \
                          WHERE coordinator_index_oid = idx; \
                     END IF; \
                 END $$; \
                 DROP TABLE IF EXISTS ec_spire_coord_insert_descriptor_race_remote_sql; \
                 DROP TABLE IF EXISTS ec_spire_coord_insert_descriptor_race_sql; \
                 CREATE TABLE ec_spire_coord_insert_descriptor_race_remote_sql \
                     (id bigint primary key, title text not null, embedding ecvector, \
                      source_identity bytea not null); \
                 CREATE INDEX ec_spire_coord_insert_descriptor_race_remote_idx \
                     ON ec_spire_coord_insert_descriptor_race_remote_sql USING ec_spire \
                     (embedding ecvector_spire_ip_ops); \
                 CREATE TABLE ec_spire_coord_insert_descriptor_race_sql \
                     (id bigint primary key, title text not null, embedding ecvector, \
                      source_identity bytea not null); \
                 INSERT INTO ec_spire_coord_insert_descriptor_race_sql \
                     (id, title, embedding, source_identity) VALUES \
                 (1, 'coordinator seed', encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42), \
                     decode('808182838485868788898a8b8c8d8e8f', 'hex')); \
                 CREATE INDEX ec_spire_coord_insert_descriptor_race_idx \
                     ON ec_spire_coord_insert_descriptor_race_sql USING ec_spire \
                     (embedding ecvector_spire_ip_ops);",
            )
            .expect("loopback descriptor-race fixtures should be created");

        let index_oid = setup_client
            .query_one(
                "SELECT 'ec_spire_coord_insert_descriptor_race_idx'::regclass::oid",
                &[],
            )
            .expect("descriptor-race index oid query should succeed")
            .try_get::<_, u32>(0)
            .expect("descriptor-race index oid should decode");
        let active_epoch = setup_client
            .query_one(
                "SELECT active_epoch FROM \
                 ec_spire_index_hierarchy_snapshot(\
                     'ec_spire_coord_insert_descriptor_race_idx'::regclass)",
                &[],
            )
            .expect("descriptor-race active epoch query should succeed")
            .try_get::<_, i64>(0)
            .expect("descriptor-race active epoch should decode");
        let selected_pid = setup_client
            .query_one(
                "SELECT child_pid \
                   FROM ec_spire_index_routing_centroid_snapshot(\
                        'ec_spire_coord_insert_descriptor_race_idx'::regclass) r \
                   CROSS JOIN LATERAL ( \
                        SELECT sum(q.value * c.value)::real AS score \
                          FROM unnest(ARRAY[1.0, 0.0]::real[]) WITH ORDINALITY q(value, ord) \
                          JOIN unnest(r.centroid) WITH ORDINALITY c(value, ord) USING (ord) \
                   ) scored \
                  WHERE parent_kind = 'root' AND child_kind = 'leaf' \
                  ORDER BY scored.score DESC, centroid_index, child_pid \
                  LIMIT 1",
                &[],
            )
            .expect("descriptor-race selected pid query should succeed")
            .try_get::<_, i64>(0)
            .expect("descriptor-race selected pid should decode");
        setup_client
            .batch_execute(&format!(
                "SELECT tests.ec_spire_test_rewrite_placement_node(\
                     'ec_spire_coord_insert_descriptor_race_idx'::regclass, \
                     {selected_pid}, 32)"
            ))
            .expect("descriptor-race placement rewrite should succeed");
        let remote_identity_hex = loopback_remote_index_identity_hex(
            &mut setup_client,
            "ec_spire_coord_insert_descriptor_race_remote_idx",
        );
        setup_client
            .batch_execute(&format!(
                "SELECT ec_spire_register_remote_node_descriptor(\
                     'ec_spire_coord_insert_descriptor_race_idx'::regclass, \
                     32, 51, '{SECRET_NAME}', \
                     decode('{remote_identity_hex}', 'hex'), \
                     'ec_spire_coord_insert_descriptor_race_remote_idx', \
                     'active', {active_epoch}, {active_epoch}, '{}', ''); \
                 SELECT ec_spire_enable_coordinator_insert(\
                     'ec_spire_coord_insert_descriptor_race_sql'::regclass, \
                     'ec_spire_coord_insert_descriptor_race_idx'::regclass, \
                     'id', 'embedding', 'source_identity')",
                env!("CARGO_PKG_VERSION")
            ))
            .expect("descriptor-race descriptor and trigger should be enabled");

        let mut winner = postgres::Client::connect(&loopback_conninfo, postgres::NoTls)
            .expect("descriptor-race winner connection should succeed");
        winner
            .execute(
                "SELECT tests.ec_spire_test_set_env_var($1::text, $2::text)",
                &[&SECRET_KEY, &loopback_conninfo],
            )
            .expect("winner backend should receive conninfo secret env var");
        winner
            .batch_execute("BEGIN")
            .expect("winner transaction should begin");
        let winner_rows = winner
            .execute(
                "INSERT INTO ec_spire_coord_insert_descriptor_race_sql \
                     (id, title, embedding, source_identity) VALUES \
                 (707, 'descriptor race winner', \
                     encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42), \
                     decode('909192939495969798999a9b9c9d9e9f', 'hex'))",
                &[],
            )
            .expect("winner coordinator INSERT should stage before commit");
        assert_eq!(winner_rows, 0, "BEFORE trigger suppresses heap insert");
        assert_eq!(
            spire_insert_prepared_count(&mut setup_client, index_oid, 32),
            1
        );

        let loser_conninfo = loopback_conninfo.clone();
        let loser_handle = std::thread::spawn(move || {
            let mut loser = postgres::Client::connect(&loser_conninfo, postgres::NoTls)
                .expect("descriptor-race loser connection should succeed");
            loser
                .execute(
                    "SELECT tests.ec_spire_test_set_env_var($1::text, $2::text)",
                    &[&SECRET_KEY, &loser_conninfo],
                )
                .expect("loser backend should receive conninfo secret env var");
            loser
                .batch_execute("SET lock_timeout = '15s'; SET statement_timeout = '30s'; BEGIN")
                .expect("loser transaction should begin");
            let insert_result = loser.execute(
                "INSERT INTO ec_spire_coord_insert_descriptor_race_sql \
                     (id, title, embedding, source_identity) VALUES \
                 (708, 'descriptor race loser', \
                     encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42), \
                     decode('a0a1a2a3a4a5a6a7a8a9aaabacadaeaf', 'hex'))",
                &[],
            );
            let result = match insert_result {
                Ok(rows) => InsertDescriptorRaceResult {
                    rows,
                    sqlstate: None,
                    message: "ok".to_owned(),
                    detail: None,
                },
                Err(error) => {
                    let db_error = error.as_db_error();
                    InsertDescriptorRaceResult {
                        rows: 0,
                        sqlstate: db_error.map(|error| error.code().code().to_owned()),
                        message: db_error
                            .map(|error| error.message().to_owned())
                            .unwrap_or_else(|| error.to_string()),
                        detail: db_error.and_then(|error| error.detail().map(str::to_owned)),
                    }
                }
            };
            let _ = loser.batch_execute("ROLLBACK");
            result
        });

        let wait_started = Instant::now();
        while spire_insert_prepared_count(&mut setup_client, index_oid, 32) < 2 {
            assert!(
                wait_started.elapsed() < Duration::from_secs(10),
                "loser should reach remote PREPARE before blocking on descriptor refresh"
            );
            std::thread::sleep(Duration::from_millis(50));
        }

        winner
            .batch_execute("COMMIT")
            .expect("winner transaction should commit");
        let loser_result = loser_handle
            .join()
            .expect("descriptor-race loser thread should join");

        let prepared_wait_started = Instant::now();
        while spire_insert_prepared_count(&mut setup_client, index_oid, 32) != 0 {
            assert!(
                prepared_wait_started.elapsed() < Duration::from_secs(10),
                "descriptor-race callbacks should resolve all remote prepared xacts"
            );
            std::thread::sleep(Duration::from_millis(50));
        }

        let remote_visibility_summary = setup_client
            .query_one(
                "WITH winner AS ( \
                     SELECT selected_count \
                       FROM ec_spire_remote_select_tuple_payload(\
                            'ec_spire_coord_insert_descriptor_race_remote_idx'::regclass, \
                            'id', int8send(707::bigint)::bytea, ARRAY['id']::text[]) \
                 ), loser AS ( \
                     SELECT selected_count \
                       FROM ec_spire_remote_select_tuple_payload(\
                            'ec_spire_coord_insert_descriptor_race_remote_idx'::regclass, \
                            'id', int8send(708::bigint)::bytea, ARRAY['id']::text[]) \
                 ) \
                 SELECT (SELECT selected_count::text FROM winner) || '|' || \
                        (SELECT selected_count::text FROM loser)",
                &[],
            )
            .expect("descriptor-race remote visibility query should succeed")
            .try_get::<_, String>(0)
            .expect("descriptor-race remote visibility summary should decode");
        let placement_summary = setup_client
            .query_one(
                "SELECT count(*) FILTER (WHERE pk_value = int8send(707::bigint)::bytea)::text \
                        || '|' || \
                        count(*) FILTER (WHERE pk_value = int8send(708::bigint)::bytea)::text \
                   FROM ec_spire_placement \
                  WHERE index_oid = 'ec_spire_coord_insert_descriptor_race_idx'::regclass \
                    AND pk_value IN (int8send(707::bigint)::bytea, \
                                     int8send(708::bigint)::bytea)",
                &[],
            )
            .expect("descriptor-race placement query should succeed")
            .try_get::<_, String>(0)
            .expect("descriptor-race placement summary should decode");
        let descriptor_generation = setup_client
            .query_one(
                "SELECT descriptor_generation::bigint \
                   FROM ec_spire_remote_node_descriptor \
                  WHERE coordinator_index_oid = \
                        'ec_spire_coord_insert_descriptor_race_idx'::regclass \
                    AND node_id = 32",
                &[],
            )
            .expect("descriptor-race descriptor generation query should succeed")
            .try_get::<_, i64>(0)
            .expect("descriptor-race descriptor generation should decode");

        assert_eq!(loser_result.rows, 0);
        assert_eq!(loser_result.sqlstate.as_deref(), Some("40001"));
        assert_eq!(
            loser_result.message,
            "ec_spire_register_remote_node_descriptor descriptor_generation must advance existing descriptor_generation"
        );
        assert_eq!(
            loser_result.detail.as_deref(),
            Some("Retry the whole coordinator write after the winning descriptor refresh commits.")
        );
        assert_eq!(remote_visibility_summary, "1|0");
        assert_eq!(placement_summary, "1|0");
        assert_eq!(descriptor_generation, 52);
    }

    #[pg_test]
    fn test_ec_spire_trigger_multirow_commits_prepares_sql() {
        let _env_lock = env_var_test_lock();
        let loopback_conninfo = current_pg_test_loopback_conninfo();
        let _conninfo_secret = ScopedEnvVar::set(
            "EC_SPIRE_REMOTE_CONNINFO_SPIRE_REMOTE_COORDINATOR_INSERT_TRIGGER_MULTIROW_SQL",
            &loopback_conninfo,
        );
        let mut loopback_client = postgres::Client::connect(&loopback_conninfo, postgres::NoTls)
            .expect("loopback client connection should succeed");
        loopback_client
            .execute(
                "SELECT tests.ec_spire_test_set_env_var(\
                     'EC_SPIRE_REMOTE_CONNINFO_SPIRE_REMOTE_COORDINATOR_INSERT_TRIGGER_MULTIROW_SQL', \
                     $1)",
                &[&loopback_conninfo],
            )
            .expect("loopback backend should receive conninfo secret env var");
        loopback_client
            .batch_execute(
                "DROP TABLE IF EXISTS ec_spire_coord_insert_trigger_multirow_remote_sql; \
                 DROP TABLE IF EXISTS ec_spire_coord_insert_trigger_multirow_sql; \
                 CREATE TABLE ec_spire_coord_insert_trigger_multirow_remote_sql \
                     (id bigint primary key, title text not null, embedding ecvector, \
                      source_identity bytea not null); \
                 CREATE INDEX ec_spire_coord_insert_trigger_multirow_remote_idx \
                     ON ec_spire_coord_insert_trigger_multirow_remote_sql USING ec_spire \
                     (embedding ecvector_spire_ip_ops); \
                 CREATE TABLE ec_spire_coord_insert_trigger_multirow_sql \
                     (id bigint primary key, title text not null, embedding ecvector, \
                      source_identity bytea not null); \
                 INSERT INTO ec_spire_coord_insert_trigger_multirow_sql \
                     (id, title, embedding, source_identity) VALUES \
                 (1, 'coordinator positive seed', encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42), \
                     decode('000102030405060708090a0b0c0d0e0f', 'hex')), \
                 (2, 'coordinator negative seed', encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42), \
                     decode('101112131415161718191a1b1c1d1e1f', 'hex')); \
                 CREATE INDEX ec_spire_coord_insert_trigger_multirow_idx \
                     ON ec_spire_coord_insert_trigger_multirow_sql USING ec_spire \
                     (embedding ecvector_spire_ip_ops) WITH (nlists = 2);",
            )
            .expect("loopback multi-row trigger fixtures should be created");

        let active_epoch = loopback_client
            .query_one(
                "SELECT active_epoch FROM \
                 ec_spire_index_hierarchy_snapshot(\
                     'ec_spire_coord_insert_trigger_multirow_idx'::regclass)",
                &[],
            )
            .expect("active epoch query should succeed")
            .try_get::<_, i64>(0)
            .expect("active epoch should decode");
        let positive_pid = loopback_client
            .query_one(
                "SELECT child_pid \
                   FROM ec_spire_index_routing_centroid_snapshot(\
                        'ec_spire_coord_insert_trigger_multirow_idx'::regclass) r \
                   CROSS JOIN LATERAL ( \
                        SELECT sum(q.value * c.value)::real AS score \
                          FROM unnest(ARRAY[1.0, 0.0]::real[]) WITH ORDINALITY q(value, ord) \
                          JOIN unnest(r.centroid) WITH ORDINALITY c(value, ord) USING (ord) \
                   ) scored \
                  WHERE parent_kind = 'root' AND child_kind = 'leaf' \
                  ORDER BY scored.score DESC, centroid_index, child_pid \
                  LIMIT 1",
                &[],
            )
            .expect("positive pid query should succeed")
            .try_get::<_, i64>(0)
            .expect("positive pid should decode");
        let negative_pid = loopback_client
            .query_one(
                "SELECT child_pid \
                   FROM ec_spire_index_routing_centroid_snapshot(\
                        'ec_spire_coord_insert_trigger_multirow_idx'::regclass) r \
                   CROSS JOIN LATERAL ( \
                        SELECT sum(q.value * c.value)::real AS score \
                          FROM unnest(ARRAY[-1.0, 0.0]::real[]) WITH ORDINALITY q(value, ord) \
                          JOIN unnest(r.centroid) WITH ORDINALITY c(value, ord) USING (ord) \
                   ) scored \
                  WHERE parent_kind = 'root' AND child_kind = 'leaf' \
                  ORDER BY scored.score DESC, centroid_index, child_pid \
                  LIMIT 1",
                &[],
            )
            .expect("negative pid query should succeed")
            .try_get::<_, i64>(0)
            .expect("negative pid should decode");
        assert_ne!(
            positive_pid, negative_pid,
            "fixture requires two leaf placements so prepared GIDs differ by node"
        );
        loopback_client
            .batch_execute(&format!(
                "SELECT tests.ec_spire_test_rewrite_placement_nodes(\
                     'ec_spire_coord_insert_trigger_multirow_idx'::regclass, \
                     ARRAY[{positive_pid}, {negative_pid}]::bigint[], \
                     ARRAY[14, 15]::integer[])"
            ))
            .expect("placement rewrite should succeed");
        let remote_identity_hex = loopback_client
            .query_one(
                "SELECT profile_fingerprint \
                   FROM ec_spire_remote_search_endpoint_identity(\
                        'ec_spire_coord_insert_trigger_multirow_remote_idx'::regclass::oid)",
                &[],
            )
            .expect("remote identity query should succeed")
            .try_get::<_, String>(0)
            .expect("remote identity should decode");
        loopback_client
            .batch_execute(&format!(
                "SELECT ec_spire_register_remote_node_descriptor(\
                     'ec_spire_coord_insert_trigger_multirow_idx'::regclass, \
                     14, 17, 'spire/remote/coordinator_insert_trigger_multirow_sql', \
                     decode('{remote_identity_hex}', 'hex'), \
                     'ec_spire_coord_insert_trigger_multirow_remote_idx', \
                     'active', {active_epoch}, {active_epoch}, '{}', ''); \
                 SELECT ec_spire_register_remote_node_descriptor(\
                     'ec_spire_coord_insert_trigger_multirow_idx'::regclass, \
                     15, 17, 'spire/remote/coordinator_insert_trigger_multirow_sql', \
                     decode('{remote_identity_hex}', 'hex'), \
                     'ec_spire_coord_insert_trigger_multirow_remote_idx', \
                     'active', {active_epoch}, {active_epoch}, '{}', ''); \
                 SELECT ec_spire_enable_coordinator_insert(\
                     'ec_spire_coord_insert_trigger_multirow_sql'::regclass, \
                     'ec_spire_coord_insert_trigger_multirow_idx'::regclass, \
                     'id', 'embedding', 'source_identity')",
                env!("CARGO_PKG_VERSION"),
                env!("CARGO_PKG_VERSION")
            ))
            .expect("remote descriptors and coordinator insert trigger should be enabled");

        loopback_client
            .batch_execute(
                "BEGIN; \
                 INSERT INTO ec_spire_coord_insert_trigger_multirow_sql \
                     (id, title, embedding, source_identity) VALUES \
                 (405, 'trigger routed positive payload', \
                     encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42), \
                     decode('404142434445464748494a4b4c4d4e4f', 'hex')), \
                 (406, 'trigger routed negative payload', \
                     encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42), \
                     decode('505152535455565758595a5b5c5d5e5f', 'hex')); \
                 COMMIT;",
            )
            .expect("multi-row coordinator insert trigger transaction should commit");

        let placement_count = loopback_client
            .query_one(
                "SELECT count(*)::bigint \
                   FROM ec_spire_placement \
                  WHERE index_oid = 'ec_spire_coord_insert_trigger_multirow_idx'::regclass \
                    AND (pk_value, node_id, centroid_id, served_epoch, source_identity) IN ( \
                        (int8send(405::bigint)::bytea, 14, $1::bigint, $3::bigint, \
                         decode('404142434445464748494a4b4c4d4e4f', 'hex')), \
                        (int8send(406::bigint)::bytea, 15, $2::bigint, $3::bigint, \
                         decode('505152535455565758595a5b5c5d5e5f', 'hex')) \
                    )",
                &[&positive_pid, &negative_pid, &active_epoch],
            )
            .expect("placement count query should succeed")
            .try_get::<_, i64>(0)
            .expect("placement count should decode");
        loopback_client
            .batch_execute("BEGIN")
            .expect("duplicate probe transaction should begin");
        let positive_duplicate_insert_count = loopback_client
            .execute(
                "INSERT INTO ec_spire_coord_insert_trigger_multirow_remote_sql \
                     (id, title, embedding, source_identity) VALUES \
                 (405, 'duplicate positive probe', encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42), \
                  decode('404142434445464748494a4b4c4d4e4f', 'hex')) \
                 ON CONFLICT DO NOTHING",
                &[],
            )
            .expect("positive duplicate probe should succeed");
        let negative_duplicate_insert_count = loopback_client
            .execute(
                "INSERT INTO ec_spire_coord_insert_trigger_multirow_remote_sql \
                     (id, title, embedding, source_identity) VALUES \
                 (406, 'duplicate negative probe', encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42), \
                  decode('505152535455565758595a5b5c5d5e5f', 'hex')) \
                 ON CONFLICT DO NOTHING",
                &[],
            )
            .expect("negative duplicate probe should succeed");
        loopback_client
            .batch_execute("ROLLBACK")
            .expect("duplicate probe transaction should roll back");
        let prepared_count = loopback_client
            .query_one(
                "SELECT count(*)::bigint \
                   FROM pg_prepared_xacts \
                  WHERE gid LIKE 'ec_spire_insert_%'",
                &[],
            )
            .expect("prepared xact query should succeed")
            .try_get::<_, i64>(0)
            .expect("prepared xact count should decode");

        assert_eq!(placement_count, 2);
        assert_eq!(
            positive_duplicate_insert_count, 0,
            "committed positive remote row should block duplicate insert"
        );
        assert_eq!(
            negative_duplicate_insert_count, 0,
            "committed negative remote row should block duplicate insert"
        );
        assert_eq!(
            prepared_count, 0,
            "local COMMIT should resolve all per-row remote prepared transactions"
        );
    }

    #[pg_test]
    fn test_ec_spire_insert_trigger_source_identity_json_roundtrip_sql() {
        Spi::run(
            "CREATE TABLE ec_spire_insert_trigger_json_roundtrip_sql \
             (id bigint primary key, title text not null, embedding ecvector, \
              source_identity bytea not null)",
        )
        .expect("json roundtrip table creation should succeed");

        let roundtrip_identity_hex = Spi::get_one::<String>(
            "WITH original AS ( \
                 SELECT ROW( \
                            515::bigint, \
                            'json bytea roundtrip'::text, \
                            encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42), \
                            decode('0001feff102030405060708090a0b0c0', 'hex') \
                        )::ec_spire_insert_trigger_json_roundtrip_sql AS row_value \
             ) \
             SELECT encode( \
                        (jsonb_populate_record( \
                             NULL::ec_spire_insert_trigger_json_roundtrip_sql, \
                             to_jsonb(row_value) \
                         )).source_identity, \
                        'hex' \
                    ) \
               FROM original",
        )
        .expect("json roundtrip query should succeed")
        .expect("json roundtrip identity should exist");

        assert_eq!(roundtrip_identity_hex, "0001feff102030405060708090a0b0c0");
    }

    #[pg_test]
    fn test_ec_spire_insert_trigger_payload_type_roundtrip_sql() {
        let _env_lock = env_var_test_lock();
        let loopback_conninfo = current_pg_test_loopback_conninfo();
        let mut loopback_client = postgres::Client::connect(&loopback_conninfo, postgres::NoTls)
            .expect("loopback client connection should succeed");
        loopback_client
            .execute(
                "SELECT tests.ec_spire_test_set_env_var(\
                     'EC_SPIRE_REMOTE_CONNINFO_SPIRE_REMOTE_TRIGGER_PAYLOAD_TYPES', \
                     $1)",
                &[&loopback_conninfo],
            )
            .expect("loopback backend should receive conninfo secret env var");
        loopback_client
            .batch_execute(
                "DROP TABLE IF EXISTS ec_spire_trig_payload_remote; \
                 DROP TABLE IF EXISTS ec_spire_trig_payload_coord; \
                 DROP DOMAIN IF EXISTS ec_spire_payload_code_dom; \
                 CREATE DOMAIN ec_spire_payload_code_dom AS text \
                     CHECK (VALUE ~ '^[A-Z]{3}[0-9]{2}$'); \
                 CREATE TABLE ec_spire_trig_payload_remote \
                     (id bigint primary key, title text not null, \
                      amount numeric(12,4) not null, event_at timestamptz not null, \
                      metadata_json json not null, metadata_jsonb jsonb not null, \
                      edge_text text not null, domain_code ec_spire_payload_code_dom not null, \
                      nullable_note text, required_label text not null, \
                      default_label text not null default 'remote-default', \
                      embedding ecvector, source_identity bytea not null); \
                 CREATE INDEX ec_spire_trig_payload_remote_idx \
                     ON ec_spire_trig_payload_remote USING ec_spire \
                     (embedding ecvector_spire_ip_ops); \
                 CREATE TABLE ec_spire_trig_payload_coord \
                     (id bigint primary key, title text not null, \
                      amount numeric(12,4) not null, event_at timestamptz not null, \
                      metadata_json json not null, metadata_jsonb jsonb not null, \
                      edge_text text not null, domain_code ec_spire_payload_code_dom not null, \
                      nullable_note text, required_label text not null, \
                      default_label text not null default 'coord-default', \
                      embedding ecvector, source_identity bytea not null); \
                 INSERT INTO ec_spire_trig_payload_coord \
                     (id, title, amount, event_at, metadata_json, metadata_jsonb, edge_text, \
                      domain_code, nullable_note, required_label, embedding, source_identity) \
                 VALUES \
                     (1, 'payload seed', 1.0000, '2026-05-12 00:00:00+00', \
                      '{\"seed\":true}'::json, '{\"seed\":true}'::jsonb, \
                      'seed text', 'ABC01', NULL, 'seed required', \
                      encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42), \
                      decode('000102030405060708090a0b0c0d0e0f', 'hex')); \
                 CREATE INDEX ec_spire_trig_payload_coord_idx \
                     ON ec_spire_trig_payload_coord USING ec_spire \
                     (embedding ecvector_spire_ip_ops);",
            )
            .expect("loopback trigger payload fixtures should be created");

        let active_epoch = loopback_client
            .query_one(
                "SELECT active_epoch FROM \
                 ec_spire_index_hierarchy_snapshot('ec_spire_trig_payload_coord_idx'::regclass)",
                &[],
            )
            .expect("active epoch query should succeed")
            .try_get::<_, i64>(0)
            .expect("active epoch should decode");
        let selected_pid = loopback_client
            .query_one(
                "SELECT child_pid \
                   FROM ec_spire_index_routing_centroid_snapshot(\
                        'ec_spire_trig_payload_coord_idx'::regclass) r \
                   CROSS JOIN LATERAL ( \
                        SELECT sum(q.value * c.value)::real AS score \
                          FROM unnest(ARRAY[1.0, 0.0]::real[]) WITH ORDINALITY q(value, ord) \
                          JOIN unnest(r.centroid) WITH ORDINALITY c(value, ord) USING (ord) \
                   ) scored \
                  WHERE parent_kind = 'root' AND child_kind = 'leaf' \
                  ORDER BY scored.score DESC, centroid_index, child_pid \
                  LIMIT 1",
                &[],
            )
            .expect("selected pid query should succeed")
            .try_get::<_, i64>(0)
            .expect("selected pid should decode");
        loopback_client
            .batch_execute(&format!(
                "SELECT tests.ec_spire_test_rewrite_placement_node(\
                     'ec_spire_trig_payload_coord_idx'::regclass, {selected_pid}, 18)"
            ))
            .expect("placement rewrite should succeed");
        let remote_identity_hex = loopback_client
            .query_one(
                "SELECT profile_fingerprint \
                   FROM ec_spire_remote_search_endpoint_identity(\
                        'ec_spire_trig_payload_remote_idx'::regclass::oid)",
                &[],
            )
            .expect("remote identity query should succeed")
            .try_get::<_, String>(0)
            .expect("remote identity should decode");
        loopback_client
            .batch_execute(&format!(
                "SELECT ec_spire_register_remote_node_descriptor(\
                     'ec_spire_trig_payload_coord_idx'::regclass, \
                     18, 17, 'spire/remote/trigger_payload_types', \
                     decode('{remote_identity_hex}', 'hex'), \
                     'ec_spire_trig_payload_remote_idx', \
                     'active', {active_epoch}, {active_epoch}, '{}', ''); \
                 SELECT ec_spire_enable_coordinator_insert(\
                     'ec_spire_trig_payload_coord'::regclass, \
                     'ec_spire_trig_payload_coord_idx'::regclass, \
                     'id', 'embedding', 'source_identity')",
                env!("CARGO_PKG_VERSION")
            ))
            .expect("remote descriptor and coordinator insert trigger should be enabled");

        loopback_client
            .batch_execute(
                "INSERT INTO ec_spire_trig_payload_coord \
                     (id, title, amount, event_at, metadata_json, metadata_jsonb, edge_text, \
                      domain_code, nullable_note, required_label, embedding, source_identity) \
                 VALUES \
                     (501, 'payload roundtrip', 12345.6789, \
                      '2026-05-12 21:30:45.123456+00', \
                      '{\"outer\":{\"answer\":42},\"list\":[true,\"json\"]}'::json, \
                      '{\"outer\":{\"answer\":84},\"list\":[false,\"jsonb\"]}'::jsonb, \
                      E'quote '' and slash \\\\ and newline\\nend', \
                      'XYZ99', NULL, 'required ok', \
                      encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42), \
                      decode('606162636465666768696a6b6c6d6e6f', 'hex'))",
            )
            .expect("payload type coordinator insert trigger should forward row");

        let remote_row = loopback_client
            .query_one(
                "SELECT amount::text, \
                        to_char(event_at AT TIME ZONE 'UTC', \
                                'YYYY-MM-DD HH24:MI:SS.US'), \
                        metadata_json::jsonb #>> '{outer,answer}', \
                        metadata_json::jsonb #>> '{list,1}', \
                        metadata_jsonb #>> '{outer,answer}', \
                        metadata_jsonb #>> '{list,1}', \
                        edge_text, domain_code::text, nullable_note IS NULL, \
                        required_label, default_label, encode(source_identity, 'hex') \
                   FROM ec_spire_trig_payload_remote \
                  WHERE id = 501",
                &[],
            )
            .expect("remote payload row query should succeed");
        assert_eq!(
            remote_row
                .try_get::<_, String>(0)
                .expect("amount should decode"),
            "12345.6789"
        );
        assert_eq!(
            remote_row
                .try_get::<_, String>(1)
                .expect("event_at should decode"),
            "2026-05-12 21:30:45.123456"
        );
        assert_eq!(
            remote_row
                .try_get::<_, String>(2)
                .expect("json answer should decode"),
            "42"
        );
        assert_eq!(
            remote_row
                .try_get::<_, String>(3)
                .expect("json list value should decode"),
            "json"
        );
        assert_eq!(
            remote_row
                .try_get::<_, String>(4)
                .expect("jsonb answer should decode"),
            "84"
        );
        assert_eq!(
            remote_row
                .try_get::<_, String>(5)
                .expect("jsonb list value should decode"),
            "jsonb"
        );
        assert_eq!(
            remote_row
                .try_get::<_, String>(6)
                .expect("edge text should decode"),
            "quote ' and slash \\ and newline\nend"
        );
        assert_eq!(
            remote_row
                .try_get::<_, String>(7)
                .expect("domain code should decode"),
            "XYZ99"
        );
        assert!(remote_row
            .try_get::<_, bool>(8)
            .expect("nullable note flag should decode"));
        assert_eq!(
            remote_row
                .try_get::<_, String>(9)
                .expect("required label should decode"),
            "required ok"
        );
        assert_eq!(
            remote_row
                .try_get::<_, String>(10)
                .expect("default label should decode"),
            "coord-default"
        );
        assert_eq!(
            remote_row
                .try_get::<_, String>(11)
                .expect("source identity should decode"),
            "606162636465666768696a6b6c6d6e6f"
        );

        let not_null_error = loopback_client
            .batch_execute(
                "INSERT INTO ec_spire_trig_payload_coord \
                     (id, title, amount, event_at, metadata_json, metadata_jsonb, edge_text, \
                      domain_code, embedding, source_identity) \
                 VALUES \
                     (502, 'payload not null failure', 2.0000, \
                      '2026-05-12 22:00:00+00', \
                      '{\"bad\":true}'::json, '{\"bad\":true}'::jsonb, \
                      'not null probe', 'DEF02', \
                      encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42), \
                      decode('707172737475767778797a7b7c7d7e7f', 'hex'))",
            )
            .expect_err("missing required_label should fail through remote payload insert");
        let not_null_message = not_null_error
            .as_db_error()
            .map(|db_error| format!("{}|{}", db_error.message(), db_error.detail().unwrap_or("")))
            .unwrap_or_else(|| not_null_error.to_string());
        assert!(
            not_null_message.contains("null value in column \"required_label\""),
            "{not_null_message}"
        );
        let failed_remote_count = loopback_client
            .query_one(
                "SELECT count(*)::bigint FROM ec_spire_trig_payload_remote WHERE id = 502",
                &[],
            )
            .expect("failed remote count query should succeed")
            .try_get::<_, i64>(0)
            .expect("failed remote count should decode");
        let prepared_count = loopback_client
            .query_one(
                "SELECT count(*)::bigint \
                   FROM pg_prepared_xacts \
                  WHERE gid LIKE 'ec_spire_insert_%'",
                &[],
            )
            .expect("prepared xact count query should succeed")
            .try_get::<_, i64>(0)
            .expect("prepared xact count should decode");
        assert_eq!(failed_remote_count, 0);
        assert_eq!(prepared_count, 0);
    }

    #[pg_test]
    fn test_ec_spire_schema_drift_fails_before_dispatch_sql() {
        let _env_lock = env_var_test_lock();
        let loopback_conninfo = current_pg_test_loopback_conninfo();
        let mut loopback_client = postgres::Client::connect(&loopback_conninfo, postgres::NoTls)
            .expect("loopback client connection should succeed");
        loopback_client
            .execute(
                "SELECT tests.ec_spire_test_set_env_var(\
                     'EC_SPIRE_REMOTE_CONNINFO_SPIRE_REMOTE_TRIGGER_SCHEMA_DRIFT', \
                     $1)",
                &[&loopback_conninfo],
            )
            .expect("loopback backend should receive conninfo secret env var");
        loopback_client
            .batch_execute(
                "DROP TABLE IF EXISTS ec_spire_trig_schema_drift_remote; \
                 DROP TABLE IF EXISTS ec_spire_trig_schema_drift_coord; \
                 CREATE TABLE ec_spire_trig_schema_drift_remote \
                     (id bigint primary key, title text not null, embedding ecvector, \
                      source_identity bytea not null); \
                 CREATE INDEX ec_spire_trig_schema_drift_remote_idx \
                     ON ec_spire_trig_schema_drift_remote USING ec_spire \
                     (embedding ecvector_spire_ip_ops); \
                 CREATE TABLE ec_spire_trig_schema_drift_coord \
                     (id bigint primary key, title text not null, embedding ecvector, \
                      source_identity bytea not null); \
                 INSERT INTO ec_spire_trig_schema_drift_coord \
                     (id, title, embedding, source_identity) VALUES \
                 (1, 'schema drift seed', encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42), \
                  decode('000102030405060708090a0b0c0d0e0f', 'hex')); \
                 CREATE INDEX ec_spire_trig_schema_drift_coord_idx \
                     ON ec_spire_trig_schema_drift_coord USING ec_spire \
                     (embedding ecvector_spire_ip_ops);",
            )
            .expect("loopback schema drift fixtures should be created");

        let active_epoch = loopback_client
            .query_one(
                "SELECT active_epoch FROM \
                 ec_spire_index_hierarchy_snapshot(\
                     'ec_spire_trig_schema_drift_coord_idx'::regclass)",
                &[],
            )
            .expect("active epoch query should succeed")
            .try_get::<_, i64>(0)
            .expect("active epoch should decode");
        let selected_pid = loopback_client
            .query_one(
                "SELECT child_pid \
                   FROM ec_spire_index_routing_centroid_snapshot(\
                        'ec_spire_trig_schema_drift_coord_idx'::regclass) r \
                   CROSS JOIN LATERAL ( \
                        SELECT sum(q.value * c.value)::real AS score \
                          FROM unnest(ARRAY[1.0, 0.0]::real[]) WITH ORDINALITY q(value, ord) \
                          JOIN unnest(r.centroid) WITH ORDINALITY c(value, ord) USING (ord) \
                   ) scored \
                  WHERE parent_kind = 'root' AND child_kind = 'leaf' \
                  ORDER BY scored.score DESC, centroid_index, child_pid \
                  LIMIT 1",
                &[],
            )
            .expect("selected pid query should succeed")
            .try_get::<_, i64>(0)
            .expect("selected pid should decode");
        loopback_client
            .batch_execute(&format!(
                "SELECT tests.ec_spire_test_rewrite_placement_node(\
                     'ec_spire_trig_schema_drift_coord_idx'::regclass, {selected_pid}, 19)"
            ))
            .expect("placement rewrite should succeed");
        let remote_identity_hex = loopback_client
            .query_one(
                "SELECT profile_fingerprint \
                   FROM ec_spire_remote_search_endpoint_identity(\
                        'ec_spire_trig_schema_drift_remote_idx'::regclass::oid)",
                &[],
            )
            .expect("remote identity query should succeed")
            .try_get::<_, String>(0)
            .expect("remote identity should decode");
        loopback_client
            .batch_execute(&format!(
                "SELECT ec_spire_register_remote_node_descriptor(\
                     'ec_spire_trig_schema_drift_coord_idx'::regclass, \
                     19, 17, 'spire/remote/trigger_schema_drift', \
                     decode('{remote_identity_hex}', 'hex'), \
                     'ec_spire_trig_schema_drift_remote_idx', \
                     'active', {active_epoch}, {active_epoch}, '{}', ''); \
                 SELECT ec_spire_enable_coordinator_insert(\
                     'ec_spire_trig_schema_drift_coord'::regclass, \
                     'ec_spire_trig_schema_drift_coord_idx'::regclass, \
                     'id', 'embedding', 'source_identity'); \
                 ALTER TABLE ec_spire_trig_schema_drift_coord \
                     ADD COLUMN coord_only text",
                env!("CARGO_PKG_VERSION")
            ))
            .expect("descriptor, trigger, and coordinator-only DDL should succeed");

        let drift_error = loopback_client
            .batch_execute(
                "INSERT INTO ec_spire_trig_schema_drift_coord \
                     (id, title, embedding, source_identity, coord_only) VALUES \
                 (601, 'schema drift payload', encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42), \
                  decode('808182838485868788898a8b8c8d8e8f', 'hex'), 'coordinator-only')",
            )
            .expect_err("coordinator-only DDL should trip schema drift guard");
        let drift_message = drift_error
            .as_db_error()
            .map(|db_error| db_error.message().to_owned())
            .unwrap_or_else(|| drift_error.to_string());
        let remote_row_count = loopback_client
            .query_one(
                "SELECT count(*)::bigint \
                   FROM ec_spire_trig_schema_drift_remote \
                  WHERE id = 601",
                &[],
            )
            .expect("remote schema drift row count query should succeed")
            .try_get::<_, i64>(0)
            .expect("remote schema drift row count should decode");
        let prepared_count = loopback_client
            .query_one(
                "SELECT count(*)::bigint \
                   FROM pg_prepared_xacts \
                  WHERE gid LIKE 'ec_spire_insert_%'",
                &[],
            )
            .expect("prepared xact count query should succeed")
            .try_get::<_, i64>(0)
            .expect("prepared xact count should decode");

        assert!(drift_message.contains("schema_drift"), "{drift_message}");
        assert!(
            drift_message.contains("coordinator side drifted"),
            "{drift_message}"
        );
        assert_eq!(remote_row_count, 0);
        assert_eq!(prepared_count, 0);
    }

    #[pg_test]
    fn test_ec_spire_remote_schema_fingerprint_pre_dispatch_sql() {
        let _env_lock = env_var_test_lock();
        const SECRET_KEY: &str =
            "EC_SPIRE_REMOTE_CONNINFO_SPIRE_REMOTE_TRIGGER_REMOTE_SCHEMA_DRIFT";
        const SECRET_NAME: &str = "spire/remote/trigger_remote_schema_drift";
        let loopback_conninfo = current_pg_test_loopback_conninfo();
        let _conninfo_secret = ScopedEnvVar::set(SECRET_KEY, &loopback_conninfo);
        let mut loopback_client = postgres::Client::connect(&loopback_conninfo, postgres::NoTls)
            .expect("loopback client connection should succeed");
        loopback_client
            .execute(
                "SELECT tests.ec_spire_test_set_env_var($1::text, $2::text)",
                &[&SECRET_KEY, &loopback_conninfo],
            )
            .expect("loopback backend should receive conninfo secret env var");
        loopback_client
            .batch_execute(
                "DROP TABLE IF EXISTS ec_spire_trig_remote_schema_drift_remote; \
                 DROP TABLE IF EXISTS ec_spire_trig_remote_schema_drift_coord; \
                 CREATE TABLE ec_spire_trig_remote_schema_drift_remote \
                     (id bigint primary key, title text not null, embedding ecvector, \
                      source_identity bytea not null); \
                 CREATE INDEX ec_spire_trig_remote_schema_drift_remote_idx \
                     ON ec_spire_trig_remote_schema_drift_remote USING ec_spire \
                     (embedding ecvector_spire_ip_ops); \
                 CREATE TABLE ec_spire_trig_remote_schema_drift_coord \
                     (id bigint primary key, title text not null, embedding ecvector, \
                      source_identity bytea not null); \
                 INSERT INTO ec_spire_trig_remote_schema_drift_coord \
                     (id, title, embedding, source_identity) VALUES \
                 (1, 'remote schema drift seed', encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42), \
                  decode('101112131415161718191a1b1c1d1e1f', 'hex')); \
                 CREATE INDEX ec_spire_trig_remote_schema_drift_coord_idx \
                     ON ec_spire_trig_remote_schema_drift_coord USING ec_spire \
                     (embedding ecvector_spire_ip_ops);",
            )
            .expect("loopback remote schema drift fixtures should be created");

        let active_epoch = loopback_client
            .query_one(
                "SELECT active_epoch FROM \
                 ec_spire_index_hierarchy_snapshot(\
                     'ec_spire_trig_remote_schema_drift_coord_idx'::regclass)",
                &[],
            )
            .expect("remote schema active epoch query should succeed")
            .try_get::<_, i64>(0)
            .expect("remote schema active epoch should decode");
        let selected_pid = loopback_client
            .query_one(
                "SELECT child_pid \
                   FROM ec_spire_index_routing_centroid_snapshot(\
                        'ec_spire_trig_remote_schema_drift_coord_idx'::regclass) r \
                   CROSS JOIN LATERAL ( \
                        SELECT sum(q.value * c.value)::real AS score \
                          FROM unnest(ARRAY[1.0, 0.0]::real[]) WITH ORDINALITY q(value, ord) \
                          JOIN unnest(r.centroid) WITH ORDINALITY c(value, ord) USING (ord) \
                   ) scored \
                  WHERE parent_kind = 'root' AND child_kind = 'leaf' \
                  ORDER BY scored.score DESC, centroid_index, child_pid \
                  LIMIT 1",
                &[],
            )
            .expect("remote schema selected pid query should succeed")
            .try_get::<_, i64>(0)
            .expect("remote schema selected pid should decode");
        loopback_client
            .batch_execute(&format!(
                "SELECT tests.ec_spire_test_rewrite_placement_node(\
                     'ec_spire_trig_remote_schema_drift_coord_idx'::regclass, {selected_pid}, 20)"
            ))
            .expect("remote schema placement rewrite should succeed");
        let remote_identity_hex = loopback_client
            .query_one(
                "SELECT profile_fingerprint \
                   FROM ec_spire_remote_search_endpoint_identity(\
                        'ec_spire_trig_remote_schema_drift_remote_idx'::regclass::oid)",
                &[],
            )
            .expect("remote schema identity query should succeed")
            .try_get::<_, String>(0)
            .expect("remote schema identity should decode");
        loopback_client
            .batch_execute(&format!(
                "SELECT ec_spire_register_remote_node_descriptor(\
                     'ec_spire_trig_remote_schema_drift_coord_idx'::regclass, \
                     20, 18, '{SECRET_NAME}', \
                     decode('{remote_identity_hex}', 'hex'), \
                     'ec_spire_trig_remote_schema_drift_remote_idx', \
                     'active', {active_epoch}, {active_epoch}, '{}', ''); \
                 SELECT ec_spire_enable_coordinator_insert(\
                     'ec_spire_trig_remote_schema_drift_coord'::regclass, \
                     'ec_spire_trig_remote_schema_drift_coord_idx'::regclass, \
                     'id', 'embedding', 'source_identity')",
                env!("CARGO_PKG_VERSION")
            ))
            .expect("remote schema descriptor and trigger should succeed");
        let stored_remote_fingerprint = loopback_client
            .query_one(
                "SELECT remote_insert_shape_fingerprint \
                   FROM ec_spire_remote_node_descriptor \
                  WHERE coordinator_index_oid = \
                        'ec_spire_trig_remote_schema_drift_coord_idx'::regclass \
                    AND node_id = 20",
                &[],
            )
            .expect("stored remote fingerprint query should succeed")
            .try_get::<_, String>(0)
            .expect("stored remote fingerprint should decode");
        assert_ne!(stored_remote_fingerprint, "unset");

        loopback_client
            .batch_execute(
                "ALTER TABLE ec_spire_trig_remote_schema_drift_remote \
                     ALTER COLUMN title TYPE varchar(128)",
            )
            .expect("remote-only ALTER TYPE should succeed");
        let drift_error = loopback_client
            .batch_execute(
                "INSERT INTO ec_spire_trig_remote_schema_drift_coord \
                     (id, title, embedding, source_identity) VALUES \
                 (611, 'remote schema drift payload', encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42), \
                  decode('909192939495969798999a9b9c9d9e9f', 'hex'))",
            )
            .expect_err("remote-only DDL should trip remote schema drift guard");
        let drift_message = drift_error
            .as_db_error()
            .map(|db_error| db_error.message().to_owned())
            .unwrap_or_else(|| drift_error.to_string());
        let remote_row_count = loopback_client
            .query_one(
                "SELECT count(*)::bigint \
                   FROM ec_spire_trig_remote_schema_drift_remote \
                  WHERE id = 611",
                &[],
            )
            .expect("remote schema drift row count query should succeed")
            .try_get::<_, i64>(0)
            .expect("remote schema drift row count should decode");
        let prepared_count = loopback_client
            .query_one(
                "SELECT count(*)::bigint \
                   FROM pg_prepared_xacts \
                  WHERE gid LIKE 'ec_spire_insert_%'",
                &[],
            )
            .expect("remote schema prepared xact count query should succeed")
            .try_get::<_, i64>(0)
            .expect("remote schema prepared xact count should decode");

        assert!(drift_message.contains("schema_drift"), "{drift_message}");
        assert!(
            drift_message.contains("remote side drifted"),
            "{drift_message}"
        );
        assert_eq!(remote_row_count, 0);
        assert_eq!(prepared_count, 0);
    }

    #[pg_test]
    fn test_ec_spire_insert_after_build_multi_row_epoch_progression() {
        Spi::run(
            "CREATE TABLE ec_spire_insert_multi_epoch (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_insert_multi_epoch (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_insert_multi_epoch_idx ON ec_spire_insert_multi_epoch \
             USING ec_spire (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("populated ec_spire index creation should succeed");

        Spi::run(
            "INSERT INTO ec_spire_insert_multi_epoch (id, embedding) VALUES \
             (3, encode_to_ecvector(ARRAY[0.9, 0.1], 4, 42)), \
             (4, encode_to_ecvector(ARRAY[0.8, 0.2], 4, 42)), \
             (5, encode_to_ecvector(ARRAY[-0.9, 0.1], 4, 42)), \
             (6, encode_to_ecvector(ARRAY[-0.8, 0.2], 4, 42)), \
             (7, encode_to_ecvector(ARRAY[0.0, 1.0], 4, 42))",
        )
        .expect("multi-row post-build insert should publish one delta epoch per row");

        let index_oid = index_oid("ec_spire_insert_multi_epoch_idx");
        let (active_epoch, next_pid, next_local_vec_seq) =
            unsafe { am::debug_spire_root_control(index_oid) };
        // This assertion documents the current no-batching contract: PostgreSQL
        // invokes `aminsert` once per row, so each row publishes its own delta
        // epoch. Insert batching should update this expectation deliberately.
        assert_eq!(active_epoch, 6);
        assert_eq!(next_pid, 9);
        assert_eq!(next_local_vec_seq, 8);
        assert_eq!(
            ec_spire_active_snapshot_i64("ec_spire_insert_multi_epoch_idx", "delta_object_count"),
            5
        );
        assert_eq!(
            ec_spire_active_snapshot_i64(
                "ec_spire_insert_multi_epoch_idx",
                "delta_assignment_count"
            ),
            5
        );

        Spi::run("SET LOCAL enable_seqscan = off").expect("SET should succeed");
        let inserted_rows_returned = Spi::get_one::<i64>(
            "SELECT count(*) FROM ( \
                 SELECT id FROM ec_spire_insert_multi_epoch \
                 ORDER BY embedding <#> ARRAY[0.0, 1.0]::real[] \
                 LIMIT 7 \
             ) ranked WHERE id BETWEEN 3 AND 7",
        )
        .expect("ordered ec_spire query should succeed")
        .expect("count should exist");
        assert_eq!(inserted_rows_returned, 5);
    }

    #[pg_test]
    #[should_panic(expected = "ec_spire aminsert failed: ec_spire vector dimensions mismatch")]
    fn test_ec_spire_insert_after_build_rejects_dimension_mismatch() {
        Spi::run(
            "CREATE TABLE ec_spire_insert_bad_dim (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_insert_bad_dim (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_insert_bad_dim_idx ON ec_spire_insert_bad_dim \
             USING ec_spire (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("populated ec_spire index creation should succeed");

        Spi::run(
            "INSERT INTO ec_spire_insert_bad_dim (id, embedding) VALUES \
             (3, encode_to_ecvector(ARRAY[1.0, 0.0, 0.0], 4, 42))",
        )
        .expect("dimension-mismatched post-build insert should fail");
    }

    #[pg_test]
    #[should_panic(expected = "ec_spire does not support NULL indexed values")]
    fn test_ec_spire_insert_after_build_rejects_null_value() {
        Spi::run(
            "CREATE TABLE ec_spire_insert_null_value (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_insert_null_value (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_insert_null_value_idx ON ec_spire_insert_null_value \
             USING ec_spire (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("populated ec_spire index creation should succeed");

        Spi::run("INSERT INTO ec_spire_insert_null_value (id, embedding) VALUES (3, NULL)")
            .expect("NULL post-build insert should fail");
    }

    #[pg_test]
    fn test_ec_spire_insert_bootstraps_empty_index_epoch() {
        Spi::run("CREATE TABLE ec_spire_insert_empty (id bigint primary key, embedding ecvector)")
            .expect("table creation should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_insert_empty_idx ON ec_spire_insert_empty \
             USING ec_spire (embedding ecvector_spire_ip_ops)",
        )
        .expect("empty ec_spire index creation should succeed");

        let index_oid = index_oid("ec_spire_insert_empty_idx");
        let (active_epoch, next_pid, next_local_vec_seq) =
            unsafe { am::debug_spire_root_control(index_oid) };
        assert_eq!(active_epoch, 0);
        assert_eq!(next_pid, 1);
        assert_eq!(next_local_vec_seq, 1);

        Spi::run(
            "INSERT INTO ec_spire_insert_empty (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42))",
        )
        .expect("first insert should bootstrap the empty ec_spire index");
        let (active_epoch, next_pid, next_local_vec_seq) =
            unsafe { am::debug_spire_root_control(index_oid) };
        assert_eq!(active_epoch, 1);
        assert_eq!(next_pid, 3);
        assert_eq!(next_local_vec_seq, 2);

        Spi::run(
            "INSERT INTO ec_spire_insert_empty (id, embedding) VALUES \
             (2, encode_to_ecvector(ARRAY[0.0, 1.0], 4, 42))",
        )
        .expect("second insert should publish a delta epoch");
        let (active_epoch, next_pid, next_local_vec_seq) =
            unsafe { am::debug_spire_root_control(index_oid) };
        assert_eq!(active_epoch, 2);
        assert_eq!(next_pid, 4);
        assert_eq!(next_local_vec_seq, 3);

        Spi::run("SET LOCAL enable_seqscan = off").expect("SET should succeed");
        let first_id = Spi::get_one::<i64>(
            "SELECT id FROM ec_spire_insert_empty \
             ORDER BY embedding <#> ARRAY[0.0, 1.0]::real[] \
             LIMIT 1",
        )
        .expect("ordered empty-bootstrap ec_spire query should succeed")
        .expect("query should return a row");
        assert_eq!(first_id, 2);
    }
