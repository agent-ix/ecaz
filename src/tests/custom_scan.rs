    fn custom_scan_json_explain_root_plan(json_plan: &str) -> serde_json::Value {
        let explain_rows = serde_json::from_str::<Vec<serde_json::Value>>(json_plan)
            .expect("CustomScan JSON EXPLAIN should parse");
        explain_rows
            .first()
            .and_then(|row| row.get("Plan"))
            .cloned()
            .expect("CustomScan JSON EXPLAIN should contain a root Plan")
    }

    fn custom_scan_json_explain_node<'a>(
        plan: &'a serde_json::Value,
        node_type: &str,
    ) -> &'a serde_json::Value {
        if plan
            .get("Node Type")
            .and_then(|value| value.as_str())
            .is_some_and(|value| value == node_type)
        {
            return plan;
        }
        if let Some(children) = plan.get("Plans").and_then(|value| value.as_array()) {
            for child in children {
                if let Some(found) = custom_scan_json_explain_node_optional(child, node_type) {
                    return found;
                }
            }
        }
        panic!("CustomScan JSON EXPLAIN should contain a {node_type} plan node: {plan:?}");
    }

    fn custom_scan_json_explain_node_optional<'a>(
        plan: &'a serde_json::Value,
        node_type: &str,
    ) -> Option<&'a serde_json::Value> {
        if plan
            .get("Node Type")
            .and_then(|value| value.as_str())
            .is_some_and(|value| value == node_type)
        {
            return Some(plan);
        }
        plan.get("Plans")
            .and_then(|value| value.as_array())
            .and_then(|children| {
                children
                    .iter()
                    .find_map(|child| custom_scan_json_explain_node_optional(child, node_type))
            })
    }

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
        let json_root_plan = custom_scan_json_explain_root_plan(&json_plan);
        assert_eq!(
            json_root_plan.get("Actual Rows").and_then(|value| value.as_u64()),
            Some(1),
            "CustomScan JSON EXPLAIN should pin Actual Rows to the LIMIT: {json_plan:?}"
        );
        assert_eq!(
            json_root_plan
                .get("Actual Loops")
                .and_then(|value| value.as_u64()),
            Some(1),
            "CustomScan JSON EXPLAIN should pin Actual Loops: {json_plan:?}"
        );
        assert!(
            json_root_plan
                .get("Actual Total Time")
                .and_then(|value| value.as_f64())
                .is_some_and(|value| value > 0.0),
            "CustomScan JSON EXPLAIN should include positive Actual Total Time: {json_plan:?}"
        );
        let json_custom_scan_plan = custom_scan_json_explain_node(&json_root_plan, "Custom Scan");
        // Contract with ec_spire_explain_custom_scan: these are the SPIRE-specific
        // JSON fields emitted in addition to PostgreSQL's standard Custom Scan keys.
        let spire_explain_fields = json_custom_scan_plan
            .as_object()
            .expect("CustomScan JSON EXPLAIN node should be an object")
            .keys()
            .filter(|key| {
                matches!(
                    key.as_str(),
                    "node" | "remote_fanout" | "tuple_transport_status" | "nprobe" | "rerank_width"
                )
            })
            .cloned()
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(
            spire_explain_fields,
            [
                "node".to_owned(),
                "nprobe".to_owned(),
                "remote_fanout".to_owned(),
                "rerank_width".to_owned(),
                "tuple_transport_status".to_owned(),
            ]
            .into_iter()
            .collect::<std::collections::BTreeSet<_>>(),
            "CustomScan JSON EXPLAIN should pin the SPIRE-specific field set: {json_plan:?}"
        );
        assert_eq!(
            json_custom_scan_plan
                .get("Actual Rows")
                .and_then(|value| value.as_u64()),
            Some(1),
            "CustomScan JSON EXPLAIN should pin Custom Scan Actual Rows: {json_plan:?}"
        );
        assert_eq!(
            json_custom_scan_plan
                .get("Actual Loops")
                .and_then(|value| value.as_u64()),
            Some(1),
            "CustomScan JSON EXPLAIN should pin Custom Scan Actual Loops: {json_plan:?}"
        );
        assert!(
            json_custom_scan_plan
                .get("Actual Total Time")
                .and_then(|value| value.as_f64())
                .is_some_and(|value| value > 0.0),
            "CustomScan JSON EXPLAIN should pin Custom Scan Actual Total Time: {json_plan:?}"
        );
        assert_eq!(row, (10, "remote alpha".to_owned()));
        assert_eq!(expression_row, (10, "remote alpha (boosted)".to_owned()));
        assert_eq!(embedding_projection, vec![1.0, 0.0]);
    }

    #[pg_test]
    fn test_ec_spire_customscan_selected_pid_payloads() {
        let _env_lock = env_var_test_lock();
        let loopback_conninfo = current_pg_test_loopback_conninfo();
        let _conninfo_secret = ScopedEnvVar::set(
            "EC_SPIRE_REMOTE_CONNINFO_SPIRE_REMOTE_CUSTOMSCAN_PID_PAYLOAD",
            &loopback_conninfo,
        );
        let mut loopback_client = postgres::Client::connect(&loopback_conninfo, postgres::NoTls)
            .expect("loopback client connection should succeed");
        loopback_client
            .batch_execute(
                "DROP TABLE IF EXISTS ec_spire_customscan_pid_payload_remote_sql; \
                 CREATE TABLE ec_spire_customscan_pid_payload_remote_sql \
                     (id bigint primary key, title text not null, embedding ecvector); \
                 INSERT INTO ec_spire_customscan_pid_payload_remote_sql (id, title, embedding) VALUES \
                     (101, 'remote payload 101', encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
                     (102, 'remote payload 102', encode_to_ecvector(ARRAY[0.8, 0.2], 4, 42)), \
                     (103, 'remote payload 103', encode_to_ecvector(ARRAY[0.6, 0.4], 4, 42)), \
                     (104, 'remote payload 104', encode_to_ecvector(ARRAY[0.4, 0.6], 4, 42)), \
                     (105, 'remote payload 105', encode_to_ecvector(ARRAY[0.2, 0.8], 4, 42)), \
                     (106, 'remote payload 106', encode_to_ecvector(ARRAY[0.0, 1.0], 4, 42)), \
                     (107, 'remote payload 107', encode_to_ecvector(ARRAY[-0.5, 0.5], 4, 42)), \
                     (108, 'remote payload 108', encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42)); \
                 CREATE INDEX ec_spire_customscan_pid_payload_remote_idx \
                     ON ec_spire_customscan_pid_payload_remote_sql USING ec_spire \
                     (embedding ecvector_spire_ip_ops) \
                     WITH (nlists = 4, nprobe = 4, storage_format = 'rabitq')",
            )
            .expect("loopback selected-PID payload fixture should be created");
        let remote_active_epoch = loopback_client
            .query_one(
                "SELECT active_epoch FROM \
                 ec_spire_index_hierarchy_snapshot('ec_spire_customscan_pid_payload_remote_idx'::regclass)",
                &[],
            )
            .expect("remote active epoch query should succeed")
            .try_get::<_, i64>(0)
            .expect("remote active epoch should decode");
        let remote_leaf_pids = loopback_client
            .query_one(
                "SELECT array_agg(leaf_pid ORDER BY leaf_pid) FROM \
                 ec_spire_index_leaf_snapshot('ec_spire_customscan_pid_payload_remote_idx'::regclass)",
                &[],
            )
            .expect("remote leaf pid query should succeed")
            .try_get::<_, Vec<i64>>(0)
            .expect("remote leaf pids should decode");
        let remote_identity_hex = loopback_remote_index_identity_hex(
            &mut loopback_client,
            "ec_spire_customscan_pid_payload_remote_idx",
        );

        Spi::run(
            "CREATE TABLE ec_spire_customscan_pid_payload_coord_sql \
             (id bigint primary key, title text not null, embedding ecvector)",
        )
        .expect("coordinator selected-PID payload table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_customscan_pid_payload_coord_sql (id, title, embedding) VALUES \
             (1, 'coordinator payload 1', encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, 'coordinator payload 2', encode_to_ecvector(ARRAY[0.8, 0.2], 4, 42)), \
             (3, 'coordinator payload 3', encode_to_ecvector(ARRAY[0.6, 0.4], 4, 42)), \
             (4, 'coordinator payload 4', encode_to_ecvector(ARRAY[0.4, 0.6], 4, 42)), \
             (5, 'coordinator payload 5', encode_to_ecvector(ARRAY[0.2, 0.8], 4, 42)), \
             (6, 'coordinator payload 6', encode_to_ecvector(ARRAY[0.0, 1.0], 4, 42)), \
             (7, 'coordinator payload 7', encode_to_ecvector(ARRAY[-0.5, 0.5], 4, 42)), \
             (8, 'coordinator payload 8', encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("coordinator selected-PID payload insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_customscan_pid_payload_coord_idx \
             ON ec_spire_customscan_pid_payload_coord_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) \
             WITH (nlists = 4, nprobe = 4, storage_format = 'rabitq')",
        )
        .expect("coordinator selected-PID payload index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_spire_customscan_pid_payload_coord_idx'::regclass::oid",
        )
        .expect("coordinator index oid query should succeed")
        .expect("coordinator index oid should exist");
        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_customscan_pid_payload_coord_idx'::regclass)",
        )
        .expect("coordinator active epoch query should succeed")
        .expect("coordinator active epoch should exist");
        let coord_leaf_pids = Spi::get_one::<Vec<i64>>(
            "SELECT array_agg(leaf_pid ORDER BY leaf_pid) FROM \
             ec_spire_index_leaf_snapshot('ec_spire_customscan_pid_payload_coord_idx'::regclass)",
        )
        .expect("coordinator leaf pid query should succeed")
        .expect("coordinator leaf pids should exist");
        assert_eq!(remote_active_epoch, active_epoch);
        assert_eq!(remote_leaf_pids, coord_leaf_pids);
        assert_eq!(coord_leaf_pids.len(), 4);

        unsafe {
            for pid in &coord_leaf_pids {
                am::debug_spire_rewrite_placement_node(index_oid, *pid as u64, 2);
            }
        }
        let register_result = Spi::get_one::<bool>(&format!(
            "SELECT ec_spire_register_remote_node_descriptor(\
                     '{}'::oid, 2, 97, 'spire/remote/customscan/pid_payload', \
                     decode('{remote_identity_hex}', 'hex'), \
                     'ec_spire_customscan_pid_payload_remote_idx', 'active', \
                     {active_epoch}, {active_epoch}, '{}', 'none')",
            u32::from(index_oid),
            env!("CARGO_PKG_VERSION")
        ))
        .expect("remote descriptor registration should succeed")
        .expect("remote descriptor registration result should exist");
        assert!(register_result);

        fn payload_rows_for_pids(
            client: &mut postgres::Client,
            active_epoch: i64,
            selected_pids: &[i64],
        ) -> Vec<(i64, i64, String)> {
            client
                .query(
                    "SELECT pid, (tuple_payload ->> 'id')::bigint AS id, \
                            tuple_payload ->> 'title' AS title \
                       FROM ec_spire_remote_search_tuple_payload(\
                            'ec_spire_customscan_pid_payload_remote_idx'::regclass::oid, \
                            $1::bigint, ARRAY[1.0, 0.0]::real[], $2::bigint[], \
                            8, 'strict', ARRAY['id','title']::text[]) \
                      ORDER BY pid, id",
                    &[&active_epoch, &selected_pids],
                )
                .expect("remote selected-PID tuple payload query should succeed")
                .into_iter()
                .map(|row| {
                    (
                        row.try_get::<_, i64>("pid")
                            .expect("payload pid should decode"),
                        row.try_get::<_, i64>("id")
                            .expect("payload id should decode"),
                        row.try_get::<_, String>("title")
                            .expect("payload title should decode"),
                    )
                })
                .collect()
        }

        let mut per_pid_payloads = Vec::new();
        for pid in &coord_leaf_pids {
            let rows = payload_rows_for_pids(&mut loopback_client, active_epoch, &[*pid]);
            assert!(
                !rows.is_empty(),
                "selected PID {pid} should produce at least one remote payload row"
            );
            assert!(
                rows.iter().all(|(row_pid, _, _)| row_pid == pid),
                "single-PID remote payload probe should return only PID {pid}: {rows:?}"
            );
            per_pid_payloads.extend(rows);
        }
        let all_pid_payloads =
            payload_rows_for_pids(&mut loopback_client, active_epoch, &coord_leaf_pids);
        let observed_pids = all_pid_payloads.iter().fold(Vec::new(), |mut pids, row| {
            if !pids.contains(&row.0) {
                pids.push(row.0);
            }
            pids
        });
        assert_eq!(observed_pids, coord_leaf_pids);
        assert_eq!(all_pid_payloads, per_pid_payloads);

        Spi::run("SET LOCAL enable_seqscan = off").expect("disable seqscan should succeed");
        Spi::run("SET LOCAL enable_indexscan = off").expect("disable indexscan should succeed");
        let plan = Spi::connect(|client| {
            let rows = client
                .select(
                    "EXPLAIN (COSTS OFF) \
                     SELECT id, title FROM ec_spire_customscan_pid_payload_coord_sql \
                     ORDER BY embedding <#> ARRAY[1.0, 0.0]::real[], id LIMIT 8",
                    None,
                    &[],
                )
                .expect("selected-PID CustomScan explain should succeed");
            let mut lines = Vec::new();
            for row in rows {
                lines.push(
                    row.get::<String>(1)
                        .expect("selected-PID explain row should decode")
                        .expect("selected-PID explain row should not be NULL"),
                );
            }
            lines.join("\n")
        });
        assert!(
            plan.contains("Custom Scan (EcSpireDistributedScan)"),
            "expected selected-PID fixture to use EcSpireDistributedScan:\n{plan}"
        );
        let mut custom_scan_rows = Spi::connect(|client| {
            client
                .select(
                    "SELECT id, title FROM ec_spire_customscan_pid_payload_coord_sql \
                     ORDER BY embedding <#> ARRAY[1.0, 0.0]::real[], id LIMIT 8",
                    None,
                    &[],
                )
                .expect("selected-PID CustomScan query should succeed")
                .into_iter()
                .map(|row| {
                    (
                        row.get::<i64>(1)
                            .expect("selected-PID CustomScan id should decode")
                            .expect("selected-PID CustomScan id should exist"),
                        row.get::<String>(2)
                            .expect("selected-PID CustomScan title should decode")
                            .expect("selected-PID CustomScan title should exist"),
                    )
                })
                .collect::<Vec<_>>()
        });
        custom_scan_rows.sort_unstable_by(|left, right| left.0.cmp(&right.0));
        let mut expected_payload_rows = all_pid_payloads
            .into_iter()
            .map(|(_, id, title)| (id, title))
            .collect::<Vec<_>>();
        expected_payload_rows.sort_unstable_by(|left, right| left.0.cmp(&right.0));

        assert_eq!(custom_scan_rows, expected_payload_rows);
    }

    #[pg_test]
    fn test_ec_spire_customscan_empty_remote_result_returns_no_rows() {
        let _env_lock = env_var_test_lock();
        let loopback_conninfo = current_pg_test_loopback_conninfo();
        let _conninfo_secret = ScopedEnvVar::set(
            "EC_SPIRE_REMOTE_CONNINFO_SPIRE_REMOTE_CUSTOMSCAN_EMPTY_SELECT",
            &loopback_conninfo,
        );
        let mut loopback_client = postgres::Client::connect(&loopback_conninfo, postgres::NoTls)
            .expect("loopback client connection should succeed");
        loopback_client
            .batch_execute(
                "DROP TABLE IF EXISTS ec_spire_customscan_empty_select_remote_sql; \
                 CREATE TABLE ec_spire_customscan_empty_select_remote_sql \
                     (id bigint primary key, title text not null, embedding ecvector, \
                      source_identity bytea not null); \
                 INSERT INTO ec_spire_customscan_empty_select_remote_sql \
                     (id, title, embedding, source_identity) VALUES \
                 (808, 'remote selected payload', encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42), \
                  decode('909192939495969798999a9b9c9d9e9f', 'hex')); \
                 CREATE INDEX ec_spire_customscan_empty_select_remote_idx \
                     ON ec_spire_customscan_empty_select_remote_sql USING ec_spire \
                     (embedding ecvector_spire_ip_ops);",
            )
            .expect("loopback empty-result remote select target should be created");

        Spi::run(
            "CREATE TABLE ec_spire_customscan_empty_select_coord_sql \
             (id bigint primary key, title text not null, embedding ecvector, \
              source_identity bytea not null)",
        )
        .expect("coordinator table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_customscan_empty_select_coord_sql \
                 (id, title, embedding, source_identity) VALUES \
             (1, 'coordinator seed', encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42), \
              decode('a0a1a2a3a4a5a6a7a8a9aaabacadaeaf', 'hex'))",
        )
        .expect("coordinator insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_customscan_empty_select_coord_idx \
             ON ec_spire_customscan_empty_select_coord_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops)",
        )
        .expect("coordinator index creation should succeed");
        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_customscan_empty_select_coord_idx'::regclass)",
        )
        .expect("coordinator active epoch query should succeed")
        .expect("coordinator active epoch should exist");
        let remote_identity_hex = Spi::get_one::<String>(
            "SELECT profile_fingerprint \
               FROM ec_spire_remote_search_endpoint_identity(\
                    'ec_spire_customscan_empty_select_remote_idx'::regclass::oid)",
        )
        .expect("remote identity query should succeed")
        .expect("remote identity should exist");
        let register_result = Spi::get_one::<bool>(&format!(
            "SELECT ec_spire_register_remote_node_descriptor(\
                     'ec_spire_customscan_empty_select_coord_idx'::regclass, \
                     23, 92, 'spire/remote/customscan/empty_select', \
                     decode('{remote_identity_hex}', 'hex'), \
                     'ec_spire_customscan_empty_select_remote_idx', 'active', \
                     {active_epoch}, {active_epoch}, '{}', 'none')",
            env!("CARGO_PKG_VERSION")
        ))
        .expect("remote descriptor registration should succeed")
        .expect("remote descriptor registration result should exist");
        assert!(register_result);
        Spi::run(&format!(
            "INSERT INTO ec_spire_placement \
                 (index_oid, pk_value, node_id, centroid_id, served_epoch, source_identity) \
             VALUES ('ec_spire_customscan_empty_select_coord_idx'::regclass, \
                     int8send(999::bigint)::bytea, 23, 2, {active_epoch}, \
                     decode('f0f1f2f3f4f5f6f7f8f9fafbfcfdfeff', 'hex'))"
        ))
        .expect("empty-result remote placement row should be inserted");

        let remote_select_status = Spi::get_one::<String>(
            "WITH result AS ( \
                 SELECT * FROM ec_spire_forward_coordinator_select_tuple_payload(\
                     'ec_spire_customscan_empty_select_coord_idx'::regclass, \
                     'id', \
                     int8send(999::bigint)::bytea, \
                     ARRAY['id', 'title']::text[]) \
             ) \
             SELECT remote_select_sent::text || '|' || selected_count::text || '|' || \
                    status || '|' || next_step \
               FROM result",
        )
        .expect("empty-result coordinator select helper query should succeed")
        .expect("empty-result coordinator select helper should return a row");
        assert_eq!(
            remote_select_status,
            "true|0|remote_select_ready|done"
        );

        Spi::run("SET LOCAL enable_seqscan = off").expect("disable seqscan should succeed");
        Spi::run("SET LOCAL enable_indexscan = off").expect("disable indexscan should succeed");

        let plan = Spi::connect(|client| {
            let rows = client
                .select(
                    "EXPLAIN (COSTS OFF) \
                     SELECT id, title FROM ec_spire_customscan_empty_select_coord_sql \
                     WHERE id = 999",
                    None,
                    &[],
                )
                .expect("empty-result CustomScan explain should succeed")
                .first();
            let mut lines = Vec::new();
            for row in rows {
                lines.push(
                    row.get::<String>(1)
                        .expect("empty-result plan row should decode")
                        .expect("empty-result plan row should not be NULL"),
                );
            }
            lines.join("\n")
        });
        let json_plan = Spi::connect(|client| {
            let rows = client
                .select(
                    "EXPLAIN (FORMAT JSON, ANALYZE, COSTS OFF) \
                     SELECT id, title FROM ec_spire_customscan_empty_select_coord_sql \
                     WHERE id = 999",
                    None,
                    &[],
                )
                .expect("empty-result CustomScan JSON explain should succeed");
            let mut lines = Vec::new();
            for row in rows {
                lines.push(
                    row.get::<pgrx::datum::JsonString>(1)
                        .expect("empty-result JSON explain row should decode")
                        .expect("empty-result JSON explain row should not be NULL")
                        .0,
                );
            }
            lines.join("\n")
        });
        am::custom_scan_reset_cleanup_counters_for_test();
        let row_count = Spi::get_one::<i64>(
            "SELECT count(*) FROM (\
                 SELECT id, title FROM ec_spire_customscan_empty_select_coord_sql \
                 WHERE id = 999\
             ) AS empty_remote_result",
        )
        .expect("empty-result CustomScan count query should succeed")
        .expect("empty-result CustomScan count should exist");
        let cleanup_counters = am::custom_scan_cleanup_counters_for_test();

        assert!(
            plan.contains("node: EcSpireDistributedScan"),
            "expected EcSpireDistributedScan in plan:\n{plan}"
        );
        assert_eq!(row_count, 0);
        assert_eq!(cleanup_counters.end_custom_scan_count, 1);
        assert_eq!(cleanup_counters.pfree_count, 1);
        assert!(
            json_plan.contains("\"tuple_transport_status\": \"ready\""),
            "expected ready tuple transport in CustomScan JSON plan: {json_plan:?}"
        );
        assert!(
            !json_plan.contains("not_applicable"),
            "empty remote result must not leak not_applicable status: {json_plan:?}"
        );
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

        Spi::run("SET LOCAL enable_seqscan = off").expect("disable seqscan should succeed");
        let plan = Spi::connect(|client| {
            let rows = client
                .select(
                    "EXPLAIN (COSTS OFF) \
                     SELECT id FROM ec_spire_customscan_empty_sql \
                     ORDER BY embedding <#> ARRAY[1.0, 0.0]::real[] \
                     LIMIT 1",
                    None,
                    &[],
                )
                .expect("empty active-epoch EXPLAIN should succeed")
                .first();
            let mut lines = Vec::new();
            for row in rows {
                lines.push(
                    row.get::<String>(1)
                        .expect("empty active-epoch plan row should decode")
                        .expect("empty active-epoch plan row should not be NULL"),
                );
            }
            lines.join("\n")
        });

        assert!(
            !plan.contains("Custom Scan (EcSpireDistributedScan)"),
            "no-active-epoch plan must not use EcSpireDistributedScan:\n{plan}"
        );
        assert!(
            plan.contains("Index Scan") || plan.contains("Seq Scan"),
            "no-active-epoch plan should stay on a normal local scan path:\n{plan}"
        );
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
    fn test_ec_spire_customscan_read_cancel_releases_transport() {
        let _env_lock = env_var_test_lock();
        set_remote_governance_test_namespace(6606);
        Spi::run("SET LOCAL ec_spire.remote_search_max_concurrent_dispatches = 1")
            .expect("global governance cap SET should succeed");
        Spi::run("SET LOCAL ec_spire.remote_search_max_concurrent_dispatches_per_node = 1")
            .expect("per-node governance cap SET should succeed");
        let loopback_conninfo = current_pg_test_loopback_conninfo();
        let _conninfo_secret = ScopedEnvVar::set(
            "EC_SPIRE_REMOTE_CONNINFO_SPIRE_REMOTE_CUSTOMSCAN_READ_CANCEL",
            &loopback_conninfo,
        );
        let mut loopback_client = postgres::Client::connect(&loopback_conninfo, postgres::NoTls)
            .expect("loopback client connection should succeed");
        loopback_client
            .batch_execute(
                "DROP TABLE IF EXISTS ec_spire_customscan_read_cancel_remote_sql; \
                 CREATE TABLE ec_spire_customscan_read_cancel_remote_sql \
                     (id bigint primary key, title text not null, embedding ecvector); \
                 INSERT INTO ec_spire_customscan_read_cancel_remote_sql (id, title, embedding) VALUES \
                     (10, 'remote alpha', encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
                     (20, 'remote beta', encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42)); \
                 CREATE INDEX ec_spire_customscan_read_cancel_remote_idx \
                     ON ec_spire_customscan_read_cancel_remote_sql USING ec_spire \
                     (embedding ecvector_spire_ip_ops) \
                     WITH (nlists = 2, nprobe = 2, storage_format = 'rabitq')",
            )
            .expect("loopback remote read-cancel fixture should be created");
        let remote_identity_hex = loopback_remote_index_identity_hex(
            &mut loopback_client,
            "ec_spire_customscan_read_cancel_remote_idx",
        );

        Spi::run(
            "CREATE TABLE ec_spire_customscan_read_cancel_coord_sql \
             (id bigint primary key, title text not null, embedding ecvector)",
        )
        .expect("coordinator table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_customscan_read_cancel_coord_sql (id, title, embedding) VALUES \
             (1, 'coordinator alpha', encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, 'coordinator beta', encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("coordinator insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_customscan_read_cancel_coord_idx \
             ON ec_spire_customscan_read_cancel_coord_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) \
             WITH (nlists = 2, nprobe = 2, storage_format = 'rabitq')",
        )
        .expect("coordinator index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_spire_customscan_read_cancel_coord_idx'::regclass::oid",
        )
        .expect("coordinator index oid query should succeed")
        .expect("coordinator index oid should exist");
        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_customscan_read_cancel_coord_idx'::regclass)",
        )
        .expect("coordinator active epoch query should succeed")
        .expect("coordinator active epoch should exist");
        let coord_leaf_pids = Spi::get_one::<Vec<i64>>(
            "SELECT array_agg(leaf_pid ORDER BY leaf_pid) FROM \
             ec_spire_index_leaf_snapshot('ec_spire_customscan_read_cancel_coord_idx'::regclass)",
        )
        .expect("coordinator leaf pid query should succeed")
        .expect("coordinator leaf pids should exist");
        unsafe {
            for pid in &coord_leaf_pids {
                am::debug_spire_rewrite_placement_node(index_oid, *pid as u64, 2);
            }
        }
        let register_result = Spi::get_one::<bool>(&format!(
            "SELECT ec_spire_register_remote_node_descriptor(\
                     '{}'::oid, 2, 96, 'spire/remote/customscan/read_cancel', \
                     decode('{remote_identity_hex}', 'hex'), \
                     'ec_spire_customscan_read_cancel_remote_idx', 'active', {active_epoch}, \
                     {active_epoch}, '{}', 'none')",
            u32::from(index_oid),
            env!("CARGO_PKG_VERSION")
        ))
        .expect("remote descriptor registration should succeed")
        .expect("remote descriptor registration result should exist");
        assert!(register_result);

        Spi::run("SET LOCAL enable_seqscan = off").expect("disable seqscan should succeed");
        Spi::run("SET LOCAL enable_indexscan = off").expect("disable indexscan should succeed");
        am::custom_scan_reset_cleanup_counters_for_test();
        let cancel_error = pg_sys::PgTryBuilder::new(|| {
            let _cancel_flags = unsafe { ScopedPgQueryCancelFlags::set_pending() }
                .expect("PostgreSQL query-cancel flags should resolve inside pg_test backend");
            Spi::run(
                "SELECT id, title FROM ec_spire_customscan_read_cancel_coord_sql \
                 ORDER BY embedding <#> ARRAY[1.0, 0.0]::real[] LIMIT 1",
            )
            .expect("CustomScan read path should be interrupted by local query cancel");
            "no_error".to_owned()
        })
        .catch_others(|cause| match cause {
            pg_sys::panic::CaughtError::ErrorReport(report)
            | pg_sys::panic::CaughtError::PostgresError(report) => report.message().to_owned(),
            pg_sys::panic::CaughtError::RustPanic { ereport, .. } => {
                ereport.message().to_owned()
            }
        })
        .execute();
        assert!(
            cancel_error.contains("canceling statement due to user request"),
            "{cancel_error}"
        );
        let cleanup_counters = am::custom_scan_cleanup_counters_for_test();
        assert_eq!(cleanup_counters.end_custom_scan_count, 1);
        assert_eq!(cleanup_counters.pfree_count, 1);
        let memory_snapshot = am::custom_scan_memory_context_snapshot_for_test();
        assert!(memory_snapshot.baseline_captured);
        assert!(memory_snapshot.after_end_captured);
        assert!(
            memory_snapshot.after_end_used_bytes <= memory_snapshot.baseline_used_bytes,
            "{memory_snapshot:?}"
        );
    }

    #[pg_test]
    fn test_ec_spire_customscan_returns_array_tuple_payload_projection() {
        let _env_lock = env_var_test_lock();
        let loopback_conninfo = current_pg_test_loopback_conninfo();
        let _conninfo_secret = ScopedEnvVar::set(
            "EC_SPIRE_REMOTE_CONNINFO_SPIRE_REMOTE_CUSTOMSCAN_ARRAY",
            &loopback_conninfo,
        );
        let mut loopback_client = postgres::Client::connect(&loopback_conninfo, postgres::NoTls)
            .expect("loopback client connection should succeed");
        loopback_client
            .batch_execute(
                "DROP TABLE IF EXISTS ec_spire_customscan_array_payload_remote_sql; \
                 CREATE TABLE ec_spire_customscan_array_payload_remote_sql \
                     (id bigint primary key, tags text[] not null, embedding ecvector); \
                 INSERT INTO ec_spire_customscan_array_payload_remote_sql \
                     (id, tags, embedding) VALUES \
                     (10, ARRAY['alpha','beta'], encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
                     (20, ARRAY['gamma'], encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42)); \
                 CREATE INDEX ec_spire_customscan_array_payload_remote_idx \
                     ON ec_spire_customscan_array_payload_remote_sql USING ec_spire \
                     (embedding ecvector_spire_ip_ops) \
                     WITH (nlists = 2, nprobe = 2, storage_format = 'rabitq')",
            )
            .expect("loopback remote array payload fixture should be created");
        let remote_identity_hex = loopback_remote_index_identity_hex(
            &mut loopback_client,
            "ec_spire_customscan_array_payload_remote_idx",
        );
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
                     decode('{remote_identity_hex}', 'hex'), \
                     'ec_spire_customscan_array_payload_remote_idx', 'active', \
                     {active_epoch}, {active_epoch}, '{}', \
                     'spire/remote/customscan/array')",
            u32::from(index_oid),
            env!("CARGO_PKG_VERSION")
        ))
        .expect("remote descriptor registration should succeed")
        .expect("remote descriptor registration result should exist");
        assert!(register_result);

        Spi::run("SET enable_seqscan = off").expect("disable seqscan should succeed");
        Spi::run("SET enable_indexscan = off").expect("disable indexscan should succeed");
        let tags = Spi::get_one::<Vec<String>>(
            "SELECT tags FROM ec_spire_customscan_array_payload_sql \
             ORDER BY embedding <#> ARRAY[1.0, 0.0]::real[] \
             LIMIT 1",
        )
        .expect("CustomScan array payload projection should succeed")
        .expect("CustomScan array payload projection should return one row");
        assert_eq!(tags, vec!["alpha".to_owned(), "beta".to_owned()]);
    }
