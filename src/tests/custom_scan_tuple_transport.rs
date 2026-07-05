    fn install_retired_tuple_transport_endpoint_identity(
        loopback_client: &mut postgres::Client,
        schema: &str,
    ) {
        loopback_client
            .batch_execute(&format!(
                "DROP SCHEMA IF EXISTS {schema} CASCADE; \
                 CREATE SCHEMA {schema}; \
                 CREATE FUNCTION {schema}.ec_spire_remote_search_endpoint_identity(index_oid oid) \
                 RETURNS TABLE (\
                     protocol_version text, \
                     extension_version text, \
                     opclass_identity text, \
                     storage_format text, \
                     assignment_payload_format text, \
                     quantizer_profile text, \
                     scoring_profile text, \
                     tuple_transport_capabilities text[], \
                     tuple_transport_default text, \
                     tuple_transport_status text, \
                     profile_fingerprint text, \
                     status text, \
                     recommendation text) \
                 LANGUAGE sql STABLE STRICT AS $function$ \
                     SELECT protocol_version, \
                            extension_version, \
                            opclass_identity, \
                            storage_format, \
                            assignment_payload_format, \
                            quantizer_profile, \
                            scoring_profile, \
                            ARRAY['json_tuple_payload_v1']::text[], \
                            'json_tuple_payload_v1'::text, \
                            'ready'::text, \
                            profile_fingerprint, \
                            status, \
                            recommendation \
                       FROM public.ec_spire_remote_search_endpoint_identity(index_oid) \
                 $function$"
            ))
            .expect("retired tuple transport endpoint identity should be installed");
    }

    #[pg_test]
    fn test_ec_spire_customscan_tuple_transport_retired_live_sql() {
        let _env_lock = env_var_test_lock();
        let loopback_conninfo = current_pg_test_loopback_conninfo();
        let retired_schema = "ec_spire_customscan_tuple_retired";
        let retired_conninfo =
            format!("{loopback_conninfo} options='-c search_path={retired_schema},public'");
        let _conninfo_secret = ScopedEnvVar::set(
            "EC_SPIRE_REMOTE_CONNINFO_SPIRE_REMOTE_CUSTOMSCAN_TUPLE_RETIRED",
            &retired_conninfo,
        );
        let mut loopback_client = postgres::Client::connect(&loopback_conninfo, postgres::NoTls)
            .expect("loopback retired-tuple transport connection should succeed");
        let fixture = setup_custom_scan_execution_fixture(
            &mut loopback_client,
            "ec_spire_customscan_tuple_retired",
            "(5401, 'remote tuple retired alpha', encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (5402, 'remote tuple retired beta', encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
            "(1, 'coord tuple retired alpha', encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, 'coord tuple retired beta', encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        );
        install_retired_tuple_transport_endpoint_identity(&mut loopback_client, retired_schema);
        route_custom_scan_fixture_to_remote(
            &fixture,
            2,
            1,
            "spire/remote/customscan/tuple_retired",
            "ec_spire_customscan_tuple_retired_remote_idx",
        );

        let query_sql = "SELECT id, title \
                           FROM ec_spire_customscan_tuple_retired_coord_sql \
                          ORDER BY embedding <#> ARRAY[1.0, 0.0]::real[], id \
                          LIMIT 2";
        let mut query_client = postgres::Client::connect(&loopback_conninfo, postgres::NoTls)
            .expect("retired tuple transport query connection should succeed");
        query_client
            .batch_execute(
                "SET enable_seqscan = off; \
                 SET enable_indexscan = off; \
                 SET ec_spire.remote_search_consistency_mode = 'strict'",
            )
            .expect("strict CustomScan GUCs should be set");
        let strict_error = query_client
            .query(query_sql, &[])
            .expect_err("strict CustomScan should fail closed on retired tuple transport");
        let strict_message = strict_error.to_string();
        assert!(
            strict_message.contains("tuple_transport_retired"),
            "{strict_message}"
        );

        query_client
            .batch_execute("SET ec_spire.remote_search_consistency_mode = 'degraded'")
            .expect("degraded CustomScan GUC should be set");
        let degraded_rows = query_client
            .query(query_sql, &[])
            .expect("degraded CustomScan should skip retired tuple transport dispatch");
        assert!(degraded_rows.is_empty());

        Spi::run("SET LOCAL ec_spire.remote_search_consistency_mode = 'degraded'")
            .expect("degraded skip-report GUC should be set");
        let selected_pids = fixture
            .coord_leaf_pids
            .iter()
            .map(i64::to_string)
            .collect::<Vec<_>>()
            .join(",");
        let skip_report = Spi::connect(|client| {
            let mut rows = client
                .select(
                    &format!(
                        "SELECT skipped_pid_count, first_skip_category, first_skip_hint \
                           FROM ec_spire_remote_search_degraded_skip_report(\
                                'ec_spire_customscan_tuple_retired_coord_idx'::regclass, \
                                {}, ARRAY[1.0, 0.0]::real[], ARRAY[{selected_pids}]::bigint[], \
                                2, 'degraded')",
                        fixture.active_epoch
                    ),
                    None,
                    &[],
                )
                .expect("degraded skip report should query retired tuple transport")
                .first();
            rows.next().map(|row| {
                (
                    row.get::<i64>(1)
                        .expect("skip report PID count should decode")
                        .expect("skip report PID count should not be NULL"),
                    row.get::<String>(2)
                        .expect("skip report category should decode")
                        .expect("skip report category should not be NULL"),
                    row.get::<String>(3)
                        .expect("skip report hint should decode")
                        .expect("skip report hint should not be NULL"),
                )
            })
        })
        .expect("retired tuple transport skip report should return one row");
        assert_eq!(
            skip_report.0,
            i64::try_from(fixture.coord_leaf_pids.len())
                .expect("fixture PID count should fit in i64")
        );
        assert_eq!(skip_report.1, "tuple_transport_retired");
        assert!(
            skip_report.2.contains("pg_binary_attr_v1"),
            "{}",
            skip_report.2
        );
    }
