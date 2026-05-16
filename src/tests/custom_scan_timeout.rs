    fn install_slow_remote_search(
        loopback_client: &mut postgres::Client,
        schema: &str,
    ) {
        loopback_client
            .batch_execute(&format!(
                "DROP SCHEMA IF EXISTS {schema} CASCADE; \
                 CREATE SCHEMA {schema}; \
                 CREATE FUNCTION {schema}.ec_spire_remote_search(\
                     index_oid oid, \
                     requested_epoch bigint, \
                     query real[], \
                     selected_pids bigint[], \
                     top_k integer, \
                     consistency_mode text) \
                 RETURNS TABLE (\
                     served_epoch bigint, \
                     node_id bigint, \
                     pid bigint, \
                     object_version bigint, \
                     row_index bigint, \
                     assignment_flags smallint, \
                     vec_id bytea, \
                     row_locator bytea, \
                     score real, \
                     protocol_version text, \
                     extension_version text, \
                     opclass_identity text, \
                     storage_format text, \
                     assignment_payload_format text, \
                     quantizer_profile text, \
                     scoring_profile text, \
                     profile_fingerprint text, \
                     endpoint_status text) \
                 LANGUAGE sql STABLE STRICT AS $function$ \
                     SELECT remote_rows.* \
                       FROM pg_sleep(0.30), \
                            public.ec_spire_remote_search(\
                                index_oid, requested_epoch, query, selected_pids, \
                                top_k, consistency_mode) AS remote_rows \
                 $function$"
            ))
            .expect("slow remote search shim should be installed");
    }

    #[pg_test]
    fn test_ec_spire_customscan_local_statement_timeout_sql() {
        let _env_lock = env_var_test_lock();
        let loopback_conninfo = current_pg_test_loopback_conninfo();
        let timeout_schema = "ec_spire_customscan_local_timeout";
        let slow_conninfo =
            format!("{loopback_conninfo} options='-c search_path={timeout_schema},public'");
        let _conninfo_secret = ScopedEnvVar::set(
            "EC_SPIRE_REMOTE_CONNINFO_SPIRE_REMOTE_CUSTOMSCAN_LOCAL_TIMEOUT",
            &slow_conninfo,
        );
        let mut loopback_client = postgres::Client::connect(&loopback_conninfo, postgres::NoTls)
            .expect("loopback local-timeout connection should succeed");
        let fixture = setup_custom_scan_execution_fixture(
            &mut loopback_client,
            "ec_spire_customscan_local_timeout",
            "(5501, 'remote timeout alpha', encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (5502, 'remote timeout beta', encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
            "(1, 'coord timeout alpha', encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, 'coord timeout beta', encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        );
        install_slow_remote_search(&mut loopback_client, timeout_schema);
        route_custom_scan_fixture_to_remote(
            &fixture,
            2,
            1,
            "spire/remote/customscan/local_timeout",
            "ec_spire_customscan_local_timeout_remote_idx",
        );

        let query_sql = "SELECT id, title \
                           FROM ec_spire_customscan_local_timeout_coord_sql \
                          ORDER BY embedding <#> ARRAY[1.0, 0.0]::real[], id \
                          LIMIT 1";
        Spi::run("SET LOCAL enable_seqscan = off").expect("disable seqscan should succeed");
        Spi::run("SET LOCAL enable_indexscan = off").expect("disable indexscan should succeed");
        Spi::run("SET LOCAL statement_timeout = '20ms'")
            .expect("low local statement timeout should be set");
        let timeout_error = pg_sys::PgTryBuilder::new(|| {
            Spi::run(query_sql)
                .expect("CustomScan should be interrupted by local statement timeout");
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
            timeout_error.contains("statement timeout"),
            "{timeout_error}"
        );

        Spi::run("SET LOCAL statement_timeout = 0")
            .expect("statement timeout reset should succeed");
        std::env::set_var(
            "EC_SPIRE_REMOTE_CONNINFO_SPIRE_REMOTE_CUSTOMSCAN_LOCAL_TIMEOUT",
            &loopback_conninfo,
        );
        let rejoined_row = Spi::get_one::<String>(
            "SELECT id::text || ':' || title \
               FROM ec_spire_customscan_local_timeout_coord_sql \
              ORDER BY embedding <#> ARRAY[1.0, 0.0]::real[], id \
              LIMIT 1",
        )
        .expect("CustomScan should succeed after clearing local timeout")
        .expect("CustomScan should return a row after timeout cleanup");
        assert_eq!(rejoined_row, "5501:remote timeout alpha");
    }
