    fn custom_scan_planner_json_explain(sql: &str) -> serde_json::Value {
        let json_plan = Spi::get_one::<String>(sql)
            .expect("CustomScan planner JSON EXPLAIN should succeed")
            .expect("CustomScan planner JSON EXPLAIN should return a row");
        custom_scan_json_explain_root_plan(&json_plan)
    }

    fn custom_scan_plan_has_node(plan: &serde_json::Value, node_type: &str) -> bool {
        plan.get("Node Type")
            .and_then(|value| value.as_str())
            .is_some_and(|value| value == node_type)
            || plan
                .get("Plans")
                .and_then(|value| value.as_array())
                .is_some_and(|children| {
                    children
                        .iter()
                        .any(|child| custom_scan_plan_has_node(child, node_type))
                })
    }

    fn custom_scan_child_has_node(plan: &serde_json::Value, node_type: &str) -> bool {
        plan.get("Plans")
            .and_then(|value| value.as_array())
            .is_some_and(|children| {
                children
                    .iter()
                    .any(|child| custom_scan_plan_has_node(child, node_type))
            })
    }

    fn custom_scan_plan_has_parent_child(
        plan: &serde_json::Value,
        parent_node_type: &str,
        child_node_type: &str,
    ) -> bool {
        let current_is_parent = plan
            .get("Node Type")
            .and_then(|value| value.as_str())
            .is_some_and(|value| value == parent_node_type);
        if current_is_parent && custom_scan_child_has_node(plan, child_node_type) {
            return true;
        }
        plan.get("Plans")
            .and_then(|value| value.as_array())
            .is_some_and(|children| {
                children.iter().any(|child| {
                    custom_scan_plan_has_parent_child(child, parent_node_type, child_node_type)
                })
            })
    }

    fn rewrite_all_leaf_placements_to_remote(index_name: &str, node_id: u32) {
        let index_oid = Spi::get_one::<pg_sys::Oid>(&format!(
            "SELECT '{index_name}'::regclass::oid"
        ))
        .expect("planner exclusion index oid query should succeed")
        .expect("planner exclusion index oid should exist");
        let leaf_pids = Spi::get_one::<Vec<i64>>(&format!(
            "SELECT array_agg(leaf_pid ORDER BY leaf_pid) \
               FROM ec_spire_index_leaf_snapshot('{index_name}'::regclass)"
        ))
        .expect("planner exclusion leaf pid query should succeed")
        .expect("planner exclusion leaf pids should exist");

        unsafe {
            for pid in leaf_pids {
                am::debug_spire_rewrite_placement_node(index_oid, pid as u64, node_id);
            }
        }
    }

    #[pg_test]
    fn test_ec_spire_customscan_not_below_mergeappend_sql() {
        Spi::run(
            "CREATE TABLE ec_spire_customscan_mergeappend_parent_sql \
                 (id bigint not null, title text not null, embedding ecvector) \
                 PARTITION BY RANGE (id); \
             CREATE TABLE ec_spire_customscan_mergeappend_p1_sql \
                 PARTITION OF ec_spire_customscan_mergeappend_parent_sql \
                 FOR VALUES FROM (0) TO (100); \
             CREATE TABLE ec_spire_customscan_mergeappend_p2_sql \
                 PARTITION OF ec_spire_customscan_mergeappend_parent_sql \
                 FOR VALUES FROM (100) TO (200); \
             INSERT INTO ec_spire_customscan_mergeappend_parent_sql \
                 (id, title, embedding) VALUES \
                 (1, 'mergeappend alpha', encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
                 (2, 'mergeappend beta', encode_to_ecvector(ARRAY[0.8, 0.2], 4, 42)), \
                 (101, 'mergeappend gamma', encode_to_ecvector(ARRAY[0.7, 0.3], 4, 42)), \
                 (102, 'mergeappend delta', encode_to_ecvector(ARRAY[0.6, 0.4], 4, 42)); \
             CREATE INDEX ec_spire_customscan_mergeappend_p1_idx \
                 ON ec_spire_customscan_mergeappend_p1_sql USING ec_spire \
                 (embedding ecvector_spire_ip_ops) WITH (nlists = 2, nprobe = 2); \
             CREATE INDEX ec_spire_customscan_mergeappend_p2_idx \
                 ON ec_spire_customscan_mergeappend_p2_sql USING ec_spire \
                 (embedding ecvector_spire_ip_ops) WITH (nlists = 2, nprobe = 2)",
        )
        .expect("MergeAppend planner exclusion fixture should be created");
        rewrite_all_leaf_placements_to_remote("ec_spire_customscan_mergeappend_p1_idx", 2);
        rewrite_all_leaf_placements_to_remote("ec_spire_customscan_mergeappend_p2_idx", 3);

        Spi::run("SET LOCAL enable_seqscan = off")
            .expect("MergeAppend planner exclusion should disable seqscan");
        let plan = custom_scan_planner_json_explain(
            "EXPLAIN (FORMAT JSON, COSTS OFF) \
             SELECT id \
               FROM ec_spire_customscan_mergeappend_parent_sql \
              ORDER BY embedding <#> encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42) \
              LIMIT 4",
        );

        assert!(
            custom_scan_plan_has_node(&plan, "Merge Append"),
            "partitioned ORDER BY LIMIT fixture should exercise MergeAppend: {plan:?}"
        );
        assert!(
            !custom_scan_plan_has_parent_child(&plan, "Merge Append", "Custom Scan"),
            "MergeAppend must not be planned above SPIRE CustomScan because \
             CustomScan does not advertise MarkPos/RestrPos callbacks: {plan:?}"
        );
    }

    #[pg_test]
    fn test_ec_spire_customscan_not_inner_rescan_nested_loop_sql() {
        let fixture_prefix = "ec_spire_customscan_nested_loop_exclusion";
        Spi::run(
            "CREATE TABLE ec_spire_customscan_nested_loop_outer_sql \
                 (outer_id bigint primary key); \
             INSERT INTO ec_spire_customscan_nested_loop_outer_sql (outer_id) \
             VALUES (1), (2)",
        )
        .expect("Nested Loop planner exclusion outer fixture should be created");

        let mut loopback_client =
            postgres::Client::connect(&current_pg_test_loopback_conninfo(), postgres::NoTls)
                .expect("Nested Loop planner exclusion loopback connection should succeed");
        let fixture = setup_custom_scan_execution_fixture(
            &mut loopback_client,
            fixture_prefix,
            "(1, 'nested-loop alpha', encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, 'nested-loop beta', encode_to_ecvector(ARRAY[0.8, 0.2], 4, 42))",
            "(1, 'nested-loop alpha', encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, 'nested-loop beta', encode_to_ecvector(ARRAY[0.8, 0.2], 4, 42))",
        );
        route_custom_scan_fixture_to_remote(
            &fixture,
            2,
            31,
            "spire/remote/customscan_nested_loop_exclusion",
            &format!("{fixture_prefix}_remote_idx"),
        );

        Spi::run("SET LOCAL enable_hashjoin = off; SET LOCAL enable_mergejoin = off")
            .expect("Nested Loop planner exclusion should force Nested Loop join choice");
        let plan = custom_scan_planner_json_explain(
            "EXPLAIN (FORMAT JSON, COSTS OFF) \
             SELECT outer_rows.outer_id, inner_rows.id \
               FROM ec_spire_customscan_nested_loop_outer_sql outer_rows \
               JOIN LATERAL ( \
                    SELECT id \
                      FROM ec_spire_customscan_nested_loop_exclusion_coord_sql inner_rows \
                     WHERE inner_rows.id >= outer_rows.outer_id \
                     ORDER BY embedding <#> encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42) \
                     LIMIT 1 \
               ) inner_rows ON true",
        );

        assert!(
            custom_scan_plan_has_node(&plan, "Nested Loop"),
            "correlated LATERAL fixture should exercise a Nested Loop plan: {plan:?}"
        );
        assert!(
            !custom_scan_plan_has_parent_child(&plan, "Nested Loop", "Custom Scan"),
            "Nested Loop must not rescan a SPIRE CustomScan inner path because \
             CustomScan does not advertise MarkPos/RestrPos callbacks: {plan:?}"
        );
    }
