    fn assert_ec_spire_customscan_remote_fanout_sql(fixture: &str, node_ids: &[i32]) {
        let _env_lock = env_var_test_lock();
        let loopback_conninfo = current_pg_test_loopback_conninfo();
        let _conninfo_secret = ScopedEnvVar::set(
            "EC_SPIRE_REMOTE_CONNINFO_SPIRE_REMOTE_CUSTOMSCAN_FANOUT",
            &loopback_conninfo,
        );
        let mut loopback_client = postgres::Client::connect(&loopback_conninfo, postgres::NoTls)
            .expect("loopback client connection should succeed");

        let coord_table = format!("ec_spire_customscan_fanout_{fixture}_coord_sql");
        let coord_index = format!("ec_spire_customscan_fanout_{fixture}_coord_idx");
        let remote_tables = node_ids
            .iter()
            .map(|node_id| {
                (
                    *node_id,
                    format!("ec_spire_customscan_fanout_{fixture}_remote_{node_id}_sql"),
                    format!("ec_spire_customscan_fanout_{fixture}_remote_{node_id}_idx"),
                )
            })
            .collect::<Vec<_>>();

        let nlists = node_ids.len();
        let coord_rows = (1..=nlists)
            .map(|idx| {
                let x = idx as f32;
                let y = (nlists + 1 - idx) as f32;
                format!(
                    "({idx}, 'coord row {idx}', encode_to_ecvector(ARRAY[{x}, {y}], 4, 42))"
                )
            })
            .collect::<Vec<_>>()
            .join(", ");

        Spi::run(&format!(
            "CREATE TABLE {coord_table} \
                 (id bigint primary key, title text not null, embedding ecvector); \
             INSERT INTO {coord_table} (id, title, embedding) VALUES {coord_rows}; \
             CREATE INDEX {coord_index} ON {coord_table} USING ec_spire \
                 (embedding ecvector_spire_ip_ops) \
                 WITH (nlists = {nlists}, nprobe = {nlists}, storage_format = 'rabitq')"
        ))
        .expect("coordinator fanout fixture should be created");

        for (node_id, remote_table, remote_index) in &remote_tables {
            let remote_rows = (1..=nlists)
                .map(|idx| {
                    let x = idx as f32;
                    let y = (nlists + 1 - idx) as f32;
                    let id = i64::from(*node_id) * 1_000
                        + i64::try_from(idx).expect("fixture row index should fit i64");
                    format!(
                        "({id}, 'remote node {node_id} row {idx}', \
                         encode_to_ecvector(ARRAY[{x}, {y}], 4, 42))"
                    )
                })
                .collect::<Vec<_>>()
                .join(", ");
            loopback_client
                .batch_execute(&format!(
                    "DROP TABLE IF EXISTS {remote_table}; \
                     CREATE TABLE {remote_table} \
                         (id bigint primary key, title text not null, embedding ecvector); \
                     INSERT INTO {remote_table} (id, title, embedding) VALUES {remote_rows}; \
                     CREATE INDEX {remote_index} ON {remote_table} USING ec_spire \
                         (embedding ecvector_spire_ip_ops) \
                         WITH (nlists = {nlists}, nprobe = {nlists}, storage_format = 'rabitq')"
                ))
                .expect("loopback remote fanout fixture should be created");
        }

        let index_oid =
            Spi::get_one::<pg_sys::Oid>(&format!("SELECT '{coord_index}'::regclass::oid"))
                .expect("coordinator fanout index oid query should succeed")
                .expect("coordinator fanout index oid should exist");
        let active_epoch = Spi::get_one::<i64>(&format!(
            "SELECT active_epoch FROM ec_spire_index_hierarchy_snapshot('{coord_index}'::regclass)"
        ))
        .expect("coordinator fanout active epoch query should succeed")
        .expect("coordinator fanout active epoch should exist");
        let coord_leaf_pids = Spi::get_one::<Vec<i64>>(&format!(
            "SELECT array_agg(leaf_pid ORDER BY leaf_pid) \
               FROM ec_spire_index_leaf_snapshot('{coord_index}'::regclass)"
        ))
        .expect("coordinator fanout leaf pid query should succeed")
        .expect("coordinator fanout leaf pids should exist");
        assert_eq!(coord_leaf_pids.len(), node_ids.len());

        unsafe {
            for (pid, node_id) in coord_leaf_pids.iter().zip(node_ids.iter()) {
                let node_id =
                    u32::try_from(*node_id).expect("fixture node_id should fit u32");
                am::debug_spire_rewrite_placement_node(index_oid, *pid as u64, node_id);
            }
        }

        let mut expected_rows = std::collections::BTreeSet::new();
        for ((node_id, _remote_table, remote_index), selected_pid) in
            remote_tables.iter().zip(coord_leaf_pids.iter())
        {
            let remote_active_epoch = loopback_client
                .query_one(
                    &format!(
                        "SELECT active_epoch \
                           FROM ec_spire_index_hierarchy_snapshot('{remote_index}'::regclass)"
                    ),
                    &[],
                )
                .expect("remote fanout active epoch query should succeed")
                .try_get::<_, i64>(0)
                .expect("remote fanout active epoch should decode");
            let remote_leaf_pids = loopback_client
                .query_one(
                    &format!(
                        "SELECT array_agg(leaf_pid ORDER BY leaf_pid) \
                           FROM ec_spire_index_leaf_snapshot('{remote_index}'::regclass)"
                    ),
                    &[],
                )
                .expect("remote fanout leaf pid query should succeed")
                .try_get::<_, Vec<i64>>(0)
                .expect("remote fanout leaf pids should decode");
            assert_eq!(remote_active_epoch, active_epoch);
            assert_eq!(remote_leaf_pids, coord_leaf_pids);

            let remote_identity_hex =
                loopback_remote_index_identity_hex(&mut loopback_client, remote_index);
            let register_result = Spi::get_one::<bool>(&format!(
                "SELECT ec_spire_register_remote_node_descriptor(\
                         '{}'::oid, {node_id}, {}, \
                         'spire/remote/customscan/fanout', \
                         decode('{remote_identity_hex}', 'hex'), \
                         '{remote_index}', 'active', {active_epoch}, {active_epoch}, \
                         '{}', 'none')",
                u32::from(index_oid),
                300 + node_id,
                env!("CARGO_PKG_VERSION")
            ))
            .expect("fanout remote descriptor registration should succeed")
            .expect("fanout remote descriptor registration result should exist");
            assert!(register_result);

            let selected_pids = vec![*selected_pid];
            let payload_rows = loopback_client
                .query(
                    &format!(
                        "SELECT (tuple_payload ->> 'id')::bigint AS id, \
                                tuple_payload ->> 'title' AS title \
                           FROM ec_spire_remote_search_tuple_payload(\
                                '{remote_index}'::regclass::oid, \
                                $1::bigint, ARRAY[100.0, 0.0]::real[], \
                                $2::bigint[], 1, 'strict', ARRAY['id','title']::text[]) \
                          WHERE status = 'ready'"
                    ),
                    &[&active_epoch, &selected_pids],
                )
                .expect("remote fanout payload probe should succeed");
            assert_eq!(
                payload_rows.len(),
                1,
                "remote node {node_id} selected PID {selected_pid} should yield one payload row"
            );
            expected_rows.insert((
                payload_rows[0]
                    .try_get::<_, i64>("id")
                    .expect("remote fanout payload id should decode"),
                payload_rows[0]
                    .try_get::<_, String>("title")
                    .expect("remote fanout payload title should decode"),
            ));
        }

        Spi::run("SET LOCAL enable_seqscan = off").expect("disable seqscan should succeed");
        Spi::run("SET LOCAL enable_indexscan = off").expect("disable indexscan should succeed");

        let json_plan = Spi::connect(|client| {
            let rows = client
                .select(
                    &format!(
                        "EXPLAIN (FORMAT JSON, ANALYZE, COSTS OFF) \
                         SELECT id, title FROM {coord_table} \
                         ORDER BY embedding <#> ARRAY[100.0, 0.0]::real[], id \
                         LIMIT {nlists}"
                    ),
                    None,
                    &[],
                )
                .expect("fanout CustomScan JSON explain should succeed");
            rows.into_iter()
                .map(|row| {
                    row.get::<pgrx::datum::JsonString>(1)
                        .expect("fanout CustomScan JSON explain row should decode")
                        .expect("fanout CustomScan JSON explain row should not be NULL")
                        .0
                })
                .collect::<Vec<_>>()
                .join("\n")
        });
        let json_root_plan = custom_scan_json_explain_root_plan(&json_plan);
        let json_custom_scan_plan = custom_scan_json_explain_node(&json_root_plan, "Custom Scan");
        assert_eq!(
            json_custom_scan_plan
                .get("remote_fanout")
                .and_then(|value| value.as_u64()),
            Some(u64::try_from(node_ids.len()).expect("node count should fit u64")),
            "CustomScan JSON EXPLAIN should report one dispatch per remote node: {json_plan:?}"
        );

        let actual_rows = Spi::connect(|client| {
            let rows = client
                .select(
                    &format!(
                        "SELECT id, title FROM {coord_table} \
                         ORDER BY embedding <#> ARRAY[100.0, 0.0]::real[], id \
                         LIMIT {nlists}"
                    ),
                    None,
                    &[],
                )
                .expect("fanout CustomScan query should succeed");
            rows.into_iter()
                .map(|row| {
                    let id = row
                        .get::<i64>(1)
                        .expect("fanout CustomScan id should decode")
                        .expect("fanout CustomScan id should not be NULL");
                    let title = row
                        .get::<String>(2)
                        .expect("fanout CustomScan title should decode")
                        .expect("fanout CustomScan title should not be NULL");
                    (id, title)
                })
                .collect::<std::collections::BTreeSet<_>>()
        });
        let actual_node_ids = actual_rows
            .iter()
            .map(|(id, _title)| id / 1_000)
            .collect::<std::collections::BTreeSet<_>>();
        let expected_node_ids = node_ids
            .iter()
            .map(|node_id| i64::from(*node_id))
            .collect::<std::collections::BTreeSet<_>>();

        assert_eq!(actual_node_ids, expected_node_ids);
        assert_eq!(actual_rows, expected_rows);
    }

    #[pg_test]
    fn test_ec_spire_customscan_three_remote_fanout_sql() {
        assert_ec_spire_customscan_remote_fanout_sql("three", &[11, 12, 13]);
    }

    #[pg_test]
    fn test_ec_spire_customscan_eight_remote_fanout_sql() {
        assert_ec_spire_customscan_remote_fanout_sql("eight", &[21, 22, 23, 24, 25, 26, 27, 28]);
    }
