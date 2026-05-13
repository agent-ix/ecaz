    #[pg_test]
    fn test_ec_spire_dml_frontdoor_hook_status_installed_pass_through() {
        let status_from = "FROM ec_spire_dml_frontdoor_hook_status()";
        let hook_name = Spi::get_one::<String>(&format!("SELECT hook_name {status_from}"))
            .expect("DML frontdoor hook name query should succeed")
            .expect("DML frontdoor hook name should exist");
        let planner_hook_installed =
            Spi::get_one::<bool>(&format!("SELECT planner_hook_installed {status_from}"))
                .expect("DML frontdoor planner hook query should succeed")
                .expect("DML frontdoor planner hook value should exist");
        let query_shape_classifier_enabled = Spi::get_one::<bool>(&format!(
            "SELECT query_shape_classifier_enabled {status_from}"
        ))
        .expect("DML frontdoor query classifier query should succeed")
        .expect("DML frontdoor query classifier value should exist");
        let query_shape_classifier_invoked_by_hook = Spi::get_one::<bool>(&format!(
            "SELECT query_shape_classifier_invoked_by_hook {status_from}"
        ))
        .expect("DML frontdoor hook classifier observation query should succeed")
        .expect("DML frontdoor hook classifier observation value should exist");
        let plan_rewrite_enabled =
            Spi::get_one::<bool>(&format!("SELECT plan_rewrite_enabled {status_from}"))
                .expect("DML frontdoor plan rewrite query should succeed")
                .expect("DML frontdoor plan rewrite value should exist");
        let unsupported_shape_fail_closed_enabled = Spi::get_one::<bool>(&format!(
            "SELECT unsupported_shape_fail_closed_enabled {status_from}"
        ))
        .expect("DML frontdoor fail-closed guard query should succeed")
        .expect("DML frontdoor fail-closed guard value should exist");
        let status = Spi::get_one::<String>(&format!("SELECT status {status_from}"))
            .expect("DML frontdoor status query should succeed")
            .expect("DML frontdoor status should exist");

        assert_eq!(hook_name, "ec_spire_dml_frontdoor_planner_hook");
        assert!(planner_hook_installed);
        assert!(query_shape_classifier_enabled);
        assert!(unsupported_shape_fail_closed_enabled);
        assert!(plan_rewrite_enabled);
        assert_eq!(
            query_shape_classifier_invoked_by_hook,
            status != "fail_closed_guard_ready"
        );
        assert!(
            status == "fail_closed_guard_ready"
                || status == "pass_through_until_rewrite"
                || status == "pass_through_not_spire_frontdoor"
                || status == "planner_error_fail_closed"
                || status == "plan_tree_replaced_customscan",
            "{status}"
        );
    }
    #[pg_test]
    fn test_ec_spire_dml_frontdoor_relation_context_sql() {
        Spi::run(
            "CREATE TABLE ec_spire_dml_frontdoor_context_sql \
             (id bigint primary key, title text not null, embedding ecvector)",
        )
        .expect("DML frontdoor context table creation should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_dml_frontdoor_context_idx \
             ON ec_spire_dml_frontdoor_context_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops)",
        )
        .expect("DML frontdoor context ec_spire index creation should succeed");

        let context = "FROM ec_spire_dml_frontdoor_relation_context(\
                       'ec_spire_dml_frontdoor_context_sql'::regclass)";
        let status = Spi::get_one::<String>(&format!("SELECT status {context}"))
            .expect("DML frontdoor relation context status query should succeed")
            .expect("DML frontdoor relation context status should exist");
        let distributed =
            Spi::get_one::<bool>(&format!("SELECT ec_spire_distributed_table {context}"))
                .expect("DML frontdoor relation context distributed query should succeed")
                .expect("DML frontdoor relation context distributed value should exist");
        let pk_column = Spi::get_one::<String>(&format!("SELECT pk_column {context}"))
            .expect("DML frontdoor relation context PK query should succeed")
            .expect("DML frontdoor relation context PK should exist");
        let pk_type = Spi::get_one::<String>(&format!("SELECT pk_type {context}"))
            .expect("DML frontdoor relation context PK type query should succeed")
            .expect("DML frontdoor relation context PK type should exist");
        let ordinary_column_count =
            Spi::get_one::<i64>(&format!("SELECT ordinary_column_count {context}"))
                .expect("DML frontdoor relation context column count query should succeed")
                .expect("DML frontdoor relation context column count should exist");
        let embedding_columns = Spi::get_one::<String>(&format!(
            "SELECT array_to_string(embedding_columns, ',') {context}"
        ))
        .expect("DML frontdoor relation context embedding query should succeed")
        .expect("DML frontdoor relation context embedding columns should exist");

        assert_eq!(status, "relation_context_ready");
        assert!(distributed);
        assert_eq!(pk_column, "id");
        assert_eq!(pk_type, "bigint");
        assert_eq!(ordinary_column_count, 3);
        assert_eq!(embedding_columns, "embedding");

        Spi::run(
            "CREATE TABLE ec_spire_dml_frontdoor_include_context_sql \
             (id bigint primary key, title text not null, embedding ecvector, source_identity bytea not null)",
        )
        .expect("DML frontdoor INCLUDE context table creation should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_dml_frontdoor_include_context_idx \
             ON ec_spire_dml_frontdoor_include_context_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) INCLUDE (source_identity) \
             WITH (source_identity = 'include')",
        )
        .expect("DML frontdoor INCLUDE context ec_spire index creation should succeed");
        let include_context = "FROM ec_spire_dml_frontdoor_relation_context(\
                               'ec_spire_dml_frontdoor_include_context_sql'::regclass)";
        let include_embedding_columns = Spi::get_one::<String>(&format!(
            "SELECT array_to_string(embedding_columns, ',') {include_context}"
        ))
        .expect("DML frontdoor INCLUDE context embedding query should succeed")
        .expect("DML frontdoor INCLUDE context embedding columns should exist");
        assert_eq!(include_embedding_columns, "embedding");

        let catalog_context = "FROM ec_spire_dml_frontdoor_relation_context_catalog(\
                               'ec_spire_dml_frontdoor_context_sql'::regclass)";
        let catalog_status = Spi::get_one::<String>(&format!("SELECT status {catalog_context}"))
            .expect("DML frontdoor catalog relation context status query should succeed")
            .expect("DML frontdoor catalog relation context status should exist");
        let catalog_pk_column =
            Spi::get_one::<String>(&format!("SELECT pk_column {catalog_context}"))
                .expect("DML frontdoor catalog relation context PK query should succeed")
                .expect("DML frontdoor catalog relation context PK should exist");
        let catalog_embedding_columns = Spi::get_one::<String>(&format!(
            "SELECT array_to_string(embedding_columns, ',') {catalog_context}"
        ))
        .expect("DML frontdoor catalog relation context embedding query should succeed")
        .expect("DML frontdoor catalog relation context embedding columns should exist");
        assert_eq!(catalog_status, status);
        assert_eq!(catalog_pk_column, pk_column);
        assert_eq!(catalog_embedding_columns, embedding_columns);
    }

    #[pg_test]
    fn test_ec_spire_dml_context_cache_invalidation_sql() {
        Spi::run(
            "CREATE TABLE ec_spire_dml_frontdoor_context_cache_sql \
             (id bigint primary key, title text not null, embedding ecvector)",
        )
        .expect("DML frontdoor context cache table creation should succeed");

        let cache = "FROM ec_spire_dml_frontdoor_relation_context_cache()";
        let context = "FROM ec_spire_dml_frontdoor_relation_context_catalog(\
                       'ec_spire_dml_frontdoor_context_cache_sql'::regclass)";
        let before_hits = Spi::get_one::<i64>(&format!("SELECT hit_count {cache}"))
            .expect("DML frontdoor context cache hit count query should succeed")
            .expect("DML frontdoor context cache hit count should exist");
        let before_misses = Spi::get_one::<i64>(&format!("SELECT miss_count {cache}"))
            .expect("DML frontdoor context cache miss count query should succeed")
            .expect("DML frontdoor context cache miss count should exist");

        let initial_status = Spi::get_one::<String>(&format!("SELECT status {context}"))
            .expect("DML frontdoor uncached context query should succeed")
            .expect("DML frontdoor uncached context status should exist");
        let after_first_misses = Spi::get_one::<i64>(&format!("SELECT miss_count {cache}"))
            .expect("DML frontdoor first miss count query should succeed")
            .expect("DML frontdoor first miss count should exist");
        assert_eq!(initial_status, "no_ec_spire_index");
        assert_eq!(after_first_misses, before_misses + 1);

        let cached_status = Spi::get_one::<String>(&format!("SELECT status {context}"))
            .expect("DML frontdoor cached context query should succeed")
            .expect("DML frontdoor cached context status should exist");
        let after_second_hits = Spi::get_one::<i64>(&format!("SELECT hit_count {cache}"))
            .expect("DML frontdoor second hit count query should succeed")
            .expect("DML frontdoor second hit count should exist");
        assert_eq!(cached_status, "no_ec_spire_index");
        assert_eq!(after_second_hits, before_hits + 1);

        Spi::run(
            "CREATE INDEX ec_spire_dml_frontdoor_context_cache_idx \
             ON ec_spire_dml_frontdoor_context_cache_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops)",
        )
        .expect("DML frontdoor context cache ec_spire index creation should succeed");

        let refreshed_status = Spi::get_one::<String>(&format!("SELECT status {context}"))
            .expect("DML frontdoor refreshed context query should succeed")
            .expect("DML frontdoor refreshed context status should exist");
        let refreshed_index_oid =
            Spi::get_one::<pg_sys::Oid>(&format!("SELECT index_oid {context}"))
                .expect("DML frontdoor refreshed context index query should succeed")
                .expect("DML frontdoor refreshed context index should exist");
        let after_refresh_misses = Spi::get_one::<i64>(&format!("SELECT miss_count {cache}"))
            .expect("DML frontdoor refresh miss count query should succeed")
            .expect("DML frontdoor refresh miss count should exist");
        let cache_status = Spi::get_one::<String>(&format!("SELECT status {cache}"))
            .expect("DML frontdoor cache status query should succeed")
            .expect("DML frontdoor cache status should exist");

        assert_eq!(refreshed_status, "relation_context_ready");
        assert_ne!(refreshed_index_oid, pg_sys::InvalidOid);
        assert_eq!(after_refresh_misses, before_misses + 2);
        assert_eq!(cache_status, "relcache_invalidated_cache_ready");
    }

    #[pg_test]
    #[should_panic(expected = "requires at most one ec_spire index")]
    fn test_ec_spire_dml_frontdoor_rejects_multi_index() {
        Spi::run(
            "CREATE TABLE ec_spire_dml_frontdoor_multi_index_sql \
             (id bigint primary key, title text not null, embedding ecvector)",
        )
        .expect("DML frontdoor multi-index table creation should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_dml_frontdoor_multi_index_a_idx \
             ON ec_spire_dml_frontdoor_multi_index_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops)",
        )
        .expect("DML frontdoor multi-index first ec_spire index creation should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_dml_frontdoor_multi_index_b_idx \
             ON ec_spire_dml_frontdoor_multi_index_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops)",
        )
        .expect("DML frontdoor multi-index second ec_spire index creation should succeed");

        Spi::run(
            "SELECT * FROM ec_spire_dml_frontdoor_relation_context(\
                 'ec_spire_dml_frontdoor_multi_index_sql'::regclass)",
        )
        .expect("DML frontdoor multi-index relation context should fail");
    }

    #[pg_test]
    fn test_ec_spire_dml_frontdoor_target_relation_oid_sql() {
        Spi::run(
            "CREATE TABLE ec_spire_dml_target_oid_sql \
             (id bigint primary key, title text not null)",
        )
        .expect("DML target oid table creation should succeed");
        let relation_oid =
            Spi::get_one::<pg_sys::Oid>("SELECT 'ec_spire_dml_target_oid_sql'::regclass::oid")
                .expect("DML target oid relation lookup should succeed")
                .expect("DML target oid relation should exist");

        for sql in [
            "UPDATE ec_spire_dml_target_oid_sql SET title = 'updated' WHERE id = 1",
            "DELETE FROM ec_spire_dml_target_oid_sql WHERE id = 1",
            "SELECT id, title FROM ec_spire_dml_target_oid_sql WHERE id = 1",
        ] {
            let query = unsafe { analyzed_query(sql) };
            assert_eq!(
                unsafe { am::spire_dml_frontdoor_target_relation_oid(query) },
                Some(relation_oid),
                "{sql}"
            );
        }

        let join_query = unsafe {
            analyzed_query(
                "SELECT l.id \
                   FROM ec_spire_dml_target_oid_sql AS l \
                   JOIN ec_spire_dml_target_oid_sql AS r ON l.id = r.id",
            )
        };
        assert_eq!(
            unsafe { am::spire_dml_frontdoor_target_relation_oid(join_query) },
            None
        );
    }

    #[pg_test]
    fn test_ec_spire_dml_frontdoor_const_coercion_and_cte() {
        Spi::run(
            "CREATE TABLE ec_spire_dml_query_shape_sql \
             (id bigint primary key, title text not null, embedding ecvector)",
        )
        .expect("DML query shape table creation should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_dml_query_shape_idx \
             ON ec_spire_dml_query_shape_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops)",
        )
        .expect("DML query shape ec_spire index creation should succeed");

        let context = am::SpireDmlFrontdoorQueryContext {
            ec_spire_distributed_table: true,
            pk_column: "id",
            column_names: &[(1, "id"), (2, "title"), (3, "embedding")],
            embedding_columns: &["embedding"],
        };

        let coerced_const_query =
            unsafe { analyzed_query("SELECT id FROM ec_spire_dml_query_shape_sql WHERE id = 5") };
        let coerced_const_shape =
            unsafe { am::spire_classify_dml_frontdoor_query(coerced_const_query, context) }
                .expect("coerced const query should classify");
        assert!(
            coerced_const_shape.supported,
            "coerced const shape: {:?}",
            coerced_const_shape
        );
        assert_eq!(coerced_const_shape.kind, "pk_select_by_pk");

        let cte_query = unsafe {
            analyzed_query(
                "WITH marker AS (SELECT 1) \
                 SELECT id FROM ec_spire_dml_query_shape_sql WHERE id = 5",
            )
        };
        let cte_shape = unsafe { am::spire_classify_dml_frontdoor_query(cte_query, context) }
            .expect("CTE-prefixed query should classify");
        assert!(!cte_shape.supported);
        assert_eq!(cte_shape.kind, "unsupported_subquery_shape");

        let diagnostic_kind = Spi::get_one::<String>(
            "SELECT kind FROM ec_spire_dml_frontdoor_classify_sql(\
             $$SELECT id FROM ec_spire_dml_query_shape_sql WHERE id = 5$$)",
        )
        .expect("DML frontdoor diagnostic classifier should succeed")
        .expect("DML frontdoor diagnostic classifier should return a kind");
        let diagnostic_cte_kind = Spi::get_one::<String>(
            "SELECT kind FROM ec_spire_dml_frontdoor_classify_sql(\
             $$WITH marker AS (SELECT 1) \
               SELECT id FROM ec_spire_dml_query_shape_sql WHERE id = 5$$)",
        )
        .expect("DML frontdoor diagnostic CTE classifier should succeed")
        .expect("DML frontdoor diagnostic CTE classifier should return a kind");
        assert_eq!(diagnostic_kind, "pk_select_by_pk");
        assert_eq!(diagnostic_cte_kind, "unsupported_subquery_shape");

        Spi::run("SELECT id FROM ec_spire_dml_query_shape_sql WHERE id = 5")
            .expect("DML frontdoor planner hook SELECT should pass through");
        let hook_status = "FROM ec_spire_dml_frontdoor_hook_status()";
        let hook_classifier_invoked = Spi::get_one::<bool>(&format!(
            "SELECT query_shape_classifier_invoked_by_hook {hook_status}"
        ))
        .expect("DML frontdoor hook classifier observation query should succeed")
        .expect("DML frontdoor hook classifier observation value should exist");
        let hook_classification_supported = Spi::get_one::<bool>(&format!(
            "SELECT last_classification_supported {hook_status}"
        ))
        .expect("DML frontdoor hook last supported query should succeed")
        .expect("DML frontdoor hook last supported value should exist");
        let hook_classification_kind =
            Spi::get_one::<String>(&format!("SELECT last_classification_kind {hook_status}"))
                .expect("DML frontdoor hook last kind query should succeed")
                .expect("DML frontdoor hook last kind should exist");
        let hook_classification_status =
            Spi::get_one::<String>(&format!("SELECT last_classification_status {hook_status}"))
                .expect("DML frontdoor hook last status query should succeed")
                .expect("DML frontdoor hook last status should exist");
        assert!(hook_classifier_invoked);
        assert!(hook_classification_supported);
        assert_eq!(hook_classification_kind, "pk_select_by_pk");
        assert_eq!(hook_classification_status, "supported_v1_shape");

        Spi::run("SELECT id FROM ec_spire_dml_query_shape_sql ORDER BY id")
            .expect("ordinary non-PK SELECT should pass through the DML frontdoor hook");
        let non_pk_hook_action =
            Spi::get_one::<String>(&format!("SELECT last_hook_action {hook_status}"))
                .expect("DML frontdoor hook action after non-PK SELECT should succeed")
                .expect("DML frontdoor hook action after non-PK SELECT should exist");
        let non_pk_hook_kind =
            Spi::get_one::<String>(&format!("SELECT last_classification_kind {hook_status}"))
                .expect("DML frontdoor hook kind after non-PK SELECT should succeed")
                .expect("DML frontdoor hook kind after non-PK SELECT should exist");
        assert_eq!(non_pk_hook_action, "pass_through_not_spire_frontdoor");
        assert_eq!(non_pk_hook_kind, "non_pk_select_pass_through");
    }

    #[pg_test]
    fn test_ec_spire_dml_frontdoor_rejects_pk_predicate_edge_shapes() {
        Spi::run(
            "CREATE TABLE ec_spire_dml_pk_edge_sql \
             (id bigint primary key, title text not null, embedding ecvector)",
        )
        .expect("DML PK edge table creation should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_dml_pk_edge_idx \
             ON ec_spire_dml_pk_edge_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops)",
        )
        .expect("DML PK edge ec_spire index creation should succeed");

        let cases = [
            (
                "numeric_outside_int8",
                "SELECT id FROM ec_spire_dml_pk_edge_sql \
                  WHERE id = 9223372036854775808::numeric",
            ),
            (
                "null_int8",
                "SELECT id FROM ec_spire_dml_pk_edge_sql WHERE id = NULL::int8",
            ),
            (
                "in_list",
                "SELECT id FROM ec_spire_dml_pk_edge_sql WHERE id IN (5, 6)",
            ),
            (
                "or_equality",
                "SELECT id FROM ec_spire_dml_pk_edge_sql WHERE id = 5 OR id = 6",
            ),
            (
                "numeric_equality",
                "SELECT id FROM ec_spire_dml_pk_edge_sql \
                  WHERE id = 5::numeric",
            ),
        ];

        for (label, sql) in cases {
            let escaped_sql = sql.replace('\'', "''");
            let summary = Spi::get_one::<String>(&format!(
                "SELECT supported::text || '|' || kind || '|' || status \
                   FROM ec_spire_dml_frontdoor_classify_sql('{escaped_sql}')"
            ))
            .expect("DML PK edge classifier query should succeed")
            .unwrap_or_else(|| panic!("DML PK edge classifier returned no row for {label}"));
            assert_eq!(
                summary, "false|unsupported_pk_predicate|unsupported_shape",
                "{label}"
            );
        }

        Spi::run(
            "PREPARE ec_spire_dml_pk_edge_numeric(numeric) AS \
             SELECT id FROM ec_spire_dml_pk_edge_sql WHERE id = $1::numeric",
        )
        .expect("numeric-parameter PK SELECT should prepare");
        let prepared_param_error = pg_sys::PgTryBuilder::new(|| {
            Spi::run("EXECUTE ec_spire_dml_pk_edge_numeric(9223372036854775808::numeric)")
                .expect("numeric-parameter PK SELECT should fail closed during EXECUTE");
            "no_error".to_owned()
        })
        .catch_when(
            pg_sys::errcodes::PgSqlErrorCode::ERRCODE_FEATURE_NOT_SUPPORTED,
            |cause| match cause {
                pg_sys::panic::CaughtError::ErrorReport(report)
                | pg_sys::panic::CaughtError::PostgresError(report) => {
                    format!("{}|{}", report.message(), report.hint().unwrap_or(""))
                }
                pg_sys::panic::CaughtError::RustPanic { ereport, .. } => {
                    format!("{}|{}", ereport.message(), ereport.hint().unwrap_or(""))
                }
            },
        )
        .catch_others(|cause| cause.rethrow())
        .execute();
        assert_eq!(
            prepared_param_error,
            "ec_spire_distributed: DML requires a bigint primary-key equality predicate in v1|See ADR-069 for the v1 SPIRE distributed DML shape."
        );

        let in_list_error = pg_sys::PgTryBuilder::new(|| {
            Spi::run("SELECT id FROM ec_spire_dml_pk_edge_sql WHERE id IN (5, 6)")
                .expect("IN-list PK SELECT should fail closed in the planner hook");
            "no_error".to_owned()
        })
        .catch_when(
            pg_sys::errcodes::PgSqlErrorCode::ERRCODE_FEATURE_NOT_SUPPORTED,
            |cause| match cause {
                pg_sys::panic::CaughtError::ErrorReport(report)
                | pg_sys::panic::CaughtError::PostgresError(report) => {
                    format!("{}|{}", report.message(), report.hint().unwrap_or(""))
                }
                pg_sys::panic::CaughtError::RustPanic { ereport, .. } => {
                    format!("{}|{}", ereport.message(), ereport.hint().unwrap_or(""))
                }
            },
        )
        .catch_others(|cause| cause.rethrow())
        .execute();
        assert_eq!(
            in_list_error,
            "ec_spire_distributed: DML requires a bigint primary-key equality predicate in v1|See ADR-069 for the v1 SPIRE distributed DML shape."
        );

        let action = Spi::get_one::<String>(
            "SELECT last_hook_action FROM ec_spire_dml_frontdoor_hook_status()",
        )
        .expect("DML PK edge hook action query should succeed")
        .expect("DML PK edge hook action should exist");
        let kind = Spi::get_one::<String>(
            "SELECT last_classification_kind FROM ec_spire_dml_frontdoor_hook_status()",
        )
        .expect("DML PK edge hook kind query should succeed")
        .expect("DML PK edge hook kind should exist");
        assert_eq!(action, "planner_error_fail_closed");
        assert_eq!(kind, "unsupported_pk_predicate");
    }

    #[pg_test]
    fn test_ec_spire_dml_frontdoor_hook_fail_closed_unsupported_shape() {
        Spi::run(
            "CREATE TABLE ec_spire_dml_failclosed_sql \
             (id bigint primary key, title text not null, embedding ecvector)",
        )
        .expect("DML fail-closed table creation should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_dml_failclosed_idx \
             ON ec_spire_dml_failclosed_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops)",
        )
        .expect("DML fail-closed ec_spire index creation should succeed");

        let error = pg_sys::PgTryBuilder::new(|| {
            Spi::run(
                "UPDATE ec_spire_dml_failclosed_sql \
                    SET embedding = '[1,2,3]'::ecvector \
                  WHERE id = 5",
            )
            .expect("embedding UPDATE should fail closed in the planner hook");
            "no_error".to_owned()
        })
        .catch_others(|cause| match cause {
            pg_sys::panic::CaughtError::ErrorReport(report)
            | pg_sys::panic::CaughtError::PostgresError(report) => {
                format!("{}|{}", report.message(), report.hint().unwrap_or(""))
            }
            pg_sys::panic::CaughtError::RustPanic { ereport, .. } => {
                format!("{}|{}", ereport.message(), ereport.hint().unwrap_or(""))
            }
        })
        .execute();

        assert_eq!(
            error,
            "ec_spire_distributed: UPDATE of indexed embedding column is not supported on a distributed ec_spire table. Use DELETE + INSERT.|Cross-shard atomic moves will be available in a future release."
        );

        let action = Spi::get_one::<String>(
            "SELECT last_hook_action FROM ec_spire_dml_frontdoor_hook_status()",
        )
        .expect("DML frontdoor hook action query should succeed")
        .expect("DML frontdoor hook action should exist");
        assert_eq!(action, "planner_error_fail_closed");

        Spi::run(
            "CREATE TABLE ec_spire_dml_failclosed_plain_sql \
             (id bigint primary key, title text not null, embedding ecvector)",
        )
        .expect("plain fail-closed table creation should succeed");
        Spi::run(
            "UPDATE ec_spire_dml_failclosed_plain_sql \
                SET embedding = '[1,2,3]'::ecvector \
              WHERE id = 5",
        )
        .expect("plain non-ec_spire table should pass through the DML frontdoor hook");
    }

    #[pg_test]
    fn test_ec_spire_dml_frontdoor_hook_fail_closed_context_error() {
        Spi::run(
            "CREATE TABLE ec_spire_dml_failclosed_context_sql \
             (id bigint primary key, title text not null, embedding ecvector)",
        )
        .expect("DML fail-closed context table creation should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_dml_failclosed_context_a_idx \
             ON ec_spire_dml_failclosed_context_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops)",
        )
        .expect("DML fail-closed context first ec_spire index creation should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_dml_failclosed_context_b_idx \
             ON ec_spire_dml_failclosed_context_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops)",
        )
        .expect("DML fail-closed context second ec_spire index creation should succeed");

        let error = pg_sys::PgTryBuilder::new(|| {
            Spi::run("SELECT id FROM ec_spire_dml_failclosed_context_sql WHERE id = 5")
                .expect("multi-index context error should fail closed in the planner hook");
            "no_error".to_owned()
        })
        .catch_others(|cause| match cause {
            pg_sys::panic::CaughtError::ErrorReport(report)
            | pg_sys::panic::CaughtError::PostgresError(report) => {
                format!("{}|{}", report.message(), report.hint().unwrap_or(""))
            }
            pg_sys::panic::CaughtError::RustPanic { ereport, .. } => {
                format!("{}|{}", ereport.message(), ereport.hint().unwrap_or(""))
            }
        })
        .execute();

        assert_eq!(
            error,
            "ec_spire_distributed: relation context could not be loaded|See ADR-069 for the v1 SPIRE distributed DML shape."
        );

        let action = Spi::get_one::<String>(
            "SELECT last_hook_action FROM ec_spire_dml_frontdoor_hook_status()",
        )
        .expect("DML frontdoor context hook action query should succeed")
        .expect("DML frontdoor context hook action should exist");
        let kind = Spi::get_one::<String>(
            "SELECT last_classification_kind FROM ec_spire_dml_frontdoor_hook_status()",
        )
        .expect("DML frontdoor context hook kind query should succeed")
        .expect("DML frontdoor context hook kind should exist");
        assert_eq!(action, "planner_error_fail_closed");
        assert_eq!(kind, "relation_context_error");
    }

    #[pg_test]
    fn test_ec_spire_dml_plan_tree_replace_scaffold() {
        Spi::run(
            "CREATE TABLE ec_spire_dml_plan_replace_sql \
             (id bigint primary key, title text not null, body text, embedding ecvector)",
        )
        .expect("DML plan replacement table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_dml_plan_replace_sql \
                 (id, title, body, embedding) VALUES \
             (5, 'before', 'body before', encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42))",
        )
        .expect("DML plan replacement seed row should be inserted");
        Spi::run(
            "CREATE INDEX ec_spire_dml_plan_replace_idx \
             ON ec_spire_dml_plan_replace_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops)",
        )
        .expect("DML plan replacement ec_spire index creation should succeed");
        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_dml_plan_replace_idx'::regclass)",
        )
        .expect("DML plan replacement active epoch query should succeed")
        .expect("DML plan replacement active epoch should exist");
        Spi::run(&format!(
            "INSERT INTO ec_spire_placement \
                 (index_oid, pk_value, node_id, centroid_id, served_epoch, source_identity) \
             VALUES ('ec_spire_dml_plan_replace_idx'::regclass, int8send(5::bigint)::bytea, \
                     0, 2, {active_epoch}, decode('000102030405060708090a0b0c0d0e0f', 'hex'))"
        ))
        .expect("DML plan replacement local placement row should be inserted");

        Spi::run(
            "EXPLAIN (COSTS OFF) \
             UPDATE ec_spire_dml_plan_replace_sql SET title = 'updated' WHERE id = 5",
        )
        .expect("DML UPDATE plan replacement EXPLAIN should succeed");
        let action = Spi::get_one::<String>(
            "SELECT last_hook_action FROM ec_spire_dml_frontdoor_hook_status()",
        )
        .expect("DML UPDATE plan replacement hook action query should succeed")
        .expect("DML UPDATE plan replacement hook action should exist");
        assert_eq!(action, "plan_tree_replaced_customscan");

        Spi::run(
            "EXPLAIN (COSTS OFF) \
             DELETE FROM ec_spire_dml_plan_replace_sql WHERE id = 5",
        )
        .expect("DML DELETE plan replacement EXPLAIN should succeed");
        let action = Spi::get_one::<String>(
            "SELECT last_hook_action FROM ec_spire_dml_frontdoor_hook_status()",
        )
        .expect("DML DELETE plan replacement hook action query should succeed")
        .expect("DML DELETE plan replacement hook action should exist");
        assert_eq!(action, "plan_tree_replaced_customscan");

        Spi::run(
            "DO $$ \
             DECLARE row_count bigint; \
             BEGIN \
                 UPDATE ec_spire_dml_plan_replace_sql \
                    SET title = 'updated', body = 'body updated' \
                  WHERE id = 5; \
                 GET DIAGNOSTICS row_count = ROW_COUNT; \
                 IF row_count != 1 THEN \
                     RAISE EXCEPTION 'unexpected DML CustomScan row count %', row_count; \
                 END IF; \
             END $$",
        )
        .expect("DML UPDATE CustomScan execution should succeed");
        let tuple = Spi::get_one::<String>(
            "SELECT title || '|' || body \
               FROM ec_spire_dml_plan_replace_sql \
              WHERE id = 5",
        )
        .expect("DML plan replacement tuple query should succeed")
        .expect("DML plan replacement tuple should exist");
        assert_eq!(tuple, "updated|body updated");

        Spi::run(
            "PREPARE ec_spire_dml_plan_replace_update(text, text) AS \
             UPDATE ec_spire_dml_plan_replace_sql \
                SET title = $1, body = $2 \
              WHERE id = 5",
        )
        .expect("DML parameterized UPDATE should prepare");
        Spi::run("EXECUTE ec_spire_dml_plan_replace_update('param updated', 'param body updated')")
            .expect("DML parameterized UPDATE CustomScan execution should succeed");
        let tuple = Spi::get_one::<String>(
            "SELECT title || '|' || body \
               FROM ec_spire_dml_plan_replace_sql \
              WHERE id = 5",
        )
        .expect("DML parameterized tuple query should succeed")
        .expect("DML parameterized tuple should exist");
        assert_eq!(tuple, "param updated|param body updated");

        let expression_error = pg_sys::PgTryBuilder::new(|| {
            Spi::run(
                "UPDATE ec_spire_dml_plan_replace_sql \
                    SET title = title || 'x' \
                  WHERE id = 5",
            )
            .expect("DML row-dependent UPDATE should fail closed");
            "no_error".to_owned()
        })
        .catch_others(|cause| match cause {
            pg_sys::panic::CaughtError::ErrorReport(report)
            | pg_sys::panic::CaughtError::PostgresError(report) => report.message().to_owned(),
            pg_sys::panic::CaughtError::RustPanic { ereport, .. } => ereport.message().to_owned(),
        })
        .execute();
        assert_eq!(
            expression_error,
            "EcSpireDistributedScan DML UPDATE supports only constant or parameter SET values in v1"
        );

        Spi::run(
            "DO $$ \
             DECLARE row_count bigint; \
             BEGIN \
                 DELETE FROM ec_spire_dml_plan_replace_sql WHERE id = 5; \
                 GET DIAGNOSTICS row_count = ROW_COUNT; \
                 IF row_count != 1 THEN \
                     RAISE EXCEPTION 'unexpected DML CustomScan delete row count %', row_count; \
                 END IF; \
             END $$",
        )
        .expect("DML DELETE CustomScan execution should succeed");
        let remaining = Spi::get_one::<String>(
            "SELECT \
                  (SELECT count(*) FROM ec_spire_dml_plan_replace_sql WHERE id = 5)::text \
                  || '|' || \
                  (SELECT count(*) FROM ec_spire_placement \
                    WHERE index_oid = 'ec_spire_dml_plan_replace_idx'::regclass \
                      AND pk_value = int8send(5::bigint)::bytea)::text",
        )
        .expect("DML DELETE remaining row query should succeed")
        .expect("DML DELETE remaining row summary should exist");
        assert_eq!(remaining, "0|0");
    }

    #[pg_test]
    fn test_ec_spire_dml_customscan_remote_update_sql() {
        let _env_lock = env_var_test_lock();
        let loopback_conninfo = current_pg_test_loopback_conninfo();
        let _conninfo_secret = ScopedEnvVar::set(
            "EC_SPIRE_REMOTE_CONNINFO_SPIRE_REMOTE_CUSTOMSCAN_UPDATE_REMOTE_SQL",
            &loopback_conninfo,
        );
        let mut loopback_client = postgres::Client::connect(&loopback_conninfo, postgres::NoTls)
            .expect("loopback client connection should succeed");
        loopback_client
            .batch_execute(
                "DROP TABLE IF EXISTS ec_spire_dml_customscan_update_remote_sql; \
                 CREATE TABLE ec_spire_dml_customscan_update_remote_sql \
                     (id bigint primary key, title text not null, embedding ecvector, \
                      source_identity bytea not null); \
                 INSERT INTO ec_spire_dml_customscan_update_remote_sql \
                     (id, title, embedding, source_identity) VALUES \
                 (1505, 'remote before transparent update', \
                  encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42), \
                  decode('15161718191a1b1c1d1e1f2021222324', 'hex')); \
                 CREATE INDEX ec_spire_dml_customscan_update_remote_idx \
                     ON ec_spire_dml_customscan_update_remote_sql USING ec_spire \
                     (embedding ecvector_spire_ip_ops);",
            )
            .expect("loopback remote transparent update target should be created");

        Spi::run(
            "CREATE TABLE ec_spire_dml_customscan_update_coord_sql \
             (id bigint primary key, title text not null, embedding ecvector, \
              source_identity bytea not null)",
        )
        .expect("transparent update coordinator table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_dml_customscan_update_coord_sql \
                 (id, title, embedding, source_identity) VALUES \
             (1, 'coordinator seed', encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42), \
              decode('25262728292a2b2c2d2e2f3031323334', 'hex'))",
        )
        .expect("transparent update coordinator seed insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_dml_customscan_update_coord_idx \
             ON ec_spire_dml_customscan_update_coord_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops)",
        )
        .expect("transparent update coordinator ec_spire index creation should succeed");
        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_dml_customscan_update_coord_idx'::regclass)",
        )
        .expect("transparent update active epoch query should succeed")
        .expect("transparent update active epoch should exist");
        let remote_identity_hex = Spi::get_one::<String>(
            "SELECT profile_fingerprint \
               FROM ec_spire_remote_search_endpoint_identity(\
                    'ec_spire_dml_customscan_update_remote_idx'::regclass::oid)",
        )
        .expect("transparent update remote identity query should succeed")
        .expect("transparent update remote identity should exist");
        Spi::run(&format!(
            "SELECT ec_spire_register_remote_node_descriptor(\
                 'ec_spire_dml_customscan_update_coord_idx'::regclass, \
                 25, 29, 'spire/remote/customscan_update_remote_sql', \
                 decode('{remote_identity_hex}', 'hex'), \
                 'ec_spire_dml_customscan_update_remote_idx', \
                 'active', {active_epoch}, {active_epoch}, '0.1.1', '')"
        ))
        .expect("transparent update remote descriptor registration should succeed");
        Spi::run(&format!(
            "INSERT INTO ec_spire_placement \
                 (index_oid, pk_value, node_id, centroid_id, served_epoch, source_identity) \
             VALUES ('ec_spire_dml_customscan_update_coord_idx'::regclass, \
                     int8send(1505::bigint)::bytea, 25, 2, {active_epoch}, \
                     decode('15161718191a1b1c1d1e1f2021222324', 'hex'))"
        ))
        .expect("transparent update remote placement row should be inserted");

        Spi::run(
            "EXPLAIN (COSTS OFF) \
             UPDATE ec_spire_dml_customscan_update_coord_sql \
                SET title = 'remote after transparent update' \
              WHERE id = 1505",
        )
        .expect("transparent remote UPDATE EXPLAIN should succeed");
        let action = Spi::get_one::<String>(
            "SELECT last_hook_action FROM ec_spire_dml_frontdoor_hook_status()",
        )
        .expect("transparent remote update hook status query should succeed")
        .expect("transparent remote update hook action should exist");
        assert_eq!(action, "plan_tree_replaced_customscan");

        Spi::run(
            "DO $$ \
             DECLARE row_count bigint; \
             BEGIN \
                 UPDATE ec_spire_dml_customscan_update_coord_sql \
                    SET title = 'remote after transparent update' \
                  WHERE id = 1505; \
                 GET DIAGNOSTICS row_count = ROW_COUNT; \
                 IF row_count != 1 THEN \
                     RAISE EXCEPTION 'unexpected remote transparent update row count %', row_count; \
                 END IF; \
             END $$",
        )
        .expect("transparent remote UPDATE through CustomScan should succeed");
        let remote_title = loopback_client
            .query_one(
                "SELECT title \
                   FROM ec_spire_dml_customscan_update_remote_sql \
                  WHERE id = 1505",
                &[],
            )
            .expect("transparent remote update title query should succeed")
            .try_get::<_, String>(0)
            .expect("transparent remote update title should decode");
        assert_eq!(remote_title, "remote after transparent update");
    }

    #[pg_test]
    fn test_ec_spire_dml_customscan_remote_delete_sql() {
        let _env_lock = env_var_test_lock();
        let loopback_conninfo = current_pg_test_loopback_conninfo();
        let _conninfo_secret = ScopedEnvVar::set(
            "EC_SPIRE_REMOTE_CONNINFO_SPIRE_REMOTE_CUSTOMSCAN_DELETE_REMOTE_SQL",
            &loopback_conninfo,
        );
        let mut loopback_client = postgres::Client::connect(&loopback_conninfo, postgres::NoTls)
            .expect("loopback client connection should succeed");
        loopback_client
            .batch_execute(
                "DROP TABLE IF EXISTS ec_spire_dml_customscan_delete_remote_sql; \
                 CREATE TABLE ec_spire_dml_customscan_delete_remote_sql \
                     (id bigint primary key, title text not null, embedding ecvector, \
                      source_identity bytea not null); \
                 INSERT INTO ec_spire_dml_customscan_delete_remote_sql \
                     (id, title, embedding, source_identity) VALUES \
                 (1606, 'remote transparent delete me', \
                  encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42), \
                  decode('35363738393a3b3c3d3e3f4041424344', 'hex')); \
                 CREATE INDEX ec_spire_dml_customscan_delete_remote_idx \
                     ON ec_spire_dml_customscan_delete_remote_sql USING ec_spire \
                     (embedding ecvector_spire_ip_ops);",
            )
            .expect("loopback remote transparent delete target should be created");

        Spi::run(
            "CREATE TABLE ec_spire_dml_customscan_delete_coord_sql \
             (id bigint primary key, title text not null, embedding ecvector, \
              source_identity bytea not null)",
        )
        .expect("transparent delete coordinator table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_dml_customscan_delete_coord_sql \
                 (id, title, embedding, source_identity) VALUES \
             (1, 'coordinator seed', encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42), \
              decode('45464748494a4b4c4d4e4f5051525354', 'hex'))",
        )
        .expect("transparent delete coordinator seed insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_dml_customscan_delete_coord_idx \
             ON ec_spire_dml_customscan_delete_coord_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops)",
        )
        .expect("transparent delete coordinator ec_spire index creation should succeed");
        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_dml_customscan_delete_coord_idx'::regclass)",
        )
        .expect("transparent delete active epoch query should succeed")
        .expect("transparent delete active epoch should exist");
        let remote_identity_hex = Spi::get_one::<String>(
            "SELECT profile_fingerprint \
               FROM ec_spire_remote_search_endpoint_identity(\
                    'ec_spire_dml_customscan_delete_remote_idx'::regclass::oid)",
        )
        .expect("transparent delete remote identity query should succeed")
        .expect("transparent delete remote identity should exist");
        Spi::run(&format!(
            "SELECT ec_spire_register_remote_node_descriptor(\
                 'ec_spire_dml_customscan_delete_coord_idx'::regclass, \
                 26, 30, 'spire/remote/customscan_delete_remote_sql', \
                 decode('{remote_identity_hex}', 'hex'), \
                 'ec_spire_dml_customscan_delete_remote_idx', \
                 'active', {active_epoch}, {active_epoch}, '0.1.1', '')"
        ))
        .expect("transparent delete remote descriptor registration should succeed");
        Spi::run(&format!(
            "INSERT INTO ec_spire_placement \
                 (index_oid, pk_value, node_id, centroid_id, served_epoch, source_identity) \
             VALUES ('ec_spire_dml_customscan_delete_coord_idx'::regclass, \
                     int8send(1606::bigint)::bytea, 26, 2, {active_epoch}, \
                     decode('35363738393a3b3c3d3e3f4041424344', 'hex'))"
        ))
        .expect("transparent delete remote placement row should be inserted");

        Spi::run(
            "EXPLAIN (COSTS OFF) \
             DELETE FROM ec_spire_dml_customscan_delete_coord_sql WHERE id = 1606",
        )
        .expect("transparent remote DELETE EXPLAIN should succeed");
        let action = Spi::get_one::<String>(
            "SELECT last_hook_action FROM ec_spire_dml_frontdoor_hook_status()",
        )
        .expect("transparent remote delete hook status query should succeed")
        .expect("transparent remote delete hook action should exist");
        assert_eq!(action, "plan_tree_replaced_customscan");

        Spi::run(
            "DO $$ \
             DECLARE row_count bigint; \
             BEGIN \
                 DELETE FROM ec_spire_dml_customscan_delete_coord_sql WHERE id = 1606; \
                 GET DIAGNOSTICS row_count = ROW_COUNT; \
                 IF row_count != 1 THEN \
                     RAISE EXCEPTION 'unexpected remote transparent delete row count %', row_count; \
                 END IF; \
             END $$",
        )
        .expect("transparent remote DELETE through CustomScan should succeed");
        let prepared_count = loopback_client
            .query_one(
                "SELECT count(*)::bigint \
                   FROM pg_prepared_xacts \
                  WHERE gid LIKE 'ec_spire_insert_%'",
                &[],
            )
            .expect("transparent remote delete prepared xact query should succeed")
            .try_get::<_, i64>(0)
            .expect("transparent remote delete prepared xact count should decode");
        let remote_visible_count = loopback_client
            .query_one(
                "SELECT count(*)::bigint \
                   FROM ec_spire_dml_customscan_delete_remote_sql \
                  WHERE id = 1606",
                &[],
            )
            .expect("transparent remote delete visibility query should succeed")
            .try_get::<_, i64>(0)
            .expect("transparent remote delete visibility count should decode");
        let placement_count = Spi::get_one::<i64>(
            "SELECT count(*) \
               FROM ec_spire_placement \
              WHERE index_oid = 'ec_spire_dml_customscan_delete_coord_idx'::regclass \
                AND pk_value = int8send(1606::bigint)::bytea",
        )
        .expect("transparent remote delete placement count query should succeed")
        .expect("transparent remote delete placement count should exist");

        assert_eq!(prepared_count, 1);
        assert_eq!(
            remote_visible_count, 1,
            "prepared remote DELETE should not be visible before transaction resolution"
        );
        assert_eq!(placement_count, 0);
    }
    #[pg_test]
    fn test_ec_spire_dml_frontdoor_pk_value_bytes_match_int8send() {
        for value in [0_i64, 5, -5, i64::MAX, i64::MIN] {
            let pg_hex = Spi::get_one::<String>(&format!(
                "SELECT encode(int8send('{value}'::bigint)::bytea, 'hex')"
            ))
            .expect("PostgreSQL int8send hex query should succeed")
            .expect("PostgreSQL int8send hex should exist");
            assert_eq!(
                hex::encode(am::spire_dml_frontdoor_bigint_pk_value_bytes(value)),
                pg_hex,
                "{value}"
            );
        }
    }

    #[pg_test]
    fn test_ec_spire_dml_frontdoor_pk_argument_from_decision() {
        Spi::run(
            "CREATE TABLE ec_spire_dml_pk_argument_sql \
             (id bigint primary key, title text not null, embedding ecvector)",
        )
        .expect("DML PK argument table creation should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_dml_pk_argument_idx \
             ON ec_spire_dml_pk_argument_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops)",
        )
        .expect("DML PK argument ec_spire index creation should succeed");

        let select_query =
            unsafe { analyzed_query("SELECT id FROM ec_spire_dml_pk_argument_sql WHERE id = 5") };
        let select_decision =
            unsafe { am::spire_dml_frontdoor_replacement_decision_catalog_row(select_query) }
                .expect("PK SELECT replacement decision should exist");
        let select_pk_argument =
            am::spire_dml_frontdoor_pk_argument_from_replacement_decision(&select_decision)
                .expect("PK SELECT argument should be buildable");
        assert_eq!(select_pk_argument.pk_column, "id");
        assert_eq!(
            select_pk_argument.value,
            am::SpireDmlFrontdoorPkValuePlan::ConstBigint(5)
        );

        let embedding_update_query = unsafe {
            analyzed_query(
                "UPDATE ec_spire_dml_pk_argument_sql \
                    SET embedding = '[1,2,3]'::ecvector \
                  WHERE id = 5",
            )
        };
        let embedding_update_decision = unsafe {
            am::spire_dml_frontdoor_replacement_decision_catalog_row(embedding_update_query)
        }
        .expect("embedding UPDATE replacement decision should exist");
        let error = am::spire_dml_frontdoor_pk_argument_from_replacement_decision(
            &embedding_update_decision,
        )
        .expect_err("unsupported decisions should not build PK arguments");
        assert!(error.contains("requires a supported decision"), "{error}");
    }

    #[pg_test]
    fn test_ec_spire_dml_frontdoor_primitive_plan_from_decision() {
        Spi::run(
            "CREATE TABLE ec_spire_dml_primitive_plan_sql \
             (id bigint primary key, title text not null, embedding ecvector)",
        )
        .expect("DML primitive plan table creation should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_dml_primitive_plan_idx \
             ON ec_spire_dml_primitive_plan_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops)",
        )
        .expect("DML primitive plan ec_spire index creation should succeed");

        let update_query = unsafe {
            analyzed_query(
                "UPDATE ec_spire_dml_primitive_plan_sql \
                    SET title = 'updated' \
                  WHERE id = 5",
            )
        };
        let update_decision =
            unsafe { am::spire_dml_frontdoor_replacement_decision_catalog_row(update_query) }
                .expect("UPDATE primitive plan decision should exist");
        let update_plan =
            am::spire_dml_frontdoor_primitive_plan_from_replacement_decision(&update_decision)
                .expect("UPDATE primitive plan should be buildable");
        assert_eq!(
            update_plan.mode,
            am::SpireDmlFrontdoorCustomScanMode::CoordinatorUpdateTuplePayload
        );
        assert_eq!(
            update_plan.primitive,
            "ec_spire_forward_coordinator_update_tuple_payload"
        );
        assert_eq!(update_plan.pk_argument.pk_column, "id");
        assert_eq!(update_plan.updated_columns, vec!["title".to_owned()]);
        assert!(update_plan.projected_columns.is_empty());
        let update_plan_expr =
            unsafe { am::spire_dml_frontdoor_primitive_plan_expr_catalog_row(update_query) }
                .expect("UPDATE primitive plan expression handoff should classify")
                .expect("UPDATE primitive plan expression handoff should be buildable");
        assert_eq!(
            update_plan_expr.primitive_plan.mode,
            am::SpireDmlFrontdoorCustomScanMode::CoordinatorUpdateTuplePayload
        );
        assert!(!update_plan_expr.pk_value_expr.is_null());
        assert_eq!(
            unsafe { (*update_plan_expr.pk_value_expr.cast::<pg_sys::Node>()).type_ },
            pg_sys::NodeTag::T_Const
        );

        let delete_query =
            unsafe { analyzed_query("DELETE FROM ec_spire_dml_primitive_plan_sql WHERE id = 5") };
        let delete_decision =
            unsafe { am::spire_dml_frontdoor_replacement_decision_catalog_row(delete_query) }
                .expect("DELETE primitive plan decision should exist");
        let delete_plan =
            am::spire_dml_frontdoor_primitive_plan_from_replacement_decision(&delete_decision)
                .expect("DELETE primitive plan should be buildable");
        assert_eq!(
            delete_plan.mode,
            am::SpireDmlFrontdoorCustomScanMode::CoordinatorDeleteTuplePayload
        );
        assert_eq!(
            delete_plan.primitive,
            "ec_spire_prepare_coordinator_delete_tuple_payload"
        );
        assert!(delete_plan.updated_columns.is_empty());
        assert!(delete_plan.projected_columns.is_empty());
        let delete_plan_expr =
            unsafe { am::spire_dml_frontdoor_primitive_plan_expr_catalog_row(delete_query) }
                .expect("DELETE primitive plan expression handoff should classify")
                .expect("DELETE primitive plan expression handoff should be buildable");
        assert_eq!(
            delete_plan_expr.primitive_plan.mode,
            am::SpireDmlFrontdoorCustomScanMode::CoordinatorDeleteTuplePayload
        );
        assert!(!delete_plan_expr.pk_value_expr.is_null());
        assert_eq!(
            unsafe { (*delete_plan_expr.pk_value_expr.cast::<pg_sys::Node>()).type_ },
            pg_sys::NodeTag::T_Const
        );

        let select_query = unsafe {
            analyzed_query("SELECT id, title FROM ec_spire_dml_primitive_plan_sql WHERE id = 5")
        };
        let select_decision =
            unsafe { am::spire_dml_frontdoor_replacement_decision_catalog_row(select_query) }
                .expect("PK SELECT primitive plan decision should exist");
        let select_plan =
            am::spire_dml_frontdoor_primitive_plan_from_replacement_decision(&select_decision)
                .expect("PK SELECT primitive plan should be buildable");
        assert_eq!(
            select_plan.mode,
            am::SpireDmlFrontdoorCustomScanMode::CoordinatorPkSelectTuplePayload
        );
        assert_eq!(
            select_plan.primitive,
            "ec_spire_forward_coordinator_select_tuple_payload"
        );
        assert!(select_plan.updated_columns.is_empty());
        assert_eq!(
            select_plan.projected_columns,
            vec!["id".to_owned(), "title".to_owned()]
        );
        let select_plan_expr =
            unsafe { am::spire_dml_frontdoor_primitive_plan_expr_catalog_row(select_query) }
                .expect("PK SELECT primitive plan expression handoff should classify")
                .expect("PK SELECT primitive plan expression handoff should be buildable");
        assert_eq!(
            select_plan_expr.primitive_plan.mode,
            am::SpireDmlFrontdoorCustomScanMode::CoordinatorPkSelectTuplePayload
        );
        assert!(!select_plan_expr.pk_value_expr.is_null());
        assert_eq!(
            unsafe { (*select_plan_expr.pk_value_expr.cast::<pg_sys::Node>()).type_ },
            pg_sys::NodeTag::T_Const
        );
        assert_eq!(
            hex::encode(
                am::spire_dml_frontdoor_primitive_plan_const_pk_value_bytes(&select_plan)
                    .expect("const PK primitive plan should produce bytea")
            ),
            "0000000000000005"
        );
        let select_invocation = unsafe {
            am::spire_dml_frontdoor_primitive_invocation_from_plan(
                &select_plan,
                std::ptr::null_mut(),
            )
        }
        .expect("const PK primitive invocation should be buildable");
        assert_eq!(
            select_invocation.mode,
            am::SpireDmlFrontdoorCustomScanMode::CoordinatorPkSelectTuplePayload
        );
        assert_eq!(
            select_invocation.primitive,
            "ec_spire_forward_coordinator_select_tuple_payload"
        );
        assert_eq!(select_invocation.pk_column, "id");
        assert_eq!(hex::encode(select_invocation.pk_value), "0000000000000005");
        assert!(select_invocation.updated_columns.is_empty());
        assert_eq!(
            select_invocation.projected_columns,
            vec!["id".to_owned(), "title".to_owned()]
        );

        let mut mismatched_primitive = select_decision.clone();
        mismatched_primitive.primitive = "ec_spire_forward_coordinator_update_tuple_payload";
        let mismatch_error =
            am::spire_dml_frontdoor_primitive_plan_from_replacement_decision(&mismatched_primitive)
                .expect_err("mode/primitive mismatch should fail closed");
        assert!(
            mismatch_error.contains("requires primitive"),
            "{mismatch_error}"
        );

        let mut param_decision = select_decision.clone();
        param_decision.pk_value_kind = "param_bigint";
        param_decision.pk_value_const = None;
        param_decision.pk_value_param_id = Some(1);
        let param_plan =
            am::spire_dml_frontdoor_primitive_plan_from_replacement_decision(&param_decision)
                .expect("parameter PK primitive plan should be buildable");
        let param_error = am::spire_dml_frontdoor_primitive_plan_const_pk_value_bytes(&param_plan)
            .expect_err("parameter PK primitive plan needs runtime evaluation");
        assert!(
            param_error.contains("requires executor parameter evaluation"),
            "{param_error}"
        );
        for value in [i64::MAX, i64::MIN] {
            let mut const_decision = select_decision.clone();
            const_decision.pk_value_kind = "const_bigint";
            const_decision.pk_value_const = Some(value);
            const_decision.pk_value_param_id = None;
            let const_plan =
                am::spire_dml_frontdoor_primitive_plan_from_replacement_decision(&const_decision)
                    .expect("boundary const PK primitive plan should be buildable");
            let expected = am::spire_dml_frontdoor_bigint_pk_value_bytes(value);
            assert_eq!(
                am::spire_dml_frontdoor_primitive_plan_const_pk_value_bytes(&const_plan)
                    .expect("boundary const PK primitive plan should produce bytea"),
                expected,
                "{value}"
            );
        }
        unsafe {
            let params = pg_sys::makeParamList(1);
            assert!(
                !params.is_null(),
                "bound parameter list allocation should succeed"
            );
            let param = (*params).params.as_mut_ptr();
            (*param).value = pg_sys::Int64GetDatum(-7);
            (*param).isnull = false;
            (*param).ptype = pg_sys::INT8OID;
            let param_bytes =
                am::spire_dml_frontdoor_primitive_plan_pk_value_bytes(&param_plan, params)
                    .expect("bound bigint parameter should produce bytea");
            assert_eq!(hex::encode(param_bytes), "fffffffffffffff9");
            let param_invocation =
                am::spire_dml_frontdoor_primitive_invocation_from_plan(&param_plan, params)
                    .expect("bound bigint parameter primitive invocation should be buildable");
            assert_eq!(hex::encode(param_invocation.pk_value), "fffffffffffffff9");
            assert_eq!(param_invocation.pk_column, "id");
            for value in [i64::MAX, i64::MIN] {
                (*param).value = pg_sys::Int64GetDatum(value);
                (*param).isnull = false;
                (*param).ptype = pg_sys::INT8OID;
                let expected = am::spire_dml_frontdoor_bigint_pk_value_bytes(value);
                assert_eq!(
                    am::spire_dml_frontdoor_primitive_plan_pk_value_bytes(&param_plan, params)
                        .expect("boundary bound bigint parameter should produce bytea"),
                    expected,
                    "{value}"
                );
                assert_eq!(
                    am::spire_dml_frontdoor_primitive_invocation_from_plan(&param_plan, params)
                        .expect("boundary bound bigint invocation should be buildable")
                        .pk_value,
                    expected,
                    "{value}"
                );
            }

            (*param).isnull = true;
            let null_error =
                am::spire_dml_frontdoor_primitive_plan_pk_value_bytes(&param_plan, params)
                    .expect_err("NULL bound PK parameter should fail closed");
            assert!(null_error.contains("must not be NULL"), "{null_error}");
        }

        let embedding_update_query = unsafe {
            analyzed_query(
                "UPDATE ec_spire_dml_primitive_plan_sql \
                    SET embedding = '[1,2,3]'::ecvector \
                  WHERE id = 5",
            )
        };
        let embedding_update_decision = unsafe {
            am::spire_dml_frontdoor_replacement_decision_catalog_row(embedding_update_query)
        }
        .expect("embedding UPDATE primitive plan decision should exist");
        let unsupported_error = am::spire_dml_frontdoor_primitive_plan_from_replacement_decision(
            &embedding_update_decision,
        )
        .expect_err("unsupported decisions should not build primitive plans");
        assert!(
            unsupported_error.contains("requires a supported decision"),
            "{unsupported_error}"
        );
    }

    #[pg_test]
    fn test_ec_spire_dml_frontdoor_primitive_plan_sql() {
        Spi::run(
            "CREATE TABLE ec_spire_dml_primitive_plan_diag_sql \
             (id bigint primary key, title text not null, embedding ecvector)",
        )
        .expect("DML primitive plan diagnostic table creation should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_dml_primitive_plan_diag_idx \
             ON ec_spire_dml_primitive_plan_diag_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops)",
        )
        .expect("DML primitive plan diagnostic ec_spire index creation should succeed");

        let select_plan = "FROM ec_spire_dml_frontdoor_primitive_plan_sql(\
             $$SELECT id, title FROM ec_spire_dml_primitive_plan_diag_sql WHERE id = 5$$)";
        let select_summary = Spi::get_one::<String>(&format!(
            "SELECT custom_scan_mode || '|' || primitive || '|' || \
                    pk_column || '|' || pk_value_kind || '|' || \
                    encode(pk_value_bytes, 'hex') || '|' || \
                    array_to_string(projected_columns, ',') || '|' || status \
               {select_plan}"
        ))
        .expect("DML primitive plan diagnostic query should succeed")
        .expect("DML primitive plan diagnostic row should exist");
        assert_eq!(
            select_summary,
            "coordinator_pk_select_tuple_payload|ec_spire_forward_coordinator_select_tuple_payload|id|const_bigint|0000000000000005|id,title|primitive_plan_ready"
        );

        let embedding_update_plan = "FROM ec_spire_dml_frontdoor_primitive_plan_sql(\
             $$UPDATE ec_spire_dml_primitive_plan_diag_sql \
                   SET embedding = '[1,2,3]'::ecvector \
                 WHERE id = 5$$)";
        let embedding_status = Spi::get_one::<String>(&format!(
            "SELECT supported::text || '|' || status || '|' || coalesce(error, '') \
               {embedding_update_plan}"
        ))
        .expect("DML primitive plan unsupported diagnostic query should succeed")
        .expect("DML primitive plan unsupported diagnostic row should exist");
        assert!(
            embedding_status.contains(
                "false|primitive_plan_not_ready|ec_spire DML frontdoor PK argument requires a supported decision"
            ),
            "{embedding_status}"
        );
    }
