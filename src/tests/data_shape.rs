    #[pg_test]
    fn test_ec_spire_text_projection_nul_byte_rejected_sql() {
        Spi::run(
            "CREATE TABLE ec_spire_text_nul_projection_sql \
             (id bigint primary key, title text not null, embedding ecvector)",
        )
        .expect("text-NUL projection table creation should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_text_nul_projection_idx \
             ON ec_spire_text_nul_projection_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 1)",
        )
        .expect("text-NUL projection ec_spire index creation should succeed");

        let error = pg_sys::PgTryBuilder::new(|| {
            Spi::run(
                "INSERT INTO ec_spire_text_nul_projection_sql (id, title, embedding) \
                 VALUES ( \
                     1, \
                     convert_from(decode('72656d6f74650074657874', 'hex'), 'UTF8'), \
                     encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42) \
                 )",
            )
            .expect("text projection with embedded NUL should fail before CustomScan");
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
        let row_count = Spi::get_one::<i64>(
            "SELECT count(*) FROM ec_spire_text_nul_projection_sql",
        )
        .expect("text-NUL projection row count should succeed")
        .expect("text-NUL projection row count should exist");

        assert!(
            error.contains("invalid byte sequence for encoding")
                || error.contains("null character not permitted"),
            "expected PostgreSQL text NUL rejection, got: {error}"
        );
        assert_eq!(row_count, 0);
    }

    #[pg_test]
    fn test_ec_spire_customscan_wide_projection_exact_sql() {
        let _env_lock = env_var_test_lock();
        let loopback_conninfo = current_pg_test_loopback_conninfo();
        let _conninfo_secret = ScopedEnvVar::set(
            "EC_SPIRE_REMOTE_CONNINFO_SPIRE_REMOTE_WIDE_PROJECTION",
            &loopback_conninfo,
        );
        let mut loopback_client = postgres::Client::connect(&loopback_conninfo, postgres::NoTls)
            .expect("loopback client connection should succeed");

        let projection_columns = (1..=32)
            .map(|idx| format!("p{idx:02}"))
            .collect::<Vec<_>>();
        let column_defs = projection_columns
            .iter()
            .map(|column| format!("{column} text not null"))
            .collect::<Vec<_>>()
            .join(", ");
        let projection_list = projection_columns.join(", ");
        let concat_projection = std::iter::once("id::text".to_owned())
            .chain(projection_columns.iter().cloned())
            .collect::<Vec<_>>()
            .join(", ");
        let remote_first_values = (1..=32)
            .map(|idx| format!("'remote-101-{idx:02}'"))
            .collect::<Vec<_>>()
            .join(", ");
        let remote_second_values = (1..=32)
            .map(|idx| format!("'remote-102-{idx:02}'"))
            .collect::<Vec<_>>()
            .join(", ");
        let coord_first_values = (1..=32)
            .map(|idx| format!("'coord-001-{idx:02}'"))
            .collect::<Vec<_>>()
            .join(", ");
        let coord_second_values = (1..=32)
            .map(|idx| format!("'coord-002-{idx:02}'"))
            .collect::<Vec<_>>()
            .join(", ");

        loopback_client
            .batch_execute(&format!(
                "DROP TABLE IF EXISTS ec_spire_wide_projection_remote_sql; \
                 CREATE TABLE ec_spire_wide_projection_remote_sql \
                     (id bigint primary key, {column_defs}, embedding ecvector); \
                 INSERT INTO ec_spire_wide_projection_remote_sql \
                     (id, {projection_list}, embedding) VALUES \
                     (101, {remote_first_values}, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
                     (102, {remote_second_values}, encode_to_ecvector(ARRAY[0.5, 0.5], 4, 42)); \
                 CREATE INDEX ec_spire_wide_projection_remote_idx \
                     ON ec_spire_wide_projection_remote_sql USING ec_spire \
                     (embedding ecvector_spire_ip_ops) \
                     WITH (nlists = 2, nprobe = 2, storage_format = 'rabitq')"
            ))
            .expect("loopback wide projection fixture should be created");

        let remote_active_epoch = loopback_client
            .query_one(
                "SELECT active_epoch FROM \
                 ec_spire_index_hierarchy_snapshot('ec_spire_wide_projection_remote_idx'::regclass)",
                &[],
            )
            .expect("remote active epoch query should succeed")
            .try_get::<_, i64>(0)
            .expect("remote active epoch should decode");
        let remote_leaf_pids = loopback_client
            .query_one(
                "SELECT array_agg(leaf_pid ORDER BY leaf_pid) FROM \
                 ec_spire_index_leaf_snapshot('ec_spire_wide_projection_remote_idx'::regclass)",
                &[],
            )
            .expect("remote leaf pid query should succeed")
            .try_get::<_, Vec<i64>>(0)
            .expect("remote leaf pids should decode");
        let remote_identity_hex = loopback_remote_index_identity_hex(
            &mut loopback_client,
            "ec_spire_wide_projection_remote_idx",
        );

        Spi::run(&format!(
            "CREATE TABLE ec_spire_wide_projection_coord_sql \
             (id bigint primary key, {column_defs}, embedding ecvector)"
        ))
        .expect("coordinator wide projection table creation should succeed");
        Spi::run(&format!(
            "INSERT INTO ec_spire_wide_projection_coord_sql \
             (id, {projection_list}, embedding) VALUES \
             (1, {coord_first_values}, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, {coord_second_values}, encode_to_ecvector(ARRAY[0.5, 0.5], 4, 42))"
        ))
        .expect("coordinator wide projection insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_wide_projection_coord_idx \
             ON ec_spire_wide_projection_coord_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) \
             WITH (nlists = 2, nprobe = 2, storage_format = 'rabitq')",
        )
        .expect("coordinator wide projection index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_spire_wide_projection_coord_idx'::regclass::oid",
        )
        .expect("coordinator index oid query should succeed")
        .expect("coordinator index oid should exist");
        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_wide_projection_coord_idx'::regclass)",
        )
        .expect("coordinator active epoch query should succeed")
        .expect("coordinator active epoch should exist");
        let coord_leaf_pids = Spi::get_one::<Vec<i64>>(
            "SELECT array_agg(leaf_pid ORDER BY leaf_pid) FROM \
             ec_spire_index_leaf_snapshot('ec_spire_wide_projection_coord_idx'::regclass)",
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
                     '{}'::oid, 2, 103, 'spire/remote/customscan/wide_projection', \
                     decode('{remote_identity_hex}', 'hex'), \
                     'ec_spire_wide_projection_remote_idx', 'active', {active_epoch}, \
                     {active_epoch}, '{}', 'none')",
            u32::from(index_oid),
            env!("CARGO_PKG_VERSION")
        ))
        .expect("remote descriptor registration should succeed")
        .expect("remote descriptor registration result should exist");
        assert!(register_result);

        Spi::run("SET LOCAL enable_seqscan = off").expect("disable seqscan should succeed");
        Spi::run("SET LOCAL enable_indexscan = off").expect("disable indexscan should succeed");

        let plan = Spi::connect(|client| {
            let rows = client
                .select(
                    &format!(
                        "EXPLAIN (COSTS OFF) \
                         SELECT concat_ws('|', {concat_projection}) \
                           FROM ec_spire_wide_projection_coord_sql \
                          ORDER BY embedding <#> ARRAY[1.0, 0.0]::real[], id \
                          LIMIT 2"
                    ),
                    None,
                    &[],
                )
                .expect("wide projection CustomScan explain should succeed");
            rows.into_iter()
                .map(|row| {
                    row.get::<String>(1)
                        .expect("wide projection explain row should decode")
                        .expect("wide projection explain row should not be NULL")
                })
                .collect::<Vec<_>>()
                .join("\n")
        });
        assert!(
            plan.contains("Custom Scan (EcSpireDistributedScan)"),
            "expected wide projection fixture to use EcSpireDistributedScan:\n{plan}"
        );

        let expected_rows = loopback_client
            .query(
                &format!(
                    "SELECT concat_ws('|', {concat_projection}) AS payload_row \
                       FROM ec_spire_wide_projection_remote_sql \
                      ORDER BY embedding <#> ARRAY[1.0, 0.0]::real[], id \
                      LIMIT 2"
                ),
                &[],
            )
            .expect("remote exact wide projection query should succeed")
            .into_iter()
            .map(|row| {
                row.try_get::<_, String>("payload_row")
                    .expect("remote wide projection row should decode")
            })
            .collect::<Vec<_>>();
        let custom_scan_rows = Spi::connect(|client| {
            client
                .select(
                    &format!(
                        "SELECT concat_ws('|', {concat_projection}) \
                           FROM ec_spire_wide_projection_coord_sql \
                          ORDER BY embedding <#> ARRAY[1.0, 0.0]::real[], id \
                          LIMIT 2"
                    ),
                    None,
                    &[],
                )
                .expect("wide projection CustomScan query should succeed")
                .into_iter()
                .map(|row| {
                    row.get::<String>(1)
                        .expect("wide projection CustomScan row should decode")
                        .expect("wide projection CustomScan row should exist")
                })
                .collect::<Vec<_>>()
        });

        assert_eq!(custom_scan_rows, expected_rows);
    }

    #[pg_test]
    fn test_ec_spire_customscan_large_text_projection_cap_sql() {
        let _env_lock = env_var_test_lock();
        let loopback_conninfo = current_pg_test_loopback_conninfo();
        let _conninfo_secret = ScopedEnvVar::set(
            "EC_SPIRE_REMOTE_CONNINFO_SPIRE_REMOTE_LARGE_TEXT_PROJECTION",
            &loopback_conninfo,
        );
        let mut loopback_client = postgres::Client::connect(&loopback_conninfo, postgres::NoTls)
            .expect("loopback client connection should succeed");

        loopback_client
            .batch_execute(
                "DROP TABLE IF EXISTS ec_spire_large_text_projection_remote_sql; \
                 CREATE TABLE ec_spire_large_text_projection_remote_sql \
                     (id bigint primary key, body text not null, embedding ecvector); \
                 INSERT INTO ec_spire_large_text_projection_remote_sql \
                     (id, body, embedding) VALUES \
                     (101, repeat('x', 1048576), encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
                     (102, repeat('y', 64), encode_to_ecvector(ARRAY[0.5, 0.5], 4, 42)), \
                     (103, repeat('z', 1048577), encode_to_ecvector(ARRAY[0.0, 1.0], 4, 42)); \
                 CREATE INDEX ec_spire_large_text_projection_remote_idx \
                     ON ec_spire_large_text_projection_remote_sql USING ec_spire \
                     (embedding ecvector_spire_ip_ops) \
                     WITH (nlists = 3, nprobe = 3, storage_format = 'rabitq')",
            )
            .expect("loopback large text projection remote fixture should be created");

        let remote_active_epoch = loopback_client
            .query_one(
                "SELECT active_epoch FROM \
                 ec_spire_index_hierarchy_snapshot('ec_spire_large_text_projection_remote_idx'::regclass)",
                &[],
            )
            .expect("remote large text active epoch query should succeed")
            .try_get::<_, i64>(0)
            .expect("remote large text active epoch should decode");
        let remote_leaf_pids = loopback_client
            .query_one(
                "SELECT array_agg(leaf_pid ORDER BY leaf_pid) FROM \
                 ec_spire_index_leaf_snapshot('ec_spire_large_text_projection_remote_idx'::regclass)",
                &[],
            )
            .expect("remote large text leaf pid query should succeed")
            .try_get::<_, Vec<i64>>(0)
            .expect("remote large text leaf pids should decode");
        let remote_identity_hex = loopback_remote_index_identity_hex(
            &mut loopback_client,
            "ec_spire_large_text_projection_remote_idx",
        );

        Spi::run(
            "CREATE TABLE ec_spire_large_text_projection_coord_sql \
             (id bigint primary key, body text not null, embedding ecvector)",
        )
        .expect("coordinator large text projection table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_large_text_projection_coord_sql \
             (id, body, embedding) VALUES \
             (1, repeat('coord-x', 16), encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, repeat('coord-y', 16), encode_to_ecvector(ARRAY[0.5, 0.5], 4, 42)), \
             (3, repeat('coord-z', 16), encode_to_ecvector(ARRAY[0.0, 1.0], 4, 42))",
        )
        .expect("coordinator large text projection insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_large_text_projection_coord_idx \
             ON ec_spire_large_text_projection_coord_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) \
             WITH (nlists = 3, nprobe = 3, storage_format = 'rabitq')",
        )
        .expect("coordinator large text projection index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_spire_large_text_projection_coord_idx'::regclass::oid",
        )
        .expect("coordinator large text index oid query should succeed")
        .expect("coordinator large text index oid should exist");
        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_large_text_projection_coord_idx'::regclass)",
        )
        .expect("coordinator large text active epoch query should succeed")
        .expect("coordinator large text active epoch should exist");
        let coord_leaf_pids = Spi::get_one::<Vec<i64>>(
            "SELECT array_agg(leaf_pid ORDER BY leaf_pid) FROM \
             ec_spire_index_leaf_snapshot('ec_spire_large_text_projection_coord_idx'::regclass)",
        )
        .expect("coordinator large text leaf pid query should succeed")
        .expect("coordinator large text leaf pids should exist");
        assert_eq!(remote_active_epoch, active_epoch);
        assert_eq!(remote_leaf_pids, coord_leaf_pids);

        unsafe {
            for pid in &coord_leaf_pids {
                am::debug_spire_rewrite_placement_node(index_oid, *pid as u64, 2);
            }
        }
        let register_result = Spi::get_one::<bool>(&format!(
            "SELECT ec_spire_register_remote_node_descriptor(\
                     '{}'::oid, 2, 107, 'spire/remote/customscan/large_text_projection', \
                     decode('{remote_identity_hex}', 'hex'), \
                     'ec_spire_large_text_projection_remote_idx', 'active', {active_epoch}, \
                     {active_epoch}, '{}', 'none')",
            u32::from(index_oid),
            env!("CARGO_PKG_VERSION")
        ))
        .expect("large text remote descriptor registration should succeed")
        .expect("large text remote descriptor registration result should exist");
        assert!(register_result);

        Spi::run("SET LOCAL enable_seqscan = off").expect("disable seqscan should succeed");
        Spi::run("SET LOCAL enable_indexscan = off").expect("disable indexscan should succeed");
        Spi::run("SET LOCAL ec_spire.max_remote_payload_bytes_per_row = 1049000")
            .expect("raise remote payload cap for 1 MiB text should succeed");

        let plan = Spi::connect(|client| {
            let rows = client
                .select(
                    "EXPLAIN (COSTS OFF) \
                     SELECT id, octet_length(body), left(body, 1), right(body, 1) \
                       FROM ec_spire_large_text_projection_coord_sql \
                      ORDER BY embedding <#> ARRAY[1.0, 0.0]::real[], id \
                      LIMIT 1",
                    None,
                    &[],
                )
                .expect("large text CustomScan explain should succeed");
            rows.into_iter()
                .map(|row| {
                    row.get::<String>(1)
                        .expect("large text explain row should decode")
                        .expect("large text explain row should not be NULL")
                })
                .collect::<Vec<_>>()
                .join("\n")
        });
        assert!(
            plan.contains("Custom Scan (EcSpireDistributedScan)"),
            "expected large text fixture to use EcSpireDistributedScan:\n{plan}"
        );

        let success_row = Spi::connect(|client| {
            let rows = client
                .select(
                    "SELECT id::text || '|' || octet_length(body)::text || '|' || \
                            left(body, 1) || '|' || right(body, 1) \
                       FROM ec_spire_large_text_projection_coord_sql \
                      ORDER BY embedding <#> ARRAY[1.0, 0.0]::real[], id \
                      LIMIT 1",
                    None,
                    &[],
                )
                .expect("large text CustomScan query should succeed");
            rows.into_iter()
                .next()
                .expect("large text CustomScan row should exist")
                .get::<String>(1)
                .expect("large text CustomScan row should decode")
                .expect("large text CustomScan row should not be NULL")
        });
        assert_eq!(success_row, "101|1048576|x|x");

        Spi::run("SET LOCAL ec_spire.max_remote_payload_bytes_per_row = 1048576")
            .expect("lower remote payload cap should succeed");
        let cap_error = pg_sys::PgTryBuilder::new(|| {
            Spi::connect(|client| {
                client
                    .select(
                        "SELECT id, body \
                           FROM ec_spire_large_text_projection_coord_sql \
                          ORDER BY embedding <#> ARRAY[0.0, 1.0]::real[], id \
                          LIMIT 1",
                        None,
                        &[],
                    )
                    .expect("oversize large text CustomScan should trip the payload cap");
            });
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
        assert!(cap_error.contains("remote_payload_too_large"), "{cap_error}");
        assert!(
            cap_error.contains("ec_spire.max_remote_payload_bytes_per_row"),
            "{cap_error}"
        );
    }
