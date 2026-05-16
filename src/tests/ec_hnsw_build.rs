    #[pg_test]
    fn test_fr020_empty_index_remains_planner_gated() {
        Spi::run("CREATE TABLE ec_hnsw_empty_cost (id bigint primary key, embedding ecvector)")
            .expect("table creation should succeed");
        Spi::run(
            "CREATE INDEX ec_hnsw_empty_cost_idx ON ec_hnsw_empty_cost USING ec_hnsw \
             (embedding ecvector_ip_ops)",
        )
        .expect("empty-index creation should succeed");

        let modeled_startup = Spi::get_one::<f64>(
            "SELECT modeled_startup_cost FROM ec_hnsw_index_cost_snapshot('ec_hnsw_empty_cost_idx'::regclass)",
        )
        .expect("snapshot query should succeed")
        .expect("modeled startup should be non-null");
        let modeled_total = Spi::get_one::<f64>(
            "SELECT modeled_total_cost FROM ec_hnsw_index_cost_snapshot('ec_hnsw_empty_cost_idx'::regclass)",
        )
        .expect("snapshot query should succeed")
        .expect("modeled total should be non-null");
        assert_eq!(
            modeled_startup,
            f64::MAX,
            "empty ec_hnsw index must keep the FR-020 gate active even after D2 activation"
        );
        assert_eq!(
            modeled_total,
            f64::MAX,
            "empty ec_hnsw index must keep the FR-020 gate active even after D2 activation"
        );

        let plan = Spi::connect(|client| {
            let rows = client
                .select(
                    "EXPLAIN (COSTS OFF) \
                     SELECT id FROM ec_hnsw_empty_cost \
                     ORDER BY embedding <#> ARRAY[1.0, 0.0, 0.5, -1.0]::real[] \
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
            !plan.contains("Index Scan") && !plan.contains("Index Only Scan"),
            "planner must not pick an empty ec_hnsw index even with D2 activation: {plan}"
        );
    }

    #[pg_test]
    fn test_fr020_ac2_planner_prefers_seqscan_for_small_tables() {
        Spi::run("CREATE TABLE ec_hnsw_small_seqscan (id bigint primary key, embedding ecvector)")
            .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_hnsw_small_seqscan \
             SELECT g, encode_to_ecvector(ARRAY[g::real, (g * 0.25)::real, (g * -0.5)::real, 1.0::real], 4, 42) \
             FROM generate_series(1, 50) AS g",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_hnsw_small_seqscan_idx ON ec_hnsw_small_seqscan USING ec_hnsw \
             (embedding ecvector_ip_ops)",
        )
        .expect("index creation should succeed");
        Spi::run("ANALYZE ec_hnsw_small_seqscan").expect("analyze should succeed");

        let plan = Spi::connect(|client| {
            let rows = client
                .select(
                    "EXPLAIN (COSTS OFF) \
                     SELECT id FROM ec_hnsw_small_seqscan \
                     ORDER BY embedding <#> ARRAY[1.0, 0.0, 0.5, -1.0]::real[] \
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
            !plan.contains("Index Scan") && !plan.contains("Index Only Scan"),
            "planner should prefer sequential scan on a 50-row table even with FR-020 activated (AC-2): {plan}"
        );
    }

    #[pg_test]
    fn test_ech_planner_chooses_index_scan_for_ordered_query() {
        Spi::run("CREATE TABLE ec_hnsw_scan_plan (id bigint primary key, embedding ecvector)")
            .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_hnsw_scan_plan VALUES
             (1, encode_to_ecvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_ecvector(ARRAY[0.0, 1.0, 0.25, -0.5], 4, 42)),
             (3, encode_to_ecvector(ARRAY[-1.0, 0.5, 0.0, 1.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_hnsw_scan_plan_idx ON ec_hnsw_scan_plan USING ec_hnsw \
             (embedding ecvector_ip_ops)",
        )
        .expect("index creation should succeed");
        Spi::run("SET LOCAL enable_seqscan = off").expect("SET LOCAL should succeed");

        let plan = Spi::connect(|client| {
            let rows = client
                .select(
                    "EXPLAIN (COSTS OFF) \
                     SELECT id FROM ec_hnsw_scan_plan \
                     ORDER BY embedding <#> ARRAY[1.0, 0.0, 0.5, -1.0]::real[] \
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
            plan.contains("Index Scan") || plan.contains("Index Only Scan"),
            "planner should select the ec_hnsw index scan once FR-020 cost activation is live: {plan}"
        );
    }

    #[pg_test]
    fn test_fr020_ac1_planner_chooses_index_scan_for_large_table() {
        Spi::run("CREATE TABLE ec_hnsw_ac1_large (id bigint primary key, embedding ecvector)")
            .expect("table creation should succeed");
        // Build 10K 64-dim vectors from four MD5 digests per row so each row
        // gets 64 distinct byte values without 10K × N hashtext calls. Keeping
        // this at 64 dimensions is deliberate: lowering the dimension count
        // flips the FR-020 crossover back toward seqscan at 10K rows.
        Spi::run(
            "INSERT INTO ec_hnsw_ac1_large \
             SELECT g, encode_to_ecvector( \
                 ARRAY( \
                     SELECT ((get_byte( \
                              decode(md5(g::text) \
                                     || md5((g + 999983)::text) \
                                     || md5((g + 1999993)::text) \
                                     || md5((g + 2999999)::text), 'hex'), \
                              i)::real - 128.0) / 128.0)::real \
                     FROM generate_series(0, 63) AS i), \
                 4, 42) \
             FROM generate_series(1, 10000) AS g",
        )
        .expect("10k-row insert should succeed");
        Spi::run(
            "CREATE INDEX ec_hnsw_ac1_large_idx ON ec_hnsw_ac1_large USING ec_hnsw \
             (embedding ecvector_ip_ops)",
        )
        .expect("index creation should succeed");
        Spi::run("ANALYZE ec_hnsw_ac1_large").expect("analyze should succeed");

        let query_array = {
            let mut s = String::from("ARRAY[");
            for i in 0..64 {
                if i > 0 {
                    s.push(',');
                }
                s.push_str(&format!("{:.6}", (i as f32 * 0.05 - 1.5)));
            }
            s.push_str("]::real[]");
            s
        };
        let explain_sql = format!(
            "EXPLAIN (COSTS OFF) SELECT id FROM ec_hnsw_ac1_large \
             ORDER BY embedding <#> {query_array} LIMIT 10"
        );

        let plan = Spi::connect(|client| {
            let rows = client
                .select(&explain_sql, None, &[])
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
            plan.contains("Index Scan") && plan.contains("ec_hnsw_ac1_large_idx"),
            "FR-020-AC-1 / TC-206: planner must naturally pick the ec_hnsw index on a 10K-row table: {plan}"
        );
    }

    #[pg_test]
    fn test_ech_index_admin_snapshot_tracks_insert_drift() {
        Spi::run("CREATE TABLE ec_hnsw_admin_snapshot (id bigint primary key, embedding ecvector)")
            .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_hnsw_admin_snapshot VALUES
             (1, encode_to_ecvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_ecvector(ARRAY[0.0, 1.0, 0.25, -0.5], 4, 42)),
             (3, encode_to_ecvector(ARRAY[0.5, 0.5, -0.5, 1.0], 4, 42))",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX ec_hnsw_admin_snapshot_idx ON ec_hnsw_admin_snapshot USING ec_hnsw \
             (embedding ecvector_ip_ops) WITH (ef_search = 77)",
        )
        .expect("index creation should succeed");
        Spi::run("SET ec_hnsw.ef_search = 19").expect("set should succeed");

        assert_eq!(
            Spi::get_one::<i64>(
                "SELECT total_live_nodes FROM ec_hnsw_index_admin_snapshot('ec_hnsw_admin_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("live-node count should be non-null"),
            3
        );
        assert_eq!(
            Spi::get_one::<i64>(
                "SELECT inserted_since_rebuild FROM ec_hnsw_index_admin_snapshot('ec_hnsw_admin_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("inserted-since-rebuild should be non-null"),
            0
        );
        assert_eq!(
            Spi::get_one::<f64>(
                "SELECT insert_drift_fraction FROM ec_hnsw_index_admin_snapshot('ec_hnsw_admin_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("insert drift fraction should be non-null"),
            0.0
        );
        assert_eq!(
            Spi::get_one::<i32>(
                "SELECT relation_ef_search FROM ec_hnsw_index_admin_snapshot('ec_hnsw_admin_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("relation ef_search should be non-null"),
            77
        );
        assert_eq!(
            Spi::get_one::<i32>(
                "SELECT session_ef_search FROM ec_hnsw_index_admin_snapshot('ec_hnsw_admin_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed"),
            Some(19)
        );
        assert_eq!(
            Spi::get_one::<String>(
                "SELECT effective_source FROM ec_hnsw_index_admin_snapshot('ec_hnsw_admin_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("effective source should be non-null"),
            "session"
        );

        Spi::run(
            "INSERT INTO ec_hnsw_admin_snapshot VALUES
             (4, encode_to_ecvector(ARRAY[0.9, 0.1, 0.25, -0.9], 4, 42))",
        )
        .expect("live insert should succeed");

        assert_eq!(
            Spi::get_one::<i64>(
                "SELECT total_live_nodes FROM ec_hnsw_index_admin_snapshot('ec_hnsw_admin_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("live-node count should be non-null"),
            4
        );
        assert_eq!(
            Spi::get_one::<i64>(
                "SELECT inserted_since_rebuild FROM ec_hnsw_index_admin_snapshot('ec_hnsw_admin_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("inserted-since-rebuild should be non-null"),
            1
        );
        assert!(
            (Spi::get_one::<f64>(
                "SELECT insert_drift_fraction FROM ec_hnsw_index_admin_snapshot('ec_hnsw_admin_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("insert drift fraction should be non-null")
                - 0.25)
                .abs()
                < 1e-9,
            "one live insert after a three-row build should report 25% drift",
        );

        Spi::run(
            "INSERT INTO ec_hnsw_admin_snapshot VALUES
             (5, encode_to_ecvector(ARRAY[0.9, 0.1, 0.25, -0.9], 4, 42))",
        )
        .expect("duplicate insert should succeed");

        assert_eq!(
            Spi::get_one::<i64>(
                "SELECT total_live_nodes FROM ec_hnsw_index_admin_snapshot('ec_hnsw_admin_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("live-node count should be non-null"),
            4,
            "duplicate coalescing should not create a new live node",
        );
        assert_eq!(
            Spi::get_one::<i64>(
                "SELECT inserted_since_rebuild FROM ec_hnsw_index_admin_snapshot('ec_hnsw_admin_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("inserted-since-rebuild should be non-null"),
            1,
            "duplicate coalescing should not advance the insert-drift counter",
        );

        Spi::run("RESET ec_hnsw.ef_search").expect("reset should succeed");
    }

    #[pg_test]
    fn test_ech_index_admin_snapshot_counts_empty_first_insert() {
        Spi::run(
            "CREATE TABLE ec_hnsw_admin_snapshot_empty (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "CREATE INDEX ec_hnsw_admin_snapshot_empty_idx ON ec_hnsw_admin_snapshot_empty USING ec_hnsw \
             (embedding ecvector_ip_ops)",
        )
        .expect("index creation should succeed");

        assert_eq!(
            Spi::get_one::<i64>(
                "SELECT total_live_nodes FROM ec_hnsw_index_admin_snapshot('ec_hnsw_admin_snapshot_empty_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("live-node count should be non-null"),
            0
        );
        assert_eq!(
            Spi::get_one::<i64>(
                "SELECT inserted_since_rebuild FROM ec_hnsw_index_admin_snapshot('ec_hnsw_admin_snapshot_empty_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("inserted-since-rebuild should be non-null"),
            0
        );

        Spi::run(
            "INSERT INTO ec_hnsw_admin_snapshot_empty VALUES
             (1, encode_to_ecvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42))",
        )
        .expect("first insert should succeed");

        assert_eq!(
            Spi::get_one::<i64>(
                "SELECT total_live_nodes FROM ec_hnsw_index_admin_snapshot('ec_hnsw_admin_snapshot_empty_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("live-node count should be non-null"),
            1
        );
        assert_eq!(
            Spi::get_one::<i64>(
                "SELECT inserted_since_rebuild FROM ec_hnsw_index_admin_snapshot('ec_hnsw_admin_snapshot_empty_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("inserted-since-rebuild should be non-null"),
            1,
            "the first successful live insert should start the post-build drift counter",
        );
        assert_eq!(
            Spi::get_one::<f64>(
                "SELECT insert_drift_fraction FROM ec_hnsw_index_admin_snapshot('ec_hnsw_admin_snapshot_empty_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("insert drift fraction should be non-null"),
            1.0
        );
    }

    #[pg_test]
    #[should_panic(expected = "ec_hnsw_index_admin_snapshot requires a ec_hnsw index")]
    fn test_ech_index_admin_snapshot_rejects_non_ec_hnsw_index() {
        Spi::run(
            "CREATE TABLE ec_hnsw_admin_snapshot_wrong_am (id bigint primary key, value bigint)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "CREATE INDEX ec_hnsw_admin_snapshot_wrong_am_idx ON ec_hnsw_admin_snapshot_wrong_am USING btree (value)",
        )
        .expect("index creation should succeed");

        let _ = Spi::get_one::<i64>(
            "SELECT total_live_nodes FROM ec_hnsw_index_admin_snapshot('ec_hnsw_admin_snapshot_wrong_am_idx'::regclass)",
        );
    }

    #[pg_test]
    fn test_ech_index_cost_snapshot_reports_modeled_and_gated_costs() {
        Spi::run("CREATE TABLE ec_hnsw_cost_snapshot (id bigint primary key, embedding ecvector)")
            .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_hnsw_cost_snapshot VALUES
             (1, encode_to_ecvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_ecvector(ARRAY[0.0, 1.0, 0.25, -0.5], 4, 42)),
             (3, encode_to_ecvector(ARRAY[0.5, 0.5, -0.5, 1.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_hnsw_cost_snapshot_idx ON ec_hnsw_cost_snapshot USING ec_hnsw \
             (embedding ecvector_ip_ops) WITH (m = 12, ef_search = 77)",
        )
        .expect("index creation should succeed");
        Spi::run("SET ec_hnsw.ef_search = 19").expect("set should succeed");
        Spi::run("ANALYZE ec_hnsw_cost_snapshot").expect("analyze should succeed");

        assert!(
            Spi::get_one::<bool>(
                "SELECT planner_scan_enabled FROM ec_hnsw_index_cost_snapshot('ec_hnsw_cost_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("planner flag should be non-null"),
            "planner gate should be live after D2 cost-model activation"
        );
        assert!(
            Spi::get_one::<String>(
                "SELECT planner_gate_reason FROM ec_hnsw_index_cost_snapshot('ec_hnsw_cost_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("gate reason should be non-null")
                .contains("FR-020"),
            "cost snapshot should reference FR-020 once the planner gate is retired"
        );
        assert_eq!(
            Spi::get_one::<i32>(
                "SELECT relation_ef_search FROM ec_hnsw_index_cost_snapshot('ec_hnsw_cost_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("relation ef_search should be non-null"),
            77
        );
        assert_eq!(
            Spi::get_one::<i32>(
                "SELECT session_ef_search FROM ec_hnsw_index_cost_snapshot('ec_hnsw_cost_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed"),
            Some(19)
        );
        assert_eq!(
            Spi::get_one::<i32>(
                "SELECT effective_ef_search FROM ec_hnsw_index_cost_snapshot('ec_hnsw_cost_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("effective ef_search should be non-null"),
            19
        );
        assert_eq!(
            Spi::get_one::<String>(
                "SELECT effective_source FROM ec_hnsw_index_cost_snapshot('ec_hnsw_cost_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("effective source should be non-null"),
            "session"
        );
        assert_eq!(
            Spi::get_one::<i32>(
                "SELECT m FROM ec_hnsw_index_cost_snapshot('ec_hnsw_cost_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("m should be non-null"),
            12
        );
        assert_eq!(
            Spi::get_one::<i32>(
                "SELECT dimensions FROM ec_hnsw_index_cost_snapshot('ec_hnsw_cost_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("dimensions should be non-null"),
            4
        );
        let max_level = Spi::get_one::<i32>(
            "SELECT max_level FROM ec_hnsw_index_cost_snapshot('ec_hnsw_cost_snapshot_idx'::regclass)",
        )
        .expect("snapshot query should succeed")
        .expect("max level should be non-null");
        assert_eq!(
            Spi::get_one::<f64>(
                "SELECT resolved_tree_height FROM ec_hnsw_index_cost_snapshot('ec_hnsw_cost_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("resolved tree height should be non-null"),
            f64::from(max_level)
        );
        assert_eq!(
            Spi::get_one::<String>(
                "SELECT tree_height_source FROM ec_hnsw_index_cost_snapshot('ec_hnsw_cost_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("tree height source should be non-null"),
            if cfg!(feature = "pg18") {
                "amgettreeheight_callback"
            } else {
                "metadata_fallback"
            }
        );
        assert_eq!(
            Spi::get_one::<bool>(
                "SELECT pg18_tree_height_callback_ready FROM ec_hnsw_index_cost_snapshot('ec_hnsw_cost_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("pg18 tree-height callback flag should be non-null"),
            cfg!(feature = "pg18")
        );
        assert!(
            Spi::get_one::<f64>(
                "SELECT index_pages FROM ec_hnsw_index_cost_snapshot('ec_hnsw_cost_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("index pages should be non-null")
                >= 1.0
        );
        assert!(
            Spi::get_one::<f64>(
                "SELECT reltuples FROM ec_hnsw_index_cost_snapshot('ec_hnsw_cost_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("reltuples should be non-null")
                >= 3.0
        );
        let modeled_startup = Spi::get_one::<f64>(
            "SELECT modeled_startup_cost FROM ec_hnsw_index_cost_snapshot('ec_hnsw_cost_snapshot_idx'::regclass)",
        )
        .expect("snapshot query should succeed")
        .expect("modeled startup should be non-null");
        let modeled_total = Spi::get_one::<f64>(
            "SELECT modeled_total_cost FROM ec_hnsw_index_cost_snapshot('ec_hnsw_cost_snapshot_idx'::regclass)",
        )
        .expect("snapshot query should succeed")
        .expect("modeled total should be non-null");
        assert!(
            modeled_startup.is_finite(),
            "modeled startup should be finite"
        );
        assert!(modeled_total.is_finite(), "modeled total should be finite");
        assert!(
            modeled_total >= modeled_startup,
            "modeled total cost should include startup cost"
        );
        assert_eq!(
            Spi::get_one::<f64>(
                "SELECT modeled_selectivity FROM ec_hnsw_index_cost_snapshot('ec_hnsw_cost_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("modeled selectivity should be non-null"),
            1.0
        );
        assert_eq!(
            Spi::get_one::<f64>(
                "SELECT modeled_correlation FROM ec_hnsw_index_cost_snapshot('ec_hnsw_cost_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("modeled correlation should be non-null"),
            0.0
        );
        assert_eq!(
            Spi::get_one::<f64>(
                "SELECT gated_startup_cost FROM ec_hnsw_index_cost_snapshot('ec_hnsw_cost_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("gated startup should be non-null"),
            f64::MAX
        );
        assert_eq!(
            Spi::get_one::<f64>(
                "SELECT gated_total_cost FROM ec_hnsw_index_cost_snapshot('ec_hnsw_cost_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("gated total should be non-null"),
            f64::MAX
        );
        assert_eq!(
            Spi::get_one::<f64>(
                "SELECT gated_selectivity FROM ec_hnsw_index_cost_snapshot('ec_hnsw_cost_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("gated selectivity should be non-null"),
            0.0
        );
        assert_eq!(
            Spi::get_one::<f64>(
                "SELECT gated_correlation FROM ec_hnsw_index_cost_snapshot('ec_hnsw_cost_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("gated correlation should be non-null"),
            0.0
        );

        Spi::run("RESET ec_hnsw.ef_search").expect("reset should succeed");
    }

    #[pg_test]
    #[should_panic(expected = "ec_hnsw_index_cost_snapshot requires a ec_hnsw index")]
    fn test_ech_index_cost_snapshot_rejects_non_ec_hnsw_index() {
        Spi::run(
            "CREATE TABLE ec_hnsw_cost_snapshot_wrong_am (id bigint primary key, value bigint)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "CREATE INDEX ec_hnsw_cost_snapshot_wrong_am_idx ON ec_hnsw_cost_snapshot_wrong_am USING btree (value)",
        )
        .expect("index creation should succeed");

        let _ = Spi::get_one::<f64>(
            "SELECT modeled_total_cost FROM ec_hnsw_index_cost_snapshot('ec_hnsw_cost_snapshot_wrong_am_idx'::regclass)",
        );
    }

    #[pg_test]
    fn test_ech_planner_integration_snapshot_reports_blockers() {
        Spi::run(
            "CREATE TABLE ec_hnsw_planner_integration_snapshot (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "CREATE INDEX ec_hnsw_planner_integration_snapshot_idx ON ec_hnsw_planner_integration_snapshot USING ec_hnsw (embedding ecvector_ip_ops)",
        )
        .expect("index creation should succeed");

        assert!(
            Spi::get_one::<bool>(
                "SELECT planner_scan_enabled FROM ec_hnsw_planner_integration_snapshot('ec_hnsw_planner_integration_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("planner scan flag should be non-null")
        );
        assert!(
            Spi::get_one::<bool>(
                "SELECT ordered_scan_ready FROM ec_hnsw_planner_integration_snapshot('ec_hnsw_planner_integration_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("ordered scan readiness should be non-null")
        );
        assert!(
            Spi::get_one::<bool>(
                "SELECT runtime_ordered_scan_ready FROM ec_hnsw_planner_integration_snapshot('ec_hnsw_planner_integration_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("runtime ordered scan readiness should be non-null")
        );
        assert!(
            Spi::get_one::<bool>(
                "SELECT planner_cost_model_ready FROM ec_hnsw_planner_integration_snapshot('ec_hnsw_planner_integration_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("planner cost model readiness should be non-null")
        );
        assert!(
            Spi::get_one::<bool>(
                "SELECT planner_cost_callback_live FROM ec_hnsw_planner_integration_snapshot('ec_hnsw_planner_integration_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("planner cost callback live flag should be non-null")
        );
        assert_eq!(
            Spi::get_one::<bool>(
                "SELECT pg18_callback_surface_ready FROM ec_hnsw_planner_integration_snapshot('ec_hnsw_planner_integration_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("pg18 callback readiness should be non-null"),
            cfg!(feature = "pg18")
        );
        assert!(
            !Spi::get_one::<bool>(
                "SELECT pg18_diagnostics_surface_ready FROM ec_hnsw_planner_integration_snapshot('ec_hnsw_planner_integration_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("pg18 diagnostics readiness should be non-null")
        );
        assert_eq!(
            Spi::get_one::<bool>(
                "SELECT pg18_read_stream_surface_ready FROM ec_hnsw_planner_integration_snapshot('ec_hnsw_planner_integration_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("pg18 read stream readiness should be non-null"),
            cfg!(feature = "pg18")
        );
        assert_eq!(
            Spi::get_one::<i32>(
                "SELECT effective_ef_search FROM ec_hnsw_planner_integration_snapshot('ec_hnsw_planner_integration_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("effective ef_search should be non-null"),
            40
        );
        assert_eq!(
            Spi::get_one::<String>(
                "SELECT effective_source FROM ec_hnsw_planner_integration_snapshot('ec_hnsw_planner_integration_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("effective source should be non-null"),
            "relation"
        );
        assert_eq!(
            Spi::get_one::<String>(
                "SELECT planner_gate_reason FROM ec_hnsw_planner_integration_snapshot('ec_hnsw_planner_integration_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("planner gate reason should be non-null"),
            "planner scan selection is live: FR-020 cost model active (ADR-011 superseded)"
        );
        assert_eq!(
            Spi::get_one::<String>(
                "SELECT next_runtime_blocker FROM ec_hnsw_planner_integration_snapshot('ec_hnsw_planner_integration_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("runtime blocker should be non-null"),
            "no merged runtime blocker remains on main; post-vacuum benchmark/reporting is next"
        );
        assert_eq!(
            Spi::get_one::<String>(
                "SELECT next_pg18_blocker FROM ec_hnsw_planner_integration_snapshot('ec_hnsw_planner_integration_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("pg18 blocker should be non-null"),
            if cfg!(feature = "pg18") {
                "custom pgstat kind registration requires loading ecaz via shared_preload_libraries on PG18 and restarting PostgreSQL"
            } else {
                "custom pgstat kind registration remains gated outside this build"
            }
        );
    }

    #[pg_test]
    #[should_panic(expected = "ec_hnsw_planner_integration_snapshot requires a ec_hnsw index")]
    fn test_ech_planner_integration_snapshot_rejects_wrong_am() {
        Spi::run(
            "CREATE TABLE ec_hnsw_planner_integration_snapshot_wrong_am (id bigint primary key, value bigint)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "CREATE INDEX ec_hnsw_planner_integration_snapshot_wrong_am_idx ON ec_hnsw_planner_integration_snapshot_wrong_am USING btree (value)",
        )
        .expect("index creation should succeed");

        let _ = Spi::get_one::<bool>(
            "SELECT planner_scan_enabled FROM ec_hnsw_planner_integration_snapshot('ec_hnsw_planner_integration_snapshot_wrong_am_idx'::regclass)",
        );
    }

    #[cfg(feature = "pg18")]
    #[pg_test]
    fn test_pg18_ecaz_stats_reports_backend_local_counters() {
        Spi::run(
            "CREATE TABLE pg18_tqvector_stats_fixture (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO pg18_tqvector_stats_fixture VALUES
             (1, encode_to_ecvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_ecvector(ARRAY[0.0, 1.0, 0.25, -0.5], 4, 42)),
             (3, encode_to_ecvector(ARRAY[0.5, 0.5, -0.5, 1.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX pg18_tqvector_stats_fixture_idx ON pg18_tqvector_stats_fixture USING ec_hnsw (embedding ecvector_ip_ops)",
        )
        .expect("index creation should succeed");
        Spi::run("SET enable_seqscan = off").expect("set should succeed");

        let _ = Spi::get_one::<i64>(
            "SELECT id FROM pg18_tqvector_stats_fixture
             ORDER BY embedding <#> ARRAY[1.0, 0.0, 0.5, -1.0]::real[]
             LIMIT 1",
        )
        .expect("query should succeed");

        assert!(
            Spi::get_one::<i64>("SELECT total_scans_started FROM ecaz_stats()")
                .expect("stats query should succeed")
                .expect("scan counter should be non-null")
                >= 1
        );
        assert!(
            Spi::get_one::<i64>("SELECT total_distance_calcs FROM ecaz_stats()")
                .expect("stats query should succeed")
                .expect("distance counter should be non-null")
                > 0
        );

        Spi::run("RESET enable_seqscan").expect("reset should succeed");
    }

    #[cfg(feature = "pg18")]
    #[pg_test]
    fn test_pg18_ecaz_stats_reports_backend_local_counters_for_ec_ivf() {
        Spi::run("CREATE TABLE pg18_ivf_stats_fixture (id bigint primary key, embedding ecvector)")
            .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO pg18_ivf_stats_fixture VALUES
             (1, '[1.0,0.0]'::ecvector),
             (2, '[0.0,1.0]'::ecvector),
             (3, '[-1.0,0.0]'::ecvector),
             (4, '[0.0,-1.0]'::ecvector)",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX pg18_ivf_stats_fixture_idx ON pg18_ivf_stats_fixture USING ec_ivf
             (embedding ecvector_ip_ops)
             WITH (nlists = 2, nprobe = 2, training_sample_rows = 4)",
        )
        .expect("index creation should succeed");

        let scans_before = Spi::get_one::<i64>("SELECT total_scans_started FROM ecaz_stats()")
            .expect("stats query should succeed")
            .expect("scan counter should be non-null");
        let distance_before = Spi::get_one::<i64>("SELECT total_distance_calcs FROM ecaz_stats()")
            .expect("stats query should succeed")
            .expect("distance counter should be non-null");
        let posting_pages_before =
            Spi::get_one::<i64>("SELECT total_linear_pages FROM ecaz_stats()")
                .expect("stats query should succeed")
                .expect("posting page counter should be non-null");

        Spi::run("SET enable_seqscan = off").expect("set should succeed");
        let _ = Spi::get_one::<i64>(
            "SELECT id FROM pg18_ivf_stats_fixture
             ORDER BY embedding <#> ARRAY[1.0, 0.0]::real[]
             LIMIT 1",
        )
        .expect("query should succeed");

        assert!(
            Spi::get_one::<i64>("SELECT total_scans_started FROM ecaz_stats()")
                .expect("stats query should succeed")
                .expect("scan counter should be non-null")
                > scans_before
        );
        assert!(
            Spi::get_one::<i64>("SELECT total_distance_calcs FROM ecaz_stats()")
                .expect("stats query should succeed")
                .expect("distance counter should be non-null")
                > distance_before
        );
        assert!(
            Spi::get_one::<i64>("SELECT total_linear_pages FROM ecaz_stats()")
                .expect("stats query should succeed")
                .expect("posting page counter should be non-null")
                > posting_pages_before
        );

        Spi::run("RESET enable_seqscan").expect("reset should succeed");
    }

    #[cfg(feature = "pg18")]
    #[pg_test]
    fn test_pg18_module_identity_reports_loaded_module_version() {
        let version = Spi::get_one::<String>(
            "SELECT version FROM pg_get_loaded_modules() WHERE module_name = 'ecaz'",
        )
        .expect("module query should succeed")
        .expect("module version should be visible");
        assert_eq!(version, env!("CARGO_PKG_VERSION"));
    }

    #[cfg(feature = "pg18")]
    #[pg_test]
    fn test_pg18_explain_option_emits_ecaz_stats_group() {
        Spi::run("CREATE TABLE pg18_explain_fixture (id bigint primary key, embedding ecvector)")
            .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO pg18_explain_fixture VALUES
             (1, encode_to_ecvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_ecvector(ARRAY[0.0, 1.0, 0.25, -0.5], 4, 42)),
             (3, encode_to_ecvector(ARRAY[0.5, 0.5, -0.5, 1.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX pg18_explain_fixture_idx ON pg18_explain_fixture USING ec_hnsw (embedding ecvector_ip_ops)",
        )
        .expect("index creation should succeed");
        Spi::run("SET enable_seqscan = off").expect("set should succeed");

        let plan = Spi::connect(|client| {
            let rows = client
                .select(
                    "EXPLAIN (FORMAT JSON, ecaz, ANALYZE, COSTS OFF)
                     SELECT id FROM pg18_explain_fixture
                     ORDER BY embedding <#> ARRAY[1.0, 0.0, 0.5, -1.0]::real[]
                     LIMIT 1",
                    None,
                    &[],
                )
                .expect("EXPLAIN should succeed");
            let mut lines = Vec::new();
            for row in rows {
                lines.push(
                    row.get::<pgrx::datum::JsonString>(1)
                        .expect("plan row should decode")
                        .expect("plan row should not be NULL")
                        .0,
                );
            }
            lines.join("\n")
        });

        if !plan.contains("Ecaz Stats") {
            panic!("missing Ecaz Stats in plan: {plan:?}");
        }
        if !plan.contains("Elements Scored") {
            panic!("missing Elements Scored in plan: {plan:?}");
        }

        Spi::run("RESET enable_seqscan").expect("reset should succeed");
    }

    #[cfg(feature = "pg18")]
    #[pg_test]
    fn test_pg18_explain_option_emits_ecaz_stats_group_for_ec_ivf() {
        Spi::run(
            "CREATE TABLE pg18_explain_ivf_fixture (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO pg18_explain_ivf_fixture VALUES
             (1, '[1.0,0.0]'::ecvector),
             (2, '[0.0,1.0]'::ecvector),
             (3, '[-1.0,0.0]'::ecvector),
             (4, '[0.0,-1.0]'::ecvector)",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX pg18_explain_ivf_fixture_idx ON pg18_explain_ivf_fixture USING ec_ivf \
             (embedding ecvector_ip_ops) \
             WITH (
                nlists = 2,
                nprobe = 2,
                training_sample_rows = 4,
                rerank = 'heap_f32',
                rerank_width = 2
             )",
        )
        .expect("index creation should succeed");
        Spi::run("SET enable_seqscan = off").expect("set should succeed");

        let plan = Spi::connect(|client| {
            let rows = client
                .select(
                    "EXPLAIN (FORMAT JSON, ecaz, ANALYZE, COSTS OFF)
                     SELECT id FROM pg18_explain_ivf_fixture
                     ORDER BY embedding <#> ARRAY[1.0, 0.0]::real[]
                     LIMIT 2",
                    None,
                    &[],
                )
                .expect("EXPLAIN should succeed");
            let mut lines = Vec::new();
            for row in rows {
                lines.push(
                    row.get::<pgrx::datum::JsonString>(1)
                        .expect("plan row should decode")
                        .expect("plan row should not be NULL")
                        .0,
                );
            }
            lines.join("\n")
        });

        for expected in [
            "Ecaz Stats",
            "Centroid Scores",
            "Selected Lists",
            "Posting Pages Read",
            "Postings Visited",
            "Postings Scored",
            "Postings Pruned By Bound",
            "Heap TIDs Scored",
            "Candidates Scored",
            "Candidates Inserted",
            "\"Rerank Rows\": 2",
            "Filtered Duplicates",
        ] {
            if !plan.contains(expected) {
                panic!("missing {expected} in IVF EXPLAIN plan: {plan:?}");
            }
        }

        Spi::run("RESET enable_seqscan").expect("reset should succeed");
    }

    #[pg_test]
    fn test_empty_index_build_initializes_metadata_page() {
        Spi::run("CREATE TABLE ec_hnsw_empty_build (id bigint primary key, embedding ecvector)")
            .expect("table creation should succeed");
        Spi::run("CREATE EXTENSION pageinspect").expect("pageinspect should be available");
        Spi::run(
            "CREATE INDEX ec_hnsw_empty_build_idx ON ec_hnsw_empty_build USING ec_hnsw \
             (embedding ecvector_ip_ops) WITH (m = 12, ef_construction = 80)",
        )
        .expect("index creation should succeed");

        let reloptions = Spi::get_one::<Vec<String>>(
            "SELECT reloptions FROM pg_class WHERE oid = 'ec_hnsw_empty_build_idx'::regclass",
        )
        .expect("SPI query should succeed")
        .expect("reloptions should exist");

        let first_page =
            Spi::get_one::<Vec<u8>>("SELECT get_raw_page('ec_hnsw_empty_build_idx', 0)::bytea")
                .expect("SPI query should succeed")
                .expect("raw index page should exist");

        let metadata = am::page::MetadataPage::decode_page(&first_page)
            .expect("metadata page should decode from raw relation bytes");

        assert_eq!(
            reloptions,
            vec!["m=12".to_string(), "ef_construction=80".to_string()]
        );
        assert_eq!(metadata.m, 12);
        assert_eq!(metadata.ef_construction, 80);
        assert_eq!(metadata.entry_point, am::page::ItemPointer::INVALID);
        assert_eq!(metadata.dimensions, 0);
        assert_eq!(metadata.bits, 0);
        assert_eq!(metadata.max_level, 0);
        assert_eq!(metadata.seed, 0);
    }

    #[pg_test]
    fn test_non_empty_index_build_writes_minimal_data_pages() {
        Spi::run("CREATE TABLE ec_hnsw_nonempty_build (id bigint primary key, embedding ecvector)")
            .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_hnsw_nonempty_build VALUES
             (1, encode_to_ecvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_ecvector(ARRAY[0.0, 1.0, 0.25, -0.5], 4, 42)),
             (3, encode_to_ecvector(ARRAY[-1.0, 0.5, 0.0, 1.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_hnsw_nonempty_build_idx ON ec_hnsw_nonempty_build USING ec_hnsw \
             (embedding ecvector_ip_ops) WITH (m = 10, ef_construction = 90)",
        )
        .expect("index creation should succeed");

        let index_oid =
            Spi::get_one::<pg_sys::Oid>("SELECT 'ec_hnsw_nonempty_build_idx'::regclass::oid")
                .expect("SPI query should succeed")
                .expect("index oid should exist");

        let (block_count, metadata, data_pages) = unsafe { am::debug_index_pages(index_oid) };

        assert!(
            block_count >= 2,
            "non-empty build should allocate a data page"
        );
        assert_eq!(metadata.m, 10);
        assert_eq!(metadata.ef_construction, 90);
        assert_eq!(metadata.dimensions, 4);
        assert_eq!(metadata.bits, 4);
        assert_eq!(metadata.seed, 42);
        assert_ne!(metadata.entry_point, am::page::ItemPointer::INVALID);
        assert!(metadata.max_level <= am::page::default_max_level_cap(metadata.m));

        let page_tuples = data_pages
            .iter()
            .flat_map(|page| {
                page.tuples.iter().enumerate().map(move |(idx, tuple)| {
                    (
                        am::page::ItemPointer {
                            block_number: page.block_number,
                            offset_number: (idx + 1) as u16,
                        },
                        tuple.as_slice(),
                    )
                })
            })
            .collect::<Vec<_>>();

        assert_eq!(
            page_tuples.len(),
            9,
            "each heap row should emit one neighbor, one turbo hot tuple, and one rerank tuple"
        );

        let neighbor_tids = page_tuples
            .iter()
            .filter_map(|(tid, tuple)| {
                if tuple.first().copied() == Some(am::page::TQ_NEIGHBOR_TAG) {
                    Some(*tid)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        let elements =
            decode_turboquant_elements_from_pages(&metadata, &data_pages, code_len(4, 4));

        assert_eq!(neighbor_tids.len(), 3);
        assert_eq!(elements.len(), 3);
        assert!(
            elements.iter().any(|(tid, _)| *tid == metadata.entry_point),
            "entry point should identify an element tuple"
        );

        let neighbor_map = page_tuples
            .iter()
            .filter_map(|(tid, tuple)| {
                if tuple.first().copied() == Some(am::page::TQ_NEIGHBOR_TAG) {
                    Some((
                        *tid,
                        am::page::TqNeighborTuple::decode(tuple)
                            .expect("neighbor tuple should decode"),
                    ))
                } else {
                    None
                }
            })
            .collect::<std::collections::HashMap<_, _>>();
        let element_ids = elements.iter().map(|(tid, _)| *tid).collect::<Vec<_>>();
        let entry_element = elements
            .iter()
            .find(|(tid, _)| *tid == metadata.entry_point)
            .expect("entry point should identify an element tuple");
        assert_eq!(entry_element.1.level, metadata.max_level);
        let persisted_binary_quantizer = crate::quant::prod::ProdQuantizer::cached(
            metadata.dimensions as usize,
            metadata.bits,
            metadata.seed,
        );
        let expected_binary_word_count =
            if persisted_binary_quantizer.binary_sign_no_qjl_4bit_supported() {
                (metadata.dimensions as usize).div_ceil(64)
            } else {
                0
            };
        for (element_tid, element) in &elements {
            assert!(element.level <= metadata.max_level);
            assert!(!element.deleted);
            assert_eq!(element.heaptids.len(), 1);
            assert_ne!(element.heaptids[0], am::page::ItemPointer::INVALID);
            assert_eq!(
                element.binary_words.len(),
                expected_binary_word_count,
                "builds should persist ADR-031 sidecars only on the supported no-QJL 4-bit lane",
            );
            assert!(neighbor_tids.contains(&element.neighbortid));
            let neighbor = neighbor_map
                .get(&element.neighbortid)
                .expect("neighbor tuple should exist");
            assert_eq!(
                neighbor.count as usize,
                neighbor.tids.len(),
                "neighbor tuples should persist every logical layer slot so runtime layer slicing stays stable",
            );
            assert_eq!(
                neighbor.tids.len(),
                am::page::neighbor_slots(element.level, metadata.m),
                "neighbor tuples should carry the full 2M / M slot payload for the node level instead of compacting active neighbors",
            );
            assert!(!neighbor.tids.contains(element_tid));
            assert!(neighbor.tids.iter().all(|tid| {
                *tid == am::page::ItemPointer::INVALID || element_ids.contains(tid)
            }));
        }
    }

    #[cfg(feature = "pg18")]
    #[pg_test]
    fn test_pg18_parallel_index_build_uses_workers() {
        Spi::run("SET max_parallel_maintenance_workers = 2").expect("set should succeed");
        Spi::run("SET max_parallel_workers = 4").expect("set should succeed");
        Spi::run("SET maintenance_work_mem = '128MB'").expect("set should succeed");
        Spi::run(
            "CREATE TABLE ec_hnsw_parallel_build (
                id bigint primary key,
                embedding ecvector
            )",
        )
        .expect("table creation should succeed");
        Spi::run("ALTER TABLE ec_hnsw_parallel_build SET (parallel_workers = 2)")
            .expect("parallel_workers reloption should be accepted");
        Spi::run(
            "INSERT INTO ec_hnsw_parallel_build
             SELECT id,
                    encode_to_ecvector(
                        ARRAY[
                            id::real,
                            (id * 2)::real,
                            (id * 3 + 1)::real,
                            (id * 5 + 2)::real
                        ]::real[],
                        4,
                        42
                    )
             FROM generate_series(1, 128) AS id",
        )
        .expect("insert should succeed");

        Spi::run(
            "CREATE INDEX ec_hnsw_parallel_build_idx ON ec_hnsw_parallel_build USING ec_hnsw \
             (embedding ecvector_ip_ops) WITH (m = 6, ef_construction = 40)",
        )
        .expect("parallel index creation should succeed");

        let workers_launched =
            Spi::get_one::<i32>("SELECT tests.ec_hnsw_debug_parallel_build_workers_launched()")
                .expect("SPI query should succeed")
                .expect("debug worker count should be non-null");
        assert!(
            workers_launched >= 1,
            "parallel build should launch at least one worker, launched {workers_launched}"
        );
        let build_timing = Spi::connect(|client| {
            let mut rows = client
                .select(
                    "SELECT requested_workers, workers_launched, heap_tuples, index_tuples,
                            heap_ingest_us, parallel_begin_us, parallel_drain_us,
                            parallel_sort_push_us, flush_total_us, graph_us, stage_us, write_us
                     FROM tests.ec_hnsw_debug_last_build_timing()",
                    None,
                    &[],
                )
                .expect("timing query should succeed");
            let row = rows.next().expect("timing query should return one row");
            (
                row.get::<i64>(1)
                    .expect("requested_workers should decode")
                    .unwrap(),
                row.get::<i64>(2)
                    .expect("workers_launched should decode")
                    .unwrap(),
                row.get::<i64>(3)
                    .expect("heap_tuples should decode")
                    .unwrap(),
                row.get::<i64>(4)
                    .expect("index_tuples should decode")
                    .unwrap(),
                row.get::<i64>(5)
                    .expect("heap_ingest_us should decode")
                    .unwrap(),
                row.get::<i64>(6)
                    .expect("parallel_begin_us should decode")
                    .unwrap(),
                row.get::<i64>(7)
                    .expect("parallel_drain_us should decode")
                    .unwrap(),
                row.get::<i64>(8)
                    .expect("parallel_sort_push_us should decode")
                    .unwrap(),
                row.get::<i64>(9)
                    .expect("flush_total_us should decode")
                    .unwrap(),
                row.get::<i64>(10).expect("graph_us should decode").unwrap(),
                row.get::<i64>(11).expect("stage_us should decode").unwrap(),
                row.get::<i64>(12).expect("write_us should decode").unwrap(),
            )
        });
        assert_eq!(build_timing.0, 2);
        assert_eq!(build_timing.1, workers_launched as i64);
        assert_eq!(build_timing.2, 128);
        assert_eq!(build_timing.3, 128);
        assert!(build_timing.4 > 0, "heap ingest timing should be recorded");
        assert!(
            build_timing.5 > 0,
            "parallel begin timing should be recorded"
        );
        assert!(
            build_timing.6 > 0,
            "parallel drain timing should be recorded"
        );
        assert!(
            build_timing.7 > 0,
            "parallel sort/push timing should be recorded"
        );
        assert!(build_timing.8 > 0, "flush timing should be recorded");
        assert!(build_timing.9 > 0, "graph timing should be recorded");
        assert!(build_timing.10 > 0, "stage timing should be recorded");
        assert!(build_timing.11 > 0, "write timing should be recorded");

        let index_oid =
            Spi::get_one::<pg_sys::Oid>("SELECT 'ec_hnsw_parallel_build_idx'::regclass::oid")
                .expect("SPI query should succeed")
                .expect("index oid should exist");
        let (_block_count, metadata, data_pages) = unsafe { am::debug_index_pages(index_oid) };
        let elements =
            decode_turboquant_elements_from_pages(&metadata, &data_pages, code_len(4, 4));
        let heap_tid_count = elements
            .iter()
            .map(|(_, element)| element.heaptids.len())
            .sum::<usize>();
        assert_eq!(heap_tid_count, 128);
        assert_ne!(metadata.entry_point, am::page::ItemPointer::INVALID);

        Spi::run("RESET maintenance_work_mem").expect("reset should succeed");
        Spi::run("RESET max_parallel_workers").expect("reset should succeed");
        Spi::run("RESET max_parallel_maintenance_workers").expect("reset should succeed");
    }

    #[cfg(feature = "pg18")]
    #[pg_test]
    fn test_pg18_parallel_index_build_concurrent_dsm_graph_default() {
        Spi::run("SET max_parallel_maintenance_workers = 2").expect("set should succeed");
        Spi::run("SET max_parallel_workers = 4").expect("set should succeed");
        Spi::run("SET maintenance_work_mem = '128MB'").expect("set should succeed");
        Spi::run(
            "CREATE TABLE ec_hnsw_parallel_build_dsm (
                id bigint primary key,
                embedding ecvector
            )",
        )
        .expect("table creation should succeed");
        Spi::run("ALTER TABLE ec_hnsw_parallel_build_dsm SET (parallel_workers = 2)")
            .expect("parallel_workers reloption should be accepted");
        Spi::run(
            "INSERT INTO ec_hnsw_parallel_build_dsm
             SELECT id,
                    encode_to_ecvector(
                        ARRAY[
                            id::real,
                            (id * 2)::real,
                            (id * 3 + 1)::real,
                            (id * 5 + 2)::real
                        ]::real[],
                        4,
                        42
                    )
             FROM generate_series(1, 96) AS id",
        )
        .expect("insert should succeed");

        Spi::run(
            "CREATE INDEX ec_hnsw_parallel_build_dsm_idx ON ec_hnsw_parallel_build_dsm USING ec_hnsw \
             (embedding ecvector_ip_ops) WITH (m = 6, ef_construction = 40)",
        )
        .expect("parallel concurrent DSM index creation should succeed");

        let graph_workers_launched = Spi::get_one::<i32>(
            "SELECT tests.ec_hnsw_debug_parallel_graph_build_workers_launched()",
        )
        .expect("SPI query should succeed")
        .expect("debug graph worker count should be non-null");
        assert!(
            graph_workers_launched >= 1,
            "parallel graph build should launch at least one worker, launched {graph_workers_launched}"
        );

        let index_oid =
            Spi::get_one::<pg_sys::Oid>("SELECT 'ec_hnsw_parallel_build_dsm_idx'::regclass::oid")
                .expect("SPI query should succeed")
                .expect("index oid should exist");
        let (_block_count, metadata, data_pages) = unsafe { am::debug_index_pages(index_oid) };
        let elements =
            decode_turboquant_elements_from_pages(&metadata, &data_pages, code_len(4, 4));
        let heap_tid_count = elements
            .iter()
            .map(|(_, element)| element.heaptids.len())
            .sum::<usize>();
        assert_eq!(heap_tid_count, 96);
        assert_ne!(metadata.entry_point, am::page::ItemPointer::INVALID);

        Spi::run("RESET maintenance_work_mem").expect("reset should succeed");
        Spi::run("RESET max_parallel_workers").expect("reset should succeed");
        Spi::run("RESET max_parallel_maintenance_workers").expect("reset should succeed");
    }

    #[cfg(feature = "pg18")]
    #[pg_test]
    fn test_pg18_parallel_index_build_concurrent_dsm_can_be_disabled() {
        Spi::run("SET ec_hnsw.enable_parallel_build_concurrent_dsm = off")
            .expect("set should succeed");
        Spi::run("SET max_parallel_maintenance_workers = 2").expect("set should succeed");
        Spi::run("SET max_parallel_workers = 4").expect("set should succeed");
        Spi::run("SET maintenance_work_mem = '128MB'").expect("set should succeed");
        Spi::run(
            "CREATE TABLE ec_hnsw_parallel_build_dsm_disabled (
                id bigint primary key,
                embedding ecvector
            )",
        )
        .expect("table creation should succeed");
        Spi::run("ALTER TABLE ec_hnsw_parallel_build_dsm_disabled SET (parallel_workers = 2)")
            .expect("parallel_workers reloption should be accepted");
        Spi::run(
            "INSERT INTO ec_hnsw_parallel_build_dsm_disabled
             SELECT id,
                    encode_to_ecvector(
                        ARRAY[
                            id::real,
                            (id * 2)::real,
                            (id * 3 + 1)::real,
                            (id * 5 + 2)::real
                        ]::real[],
                        4,
                        42
                    )
             FROM generate_series(1, 64) AS id",
        )
        .expect("insert should succeed");

        Spi::run(
            "CREATE INDEX ec_hnsw_parallel_build_dsm_disabled_idx
             ON ec_hnsw_parallel_build_dsm_disabled
             USING ec_hnsw (embedding ecvector_ip_ops)
             WITH (m = 6, ef_construction = 40)",
        )
        .expect("parallel fallback index creation should succeed");

        let graph_workers_launched = Spi::get_one::<i32>(
            "SELECT tests.ec_hnsw_debug_parallel_graph_build_workers_launched()",
        )
        .expect("SPI query should succeed")
        .expect("debug graph worker count should be non-null");
        assert_eq!(
            graph_workers_launched, 0,
            "disabled concurrent DSM graph build should not launch graph workers"
        );

        Spi::run("RESET maintenance_work_mem").expect("reset should succeed");
        Spi::run("RESET max_parallel_workers").expect("reset should succeed");
        Spi::run("RESET max_parallel_maintenance_workers").expect("reset should succeed");
        Spi::run("RESET ec_hnsw.enable_parallel_build_concurrent_dsm")
            .expect("reset should succeed");
    }

    #[pg_test]
    fn test_non_empty_index_build_supports_raw_source_column() {
        Spi::run(
            "CREATE TABLE ec_hnsw_source_build (
                id bigint primary key,
                source real[],
                embedding ecvector
            )",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_hnsw_source_build VALUES
             (1, ARRAY[1.0, 0.0, 0.5, -1.0], encode_to_ecvector(ARRAY[0.2, 0.1, 0.0, -0.2], 4, 42)),
             (2, ARRAY[0.9, 0.1, 0.4, -0.8], encode_to_ecvector(ARRAY[-0.1, 0.9, 0.2, -0.3], 4, 42)),
             (3, ARRAY[-1.0, 0.5, 0.0, 1.0], encode_to_ecvector(ARRAY[0.8, -0.4, 0.1, 0.7], 4, 42)),
             (4, ARRAY[-0.8, 0.4, 0.2, 0.9], encode_to_ecvector(ARRAY[-0.7, -0.2, 0.3, 0.6], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_hnsw_source_build_idx ON ec_hnsw_source_build USING ec_hnsw \
             (embedding ecvector_ip_ops) WITH (m = 6, ef_construction = 80, build_source_column = 'source')",
        )
        .expect("index creation should succeed");

        let reloptions = Spi::get_one::<Vec<String>>(
            "SELECT reloptions FROM pg_class WHERE oid = 'ec_hnsw_source_build_idx'::regclass",
        )
        .expect("SPI query should succeed")
        .expect("reloptions should exist");
        assert!(reloptions.contains(&"build_source_column=source".to_string()));

        let index_oid =
            Spi::get_one::<pg_sys::Oid>("SELECT 'ec_hnsw_source_build_idx'::regclass::oid")
                .expect("SPI query should succeed")
                .expect("index oid should exist");

        let (_block_count, metadata, data_pages) = unsafe { am::debug_index_pages(index_oid) };
        assert_eq!(metadata.m, 6);
        assert_eq!(metadata.ef_construction, 80);
        assert_eq!(metadata.dimensions, 4);
        assert_eq!(metadata.bits, 4);
        assert_eq!(metadata.seed, 42);
        assert_ne!(metadata.entry_point, am::page::ItemPointer::INVALID);

        let elements =
            decode_turboquant_elements_from_pages(&metadata, &data_pages, code_len(4, 4));

        assert_eq!(elements.len(), 4);
        assert!(
            elements.iter().any(|(tid, _)| *tid == metadata.entry_point),
            "entry point should identify an element tuple"
        );
        assert!(elements
            .iter()
            .all(|(_, element)| element.heaptids.len() == 1));
    }

    #[pg_test]
    fn test_ech_pq_fastscan_source_build_writes_grouped_pages() {
        let _lock = env_var_test_lock();

        Spi::run(
            "CREATE TABLE ec_hnsw_pq_fastscan_source_build (
                id bigint primary key,
                source real[],
                embedding ecvector
            )",
        )
        .expect("table creation should succeed");

        for id in 1..=16 {
            let source = (0..16)
                .map(|dim| format!("{:.6}", (((id * 19 + dim) as f32) * 0.05).cos()))
                .collect::<Vec<_>>()
                .join(", ");
            let embedding = (0..16)
                .map(|dim| format!("{:.6}", (((id * 11 + dim) as f32) * 0.04).sin()))
                .collect::<Vec<_>>()
                .join(", ");
            Spi::run(&format!(
                "INSERT INTO ec_hnsw_pq_fastscan_source_build VALUES \
                 ({id}, ARRAY[{source}]::real[], encode_to_ecvector(ARRAY[{embedding}]::real[], 4, 42))"
            ))
            .expect("insert should succeed");
        }

        Spi::run(
            "CREATE INDEX ec_hnsw_pq_fastscan_source_build_idx ON ec_hnsw_pq_fastscan_source_build USING ec_hnsw \
             (embedding ecvector_ip_ops) WITH (m = 6, ef_construction = 80, build_source_column = 'source', storage_format = 'pq_fastscan')",
        )
        .expect("index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_hnsw_pq_fastscan_source_build_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");
        let reloptions = Spi::get_one::<Vec<String>>(
            "SELECT reloptions FROM pg_class WHERE oid = 'ec_hnsw_pq_fastscan_source_build_idx'::regclass",
        )
        .expect("SPI query should succeed")
        .expect("reloptions should exist");

        let (_block_count, metadata, data_pages) = unsafe { am::debug_index_pages(index_oid) };

        assert!(reloptions.contains(&"storage_format=pq_fastscan".to_string()));
        assert_eq!(metadata.format_version, am::page::INDEX_FORMAT_V2_GROUPED);
        assert_eq!(metadata.transform_kind, am::page::TransformKind::Srht);
        assert_eq!(
            metadata.search_codec_kind,
            am::page::SearchCodecKind::GroupedPq
        );
        assert_eq!(
            metadata.rerank_codec_kind,
            am::page::RerankCodecKind::ScalarQuantized
        );
        assert_eq!(
            metadata.payload_flags & am::page::PAYLOAD_FLAG_GROUPED_SEARCH_CODE,
            am::page::PAYLOAD_FLAG_GROUPED_SEARCH_CODE
        );
        assert_eq!(
            metadata.payload_flags & am::page::PAYLOAD_FLAG_COLD_RERANK_PAYLOAD,
            am::page::PAYLOAD_FLAG_COLD_RERANK_PAYLOAD
        );
        assert_eq!(metadata.dimensions, 16);
        assert_eq!(metadata.bits, 4);
        assert_eq!(metadata.seed, 42);
        assert_eq!(metadata.search_bits, 4);
        assert_eq!(metadata.search_subvector_count, 1);
        assert_eq!(metadata.search_subvector_dim, 16);

        let page_tuples = data_pages
            .iter()
            .flat_map(|page| {
                page.tuples.iter().enumerate().map(move |(idx, tuple)| {
                    (
                        am::page::ItemPointer {
                            block_number: page.block_number,
                            offset_number: (idx + 1) as u16,
                        },
                        tuple.as_slice(),
                    )
                })
            })
            .collect::<Vec<_>>();

        let grouped_hot_tids = page_tuples
            .iter()
            .filter_map(|(tid, tuple)| {
                (tuple.first().copied() == Some(am::page::TQ_GROUPED_HOT_TAG)).then_some(*tid)
            })
            .collect::<Vec<_>>();
        let rerank_count = page_tuples
            .iter()
            .filter(|(_, tuple)| tuple.first().copied() == Some(am::page::TQ_RERANK_TAG))
            .count();
        let neighbor_count = page_tuples
            .iter()
            .filter(|(_, tuple)| tuple.first().copied() == Some(am::page::TQ_NEIGHBOR_TAG))
            .count();
        let grouped_codebook_count = page_tuples
            .iter()
            .filter(|(_, tuple)| tuple.first().copied() == Some(am::page::TQ_GROUPED_CODEBOOK_TAG))
            .count();

        assert_eq!(grouped_hot_tids.len(), 16);
        assert_eq!(rerank_count, 16);
        assert_eq!(neighbor_count, 16);
        assert_eq!(
            grouped_codebook_count,
            metadata.search_subvector_count as usize
        );
        assert_ne!(
            metadata.grouped_codebook_head,
            am::page::ItemPointer::INVALID
        );
        assert!(!page_tuples
            .iter()
            .any(|(_, tuple)| tuple.first().copied() == Some(am::page::TQ_ELEMENT_TAG)));
        assert!(
            grouped_hot_tids.contains(&metadata.entry_point),
            "entry point should identify a grouped hot tuple for a PqFastScan build"
        );
    }

    #[pg_test]
    fn test_ech_pq_fastscan_small_source_build_writes_grouped_pages() {
        let _lock = env_var_test_lock();

        Spi::run(
            "CREATE TABLE ec_hnsw_pq_fastscan_small_source_build (
                id bigint primary key,
                source real[],
                embedding ecvector
            )",
        )
        .expect("table creation should succeed");

        for id in 1..=4 {
            let source = (0..16)
                .map(|dim| format!("{:.6}", (((id * 13 + dim) as f32) * 0.05).sin()))
                .collect::<Vec<_>>()
                .join(", ");
            let embedding = (0..16)
                .map(|dim| format!("{:.6}", (((id * 7 + dim) as f32) * 0.04).cos()))
                .collect::<Vec<_>>()
                .join(", ");
            Spi::run(&format!(
                "INSERT INTO ec_hnsw_pq_fastscan_small_source_build VALUES \
                 ({id}, ARRAY[{source}]::real[], encode_to_ecvector(ARRAY[{embedding}]::real[], 4, 42))"
            ))
            .expect("insert should succeed");
        }

        Spi::run(
            "CREATE INDEX ec_hnsw_pq_fastscan_small_source_build_idx ON ec_hnsw_pq_fastscan_small_source_build USING ec_hnsw \
             (embedding ecvector_ip_ops) WITH (m = 6, ef_construction = 80, build_source_column = 'source', storage_format = 'pq_fastscan')",
        )
        .expect("small-table index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_hnsw_pq_fastscan_small_source_build_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");
        let (_block_count, metadata, data_pages) = unsafe { am::debug_index_pages(index_oid) };

        let page_tuples = data_pages
            .iter()
            .flat_map(|page| {
                page.tuples.iter().enumerate().map(move |(idx, tuple)| {
                    (
                        am::page::ItemPointer {
                            block_number: page.block_number,
                            offset_number: (idx + 1) as u16,
                        },
                        tuple.as_slice(),
                    )
                })
            })
            .collect::<Vec<_>>();
        let grouped_hot_tids = page_tuples
            .iter()
            .filter_map(|(tid, tuple)| {
                (tuple.first().copied() == Some(am::page::TQ_GROUPED_HOT_TAG)).then_some(*tid)
            })
            .collect::<Vec<_>>();
        let rerank_count = page_tuples
            .iter()
            .filter(|(_, tuple)| tuple.first().copied() == Some(am::page::TQ_RERANK_TAG))
            .count();
        let neighbor_count = page_tuples
            .iter()
            .filter(|(_, tuple)| tuple.first().copied() == Some(am::page::TQ_NEIGHBOR_TAG))
            .count();
        let grouped_codebook_count = page_tuples
            .iter()
            .filter(|(_, tuple)| tuple.first().copied() == Some(am::page::TQ_GROUPED_CODEBOOK_TAG))
            .count();

        assert_eq!(metadata.format_version, am::page::INDEX_FORMAT_V2_GROUPED);
        assert_eq!(grouped_hot_tids.len(), 4);
        assert_eq!(rerank_count, 4);
        assert_eq!(neighbor_count, 4);
        assert_eq!(
            grouped_codebook_count,
            metadata.search_subvector_count as usize
        );
        assert_ne!(
            metadata.grouped_codebook_head,
            am::page::ItemPointer::INVALID
        );
        assert!(
            grouped_hot_tids.contains(&metadata.entry_point),
            "small-cardinality PqFastScan build should still pick a grouped hot tuple entry point",
        );
    }

    #[pg_test]
    fn test_ech_pq_fastscan_small_dim_build_derives_group_size() {
        let _lock = env_var_test_lock();

        Spi::run(
            "CREATE TABLE ec_hnsw_pq_fastscan_small_dim_build (
                id bigint primary key,
                source real[],
                embedding ecvector
            )",
        )
        .expect("table creation should succeed");

        for id in 1..=8 {
            let source = (0..8)
                .map(|dim| format!("{:.6}", (((id * 11 + dim) as f32) * 0.08).sin()))
                .collect::<Vec<_>>()
                .join(", ");
            let embedding = (0..8)
                .map(|dim| format!("{:.6}", (((id * 5 + dim) as f32) * 0.06).cos()))
                .collect::<Vec<_>>()
                .join(", ");
            Spi::run(&format!(
                "INSERT INTO ec_hnsw_pq_fastscan_small_dim_build VALUES \
                 ({id}, ARRAY[{source}]::real[], encode_to_ecvector(ARRAY[{embedding}]::real[], 4, 42))"
            ))
            .expect("insert should succeed");
        }

        Spi::run(
            "CREATE INDEX ec_hnsw_pq_fastscan_small_dim_build_idx ON ec_hnsw_pq_fastscan_small_dim_build USING ec_hnsw \
             (embedding ecvector_ip_ops) WITH (m = 6, ef_construction = 80, build_source_column = 'source', storage_format = 'pq_fastscan')",
        )
        .expect("small-dimension index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_hnsw_pq_fastscan_small_dim_build_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");
        let (_block_count, metadata, data_pages) = unsafe { am::debug_index_pages(index_oid) };
        let grouped_codebook_count = data_pages
            .iter()
            .flat_map(|page| page.tuples.iter())
            .filter(|tuple| tuple.first().copied() == Some(am::page::TQ_GROUPED_CODEBOOK_TAG))
            .count();

        assert_eq!(metadata.format_version, am::page::INDEX_FORMAT_V2_GROUPED);
        assert_eq!(metadata.search_subvector_dim, 8);
        assert_eq!(metadata.search_subvector_count, 1);
        assert_eq!(grouped_codebook_count, 1);
        assert_ne!(
            metadata.grouped_codebook_head,
            am::page::ItemPointer::INVALID
        );
    }

    #[pg_test]
    fn test_ech_turboquant_storage_format_build_writes_scalar_pages() {
        let _lock = env_var_test_lock();

        Spi::run(
            "CREATE TABLE ec_hnsw_turboquant_storage_build (
                id bigint primary key,
                embedding ecvector
            )",
        )
        .expect("table creation should succeed");

        for id in 1..=16 {
            let embedding = (0..16)
                .map(|dim| format!("{:.6}", (((id * 7 + dim) as f32) * 0.06).cos()))
                .collect::<Vec<_>>()
                .join(", ");
            Spi::run(&format!(
                "INSERT INTO ec_hnsw_turboquant_storage_build VALUES \
                 ({id}, encode_to_ecvector(ARRAY[{embedding}]::real[], 4, 42))"
            ))
            .expect("insert should succeed");
        }

        Spi::run(
            "CREATE INDEX ec_hnsw_turboquant_storage_build_idx ON ec_hnsw_turboquant_storage_build USING ec_hnsw \
             (embedding ecvector_ip_ops) WITH (m = 6, ef_construction = 80, storage_format = 'turboquant')",
        )
        .expect("index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_hnsw_turboquant_storage_build_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");
        let reloptions = Spi::get_one::<Vec<String>>(
            "SELECT reloptions FROM pg_class WHERE oid = 'ec_hnsw_turboquant_storage_build_idx'::regclass",
        )
        .expect("SPI query should succeed")
        .expect("reloptions should exist");
        let (_block_count, metadata, data_pages) = unsafe { am::debug_index_pages(index_oid) };
        let (decoded_metadata, elements, neighbors) =
            decode_index_elements_and_neighbors(index_oid, code_len(16, 4));

        assert_eq!(decoded_metadata, metadata);
        assert!(reloptions.contains(&"storage_format=turboquant".to_string()));
        assert_eq!(
            metadata.format_version,
            am::page::INDEX_FORMAT_V3_TURBO_HOT_COLD
        );
        assert_eq!(metadata.transform_kind, am::page::TransformKind::Srht);
        assert_eq!(
            metadata.search_codec_kind,
            am::page::SearchCodecKind::ScalarQuantized
        );
        assert_eq!(
            metadata.rerank_codec_kind,
            am::page::RerankCodecKind::ScalarQuantized
        );
        assert_eq!(
            metadata.payload_flags & am::page::PAYLOAD_FLAG_COLD_RERANK_PAYLOAD,
            am::page::PAYLOAD_FLAG_COLD_RERANK_PAYLOAD
        );
        assert_eq!(
            metadata.grouped_codebook_head,
            am::page::ItemPointer::INVALID
        );
        assert_eq!(metadata.search_subvector_count, 0);
        assert_eq!(metadata.search_subvector_dim, 0);
        assert_eq!(elements.len(), 16);
        assert_eq!(neighbors.len(), 16);
        assert!(
            elements.iter().any(|(tid, _)| *tid == metadata.entry_point),
            "entry point should identify a scalar element tuple for a TurboQuant build",
        );
        assert!(elements
            .iter()
            .all(|(_, element)| element.heaptids.len() == 1));

        let page_tuples = data_pages
            .iter()
            .flat_map(|page| page.tuples.iter())
            .collect::<Vec<_>>();
        let scalar_count = page_tuples
            .iter()
            .filter(|tuple| tuple.first().copied() == Some(am::page::TQ_ELEMENT_TAG))
            .count();
        let turbo_hot_count = page_tuples
            .iter()
            .filter(|tuple| tuple.first().copied() == Some(am::page::TQ_TURBO_HOT_TAG))
            .count();
        let neighbor_count = page_tuples
            .iter()
            .filter(|tuple| tuple.first().copied() == Some(am::page::TQ_NEIGHBOR_TAG))
            .count();
        let grouped_hot_count = page_tuples
            .iter()
            .filter(|tuple| tuple.first().copied() == Some(am::page::TQ_GROUPED_HOT_TAG))
            .count();
        let rerank_count = page_tuples
            .iter()
            .filter(|tuple| tuple.first().copied() == Some(am::page::TQ_RERANK_TAG))
            .count();
        let codebook_count = page_tuples
            .iter()
            .filter(|tuple| tuple.first().copied() == Some(am::page::TQ_GROUPED_CODEBOOK_TAG))
            .count();

        assert_eq!(scalar_count, 0);
        assert_eq!(turbo_hot_count, 16);
        assert_eq!(neighbor_count, 16);
        assert_eq!(grouped_hot_count, 0);
        assert_eq!(rerank_count, 16);
        assert_eq!(codebook_count, 0);
    }

    #[pg_test]
    fn test_pq_fastscan_graph_reads_load_entry_and_neighbors() {
        let _lock = env_var_test_lock();

        Spi::run(
            "CREATE TABLE ec_hnsw_pq_fastscan_graph_reads (
                id bigint primary key,
                source real[],
                embedding ecvector
            )",
        )
        .expect("table creation should succeed");

        for id in 1..=16 {
            let source = (0..16)
                .map(|dim| format!("{:.6}", (((id * 23 + dim) as f32) * 0.03).sin()))
                .collect::<Vec<_>>()
                .join(", ");
            let embedding = (0..16)
                .map(|dim| format!("{:.6}", (((id * 13 + dim) as f32) * 0.06).cos()))
                .collect::<Vec<_>>()
                .join(", ");
            Spi::run(&format!(
                "INSERT INTO ec_hnsw_pq_fastscan_graph_reads VALUES \
                 ({id}, ARRAY[{source}]::real[], encode_to_ecvector(ARRAY[{embedding}]::real[], 4, 42))"
            ))
            .expect("insert should succeed");
        }

        Spi::run(
            "CREATE INDEX ec_hnsw_pq_fastscan_graph_reads_idx ON ec_hnsw_pq_fastscan_graph_reads USING ec_hnsw \
             (embedding ecvector_ip_ops) WITH (m = 6, ef_construction = 80, build_source_column = 'source', storage_format = 'pq_fastscan')",
        )
        .expect("index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_hnsw_pq_fastscan_graph_reads_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");

        let (_block_count, metadata, _data_pages) = unsafe { am::debug_index_pages(index_oid) };
        let layout = match am::graph::GraphStorageDescriptor::from_metadata(&metadata).unwrap() {
            am::graph::GraphStorageDescriptor::PqFastScan(layout) => layout,
            am::graph::GraphStorageDescriptor::TurboQuant { .. }
            | am::graph::GraphStorageDescriptor::TurboQuantHotCold(_) => {
                panic!("PqFastScan build should not decode as TurboQuant storage")
            }
        };

        let index_relation =
            unsafe { open_valid_ec_hnsw_index(index_oid, "test_pq_fastscan_graph_reads") };

        unsafe {
            am::graph::with_graph_storage_tuple(
                index_relation,
                metadata.entry_point,
                am::graph::GraphStorageDescriptor::PqFastScan(layout),
                |entry| match entry {
                    am::graph::GraphTupleRef::GroupedHot(tuple) => {
                        assert_eq!(tuple.search_code.len(), layout.search_code_len);
                        assert_eq!(tuple.collect_binary_words().len(), layout.binary_word_count);
                        assert!(tuple.heaptid_count() > 0);
                    }
                    am::graph::GraphTupleRef::Scalar(_) | am::graph::GraphTupleRef::TurboHot(_) => {
                        panic!("PqFastScan entry should decode as grouped-hot tuple")
                    }
                },
            );
        }

        let (entry, neighbors) = unsafe {
            am::graph::load_grouped_graph_adjacency(index_relation, metadata.entry_point, layout)
        };

        assert_eq!(entry.tid, metadata.entry_point);
        assert!(!entry.deleted);
        assert_eq!(entry.search_code.len(), layout.search_code_len);
        assert_eq!(entry.binary_words.len(), layout.binary_word_count);
        assert!(!entry.heaptids.is_empty());
        assert_ne!(entry.reranktid, am::page::ItemPointer::INVALID);
        assert_eq!(neighbors.tid, entry.neighbortid);
        assert!(neighbors.count > 0);
        assert!(
            neighbors
                .tids
                .iter()
                .any(|tid| *tid != am::page::ItemPointer::INVALID),
            "entry adjacency should include at least one real grouped-hot neighbor",
        );

        let first_neighbor_tid = neighbors
            .tids
            .iter()
            .copied()
            .find(|tid| *tid != am::page::ItemPointer::INVALID)
            .expect("grouped entry should expose a readable neighbor");
        let neighbor = unsafe {
            am::graph::load_grouped_graph_element(index_relation, first_neighbor_tid, layout)
        };

        assert_eq!(neighbor.search_code.len(), layout.search_code_len);
        assert_eq!(neighbor.binary_words.len(), layout.binary_word_count);
        assert!(!neighbor.heaptids.is_empty());
        assert_ne!(neighbor.reranktid, am::page::ItemPointer::INVALID);

        unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    }

    #[pg_test]
    fn test_pq_fastscan_graph_reads_load_cold_rerank_payload() {
        let _lock = env_var_test_lock();

        Spi::run(
            "CREATE TABLE ec_hnsw_pq_fastscan_rerank_reads (
                id bigint primary key,
                source real[],
                embedding ecvector
            )",
        )
        .expect("table creation should succeed");

        for id in 1..=16 {
            let source = (0..16)
                .map(|dim| format!("{:.6}", (((id * 41 + dim) as f32) * 0.03).sin()))
                .collect::<Vec<_>>()
                .join(", ");
            let embedding = (0..16)
                .map(|dim| format!("{:.6}", (((id * 17 + dim) as f32) * 0.05).cos()))
                .collect::<Vec<_>>()
                .join(", ");
            Spi::run(&format!(
                "INSERT INTO ec_hnsw_pq_fastscan_rerank_reads VALUES \
                 ({id}, ARRAY[{source}]::real[], encode_to_ecvector(ARRAY[{embedding}]::real[], 4, 42))"
            ))
            .expect("insert should succeed");
        }

        Spi::run(
            "CREATE INDEX ec_hnsw_pq_fastscan_rerank_reads_idx ON ec_hnsw_pq_fastscan_rerank_reads USING ec_hnsw \
             (embedding ecvector_ip_ops) WITH (m = 6, ef_construction = 80, build_source_column = 'source', storage_format = 'pq_fastscan')",
        )
        .expect("index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_hnsw_pq_fastscan_rerank_reads_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");

        let (_block_count, metadata, _data_pages) = unsafe { am::debug_index_pages(index_oid) };
        let layout = match am::graph::GraphStorageDescriptor::from_metadata(&metadata).unwrap() {
            am::graph::GraphStorageDescriptor::PqFastScan(layout) => layout,
            am::graph::GraphStorageDescriptor::TurboQuant { .. }
            | am::graph::GraphStorageDescriptor::TurboQuantHotCold(_) => {
                panic!("PqFastScan build should not decode as TurboQuant storage")
            }
        };

        let index_relation = unsafe {
            open_valid_ec_hnsw_index(
                index_oid,
                "test_pq_fastscan_graph_reads_load_cold_rerank_payload",
            )
        };
        let entry = unsafe {
            am::graph::load_grouped_graph_element(index_relation, metadata.entry_point, layout)
        };
        let rerank = unsafe {
            am::graph::load_grouped_rerank_payload(index_relation, entry.reranktid, layout)
        };

        assert_eq!(rerank.tid, entry.reranktid);
        assert_eq!(rerank.code.len(), layout.rerank_code_len);
        assert!(rerank.gamma.is_finite());
        assert!(
            rerank.code.iter().any(|byte| *byte != 0),
            "cold rerank payload should contain a non-empty scalar code",
        );

        unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    }

    #[pg_test]
    fn test_pq_fastscan_graph_reads_load_persisted_codebooks() {
        let _lock = env_var_test_lock();

        Spi::run(
            "CREATE TABLE ec_hnsw_pq_fastscan_codebook_reads (
                id bigint primary key,
                source real[],
                embedding ecvector
            )",
        )
        .expect("table creation should succeed");

        for id in 1..=16 {
            let source = (0..16)
                .map(|dim| format!("{:.6}", (((id * 43 + dim) as f32) * 0.02).sin()))
                .collect::<Vec<_>>()
                .join(", ");
            let embedding = (0..16)
                .map(|dim| format!("{:.6}", (((id * 19 + dim) as f32) * 0.03).cos()))
                .collect::<Vec<_>>()
                .join(", ");
            Spi::run(&format!(
                "INSERT INTO ec_hnsw_pq_fastscan_codebook_reads VALUES \
                 ({id}, ARRAY[{source}]::real[], encode_to_ecvector(ARRAY[{embedding}]::real[], 4, 42))"
            ))
            .expect("insert should succeed");
        }

        Spi::run(
            "CREATE INDEX ec_hnsw_pq_fastscan_codebook_reads_idx ON ec_hnsw_pq_fastscan_codebook_reads USING ec_hnsw \
             (embedding ecvector_ip_ops) WITH (m = 6, ef_construction = 80, build_source_column = 'source', storage_format = 'pq_fastscan')",
        )
        .expect("index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_hnsw_pq_fastscan_codebook_reads_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");

        let (_block_count, metadata, _data_pages) = unsafe { am::debug_index_pages(index_oid) };
        assert_ne!(
            metadata.grouped_codebook_head,
            am::page::ItemPointer::INVALID
        );

        let index_relation = unsafe {
            open_valid_ec_hnsw_index(
                index_oid,
                "test_pq_fastscan_graph_reads_load_persisted_codebooks",
            )
        };
        let model = unsafe { am::graph::load_grouped_codebook_model(index_relation, &metadata) };

        assert_eq!(model.head_tid, metadata.grouped_codebook_head);
        assert_eq!(model.group_count, metadata.search_subvector_count as usize);
        assert_eq!(model.group_size, metadata.search_subvector_dim as usize);
        assert_eq!(
            model.flat_codebooks.len(),
            model.group_count * model.group_size * crate::quant::grouped_pq::GROUPED_PQ_CENTROIDS
        );
        assert!(
            model.flat_codebooks.iter().all(|value| value.is_finite()),
            "persisted grouped codebooks should decode as finite f32 values",
        );

        let head = unsafe {
            am::graph::with_grouped_codebook_tuple(
                index_relation,
                model.head_tid,
                model.group_size * crate::quant::grouped_pq::GROUPED_PQ_CENTROIDS,
                |tuple| (tuple.group_index, tuple.nexttid),
            )
        };
        assert_eq!(head.0, 0);
        if model.group_count == 1 {
            assert_eq!(head.1, am::page::ItemPointer::INVALID);
        } else {
            assert_ne!(head.1, am::page::ItemPointer::INVALID);
        }

        let query = vec![0.5_f32; model.group_count * model.group_size];
        let lut = crate::quant::grouped_pq::build_grouped_pq_lut_f32(
            &query,
            &model.flat_codebooks,
            model.group_size,
        );
        assert_eq!(
            lut.len(),
            model.group_count * crate::quant::grouped_pq::GROUPED_PQ_CENTROIDS
        );

        unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    }

    #[pg_test]
    fn test_pq_fastscan_ordered_scan_smoke() {
        let _lock = env_var_test_lock();

        Spi::run(
            "CREATE TABLE ec_hnsw_pq_fastscan_runtime_reject (
                id bigint primary key,
                source real[],
                embedding ecvector
            )",
        )
        .expect("table creation should succeed");

        for id in 1..=16 {
            let source = (0..16)
                .map(|dim| format!("{:.6}", (((id * 23 + dim) as f32) * 0.03).cos()))
                .collect::<Vec<_>>()
                .join(", ");
            let embedding = (0..16)
                .map(|dim| format!("{:.6}", (((id * 13 + dim) as f32) * 0.02).sin()))
                .collect::<Vec<_>>()
                .join(", ");
            Spi::run(&format!(
                "INSERT INTO ec_hnsw_pq_fastscan_runtime_reject VALUES \
                 ({id}, ARRAY[{source}]::real[], encode_to_ecvector(ARRAY[{embedding}]::real[], 4, 42))"
            ))
            .expect("insert should succeed");
        }

        Spi::run(
            "CREATE INDEX ec_hnsw_pq_fastscan_runtime_reject_idx ON ec_hnsw_pq_fastscan_runtime_reject USING ec_hnsw \
             (embedding ecvector_ip_ops) WITH (m = 6, ef_construction = 80, build_source_column = 'source', storage_format = 'pq_fastscan')",
        )
        .expect("index creation should succeed");
        Spi::run("SET LOCAL enable_seqscan = off").expect("SET LOCAL should succeed");

        let observed = Spi::get_one::<i64>(
            "SELECT id FROM ec_hnsw_pq_fastscan_runtime_reject \
             ORDER BY embedding <#> ARRAY[0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, \
                                      0.9, 1.0, 1.1, 1.2, 1.3, 1.4, 1.5, 1.6]::real[] \
             LIMIT 1",
        )
        .expect("ordered SELECT should succeed");
        assert!(
            observed.is_some(),
            "PqFastScan ordered scans should emit at least one row",
        );
    }

    #[pg_test]
    fn test_pq_fastscan_ordered_scan_plan_smoke() {
        let _lock = env_var_test_lock();

        Spi::run(
            "CREATE TABLE ec_hnsw_pq_fastscan_runtime_enabled (
                id bigint primary key,
                source real[],
                embedding ecvector
            )",
        )
        .expect("table creation should succeed");

        for id in 1..=16 {
            let source = (0..16)
                .map(|dim| format!("{:.6}", (((id * 29 + dim) as f32) * 0.03).cos()))
                .collect::<Vec<_>>()
                .join(", ");
            let embedding = (0..16)
                .map(|dim| format!("{:.6}", (((id * 17 + dim) as f32) * 0.02).sin()))
                .collect::<Vec<_>>()
                .join(", ");
            Spi::run(&format!(
                "INSERT INTO ec_hnsw_pq_fastscan_runtime_enabled VALUES \
                 ({id}, ARRAY[{source}]::real[], encode_to_ecvector(ARRAY[{embedding}]::real[], 4, 42))"
            ))
            .expect("insert should succeed");
        }

        Spi::run(
            "CREATE INDEX ec_hnsw_pq_fastscan_runtime_enabled_idx ON ec_hnsw_pq_fastscan_runtime_enabled USING ec_hnsw \
             (embedding ecvector_ip_ops) WITH (m = 6, ef_construction = 80, build_source_column = 'source', storage_format = 'pq_fastscan')",
        )
        .expect("index creation should succeed");
        Spi::run("SET LOCAL enable_seqscan = off").expect("SET LOCAL should succeed");

        let plan = Spi::connect(|client| {
            let rows = client
                .select(
                    "EXPLAIN (COSTS OFF) \
                     SELECT id FROM ec_hnsw_pq_fastscan_runtime_enabled \
                     ORDER BY embedding <#> ARRAY[0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, \
                                              0.9, 1.0, 1.1, 1.2, 1.3, 1.4, 1.5, 1.6]::real[] \
                     LIMIT 3",
                    None,
                    &[],
                )
                .expect("EXPLAIN should succeed");
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
            plan.contains("Index Scan") || plan.contains("Index Only Scan"),
            "PqFastScan runtime smoke test should route through ec_hnsw when PqFastScan storage is selected: {plan}"
        );

        let ordered_ids = Spi::connect(|client| {
            client
                .select(
                    "SELECT id FROM ec_hnsw_pq_fastscan_runtime_enabled \
                     ORDER BY embedding <#> ARRAY[0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, \
                                              0.9, 1.0, 1.1, 1.2, 1.3, 1.4, 1.5, 1.6]::real[] \
                     LIMIT 3",
                    None,
                    &[],
                )
                .expect("ordered SELECT should succeed when PqFastScan storage is selected")
                .map(|row| {
                    row["id"]
                        .value::<i64>()
                        .expect("id should decode")
                        .expect("id should be non-null")
                })
                .collect::<Vec<_>>()
        });

        assert_eq!(ordered_ids.len(), 3);
        assert!(
            ordered_ids.windows(2).all(|pair| pair[0] != pair[1]),
            "PqFastScan runtime smoke test should emit distinct ids"
        );
    }

    #[pg_test]
    #[should_panic(
        expected = "ec_hnsw PqFastScan live rerank window must be between 1 and 64, got 0"
    )]
    fn test_pq_fastscan_runtime_rejects_invalid_live_window_env() {
        let _lock = env_var_test_lock();
        let _window_guard = ScopedEnvVar::set("TQVECTOR_PQ_FASTSCAN_SCAN_WINDOW", "0");

        Spi::run(
            "CREATE TABLE ec_hnsw_pq_fastscan_runtime_invalid_window (
                id bigint primary key,
                source real[],
                embedding ecvector
            )",
        )
        .expect("table creation should succeed");

        for id in 1..=16 {
            let source = (0..16)
                .map(|dim| format!("{:.6}", (((id * 29 + dim) as f32) * 0.03).cos()))
                .collect::<Vec<_>>()
                .join(", ");
            let embedding = (0..16)
                .map(|dim| format!("{:.6}", (((id * 17 + dim) as f32) * 0.02).sin()))
                .collect::<Vec<_>>()
                .join(", ");
            Spi::run(&format!(
                "INSERT INTO ec_hnsw_pq_fastscan_runtime_invalid_window VALUES \
                 ({id}, ARRAY[{source}]::real[], encode_to_ecvector(ARRAY[{embedding}]::real[], 4, 42))"
            ))
            .expect("insert should succeed");
        }

        Spi::run(
            "CREATE INDEX ec_hnsw_pq_fastscan_runtime_invalid_window_idx ON ec_hnsw_pq_fastscan_runtime_invalid_window USING ec_hnsw \
             (embedding ecvector_ip_ops) WITH (m = 6, ef_construction = 80, build_source_column = 'source', storage_format = 'pq_fastscan')",
        )
        .expect("index creation should succeed");
        Spi::run("SET LOCAL enable_seqscan = off").expect("SET LOCAL should succeed");

        let _ = Spi::get_one::<i64>(
            "SELECT id FROM ec_hnsw_pq_fastscan_runtime_invalid_window \
             ORDER BY embedding <#> ARRAY[0.5, 0.1, 0.4, -0.8, 0.0, 0.2, 0.4, 0.6, 0.8, 1.0, 1.2, 1.4, -0.1, -0.2, -0.3, -0.4]::real[] \
             LIMIT 1",
        )
        .expect("ordered scan should reach amrescan before rejecting invalid grouped window env");
    }

    #[pg_test]
    fn test_pq_fastscan_runtime_captures_rerank_scores() {
        let _lock = env_var_test_lock();

        Spi::run(
            "CREATE TABLE ec_hnsw_pq_fastscan_runtime_compare (
                id bigint primary key,
                source real[],
                embedding ecvector
            )",
        )
        .expect("table creation should succeed");

        for id in 1..=16 {
            let source = (0..16)
                .map(|dim| format!("{:.6}", (((id * 31 + dim) as f32) * 0.03).cos()))
                .collect::<Vec<_>>()
                .join(", ");
            let embedding = (0..16)
                .map(|dim| format!("{:.6}", (((id * 19 + dim) as f32) * 0.02).sin()))
                .collect::<Vec<_>>()
                .join(", ");
            Spi::run(&format!(
                "INSERT INTO ec_hnsw_pq_fastscan_runtime_compare VALUES \
                 ({id}, ARRAY[{source}]::real[], encode_to_ecvector(ARRAY[{embedding}]::real[], 4, 42))"
            ))
            .expect("insert should succeed");
        }

        Spi::run(
            "CREATE INDEX ec_hnsw_pq_fastscan_runtime_compare_idx ON ec_hnsw_pq_fastscan_runtime_compare USING ec_hnsw \
             (embedding ecvector_ip_ops) WITH (m = 6, ef_construction = 80, build_source_column = 'source', storage_format = 'pq_fastscan')",
        )
        .expect("index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_hnsw_pq_fastscan_runtime_compare_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");
        let query = vec![
            0.1_f32, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0, 1.1, 1.2, 1.3, 1.4, 1.5, 1.6,
        ];
        let observed = unsafe {
            am::debug_gettuple_scan_heap_tids_with_score_comparisons(index_oid, query.clone())
        };
        let exact_scores = (1..=16)
            .map(|id| {
                let source = (0..16)
                    .map(|dim| (((id * 31 + dim) as f32) * 0.03).cos())
                    .collect::<Vec<_>>();
                let heap_tid = heap_tid_for_row("ec_hnsw_pq_fastscan_runtime_compare", id);
                (
                    (heap_tid.block_number, heap_tid.offset_number),
                    -dot_product(&query, &source),
                )
            })
            .collect::<HashMap<_, _>>();

        assert!(
            !observed.is_empty(),
            "PqFastScan runtime comparison path should emit at least one ordered result"
        );
        for (heap_tid, _approx_score, comparison_score, _approx_rank) in observed {
            let comparison_score = comparison_score
                .expect("PqFastScan emitted results should carry an exact rerank comparison score");
            let expected = exact_scores
                .get(&heap_tid)
                .copied()
                .expect("every emitted heap tid should map back to an exact source score");
            assert_f32_close(
                comparison_score,
                expected,
                "PqFastScan comparison score should match the source-backed exact rerank score for the emitted tuple",
            );
        }
    }

    type DebugScanComparisonRow = ((u32, u16), f32, Option<f32>, Option<i32>);
    type DebugGroupedComparisonRow = ((u32, u16), i32, f32, Option<f32>, Option<i32>, Option<i32>);

