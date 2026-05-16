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

        let non_terminal_intent_count = setup_client
            .query_one(
                "SELECT count(*)::bigint \
                   FROM ec_spire_remote_prepared_xact_intent \
                  WHERE index_oid = \
                        'ec_spire_coord_insert_descriptor_race_idx'::regclass \
                    AND node_id = 32 \
                    AND intent_state NOT IN ('commit_local', 'rollback_local')",
                &[],
            )
            .expect("descriptor-race prepared intent query should succeed")
            .try_get::<_, i64>(0)
            .expect("descriptor-race prepared intent count should decode");

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
        assert_eq!(non_terminal_intent_count, 0);
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
