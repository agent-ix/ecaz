    #[pg_test]
    fn test_ec_spire_remote_delete_tuple_payload_idempotent_shape_sql() {
        Spi::run(
            "CREATE TABLE ec_spire_remote_delete_idempotent_sql \
             (id bigint primary key, title text not null, embedding ecvector)",
        )
        .expect("remote delete idempotent table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_remote_delete_idempotent_sql (id, title, embedding) VALUES \
             (7001, 'delete once', encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42))",
        )
        .expect("remote delete idempotent seed insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_remote_delete_idempotent_idx \
             ON ec_spire_remote_delete_idempotent_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 1)",
        )
        .expect("remote delete idempotent ec_spire index creation should succeed");

        let first_delete = Spi::get_one::<String>(
            "SELECT deleted_count::text || '|' || status \
               FROM ec_spire_remote_delete_tuple_payload(\
                    'ec_spire_remote_delete_idempotent_idx'::regclass, \
                    'id', \
                    int8send(7001::bigint)::bytea)",
        )
        .expect("first remote delete idempotent query should succeed")
        .expect("first remote delete idempotent query should return a row");
        let second_delete = Spi::get_one::<String>(
            "SELECT deleted_count::text || '|' || status \
               FROM ec_spire_remote_delete_tuple_payload(\
                    'ec_spire_remote_delete_idempotent_idx'::regclass, \
                    'id', \
                    int8send(7001::bigint)::bytea)",
        )
        .expect("second remote delete idempotent query should succeed")
        .expect("second remote delete idempotent query should return a row");
        let remaining_rows = Spi::get_one::<i64>(
            "SELECT count(*) FROM ec_spire_remote_delete_idempotent_sql WHERE id = 7001",
        )
        .expect("remote delete idempotent remaining count query should succeed")
        .expect("remote delete idempotent remaining count should exist");

        assert_eq!(first_delete, "1|ready");
        assert_eq!(second_delete, "0|ready");
        assert_eq!(remaining_rows, 0);
    }

    #[pg_test]
    fn test_ec_spire_coord_remote_delete_idem_no_redispatch_sql() {
        let _env_lock = env_var_test_lock();
        let loopback_conninfo = current_pg_test_loopback_conninfo();
        let _conninfo_secret = ScopedEnvVar::set(
            "EC_SPIRE_REMOTE_CONNINFO_SPIRE_REMOTE_DELETE_IDEMPOTENT_NO_REDISPATCH",
            &loopback_conninfo,
        );
        let mut loopback_client = postgres::Client::connect(&loopback_conninfo, postgres::NoTls)
            .expect("loopback client connection should succeed");
        loopback_client
            .batch_execute(
                "DROP TABLE IF EXISTS ec_spire_coord_remote_delete_idem_remote_sql; \
                 CREATE TABLE ec_spire_coord_remote_delete_idem_remote_sql \
                     (id bigint primary key, title text not null, embedding ecvector, \
                      source_identity bytea not null); \
                 INSERT INTO ec_spire_coord_remote_delete_idem_remote_sql \
                     (id, title, embedding, source_identity) VALUES \
                     (7101, 'remote delete once', encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42), \
                      decode('7172737475767778797a7b7c7d7e7f80', 'hex')); \
                 CREATE INDEX ec_spire_coord_remote_delete_idem_remote_idx \
                     ON ec_spire_coord_remote_delete_idem_remote_sql USING ec_spire \
                     (embedding ecvector_spire_ip_ops) WITH (nlists = 1);",
            )
            .expect("loopback remote delete idempotent target should be created");

        Spi::run(
            "CREATE TABLE ec_spire_coord_remote_delete_idem_coord_sql \
             (id bigint primary key, title text not null, embedding ecvector, \
              source_identity bytea not null)",
        )
        .expect("coordinator remote delete idempotent table should be created");
        Spi::run(
            "CREATE INDEX ec_spire_coord_remote_delete_idem_coord_idx \
             ON ec_spire_coord_remote_delete_idem_coord_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 1)",
        )
        .expect("coordinator remote delete idempotent index should be created");

        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot(\
                 'ec_spire_coord_remote_delete_idem_coord_idx'::regclass)",
        )
        .expect("coordinator remote delete active epoch query should succeed")
        .expect("coordinator remote delete active epoch should exist");
        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_spire_coord_remote_delete_idem_coord_idx'::regclass::oid",
        )
        .expect("coordinator remote delete index oid query should succeed")
        .expect("coordinator remote delete index oid should exist");
        let remote_identity_hex = Spi::get_one::<String>(
            "SELECT profile_fingerprint \
               FROM ec_spire_remote_search_endpoint_identity(\
                    'ec_spire_coord_remote_delete_idem_remote_idx'::regclass::oid)",
        )
        .expect("remote delete idempotent identity query should succeed")
        .expect("remote delete idempotent identity should exist");
        Spi::run(&format!(
            "SELECT ec_spire_register_remote_node_descriptor(\
                 'ec_spire_coord_remote_delete_idem_coord_idx'::regclass, \
                 41, 43, 'spire/remote/delete_idempotent_no_redispatch', \
                 decode('{remote_identity_hex}', 'hex'), \
                 'ec_spire_coord_remote_delete_idem_remote_idx', \
                 'active', {active_epoch}, {active_epoch}, '{}', '')",
            env!("CARGO_PKG_VERSION")
        ))
        .expect("remote delete idempotent descriptor registration should succeed");
        Spi::run(&format!(
            "INSERT INTO ec_spire_placement \
                 (index_oid, pk_value, node_id, centroid_id, served_epoch, source_identity) \
             VALUES ('ec_spire_coord_remote_delete_idem_coord_idx'::regclass, \
                     int8send(7101::bigint)::bytea, 41, 2, {active_epoch}, \
                     decode('7172737475767778797a7b7c7d7e7f80', 'hex'))"
        ))
        .expect("remote delete idempotent placement row should be inserted");

        let first_delete = Spi::get_one::<String>(
            "WITH result AS ( \
                 SELECT * FROM ec_spire_prepare_coordinator_delete_tuple_payload(\
                     'ec_spire_coord_remote_delete_idem_coord_idx'::regclass, \
                     'id', \
                     int8send(7101::bigint)::bytea) \
             ) \
             SELECT node_id::text || '|' || remote_delete_sent::text || '|' || \
                    remote_prepared::text || '|' || remote_deleted_count::text || '|' || \
                    placement_deleted::text || '|' || status || '|' || next_step \
               FROM result",
        )
        .expect("first coordinator remote delete idempotent query should succeed")
        .expect("first coordinator remote delete idempotent row should exist");
        let prepared_prefix = format!("ec_spire_insert_{}_41_{active_epoch}_%", u32::from(index_oid));
        let prepared_after_first = loopback_client
            .query_one(
                "SELECT count(*)::bigint \
                   FROM pg_prepared_xacts \
                  WHERE gid LIKE $1",
                &[&prepared_prefix],
            )
            .expect("prepared count after first delete should succeed")
            .try_get::<_, i64>(0)
            .expect("prepared count after first delete should decode");
        let second_delete = Spi::get_one::<String>(
            "WITH result AS ( \
                 SELECT * FROM ec_spire_prepare_coordinator_delete_tuple_payload(\
                     'ec_spire_coord_remote_delete_idem_coord_idx'::regclass, \
                     'id', \
                     int8send(7101::bigint)::bytea) \
             ) \
             SELECT node_id::text || '|' || remote_delete_sent::text || '|' || \
                    remote_prepared::text || '|' || remote_deleted_count::text || '|' || \
                    placement_deleted::text || '|' || status || '|' || next_step \
               FROM result",
        )
        .expect("second coordinator remote delete idempotent query should succeed")
        .expect("second coordinator remote delete idempotent row should exist");
        let prepared_after_second = loopback_client
            .query_one(
                "SELECT count(*)::bigint \
                   FROM pg_prepared_xacts \
                  WHERE gid LIKE $1",
                &[&prepared_prefix],
            )
            .expect("prepared count after second delete should succeed")
            .try_get::<_, i64>(0)
            .expect("prepared count after second delete should decode");
        let cleanup_summary = Spi::get_one::<String>(
            "SELECT \
                (SELECT count(*)::bigint \
                   FROM ec_spire_placement \
                  WHERE index_oid = 'ec_spire_coord_remote_delete_idem_coord_idx'::regclass \
                    AND pk_value = int8send(7101::bigint)::bytea)::text",
        )
        .expect("coordinator remote delete cleanup query should succeed")
        .expect("coordinator remote delete cleanup summary should exist");

        assert_eq!(
            first_delete,
            "41|true|true|1|true|remote_delete_prepared_pending_local_commit|await_local_commit"
        );
        assert_eq!(prepared_after_first, 1);
        assert_eq!(
            second_delete,
            "-1|false|false|0|false|delete_not_found_noop|done"
        );
        assert_eq!(
            prepared_after_second, prepared_after_first,
            "idempotent re-DELETE must not dispatch or prepare a second remote delete"
        );
        assert_eq!(cleanup_summary, "0");
    }
