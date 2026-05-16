    fn spire_cost_snapshot_f64(index_name: &str, column_name: &str) -> f64 {
        Spi::get_one::<f64>(&format!(
            "SELECT {column_name} FROM ec_spire_index_cost_snapshot('{index_name}'::regclass)"
        ))
        .expect("SPIRE cost snapshot query should succeed")
        .expect("SPIRE cost snapshot value should exist")
    }

    fn spire_cost_tuning_snapshot_f64(index_name: &str, column_name: &str) -> f64 {
        Spi::get_one::<f64>(&format!(
            "SELECT {column_name} FROM ec_spire_index_cost_tuning_snapshot('{index_name}'::regclass)"
        ))
        .expect("SPIRE cost tuning snapshot query should succeed")
        .expect("SPIRE cost tuning snapshot value should exist")
    }

    fn assert_spire_cost_close(label: &str, actual: f64, expected: f64) {
        assert!(
            (actual - expected).abs() < 1e-9,
            "{label} expected {expected}, got {actual}"
        );
    }

    fn spire_setting_f64(name: &str) -> f64 {
        Spi::get_one::<String>(&format!("SHOW {name}"))
            .expect("SPIRE setting query should succeed")
            .expect("SPIRE setting should exist")
            .parse::<f64>()
            .expect("SPIRE setting should parse as f64")
    }

    fn spire_customscan_explain_total_cost(table_name: &str) -> f64 {
        let plan = Spi::connect(|client| {
            let rows = client
                .select(
                    &format!(
                        "EXPLAIN \
                         SELECT id FROM {table_name} \
                         ORDER BY embedding <#> ARRAY[1.0, 0.0]::real[] \
                         LIMIT 4"
                    ),
                    None,
                    &[],
                )
                .expect("SPIRE EXPLAIN should succeed")
                .first();
            let mut lines = Vec::new();
            for row in rows {
                lines.push(
                    row.get::<String>(1)
                        .expect("SPIRE EXPLAIN row should decode")
                        .expect("SPIRE EXPLAIN row should not be NULL"),
                );
            }
            lines.join("\n")
        });
        let custom_scan_line = plan
            .lines()
            .find(|line| line.contains("Custom Scan (EcSpireDistributedScan)"))
            .unwrap_or_else(|| panic!("SPIRE EXPLAIN should use CustomScan:\n{plan}"));
        let cost_start = custom_scan_line
            .find("(cost=")
            .expect("CustomScan line should include cost")
            + "(cost=".len();
        let cost_text = &custom_scan_line[cost_start..];
        let (_, total_and_rest) = cost_text
            .split_once("..")
            .expect("CustomScan cost should include startup and total");
        let total_text = total_and_rest
            .split_whitespace()
            .next()
            .expect("CustomScan total cost should exist");
        total_text
            .parse::<f64>()
            .expect("CustomScan total cost should parse")
    }

    #[pg_test]
    fn test_ec_spire_cost_tuning_snapshot_reflects_session_gucs_sql() {
        Spi::run(
            "CREATE TABLE ec_spire_cost_tuning_snapshot_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("cost tuning snapshot table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_cost_tuning_snapshot_sql (id, embedding) \
             SELECT id, encode_to_ecvector(\
                    ARRAY[(id::real / 32.0)::real, ((33 - id)::real / 32.0)::real], 4, 42) \
               FROM generate_series(1, 32) AS id",
        )
        .expect("cost tuning snapshot corpus insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_cost_tuning_snapshot_idx \
             ON ec_spire_cost_tuning_snapshot_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) \
             WITH (nlists = 4, nprobe = 4, rerank_width = 0, storage_format = 'rabitq')",
        )
        .expect("cost tuning snapshot index creation should succeed");

        let index_name = "ec_spire_cost_tuning_snapshot_idx";
        let baseline_total = spire_cost_snapshot_f64(index_name, "modeled_total_cost");

        Spi::run("SET LOCAL ec_spire.cost_routing_dimension_scale = 2.0")
            .expect("routing cost GUC should set");
        Spi::run("SET LOCAL ec_spire.cost_leaf_dimension_scale = 2.0")
            .expect("leaf cost GUC should set");
        Spi::run("SET LOCAL ec_spire.cost_index_page_scale = 2.0")
            .expect("index page cost GUC should set");
        Spi::run("SET LOCAL ec_spire.cost_local_store_page_fanout_scale = 2.0")
            .expect("local-store fanout cost GUC should set");
        Spi::run("SET LOCAL ec_spire.cost_storage_scoring_multiplier = 2.0")
            .expect("storage scoring cost GUC should set");
        Spi::run("SET LOCAL ec_spire.cost_rerank_multiplier = 2.0")
            .expect("rerank cost GUC should set");

        let tuned_total = spire_cost_snapshot_f64(index_name, "modeled_total_cost");
        assert!(
            tuned_total > baseline_total,
            "tuned SQL cost snapshot should increase modeled total cost: \
             baseline={baseline_total}, tuned={tuned_total}"
        );

        for column_name in [
            "cost_routing_dimension_scale",
            "cost_leaf_dimension_scale",
            "cost_index_page_scale",
            "cost_local_store_page_fanout_scale",
            "cost_storage_scoring_multiplier",
            "cost_rerank_multiplier",
        ] {
            assert_spire_cost_close(
                column_name,
                spire_cost_tuning_snapshot_f64(index_name, column_name),
                2.0,
            );
        }
        assert_spire_cost_close(
            "effective_storage_scoring_multiplier",
            spire_cost_tuning_snapshot_f64(index_name, "effective_storage_scoring_multiplier"),
            0.90,
        );
        assert_spire_cost_close(
            "effective_rerank_multiplier",
            spire_cost_tuning_snapshot_f64(index_name, "effective_rerank_multiplier"),
            2.0,
        );
    }

    #[pg_test]
    fn test_ec_spire_cost_gucs_reflect_in_explain_sql() {
        Spi::run(
            "CREATE TABLE ec_spire_cost_tuning_explain_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("cost tuning EXPLAIN table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_cost_tuning_explain_sql (id, embedding) \
             SELECT id, encode_to_ecvector(\
                    ARRAY[(id::real / 64.0)::real, ((65 - id)::real / 64.0)::real], 4, 42) \
               FROM generate_series(1, 64) AS id",
        )
        .expect("cost tuning EXPLAIN corpus insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_cost_tuning_explain_idx \
             ON ec_spire_cost_tuning_explain_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) \
             WITH (nlists = 4, nprobe = 4, rerank_width = 0, storage_format = 'rabitq')",
        )
        .expect("cost tuning EXPLAIN index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_spire_cost_tuning_explain_idx'::regclass::oid",
        )
        .expect("cost tuning EXPLAIN index oid query should succeed")
        .expect("cost tuning EXPLAIN index oid should exist");
        let selected_pid = Spi::get_one::<i64>(
            "SELECT leaf_pid FROM \
             ec_spire_index_leaf_snapshot('ec_spire_cost_tuning_explain_idx'::regclass) \
             ORDER BY leaf_pid LIMIT 1",
        )
        .expect("cost tuning EXPLAIN leaf pid query should succeed")
        .expect("cost tuning EXPLAIN leaf pid should exist");
        unsafe { am::debug_spire_rewrite_placement_node(index_oid, selected_pid as u64, 2) };

        Spi::run("SET enable_seqscan = off").expect("disable seqscan should succeed");
        Spi::run("SET enable_indexscan = off").expect("disable indexscan should succeed");

        let index_name = "ec_spire_cost_tuning_explain_idx";
        let table_name = "ec_spire_cost_tuning_explain_sql";
        let settings = [
            "ec_spire.cost_routing_dimension_scale",
            "ec_spire.cost_leaf_dimension_scale",
            "ec_spire.cost_index_page_scale",
            "ec_spire.cost_local_store_page_fanout_scale",
            "ec_spire.cost_storage_scoring_multiplier",
            "ec_spire.cost_rerank_multiplier",
        ];

        for setting in settings {
            Spi::run(&format!("SET LOCAL {setting} TO DEFAULT"))
                .expect("baseline SPIRE cost GUC should reset");
        }
        let baseline_snapshot_total = spire_cost_snapshot_f64(index_name, "modeled_total_cost");
        let baseline_explain_total = spire_customscan_explain_total_cost(table_name);

        for setting in settings {
            let default_value = spire_setting_f64(setting);
            let tuned_value = default_value * 2.0;
            Spi::run(&format!("SET LOCAL {setting} = {tuned_value}"))
                .expect("SPIRE cost GUC should set to tuned value");

            let tuned_snapshot_total = spire_cost_snapshot_f64(index_name, "modeled_total_cost");
            let tuned_explain_total = spire_customscan_explain_total_cost(table_name);
            let snapshot_delta = tuned_snapshot_total - baseline_snapshot_total;
            let explain_delta = tuned_explain_total - baseline_explain_total;

            assert!(
                explain_delta > 0.0,
                "{setting} should increase CustomScan EXPLAIN cost: \
                 baseline={baseline_explain_total}, tuned={tuned_explain_total}"
            );
            assert!(
                (explain_delta - snapshot_delta).abs() < 0.02,
                "{setting} EXPLAIN cost delta should track the cost model delta: \
                 explain_delta={explain_delta}, snapshot_delta={snapshot_delta}"
            );

            Spi::run(&format!("SET LOCAL {setting} TO DEFAULT"))
                .expect("SPIRE cost GUC should reset after tuned assertion");
        }
    }
