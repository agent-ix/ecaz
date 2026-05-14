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
