    #[pg_test]
    fn test_ec_spire_customscan_tuple_payload_stores_virtual_slot() {
        Spi::run(
            "CREATE TABLE ec_spire_customscan_payload_slot_sql \
             (id bigint primary key, title text not null, embedding ecvector)",
        )
        .expect("payload slot table creation should succeed");
        let relation_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_spire_customscan_payload_slot_sql'::regclass::oid",
        )
        .expect("payload slot relation oid query should succeed")
        .expect("payload slot relation oid should exist");

        unsafe {
            let relation =
                pg_sys::table_open(relation_oid, pg_sys::AccessShareLock as pg_sys::LOCKMODE);
            let slot = pg_sys::MakeSingleTupleTableSlot(
                (*relation).rd_att,
                pg_sys::table_slot_callbacks(relation),
            );
            am::spire_custom_scan_store_tuple_payload_json_for_test(
                slot,
                r#"{"id":42,"title":"remote alpha"}"#,
            );

            let mut id_is_null = false;
            let id_datum = pg_sys::slot_getattr(slot, 1, &mut id_is_null);
            let id = i64::from_datum(id_datum, id_is_null).expect("id should decode");
            let mut title_is_null = false;
            let title_datum = pg_sys::slot_getattr(slot, 2, &mut title_is_null);
            let title =
                String::from_datum(title_datum, title_is_null).expect("title should decode");
            let mut embedding_is_null = false;
            let _ = pg_sys::slot_getattr(slot, 3, &mut embedding_is_null);

            pg_sys::ExecDropSingleTupleTableSlot(slot);
            pg_sys::table_close(relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE);

            assert_eq!(id, 42);
            assert_eq!(title, "remote alpha");
            assert!(embedding_is_null);
        }
    }

    #[pg_test]
    fn test_ec_spire_customscan_returns_loopback_remote_tuple_payload() {
        let _env_lock = env_var_test_lock();
        let loopback_conninfo = current_pg_test_loopback_conninfo();
        let _conninfo_secret = ScopedEnvVar::set(
            "EC_SPIRE_REMOTE_CONNINFO_SPIRE_REMOTE_CUSTOMSCAN_PAYLOAD",
            &loopback_conninfo,
        );
        let mut loopback_client = postgres::Client::connect(&loopback_conninfo, postgres::NoTls)
            .expect("loopback client connection should succeed");
        loopback_client
            .batch_execute(
                "DROP TABLE IF EXISTS ec_spire_customscan_payload_remote_sql; \
                 CREATE TABLE ec_spire_customscan_payload_remote_sql \
                     (id bigint primary key, title text not null, embedding ecvector); \
                 INSERT INTO ec_spire_customscan_payload_remote_sql (id, title, embedding) VALUES \
                     (10, 'remote alpha', encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
                     (20, 'remote beta', encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42)); \
                 CREATE INDEX ec_spire_customscan_payload_remote_idx \
                     ON ec_spire_customscan_payload_remote_sql USING ec_spire \
                     (embedding ecvector_spire_ip_ops) \
                     WITH (nlists = 2, nprobe = 2, storage_format = 'rabitq')",
            )
            .expect("loopback remote CustomScan payload fixture should be created");
        let remote_active_epoch = loopback_client
            .query_one(
                "SELECT active_epoch FROM \
                 ec_spire_index_hierarchy_snapshot('ec_spire_customscan_payload_remote_idx'::regclass)",
                &[],
            )
            .expect("remote active epoch query should succeed")
            .try_get::<_, i64>(0)
            .expect("remote active epoch should decode");
        let remote_leaf_pids = loopback_client
            .query_one(
                "SELECT array_agg(leaf_pid ORDER BY leaf_pid) FROM \
                 ec_spire_index_leaf_snapshot('ec_spire_customscan_payload_remote_idx'::regclass)",
                &[],
            )
            .expect("remote leaf pid query should succeed")
            .try_get::<_, Vec<i64>>(0)
            .expect("remote leaf pids should decode");
        let remote_identity_hex = loopback_remote_index_identity_hex(
            &mut loopback_client,
            "ec_spire_customscan_payload_remote_idx",
        );

        Spi::run(
            "CREATE TABLE ec_spire_customscan_payload_coord_sql \
             (id bigint primary key, title text not null, embedding ecvector)",
        )
        .expect("coordinator table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_customscan_payload_coord_sql (id, title, embedding) VALUES \
             (1, 'coordinator alpha', encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, 'coordinator beta', encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("coordinator insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_customscan_payload_coord_idx \
             ON ec_spire_customscan_payload_coord_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) \
             WITH (nlists = 2, nprobe = 2, storage_format = 'rabitq')",
        )
        .expect("coordinator index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_spire_customscan_payload_coord_idx'::regclass::oid",
        )
        .expect("coordinator index oid query should succeed")
        .expect("coordinator index oid should exist");
        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_customscan_payload_coord_idx'::regclass)",
        )
        .expect("coordinator active epoch query should succeed")
        .expect("coordinator active epoch should exist");
        let coord_leaf_pids = Spi::get_one::<Vec<i64>>(
            "SELECT array_agg(leaf_pid ORDER BY leaf_pid) FROM \
             ec_spire_index_leaf_snapshot('ec_spire_customscan_payload_coord_idx'::regclass)",
        )
        .expect("coordinator leaf pid query should succeed")
        .expect("coordinator leaf pids should exist");
        assert_eq!(remote_active_epoch, active_epoch);
        assert_eq!(remote_leaf_pids, coord_leaf_pids);

        unsafe {
            for pid in &coord_leaf_pids {
                am::debug_spire_rewrite_placement_node(index_oid, *pid as u64, 2);
            }
        }
        let register_result = Spi::get_one::<bool>(&format!(
            "SELECT ec_spire_register_remote_node_descriptor(\
                     '{}'::oid, 2, 91, 'spire/remote/customscan/payload', \
                     decode('{remote_identity_hex}', 'hex'), \
                     'ec_spire_customscan_payload_remote_idx', 'active', {active_epoch}, \
                     {active_epoch}, '{}', 'none')",
            u32::from(index_oid),
            env!("CARGO_PKG_VERSION")
        ))
        .expect("remote descriptor registration should succeed")
        .expect("remote descriptor registration result should exist");
        assert!(register_result);
        let selected_pids_for_remote = coord_leaf_pids.clone();
        let remote_probe_rows = loopback_client
            .query(
                "SELECT payload.*, payload.tuple_payload::text AS tuple_payload_text \
                   FROM ec_spire_remote_search_tuple_payload(\
                        'ec_spire_customscan_payload_remote_idx'::regclass::oid, \
                        $1::bigint, ARRAY[1.0, 0.0]::real[], $2::bigint[], \
                        1, 'strict', ARRAY['id','title']::text[]) AS payload",
                &[&active_epoch, &selected_pids_for_remote],
            )
            .expect("loopback tuple payload probe should succeed");
        assert_eq!(remote_probe_rows.len(), 1);
        let remote_probe_status = remote_probe_rows[0]
            .try_get::<_, String>("status")
            .expect("loopback tuple payload status should decode");
        assert_eq!(remote_probe_status, "ready");
        Spi::run("SET LOCAL enable_seqscan = off").expect("disable seqscan should succeed");
        Spi::run("SET LOCAL enable_indexscan = off").expect("disable indexscan should succeed");

        let plan = Spi::connect(|client| {
            let rows = client
                .select(
                    "EXPLAIN (COSTS OFF) \
                     SELECT id, title FROM ec_spire_customscan_payload_coord_sql \
                     ORDER BY embedding <#> ARRAY[1.0, 0.0]::real[] LIMIT 1",
                    None,
                    &[],
                )
                .expect("CustomScan explain should succeed")
                .first();
            let mut lines = Vec::new();
            for row in rows {
                lines.push(
                    row.get::<String>(1)
                        .expect("CustomScan explain row should decode")
                        .expect("CustomScan explain row should not be NULL"),
                );
            }
            lines.join("\n")
        });
        let json_plan = Spi::connect(|client| {
            let rows = client
                .select(
                    "EXPLAIN (FORMAT JSON, ANALYZE, COSTS OFF) \
                     SELECT id, title FROM ec_spire_customscan_payload_coord_sql \
                     ORDER BY embedding <#> ARRAY[1.0, 0.0]::real[] LIMIT 1",
                    None,
                    &[],
                )
                .expect("CustomScan JSON explain should succeed");
            let mut lines = Vec::new();
            for row in rows {
                lines.push(
                    row.get::<pgrx::datum::JsonString>(1)
                        .expect("CustomScan JSON explain row should decode")
                        .expect("CustomScan JSON explain row should not be NULL")
                        .0,
                );
            }
            lines.join("\n")
        });
        let row = Spi::connect(|client| {
            let rows = client
                .select(
                    "SELECT id, title FROM ec_spire_customscan_payload_coord_sql \
                     ORDER BY embedding <#> ARRAY[1.0, 0.0]::real[] LIMIT 1",
                    None,
                    &[],
                )
                .expect("CustomScan remote tuple query should succeed");
            let row = rows.first();
            let id = row
                .get_one::<i64>()
                .expect("CustomScan remote id should decode")
                .expect("CustomScan remote id should exist");
            let title = row
                .get::<String>(2)
                .expect("CustomScan remote title should decode")
                .expect("CustomScan remote title should exist");
            (id, title)
        });
        let expression_row = Spi::connect(|client| {
            let rows = client
                .select(
                    "SELECT id, title || ' (boosted)' AS boosted_title \
                     FROM ec_spire_customscan_payload_coord_sql \
                     ORDER BY embedding <#> ARRAY[1.0, 0.0]::real[] LIMIT 1",
                    None,
                    &[],
                )
                .expect("CustomScan remote expression tuple query should succeed");
            let row = rows.first();
            let id = row
                .get_one::<i64>()
                .expect("CustomScan remote expression id should decode")
                .expect("CustomScan remote expression id should exist");
            let boosted_title = row
                .get::<String>(2)
                .expect("CustomScan remote expression title should decode")
                .expect("CustomScan remote expression title should exist");
            (id, boosted_title)
        });
        let embedding_projection = Spi::connect(|client| {
            let rows = client
                .select(
                    "SELECT ecvector_to_real_array(embedding, 4, false) \
                     FROM ec_spire_customscan_payload_coord_sql \
                     ORDER BY embedding <#> ARRAY[1.0, 0.0]::real[] LIMIT 1",
                    None,
                    &[],
                )
                .expect("CustomScan remote ecvector projection query should succeed");
            rows.first()
                .get_one::<Vec<f32>>()
                .expect("CustomScan remote ecvector projection should decode")
                .expect("CustomScan remote ecvector projection should exist")
        });

        assert!(
            plan.contains("Custom Scan (EcSpireDistributedScan)"),
            "expected EcSpireDistributedScan in plan:\n{plan}"
        );
        for expected in [
            "\"node\": \"EcSpireDistributedScan\"",
            "\"remote_fanout\": 1",
            "\"tuple_transport_status\": \"ready\"",
            "\"nprobe\": 2",
            "\"rerank_width\": 0",
        ] {
            assert!(
                json_plan.contains(expected),
                "missing {expected} in CustomScan JSON plan: {json_plan:?}"
            );
        }
        assert_eq!(row, (10, "remote alpha".to_owned()));
        assert_eq!(expression_row, (10, "remote alpha (boosted)".to_owned()));
        assert_eq!(embedding_projection, vec![1.0, 0.0]);
    }

    #[pg_test]
    fn test_ec_spire_customscan_eligibility_no_active_epoch() {
        Spi::run(
            "CREATE TABLE ec_spire_customscan_empty_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("empty table creation should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_customscan_empty_sql_idx \
             ON ec_spire_customscan_empty_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("empty ec_spire index creation should succeed");

        let eligibility_from = "FROM ec_spire_custom_scan_index_eligibility(\
             'ec_spire_customscan_empty_sql_idx'::regclass)";
        let status = Spi::get_one::<String>(&format!("SELECT status {eligibility_from}"))
            .expect("empty eligibility status query should succeed")
            .expect("empty eligibility status should exist");
        let eligible = Spi::get_one::<bool>(&format!(
            "SELECT eligible_for_custom_scan {eligibility_from}"
        ))
        .expect("empty eligibility query should succeed")
        .expect("empty eligibility value should exist");
        let remote_placement_count =
            Spi::get_one::<i64>(&format!("SELECT remote_placement_count {eligibility_from}"))
                .expect("empty remote placement count query should succeed")
                .expect("empty remote placement count should exist");

        assert_eq!(status, "no_active_epoch");
        assert!(!eligible);
        assert_eq!(remote_placement_count, 0);
    }

    #[pg_test]
    fn test_ec_spire_custom_scan_index_eligibility_remote() {
        Spi::run(
            "CREATE TABLE ec_spire_customscan_eligibility_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_customscan_eligibility_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_customscan_eligibility_sql_idx \
             ON ec_spire_customscan_eligibility_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("ec_spire index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_spire_customscan_eligibility_sql_idx'::regclass::oid",
        )
        .expect("index oid query should succeed")
        .expect("index oid should exist");
        let selected_pid = Spi::get_one::<i64>(
            "SELECT leaf_pid FROM \
             ec_spire_index_leaf_snapshot('ec_spire_customscan_eligibility_sql_idx'::regclass) \
             ORDER BY leaf_pid LIMIT 1",
        )
        .expect("leaf pid query should succeed")
        .expect("leaf pid should exist");

        let eligibility_from = "FROM ec_spire_custom_scan_index_eligibility(\
             'ec_spire_customscan_eligibility_sql_idx'::regclass)";
        let initial_status = Spi::get_one::<String>(&format!("SELECT status {eligibility_from}"))
            .expect("initial eligibility status query should succeed")
            .expect("initial eligibility status should exist");
        let initial_eligible = Spi::get_one::<bool>(&format!(
            "SELECT eligible_for_custom_scan {eligibility_from}"
        ))
        .expect("initial eligibility query should succeed")
        .expect("initial eligibility value should exist");
        assert_eq!(initial_status, "local_only");
        assert!(!initial_eligible);

        unsafe { am::debug_spire_rewrite_placement_node(index_oid, selected_pid as u64, 2) };

        let remote_status = Spi::get_one::<String>(&format!("SELECT status {eligibility_from}"))
            .expect("remote eligibility status query should succeed")
            .expect("remote eligibility status should exist");
        let remote_eligible = Spi::get_one::<bool>(&format!(
            "SELECT eligible_for_custom_scan {eligibility_from}"
        ))
        .expect("remote eligibility query should succeed")
        .expect("remote eligibility value should exist");
        let remote_node_count =
            Spi::get_one::<i64>(&format!("SELECT remote_node_count {eligibility_from}"))
                .expect("remote node count query should succeed")
                .expect("remote node count should exist");
        let remote_available_node_count = Spi::get_one::<i64>(&format!(
            "SELECT remote_available_node_count {eligibility_from}"
        ))
        .expect("remote available node count query should succeed")
        .expect("remote available node count should exist");
        let remote_available_placement_count = Spi::get_one::<i64>(&format!(
            "SELECT remote_available_placement_count {eligibility_from}"
        ))
        .expect("remote available placement count query should succeed")
        .expect("remote available placement count should exist");
        let remote_unavailable_placement_count = Spi::get_one::<i64>(&format!(
            "SELECT remote_unavailable_placement_count {eligibility_from}"
        ))
        .expect("remote unavailable placement count query should succeed")
        .expect("remote unavailable placement count should exist");
        let all_remote_placements_available = Spi::get_one::<bool>(&format!(
            "SELECT all_remote_placements_available {eligibility_from}"
        ))
        .expect("all remote placements available query should succeed")
        .expect("all remote placements available value should exist");

        assert_eq!(remote_status, "customscan_candidate");
        assert!(remote_eligible);
        assert_eq!(remote_node_count, 1);
        assert_eq!(remote_available_node_count, 1);
        assert_eq!(remote_available_placement_count, 1);
        assert_eq!(remote_unavailable_placement_count, 0);
        assert!(all_remote_placements_available);
    }

    #[pg_test]
    fn test_ec_spire_customscan_eligibility_no_available_remote() {
        Spi::run(
            "CREATE TABLE ec_spire_customscan_unavailable_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_customscan_unavailable_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_customscan_unavailable_sql_idx \
             ON ec_spire_customscan_unavailable_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("ec_spire index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_spire_customscan_unavailable_sql_idx'::regclass::oid",
        )
        .expect("index oid query should succeed")
        .expect("index oid should exist");
        let selected_pid = Spi::get_one::<i64>(
            "SELECT leaf_pid FROM \
             ec_spire_index_leaf_snapshot('ec_spire_customscan_unavailable_sql_idx'::regclass) \
             ORDER BY leaf_pid LIMIT 1",
        )
        .expect("leaf pid query should succeed")
        .expect("leaf pid should exist");

        unsafe {
            am::debug_spire_rewrite_placement_node(index_oid, selected_pid as u64, 2);
            am::debug_spire_rewrite_placement_state(index_oid, selected_pid as u64, "unavailable");
        }

        let eligibility_from = "FROM ec_spire_custom_scan_index_eligibility(\
             'ec_spire_customscan_unavailable_sql_idx'::regclass)";
        let status = Spi::get_one::<String>(&format!("SELECT status {eligibility_from}"))
            .expect("unavailable eligibility status query should succeed")
            .expect("unavailable eligibility status should exist");
        let eligible = Spi::get_one::<bool>(&format!(
            "SELECT eligible_for_custom_scan {eligibility_from}"
        ))
        .expect("unavailable eligibility query should succeed")
        .expect("unavailable eligibility value should exist");
        let remote_available_node_count = Spi::get_one::<i64>(&format!(
            "SELECT remote_available_node_count {eligibility_from}"
        ))
        .expect("remote available node count query should succeed")
        .expect("remote available node count should exist");
        let remote_available_placement_count = Spi::get_one::<i64>(&format!(
            "SELECT remote_available_placement_count {eligibility_from}"
        ))
        .expect("remote available placement count query should succeed")
        .expect("remote available placement count should exist");
        let remote_unavailable_placement_count = Spi::get_one::<i64>(&format!(
            "SELECT remote_unavailable_placement_count {eligibility_from}"
        ))
        .expect("remote unavailable placement count query should succeed")
        .expect("remote unavailable placement count should exist");
        let all_remote_placements_available = Spi::get_one::<bool>(&format!(
            "SELECT all_remote_placements_available {eligibility_from}"
        ))
        .expect("all remote placements available query should succeed")
        .expect("all remote placements available value should exist");

        assert_eq!(status, "no_available_remote_placements");
        assert!(!eligible);
        assert_eq!(remote_available_node_count, 0);
        assert_eq!(remote_available_placement_count, 0);
        assert_eq!(remote_unavailable_placement_count, 1);
        assert!(!all_remote_placements_available);
    }

    #[pg_test]
    fn test_ec_spire_customscan_explain_remote_order_limit() {
        Spi::run(
            "CREATE TABLE ec_spire_customscan_explain_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_customscan_explain_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_customscan_explain_sql_idx \
             ON ec_spire_customscan_explain_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("ec_spire index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_spire_customscan_explain_sql_idx'::regclass::oid",
        )
        .expect("index oid query should succeed")
        .expect("index oid should exist");
        let selected_pid = Spi::get_one::<i64>(
            "SELECT leaf_pid FROM \
             ec_spire_index_leaf_snapshot('ec_spire_customscan_explain_sql_idx'::regclass) \
             ORDER BY leaf_pid LIMIT 1",
        )
        .expect("leaf pid query should succeed")
        .expect("leaf pid should exist");

        unsafe { am::debug_spire_rewrite_placement_node(index_oid, selected_pid as u64, 2) };
        Spi::run("SET enable_seqscan = off").expect("disable seqscan should succeed");
        Spi::run("SET enable_indexscan = off").expect("disable indexscan should succeed");

        let plan = Spi::connect(|client| {
            let rows = client
                .select(
                    "EXPLAIN (COSTS OFF) \
                     SELECT id FROM ec_spire_customscan_explain_sql \
                     ORDER BY embedding <#> ARRAY[1.0, 0.0]::real[] \
                     LIMIT 1",
                    None,
                    &[],
                )
                .expect("EXPLAIN should succeed")
                .first();
            let mut lines = Vec::new();
            for row in rows {
                lines.push(
                    row.get::<String>(1)
                        .expect("plan row should decode")
                        .expect("plan row should not be NULL"),
                );
            }
            lines.join("\n")
        });

        assert!(
            plan.contains("Custom Scan (EcSpireDistributedScan)"),
            "expected EcSpireDistributedScan in plan:\n{plan}"
        );
    }

    #[pg_test]
    fn test_ec_spire_customscan_does_not_replace_local_only_index_plan() {
        Spi::run(
            "CREATE TABLE ec_spire_customscan_local_only_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("local-only table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_customscan_local_only_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("local-only insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_customscan_local_only_idx \
             ON ec_spire_customscan_local_only_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("local-only ec_spire index creation should succeed");
        Spi::run("SET enable_seqscan = off").expect("disable seqscan should succeed");

        let plan = Spi::connect(|client| {
            let rows = client
                .select(
                    "EXPLAIN (COSTS OFF) \
                     SELECT id FROM ec_spire_customscan_local_only_sql \
                     ORDER BY embedding <#> ARRAY[1.0, 0.0]::real[] \
                     LIMIT 1",
                    None,
                    &[],
                )
                .expect("local-only EXPLAIN should succeed")
                .first();
            let mut lines = Vec::new();
            for row in rows {
                lines.push(
                    row.get::<String>(1)
                        .expect("local-only plan row should decode")
                        .expect("local-only plan row should not be NULL"),
                );
            }
            lines.join("\n")
        });

        assert!(
            !plan.contains("Custom Scan (EcSpireDistributedScan)"),
            "local-only plan must not use EcSpireDistributedScan:\n{plan}"
        );
        assert!(
            plan.contains("Index Scan"),
            "local-only plan should preserve the ec_spire index AM path:\n{plan}"
        );
    }

    #[pg_test]
    #[should_panic(expected = "EcSpireDistributedScan production executor blocked")]
    fn test_ec_spire_customscan_exec_reaches_production_executor() {
        Spi::run(
            "CREATE TABLE ec_spire_customscan_exec_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_customscan_exec_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_customscan_exec_sql_idx \
             ON ec_spire_customscan_exec_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("ec_spire index creation should succeed");

        let index_oid =
            Spi::get_one::<pg_sys::Oid>("SELECT 'ec_spire_customscan_exec_sql_idx'::regclass::oid")
                .expect("index oid query should succeed")
                .expect("index oid should exist");
        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_customscan_exec_sql_idx'::regclass)",
        )
        .expect("active epoch query should succeed")
        .expect("active epoch should exist");
        let selected_pid = Spi::get_one::<i64>(
            "SELECT leaf_pid FROM \
             ec_spire_index_leaf_snapshot('ec_spire_customscan_exec_sql_idx'::regclass) \
             ORDER BY leaf_pid LIMIT 1",
        )
        .expect("leaf pid query should succeed")
        .expect("leaf pid should exist");

        unsafe { am::debug_spire_rewrite_placement_node(index_oid, selected_pid as u64, 2) };
        let register_result = Spi::get_one::<bool>(&format!(
            "SELECT ec_spire_register_remote_node_descriptor(\
                     '{}'::oid, 2, 1, 'spire/remote/customscan/exec', \
                     decode('0a', 'hex'), 'remote_spire_idx', 'active', \
                     {active_epoch}, {active_epoch}, '0.1.1', 'none')",
            u32::from(index_oid)
        ))
        .expect("remote descriptor registration should succeed")
        .expect("remote descriptor registration result should exist");
        assert!(register_result);

        Spi::run("SET enable_seqscan = off").expect("disable seqscan should succeed");
        Spi::run("SET enable_indexscan = off").expect("disable indexscan should succeed");
        Spi::run(
            "SELECT id FROM ec_spire_customscan_exec_sql \
             ORDER BY embedding <#> ARRAY[1.0, 0.0]::real[] \
             LIMIT 1",
        )
        .expect("CustomScan execution should fail in production executor, not scaffold");
    }

    #[pg_test]
    #[should_panic(expected = "EcSpireDistributedScan production executor blocked")]
    fn test_ec_spire_customscan_exec_accepts_parameter_query() {
        Spi::run(
            "CREATE TABLE ec_spire_customscan_param_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_customscan_param_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_customscan_param_sql_idx \
             ON ec_spire_customscan_param_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("ec_spire index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_spire_customscan_param_sql_idx'::regclass::oid",
        )
        .expect("index oid query should succeed")
        .expect("index oid should exist");
        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_customscan_param_sql_idx'::regclass)",
        )
        .expect("active epoch query should succeed")
        .expect("active epoch should exist");
        let selected_pid = Spi::get_one::<i64>(
            "SELECT leaf_pid FROM \
             ec_spire_index_leaf_snapshot('ec_spire_customscan_param_sql_idx'::regclass) \
             ORDER BY leaf_pid LIMIT 1",
        )
        .expect("leaf pid query should succeed")
        .expect("leaf pid should exist");

        unsafe { am::debug_spire_rewrite_placement_node(index_oid, selected_pid as u64, 2) };
        let register_result = Spi::get_one::<bool>(&format!(
            "SELECT ec_spire_register_remote_node_descriptor(\
                     '{}'::oid, 2, 1, 'spire/remote/customscan/param', \
                     decode('0a', 'hex'), 'remote_spire_idx', 'active', \
                     {active_epoch}, {active_epoch}, '0.1.1', 'none')",
            u32::from(index_oid)
        ))
        .expect("remote descriptor registration should succeed")
        .expect("remote descriptor registration result should exist");
        assert!(register_result);

        Spi::run("SET enable_seqscan = off").expect("disable seqscan should succeed");
        Spi::run("SET enable_indexscan = off").expect("disable indexscan should succeed");
        Spi::run(
            "PREPARE ec_spire_customscan_param_query(real[]) AS \
             SELECT id FROM ec_spire_customscan_param_sql \
             ORDER BY embedding <#> $1 \
             LIMIT 1",
        )
        .expect("prepare should succeed");
        Spi::run("EXECUTE ec_spire_customscan_param_query(ARRAY[1.0, 0.0]::real[])")
            .expect("parameterized CustomScan execution should fail in production executor");
    }

    #[pg_test]
    #[should_panic(
        expected = "EcSpireDistributedScan tuple payload column \"tags\" has unsupported array type"
    )]
    fn test_ec_spire_customscan_rejects_array_tuple_payload_projection() {
        Spi::run(
            "CREATE TABLE ec_spire_customscan_array_payload_sql \
             (id bigint primary key, tags text[] not null, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_customscan_array_payload_sql (id, tags, embedding) VALUES \
             (1, ARRAY['alpha','beta'], encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, ARRAY['gamma'], encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_customscan_array_payload_sql_idx \
             ON ec_spire_customscan_array_payload_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("ec_spire index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_spire_customscan_array_payload_sql_idx'::regclass::oid",
        )
        .expect("index oid query should succeed")
        .expect("index oid should exist");
        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_customscan_array_payload_sql_idx'::regclass)",
        )
        .expect("active epoch query should succeed")
        .expect("active epoch should exist");
        let selected_pid = Spi::get_one::<i64>(
            "SELECT leaf_pid FROM \
             ec_spire_index_leaf_snapshot('ec_spire_customscan_array_payload_sql_idx'::regclass) \
             ORDER BY leaf_pid LIMIT 1",
        )
        .expect("leaf pid query should succeed")
        .expect("leaf pid should exist");

        unsafe { am::debug_spire_rewrite_placement_node(index_oid, selected_pid as u64, 2) };
        let register_result = Spi::get_one::<bool>(&format!(
            "SELECT ec_spire_register_remote_node_descriptor(\
                     '{}'::oid, 2, 1, 'spire/remote/customscan/array', \
                     decode('0a', 'hex'), 'remote_spire_idx', 'active', \
                     {active_epoch}, {active_epoch}, '0.1.1', 'none')",
            u32::from(index_oid)
        ))
        .expect("remote descriptor registration should succeed")
        .expect("remote descriptor registration result should exist");
        assert!(register_result);

        Spi::run("SET enable_seqscan = off").expect("disable seqscan should succeed");
        Spi::run("SET enable_indexscan = off").expect("disable indexscan should succeed");
        Spi::run(
            "SELECT tags FROM ec_spire_customscan_array_payload_sql \
             ORDER BY embedding <#> ARRAY[1.0, 0.0]::real[] \
             LIMIT 1",
        )
        .expect("CustomScan array payload projection should fail closed");
    }
