    #[pg_test]
    fn test_ec_spire_dml_frontdoor_non_pk_select_passes_through_sql() {
        Spi::run(
            "CREATE TABLE ec_spire_dml_non_pk_select_sql \
             (id bigint primary key, title text not null, embedding ecvector)",
        )
        .expect("DML non-PK SELECT table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_dml_non_pk_select_sql (id, title, embedding) VALUES \
             (1, 'keep-alpha', encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, 'drop-beta', encode_to_ecvector(ARRAY[0.0, 1.0], 4, 42)), \
             (3, 'keep-gamma', encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("DML non-PK SELECT seed insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_dml_non_pk_select_idx \
             ON ec_spire_dml_non_pk_select_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("DML non-PK SELECT ec_spire index creation should succeed");

        let relation_status = Spi::get_one::<String>(
            "SELECT status FROM ec_spire_dml_frontdoor_relation_context(\
                'ec_spire_dml_non_pk_select_sql'::regclass)",
        )
        .expect("DML non-PK SELECT relation context should succeed")
        .expect("DML non-PK SELECT relation context should return a status");
        assert_eq!(relation_status, "relation_context_ready");

        let plan = Spi::connect(|client| {
            let rows = client
                .select(
                    "EXPLAIN (COSTS OFF) \
                     SELECT id, title FROM ec_spire_dml_non_pk_select_sql \
                     WHERE title LIKE 'keep-%' ORDER BY id",
                    None,
                    &[],
                )
                .expect("DML non-PK SELECT EXPLAIN should succeed");
            let mut lines = Vec::new();
            for row in rows {
                lines.push(
                    row.get::<String>(1)
                        .expect("DML non-PK SELECT plan row should decode")
                        .expect("DML non-PK SELECT plan row should not be NULL"),
                );
            }
            lines.join("\n")
        });
        assert!(
            plan.contains("Seq Scan") || plan.contains("Index Scan"),
            "expected an ordinary PostgreSQL scan for non-PK SELECT:\n{plan}"
        );
        assert!(
            !plan.contains("Custom Scan (EcSpireDistributedScan)"),
            "non-PK SELECT must pass through without SPIRE CustomScan rewrite:\n{plan}"
        );

        let rows = Spi::get_one::<String>(
            "SELECT string_agg(id::text || ':' || title, ',' ORDER BY id) \
             FROM ec_spire_dml_non_pk_select_sql \
             WHERE title LIKE 'keep-%'",
        )
        .expect("DML non-PK SELECT should succeed")
        .expect("DML non-PK SELECT should return matching rows");
        assert_eq!(rows, "1:keep-alpha,3:keep-gamma");
    }

    #[pg_test]
    fn test_ec_spire_dml_frontdoor_composite_pk_rejected_sql() {
        Spi::run(
            "CREATE TABLE ec_spire_dml_composite_pk_sql \
             (tenant_id bigint not null, id bigint not null, title text not null, \
              embedding ecvector, primary key (tenant_id, id))",
        )
        .expect("DML composite-PK table creation should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_dml_composite_pk_idx \
             ON ec_spire_dml_composite_pk_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("DML composite-PK ec_spire index creation should succeed");

        let context = "FROM ec_spire_dml_frontdoor_relation_context(\
            'ec_spire_dml_composite_pk_sql'::regclass)";
        let status = Spi::get_one::<String>(&format!("SELECT status {context}"))
            .expect("DML composite-PK relation context status should succeed")
            .expect("DML composite-PK relation context status should exist");
        let next_step = Spi::get_one::<String>(&format!("SELECT next_step {context}"))
            .expect("DML composite-PK relation context next step should succeed")
            .expect("DML composite-PK relation context next step should exist");
        let distributed =
            Spi::get_one::<bool>(&format!("SELECT ec_spire_distributed_table {context}"))
                .expect("DML composite-PK distributed flag should succeed")
                .expect("DML composite-PK distributed flag should exist");
        let pk_column_is_null = Spi::get_one::<bool>(&format!(
            "SELECT pk_column IS NULL {context}"
        ))
        .expect("DML composite-PK pk_column null check should succeed")
        .expect("DML composite-PK pk_column null check should exist");
        let pk_type_is_null = Spi::get_one::<bool>(&format!(
            "SELECT pk_type IS NULL {context}"
        ))
        .expect("DML composite-PK pk_type null check should succeed")
        .expect("DML composite-PK pk_type null check should exist");

        assert_eq!(status, "unsupported_pk_shape");
        assert_eq!(
            next_step,
            "define one bigint primary-key column for ADR-069 v1 routing"
        );
        assert!(!distributed);
        assert!(pk_column_is_null);
        assert!(pk_type_is_null);
    }
