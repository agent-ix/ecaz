    struct CustomScanExecutionFixture {
        index_oid: pg_sys::Oid,
        active_epoch: i64,
        coord_leaf_pids: Vec<i64>,
        remote_identity_hex: String,
    }

    fn setup_custom_scan_execution_fixture(
        loopback_client: &mut postgres::Client,
        prefix: &str,
        remote_rows: &str,
        coord_rows: &str,
    ) -> CustomScanExecutionFixture {
        loopback_client
            .batch_execute(&format!(
                "DROP TABLE IF EXISTS {prefix}_remote_sql; \
                 CREATE TABLE {prefix}_remote_sql \
                     (id bigint primary key, title text not null, embedding ecvector); \
                 INSERT INTO {prefix}_remote_sql (id, title, embedding) VALUES {remote_rows}; \
                 CREATE INDEX {prefix}_remote_idx \
                     ON {prefix}_remote_sql USING ec_spire \
                     (embedding ecvector_spire_ip_ops) \
                     WITH (nlists = 2, nprobe = 2, storage_format = 'rabitq')"
            ))
            .expect("loopback CustomScan execution fixture should be created");

        Spi::run(&format!(
            "CREATE TABLE {prefix}_coord_sql \
                 (id bigint primary key, title text not null, embedding ecvector); \
             INSERT INTO {prefix}_coord_sql (id, title, embedding) VALUES {coord_rows}; \
             CREATE INDEX {prefix}_coord_idx \
                 ON {prefix}_coord_sql USING ec_spire \
                 (embedding ecvector_spire_ip_ops) \
                 WITH (nlists = 2, nprobe = 2, storage_format = 'rabitq')"
        ))
        .expect("coordinator CustomScan execution fixture should be created");

        let index_oid =
            Spi::get_one::<pg_sys::Oid>(&format!("SELECT '{prefix}_coord_idx'::regclass::oid"))
                .expect("coordinator execution index oid query should succeed")
                .expect("coordinator execution index oid should exist");
        let active_epoch = Spi::get_one::<i64>(&format!(
            "SELECT active_epoch FROM ec_spire_index_hierarchy_snapshot('{prefix}_coord_idx'::regclass)"
        ))
        .expect("coordinator execution active epoch query should succeed")
        .expect("coordinator execution active epoch should exist");
        let coord_leaf_pids = Spi::get_one::<Vec<i64>>(&format!(
            "SELECT array_agg(leaf_pid ORDER BY leaf_pid) \
               FROM ec_spire_index_leaf_snapshot('{prefix}_coord_idx'::regclass)"
        ))
        .expect("coordinator execution leaf pid query should succeed")
        .expect("coordinator execution leaf pids should exist");
        let remote_leaf_pids = loopback_client
            .query_one(
                &format!(
                    "SELECT array_agg(leaf_pid ORDER BY leaf_pid) \
                       FROM ec_spire_index_leaf_snapshot('{prefix}_remote_idx'::regclass)"
                ),
                &[],
            )
            .expect("remote execution leaf pid query should succeed")
            .try_get::<_, Vec<i64>>(0)
            .expect("remote execution leaf pids should decode");
        assert_eq!(remote_leaf_pids, coord_leaf_pids);

        CustomScanExecutionFixture {
            index_oid,
            active_epoch,
            coord_leaf_pids,
            remote_identity_hex: loopback_remote_index_identity_hex(
                loopback_client,
                &format!("{prefix}_remote_idx"),
            ),
        }
    }

    fn route_custom_scan_fixture_to_remote(
        fixture: &CustomScanExecutionFixture,
        node_id: u32,
        descriptor_generation: i64,
        descriptor_label: &str,
        remote_index_regclass: &str,
    ) {
        unsafe {
            for pid in &fixture.coord_leaf_pids {
                am::debug_spire_rewrite_placement_node(fixture.index_oid, *pid as u64, node_id);
            }
        }

        let register_result = Spi::get_one::<bool>(&format!(
            "SELECT ec_spire_register_remote_node_descriptor(\
                     '{}'::oid, {node_id}, {descriptor_generation}, '{descriptor_label}', \
                     decode('{}', 'hex'), '{remote_index_regclass}', 'active', {}, {}, '{}', \
                     'none')",
            u32::from(fixture.index_oid),
            fixture.remote_identity_hex,
            fixture.active_epoch,
            fixture.active_epoch,
            env!("CARGO_PKG_VERSION")
        ))
        .expect("CustomScan execution descriptor registration should succeed")
        .expect("CustomScan execution descriptor registration result should exist");
        assert!(register_result);
    }

    fn fetch_cursor_payload_rows(
        client: &mut postgres::Client,
        sql: &str,
    ) -> Vec<(i64, String)> {
        client
            .query(sql, &[])
            .expect("cursor payload fetch should succeed")
            .into_iter()
            .map(|row| {
                (
                    row.try_get::<_, i64>(0)
                        .expect("cursor payload id should decode"),
                    row.try_get::<_, String>(1)
                        .expect("cursor payload title should decode"),
                )
            })
            .collect()
    }

    #[pg_test]
    fn test_ec_spire_customscan_cursor_move_first_rescans_sql() {
        let _env_lock = env_var_test_lock();
        let loopback_conninfo = current_pg_test_loopback_conninfo();
        let _conninfo_secret = ScopedEnvVar::set(
            "EC_SPIRE_REMOTE_CONNINFO_SPIRE_REMOTE_CUSTOMSCAN_CURSOR_RESCAN",
            &loopback_conninfo,
        );
        let mut loopback_client = postgres::Client::connect(&loopback_conninfo, postgres::NoTls)
            .expect("loopback CustomScan cursor-rescan connection should succeed");
        let fixture = setup_custom_scan_execution_fixture(
            &mut loopback_client,
            "ec_spire_customscan_cursor_rescan",
            "(5301, 'remote cursor alpha', encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (5302, 'remote cursor beta', encode_to_ecvector(ARRAY[0.7, 0.3], 4, 42)), \
             (5303, 'remote cursor gamma', encode_to_ecvector(ARRAY[0.3, 0.7], 4, 42)), \
             (5304, 'remote cursor delta', encode_to_ecvector(ARRAY[0.0, 1.0], 4, 42))",
            "(1, 'coordinator cursor alpha', encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, 'coordinator cursor beta', encode_to_ecvector(ARRAY[0.7, 0.3], 4, 42)), \
             (3, 'coordinator cursor gamma', encode_to_ecvector(ARRAY[0.3, 0.7], 4, 42)), \
             (4, 'coordinator cursor delta', encode_to_ecvector(ARRAY[0.0, 1.0], 4, 42))",
        );
        route_custom_scan_fixture_to_remote(
            &fixture,
            2,
            1,
            "spire/remote/customscan/cursor_rescan",
            "ec_spire_customscan_cursor_rescan_remote_idx",
        );

        am::custom_scan_reset_rescan_snapshot_for_test();
        let mut cursor_client = postgres::Client::connect(&loopback_conninfo, postgres::NoTls)
            .expect("cursor-rescan client connection should succeed");
        cursor_client
            .batch_execute(
                "BEGIN; \
                 SET LOCAL enable_seqscan = off; \
                 SET LOCAL enable_indexscan = off; \
                 DECLARE ec_spire_cursor_rescan_cursor SCROLL CURSOR FOR \
                     SELECT id, title \
                       FROM ec_spire_customscan_cursor_rescan_coord_sql \
                      ORDER BY embedding <#> ARRAY[1.0, 0.0]::real[], id \
                      LIMIT 4",
            )
            .expect("scroll cursor over CustomScan should open");

        let first_half = fetch_cursor_payload_rows(
            &mut cursor_client,
            "FETCH FORWARD 2 FROM ec_spire_cursor_rescan_cursor",
        );
        let first_tail = fetch_cursor_payload_rows(
            &mut cursor_client,
            "FETCH FORWARD ALL FROM ec_spire_cursor_rescan_cursor",
        );
        let mut first_pass = first_half.clone();
        first_pass.extend(first_tail);
        assert_eq!(first_half.len(), 2);
        assert_eq!(first_pass.len(), 4);

        let moved = cursor_client
            .execute("MOVE FIRST FROM ec_spire_cursor_rescan_cursor", &[])
            .expect("MOVE FIRST over CustomScan cursor should succeed")
            ;
        assert_eq!(moved, 1);
        let mut second_pass = fetch_cursor_payload_rows(
            &mut cursor_client,
            "FETCH RELATIVE 0 FROM ec_spire_cursor_rescan_cursor",
        );
        second_pass.extend(fetch_cursor_payload_rows(
            &mut cursor_client,
            "FETCH FORWARD ALL FROM ec_spire_cursor_rescan_cursor",
        ));
        cursor_client
            .batch_execute("CLOSE ec_spire_cursor_rescan_cursor; COMMIT")
            .expect("cursor-rescan transaction should close cleanly");

        assert_eq!(second_pass, first_pass);
        assert_eq!(
            first_pass,
            vec![
                (5301, "remote cursor alpha".to_owned()),
                (5302, "remote cursor beta".to_owned()),
                (5303, "remote cursor gamma".to_owned()),
                (5304, "remote cursor delta".to_owned()),
            ]
        );

        let rescan = am::custom_scan_rescan_snapshot_for_test();
        assert!(
            rescan.rescan_count >= 1,
            "MOVE FIRST should drive ReScanCustomScan at least once: {rescan:?}"
        );
        assert_eq!(rescan.outputs_len_after_reset, 0);
        assert_eq!(rescan.next_output_after_reset, 0);
        assert!(!rescan.loaded_outputs_after_reset);
    }

    #[pg_test]
    fn test_ec_spire_customscan_exec_returns_remote_tuple_payload_sql() {
        let _env_lock = env_var_test_lock();
        let loopback_conninfo = current_pg_test_loopback_conninfo();
        let _conninfo_secret = ScopedEnvVar::set(
            "EC_SPIRE_REMOTE_CONNINFO_SPIRE_REMOTE_CUSTOMSCAN_EXEC",
            &loopback_conninfo,
        );
        let mut loopback_client = postgres::Client::connect(&loopback_conninfo, postgres::NoTls)
            .expect("loopback CustomScan execution connection should succeed");
        let fixture = setup_custom_scan_execution_fixture(
            &mut loopback_client,
            "ec_spire_customscan_exec",
            "(5101, 'remote exec alpha', encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (5102, 'remote exec beta', encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
            "(1, 'coordinator exec alpha', encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, 'coordinator exec beta', encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        );
        route_custom_scan_fixture_to_remote(
            &fixture,
            2,
            1,
            "spire/remote/customscan/exec",
            "ec_spire_customscan_exec_remote_idx",
        );

        Spi::run("SET LOCAL enable_seqscan = off").expect("disable seqscan should succeed");
        Spi::run("SET LOCAL enable_indexscan = off").expect("disable indexscan should succeed");
        let plan = Spi::connect(|client| {
            let rows = client
                .select(
                    "EXPLAIN (COSTS OFF) \
                     SELECT id, title FROM ec_spire_customscan_exec_coord_sql \
                     ORDER BY embedding <#> ARRAY[1.0, 0.0]::real[], id LIMIT 1",
                    None,
                    &[],
                )
                .expect("CustomScan execution EXPLAIN should succeed")
                .first();
            rows.map(|row| {
                row.get::<String>(1)
                    .expect("CustomScan execution plan row should decode")
                    .expect("CustomScan execution plan row should not be NULL")
            })
            .collect::<Vec<_>>()
            .join("\n")
        });
        let row = Spi::get_one::<String>(
            "SELECT id::text || ':' || title \
               FROM ec_spire_customscan_exec_coord_sql \
              ORDER BY embedding <#> ARRAY[1.0, 0.0]::real[], id LIMIT 1",
        )
        .expect("CustomScan execution query should succeed")
        .expect("CustomScan execution query should return one row");

        assert!(
            plan.contains("Custom Scan (EcSpireDistributedScan)"),
            "expected production CustomScan plan:\n{plan}"
        );
        assert_eq!(row, "5101:remote exec alpha");
    }

    #[pg_test]
    fn test_ec_spire_customscan_exec_accepts_parameter_query_sql() {
        let _env_lock = env_var_test_lock();
        let loopback_conninfo = current_pg_test_loopback_conninfo();
        let _conninfo_secret = ScopedEnvVar::set(
            "EC_SPIRE_REMOTE_CONNINFO_SPIRE_REMOTE_CUSTOMSCAN_PARAM",
            &loopback_conninfo,
        );
        let mut loopback_client = postgres::Client::connect(&loopback_conninfo, postgres::NoTls)
            .expect("loopback CustomScan parameter connection should succeed");
        let fixture = setup_custom_scan_execution_fixture(
            &mut loopback_client,
            "ec_spire_customscan_param",
            "(5201, 'remote param alpha', encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (5202, 'remote param beta', encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
            "(1, 'coordinator param alpha', encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, 'coordinator param beta', encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        );
        route_custom_scan_fixture_to_remote(
            &fixture,
            2,
            1,
            "spire/remote/customscan/param",
            "ec_spire_customscan_param_remote_idx",
        );

        Spi::run("SET LOCAL enable_seqscan = off").expect("disable seqscan should succeed");
        Spi::run("SET LOCAL enable_indexscan = off").expect("disable indexscan should succeed");
        Spi::run(
            "PREPARE ec_spire_customscan_param_query(real[]) AS \
             SELECT id::text || ':' || title \
               FROM ec_spire_customscan_param_coord_sql \
              ORDER BY embedding <#> $1, id LIMIT 1",
        )
        .expect("parameterized CustomScan query should prepare");
        let plan = Spi::connect(|client| {
            let rows = client
                .select(
                    "EXPLAIN (COSTS OFF) \
                     EXECUTE ec_spire_customscan_param_query(ARRAY[1.0, 0.0]::real[])",
                    None,
                    &[],
                )
                .expect("parameterized CustomScan EXPLAIN should succeed")
                .first();
            rows.map(|row| {
                row.get::<String>(1)
                    .expect("parameterized CustomScan plan row should decode")
                    .expect("parameterized CustomScan plan row should not be NULL")
            })
            .collect::<Vec<_>>()
            .join("\n")
        });
        let row = Spi::get_one::<String>(
            "EXECUTE ec_spire_customscan_param_query(ARRAY[1.0, 0.0]::real[])",
        )
        .expect("parameterized CustomScan execution should succeed")
        .expect("parameterized CustomScan execution should return one row");

        assert!(
            plan.contains("Custom Scan (EcSpireDistributedScan)"),
            "expected parameterized production CustomScan plan:\n{plan}"
        );
        assert_eq!(row, "5201:remote param alpha");
    }
