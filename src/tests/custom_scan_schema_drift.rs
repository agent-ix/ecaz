    #[derive(Clone, Copy)]
    enum CustomScanSchemaDriftVariant {
        CoordinatorOnly,
        RemoteOnly,
        BothSides,
    }

    impl CustomScanSchemaDriftVariant {
        fn suffix(self) -> &'static str {
            match self {
                Self::CoordinatorOnly => "coord_only",
                Self::RemoteOnly => "remote_only",
                Self::BothSides => "both_sides",
            }
        }

        fn expected_message(self) -> &'static str {
            match self {
                Self::CoordinatorOnly => "coordinator side drifted",
                Self::RemoteOnly => "remote side drifted",
                Self::BothSides => "coordinator and remote schema fingerprints differ",
            }
        }
    }

    const CUSTOMSCAN_SCHEMA_DRIFT_SECRET_ENV: &str =
        "EC_SPIRE_REMOTE_CONNINFO_SPIRE_REMOTE_CUSTOMSCAN_SCHEMA_DRIFT";
    const CUSTOMSCAN_SCHEMA_DRIFT_SECRET_NAME: &str =
        "spire/remote/customscan/schema_drift";

    fn assert_ec_spire_customscan_schema_drift_variant_sql(
        loopback_client: &mut postgres::Client,
        loopback_conninfo: &str,
        variant: CustomScanSchemaDriftVariant,
        node_id: u32,
        descriptor_generation: i64,
    ) {
        let prefix = format!("ec_spire_customscan_schema_drift_{}", variant.suffix());
        let fixture = setup_custom_scan_execution_fixture(
            loopback_client,
            &prefix,
            "(9601, 'remote drift alpha', encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (9602, 'remote drift beta', encode_to_ecvector(ARRAY[0.0, 1.0], 4, 42))",
            "(1, 'coord drift alpha', encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, 'coord drift beta', encode_to_ecvector(ARRAY[0.0, 1.0], 4, 42))",
        );
        route_custom_scan_fixture_to_remote(
            &fixture,
            node_id,
            descriptor_generation,
            CUSTOMSCAN_SCHEMA_DRIFT_SECRET_NAME,
            &format!("{prefix}_remote_idx"),
        );

        let query_sql = format!(
            "SELECT id, title \
               FROM {prefix}_coord_sql \
              ORDER BY embedding <#> ARRAY[1.0, 0.0]::real[], id \
              LIMIT 2"
        );
        let selected_pids = fixture
            .coord_leaf_pids
            .iter()
            .map(i64::to_string)
            .collect::<Vec<_>>()
            .join(",");
        let mut query_client = postgres::Client::connect(loopback_conninfo, postgres::NoTls)
            .expect("READ schema-drift query connection should succeed");
        query_client
            .batch_execute(
                "SET enable_seqscan = off; \
                 SET enable_indexscan = off; \
                 SET ec_spire.remote_search_consistency_mode = 'strict'",
            )
            .expect("strict CustomScan GUCs should be set");
        let plan = query_client
            .query(&format!("EXPLAIN (COSTS OFF) {query_sql}"), &[])
            .expect("pre-drift CustomScan plan should explain")
            .into_iter()
            .map(|row| {
                row.try_get::<_, String>(0)
                    .expect("pre-drift CustomScan plan line should decode")
            })
            .collect::<Vec<_>>()
            .join("\n");
        assert!(
            plan.contains("Custom Scan (EcSpireDistributedScan)"),
            "expected READ schema-drift fixture to use EcSpireDistributedScan:\n{plan}"
        );

        match variant {
            CustomScanSchemaDriftVariant::CoordinatorOnly => loopback_client
                .batch_execute(&format!(
                    "ALTER TABLE {prefix}_coord_sql ADD COLUMN coord_only text"
                ))
                .expect("coordinator-only READ drift DDL should succeed"),
            CustomScanSchemaDriftVariant::RemoteOnly => loopback_client
                .batch_execute(&format!(
                    "ALTER TABLE {prefix}_remote_sql ADD COLUMN remote_only text"
                ))
                .expect("remote-only READ drift DDL should succeed"),
            CustomScanSchemaDriftVariant::BothSides => loopback_client
                .batch_execute(&format!(
                    "ALTER TABLE {prefix}_coord_sql ADD COLUMN coord_side text; \
                     ALTER TABLE {prefix}_remote_sql ADD COLUMN remote_side integer"
                ))
                .expect("both-sides READ drift DDL should succeed"),
        }

        let strict_error = query_client
            .query(&query_sql, &[])
            .expect_err("strict CustomScan READ should fail closed on schema drift");
        let strict_message = strict_error.to_string();
        assert!(strict_message.contains("schema_drift"), "{strict_message}");
        assert!(
            strict_message.contains(variant.expected_message()),
            "{strict_message}"
        );

        query_client
            .batch_execute("SET ec_spire.remote_search_consistency_mode = 'degraded'")
            .expect("degraded CustomScan GUC should be set");
        let degraded_rows = query_client
            .query(&query_sql, &[])
            .expect("degraded CustomScan READ should skip schema-drifted remote");
        assert!(degraded_rows.is_empty());

        let skip_report = Spi::connect(|client| {
            let mut rows = client
                .select(
                    &format!(
                        "SELECT skipped_pid_count, first_skip_category, first_skip_hint \
                           FROM ec_spire_remote_search_degraded_skip_report(\
                                '{prefix}_coord_idx'::regclass, {}, \
                                ARRAY[1.0, 0.0]::real[], ARRAY[{selected_pids}]::bigint[], \
                                2, 'degraded')",
                        fixture.active_epoch
                    ),
                    None,
                    &[],
                )
                .expect("schema drift degraded skip report should query")
                .first();
            rows.next().map(|row| {
                (
                    row.get::<i64>(1)
                        .expect("schema drift skip PID count should decode")
                        .expect("schema drift skip PID count should not be NULL"),
                    row.get::<String>(2)
                        .expect("schema drift skip category should decode")
                        .expect("schema drift skip category should not be NULL"),
                    row.get::<String>(3)
                        .expect("schema drift skip hint should decode")
                        .expect("schema drift skip hint should not be NULL"),
                )
            })
        })
        .expect("schema drift skip report should return one row");
        assert_eq!(
            skip_report.0,
            i64::try_from(fixture.coord_leaf_pids.len())
                .expect("fixture PID count should fit in i64")
        );
        assert_eq!(skip_report.1, "schema_drift");
        assert!(skip_report.2.contains("matching DDL"), "{}", skip_report.2);
    }

    #[pg_test]
    fn test_ec_spire_customscan_read_schema_drift_variants_sql() {
        let _env_lock = env_var_test_lock();
        let loopback_conninfo = current_pg_test_loopback_conninfo();
        let _conninfo_secret =
            ScopedEnvVar::set(CUSTOMSCAN_SCHEMA_DRIFT_SECRET_ENV, &loopback_conninfo);
        let mut loopback_client = postgres::Client::connect(&loopback_conninfo, postgres::NoTls)
            .expect("loopback READ schema-drift connection should succeed");
        loopback_client
            .execute(
                "SELECT tests.ec_spire_test_set_env_var($1, $2)",
                &[&CUSTOMSCAN_SCHEMA_DRIFT_SECRET_ENV, &loopback_conninfo],
            )
            .expect("loopback backend should receive READ schema-drift conninfo secret env var");

        assert_ec_spire_customscan_schema_drift_variant_sql(
            &mut loopback_client,
            &loopback_conninfo,
            CustomScanSchemaDriftVariant::CoordinatorOnly,
            96,
            61,
        );
        assert_ec_spire_customscan_schema_drift_variant_sql(
            &mut loopback_client,
            &loopback_conninfo,
            CustomScanSchemaDriftVariant::RemoteOnly,
            97,
            62,
        );
        assert_ec_spire_customscan_schema_drift_variant_sql(
            &mut loopback_client,
            &loopback_conninfo,
            CustomScanSchemaDriftVariant::BothSides,
            98,
            63,
        );
    }
