    #[pg_test]
    fn test_ec_spire_remote_search_libpq_req_blocked() {
        Spi::run(
            "CREATE TABLE ec_spire_remote_libpq_req_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_remote_libpq_req_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_remote_libpq_req_sql_idx \
             ON ec_spire_remote_libpq_req_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("ec_spire index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_spire_remote_libpq_req_sql_idx'::regclass::oid",
        )
        .expect("index oid query should succeed")
        .expect("index oid should exist");
        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_remote_libpq_req_sql_idx'::regclass)",
        )
        .expect("hierarchy snapshot query should succeed")
        .expect("active epoch should exist");
        let selected_pids = Spi::get_one::<Vec<i64>>(
            "SELECT array_agg(leaf_pid ORDER BY leaf_pid) FROM \
             ec_spire_index_leaf_snapshot('ec_spire_remote_libpq_req_sql_idx'::regclass)",
        )
        .expect("leaf snapshot query should succeed")
        .expect("leaf pids should exist");
        assert_eq!(selected_pids.len(), 2);

        unsafe { am::debug_spire_rewrite_placement_node(index_oid, selected_pids[1] as u64, 2) };
        let plan_from = format!(
            "FROM ec_spire_remote_search_libpq_request_plan(\
             'ec_spire_remote_libpq_req_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{}, {}]::bigint[], 3, 'strict')",
            selected_pids[0], selected_pids[1],
        );
        let summary_from = format!(
            "FROM ec_spire_remote_search_libpq_request_summary(\
             'ec_spire_remote_libpq_req_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{}, {}]::bigint[], 3, 'strict')",
            selected_pids[0], selected_pids[1],
        );
        let connection_from = format!(
            "FROM ec_spire_remote_search_libpq_connection_plan(\
             'ec_spire_remote_libpq_req_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{}, {}]::bigint[], 3, 'strict')",
            selected_pids[0], selected_pids[1],
        );
        let connection_summary_from = format!(
            "FROM ec_spire_remote_search_libpq_connection_summary(\
             'ec_spire_remote_libpq_req_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{}, {}]::bigint[], 3, 'strict')",
            selected_pids[0], selected_pids[1],
        );
        let dispatch_from = format!(
            "FROM ec_spire_remote_search_libpq_dispatch_plan(\
             'ec_spire_remote_libpq_req_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{}, {}]::bigint[], 3, 'strict')",
            selected_pids[0], selected_pids[1],
        );
        let bind_from = format!(
            "FROM ec_spire_remote_search_libpq_bind_plan(\
             'ec_spire_remote_libpq_req_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{}, {}]::bigint[], 3, 'strict')",
            selected_pids[0], selected_pids[1],
        );
        let bind_summary_from = format!(
            "FROM ec_spire_remote_search_libpq_bind_summary(\
             'ec_spire_remote_libpq_req_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{}, {}]::bigint[], 3, 'strict')",
            selected_pids[0], selected_pids[1],
        );
        let work_from = format!(
            "FROM ec_spire_remote_search_libpq_executor_work_plan(\
             'ec_spire_remote_libpq_req_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{}, {}]::bigint[], 3, 'strict')",
            selected_pids[0], selected_pids[1],
        );
        let work_summary_from = format!(
            "FROM ec_spire_remote_search_libpq_executor_work_summary(\
             'ec_spire_remote_libpq_req_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{}, {}]::bigint[], 3, 'strict')",
            selected_pids[0], selected_pids[1],
        );
        let dispatch_summary_from = format!(
            "FROM ec_spire_remote_search_libpq_dispatch_summary(\
             'ec_spire_remote_libpq_req_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{}, {}]::bigint[], 3, 'strict')",
            selected_pids[0], selected_pids[1],
        );
        let executor_from = format!(
            "FROM ec_spire_remote_search_libpq_executor_readiness(\
             'ec_spire_remote_libpq_req_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{}, {}]::bigint[], 3, 'strict')",
            selected_pids[0], selected_pids[1],
        );
        let request_count = Spi::get_one::<i64>(&format!("SELECT count(*) {plan_from}"))
            .expect("libpq request count query should succeed")
            .expect("libpq request count should exist");
        let sql_template = Spi::get_one::<String>(&format!("SELECT sql_template {plan_from}"))
            .expect("libpq request SQL template query should succeed")
            .expect("libpq request SQL template should exist");
        let parameter_count = Spi::get_one::<i64>(&format!("SELECT parameter_count {plan_from}"))
            .expect("libpq request parameter count query should succeed")
            .expect("libpq request parameter count should exist");
        let result_column_count =
            Spi::get_one::<i64>(&format!("SELECT result_column_count {plan_from}"))
                .expect("libpq request result column count query should succeed")
                .expect("libpq request result column count should exist");
        let status = Spi::get_one::<String>(&format!("SELECT status {plan_from}"))
            .expect("libpq request status query should succeed")
            .expect("libpq request status should exist");
        let summary_status = Spi::get_one::<String>(&format!("SELECT status {summary_from}"))
            .expect("libpq request summary status query should succeed")
            .expect("libpq request summary status should exist");
        let blocked_request_count =
            Spi::get_one::<i64>(&format!("SELECT blocked_request_count {summary_from}"))
                .expect("libpq request summary blocked count query should succeed")
                .expect("libpq request summary blocked count should exist");
        let blocked_pid_count =
            Spi::get_one::<i64>(&format!("SELECT blocked_pid_count {summary_from}"))
                .expect("libpq request summary blocked pid query should succeed")
                .expect("libpq request summary blocked pid count should exist");
        let connection_count = Spi::get_one::<i64>(&format!("SELECT count(*) {connection_from}"))
            .expect("libpq connection plan count query should succeed")
            .expect("libpq connection plan count should exist");
        let conninfo_resolution =
            Spi::get_one::<String>(&format!("SELECT conninfo_resolution {connection_from}"))
                .expect("libpq connection resolution query should succeed")
                .expect("libpq connection resolution should exist");
        let pipeline_mode =
            Spi::get_one::<String>(&format!("SELECT pipeline_mode {connection_from}"))
                .expect("libpq connection pipeline query should succeed")
                .expect("libpq connection pipeline should exist");
        let missing_descriptor_connection_count = Spi::get_one::<i64>(&format!(
            "SELECT missing_descriptor_connection_count {connection_summary_from}"
        ))
        .expect("libpq connection summary missing count query should succeed")
        .expect("libpq connection summary missing count should exist");
        let connection_summary_status =
            Spi::get_one::<String>(&format!("SELECT status {connection_summary_from}"))
                .expect("libpq connection summary status query should succeed")
                .expect("libpq connection summary status should exist");
        let dispatch_action =
            Spi::get_one::<String>(&format!("SELECT dispatch_action {dispatch_from}"))
                .expect("libpq dispatch action query should succeed")
                .expect("libpq dispatch action should exist");
        let bind_blocked_count = Spi::get_one::<i64>(&format!(
            "SELECT count(*) {bind_from} WHERE value_status = 'requires_remote_node_descriptor'"
        ))
        .expect("libpq bind blocked count query should succeed")
        .expect("libpq bind blocked count should exist");
        let bind_remote_index_preview = Spi::get_one::<String>(&format!(
            "SELECT value_preview {bind_from} WHERE parameter_name = 'remote_index_oid'"
        ))
        .expect("libpq bind remote index query should succeed")
        .expect("libpq bind remote index preview should exist");
        let bind_summary_bind_count =
            Spi::get_one::<i64>(&format!("SELECT bind_count {bind_summary_from}"))
                .expect("libpq bind summary count query should succeed")
                .expect("libpq bind summary count should exist");
        let bind_summary_blocked_count =
            Spi::get_one::<i64>(&format!("SELECT blocked_bind_count {bind_summary_from}"))
                .expect("libpq bind summary blocked query should succeed")
                .expect("libpq bind summary blocked count should exist");
        let bind_summary_blocked_pid_count =
            Spi::get_one::<i64>(&format!("SELECT blocked_pid_count {bind_summary_from}"))
                .expect("libpq bind summary blocked pid query should succeed")
                .expect("libpq bind summary blocked pid count should exist");
        let bind_summary_status =
            Spi::get_one::<String>(&format!("SELECT status {bind_summary_from}"))
                .expect("libpq bind summary status query should succeed")
                .expect("libpq bind summary status should exist");
        let work_bind_status = Spi::get_one::<String>(&format!("SELECT bind_status {work_from}"))
            .expect("libpq work bind status query should succeed")
            .expect("libpq work bind status should exist");
        let work_next_step =
            Spi::get_one::<String>(&format!("SELECT next_executor_step {work_from}"))
                .expect("libpq work next step query should succeed")
                .expect("libpq work next step should exist");
        let work_action = Spi::get_one::<String>(&format!("SELECT work_action {work_from}"))
            .expect("libpq work action query should succeed")
            .expect("libpq work action should exist");
        let work_status = Spi::get_one::<String>(&format!("SELECT status {work_from}"))
            .expect("libpq work status query should succeed")
            .expect("libpq work status should exist");
        let work_summary_ready_count =
            Spi::get_one::<i64>(&format!("SELECT ready_work_count {work_summary_from}"))
                .expect("libpq work summary ready count query should succeed")
                .expect("libpq work summary ready count should exist");
        let work_summary_blocked_count =
            Spi::get_one::<i64>(&format!("SELECT blocked_work_count {work_summary_from}"))
                .expect("libpq work summary blocked count query should succeed")
                .expect("libpq work summary blocked count should exist");
        let work_summary_status =
            Spi::get_one::<String>(&format!("SELECT status {work_summary_from}"))
                .expect("libpq work summary status query should succeed")
                .expect("libpq work summary status should exist");
        let dispatch_missing_count = Spi::get_one::<i64>(&format!(
            "SELECT missing_descriptor_dispatch_count {dispatch_summary_from}"
        ))
        .expect("libpq dispatch summary missing count query should succeed")
        .expect("libpq dispatch summary missing count should exist");
        let dispatch_summary_status =
            Spi::get_one::<String>(&format!("SELECT status {dispatch_summary_from}"))
                .expect("libpq dispatch summary status query should succeed")
                .expect("libpq dispatch summary status should exist");
        let executor_status = Spi::get_one::<String>(&format!("SELECT status {executor_from}"))
            .expect("libpq executor status query should succeed")
            .expect("libpq executor status should exist");
        let executor_next_step =
            Spi::get_one::<String>(&format!("SELECT next_executor_step {executor_from}"))
                .expect("libpq executor step query should succeed")
                .expect("libpq executor step should exist");

        assert_eq!(request_count, 1);
        assert!(sql_template.contains("ec_spire_remote_search"));
        assert_eq!(parameter_count, 6);
        assert_eq!(result_column_count, 18);
        assert_eq!(status, "requires_remote_node_descriptor");
        assert_eq!(summary_status, "requires_remote_node_descriptor");
        assert_eq!(blocked_request_count, 1);
        assert_eq!(blocked_pid_count, 1);
        assert_eq!(connection_count, 1);
        assert_eq!(conninfo_resolution, "requires_remote_node_descriptor");
        assert_eq!(pipeline_mode, "none");
        assert_eq!(missing_descriptor_connection_count, 1);
        assert_eq!(connection_summary_status, "requires_remote_node_descriptor");
        assert_eq!(dispatch_action, "blocked_before_dispatch");
        assert_eq!(bind_blocked_count, 6);
        assert_eq!(bind_remote_index_preview, "none");
        assert_eq!(bind_summary_bind_count, 6);
        assert_eq!(bind_summary_blocked_count, 6);
        assert_eq!(bind_summary_blocked_pid_count, 1);
        assert_eq!(bind_summary_status, "requires_remote_node_descriptor");
        assert_eq!(work_bind_status, "requires_remote_node_descriptor");
        assert_eq!(work_next_step, "remote_node_descriptor");
        assert_eq!(work_action, "blocked_before_executor");
        assert_eq!(work_status, "requires_remote_node_descriptor");
        assert_eq!(work_summary_ready_count, 0);
        assert_eq!(work_summary_blocked_count, 1);
        assert_eq!(work_summary_status, "requires_remote_node_descriptor");
        assert_eq!(dispatch_missing_count, 1);
        assert_eq!(dispatch_summary_status, "requires_remote_node_descriptor");
        assert_eq!(executor_status, "requires_remote_node_descriptor");
        assert_eq!(executor_next_step, "remote_node_descriptor");
    }

    #[pg_test]
    fn test_ec_spire_remote_search_libpq_req_local() {
        Spi::run(
            "CREATE TABLE ec_spire_remote_libpq_req_local_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_remote_libpq_req_local_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_remote_libpq_req_local_sql_idx \
             ON ec_spire_remote_libpq_req_local_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("ec_spire index creation should succeed");

        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_remote_libpq_req_local_sql_idx'::regclass)",
        )
        .expect("hierarchy snapshot query should succeed")
        .expect("active epoch should exist");
        let selected_pid = Spi::get_one::<i64>(
            "SELECT min(leaf_pid) FROM \
             ec_spire_index_leaf_snapshot('ec_spire_remote_libpq_req_local_sql_idx'::regclass)",
        )
        .expect("leaf snapshot query should succeed")
        .expect("leaf pid should exist");

        let plan_from = format!(
            "FROM ec_spire_remote_search_libpq_request_plan(\
             'ec_spire_remote_libpq_req_local_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{selected_pid}]::bigint[], 3, 'strict')",
        );
        let summary_from = format!(
            "FROM ec_spire_remote_search_libpq_request_summary(\
             'ec_spire_remote_libpq_req_local_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{selected_pid}]::bigint[], 3, 'strict')",
        );
        let connection_from = format!(
            "FROM ec_spire_remote_search_libpq_connection_plan(\
             'ec_spire_remote_libpq_req_local_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{selected_pid}]::bigint[], 3, 'strict')",
        );
        let connection_summary_from = format!(
            "FROM ec_spire_remote_search_libpq_connection_summary(\
             'ec_spire_remote_libpq_req_local_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{selected_pid}]::bigint[], 3, 'strict')",
        );
        let dispatch_from = format!(
            "FROM ec_spire_remote_search_libpq_dispatch_plan(\
             'ec_spire_remote_libpq_req_local_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{selected_pid}]::bigint[], 3, 'strict')",
        );
        let dispatch_summary_from = format!(
            "FROM ec_spire_remote_search_libpq_dispatch_summary(\
             'ec_spire_remote_libpq_req_local_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{selected_pid}]::bigint[], 3, 'strict')",
        );
        let executor_from = format!(
            "FROM ec_spire_remote_search_libpq_executor_readiness(\
             'ec_spire_remote_libpq_req_local_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{selected_pid}]::bigint[], 3, 'strict')",
        );
        let receive_summary_from = format!(
            "FROM ec_spire_remote_search_receive_summary(\
             'ec_spire_remote_libpq_req_local_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{selected_pid}]::bigint[], 3, 'strict')",
        );
        let request_count = Spi::get_one::<i64>(&format!("SELECT count(*) {plan_from}"))
            .expect("local libpq request count query should succeed")
            .expect("local libpq request count should exist");
        let summary_request_count =
            Spi::get_one::<i64>(&format!("SELECT request_count {summary_from}"))
                .expect("local libpq summary request count query should succeed")
                .expect("local libpq summary request count should exist");
        let summary_status = Spi::get_one::<String>(&format!("SELECT status {summary_from}"))
            .expect("local libpq summary status query should succeed")
            .expect("local libpq summary status should exist");
        let connection_count = Spi::get_one::<i64>(&format!("SELECT count(*) {connection_from}"))
            .expect("local libpq connection count query should succeed")
            .expect("local libpq connection count should exist");
        let connection_summary_count = Spi::get_one::<i64>(&format!(
            "SELECT connection_count {connection_summary_from}"
        ))
        .expect("local libpq connection summary count query should succeed")
        .expect("local libpq connection summary count should exist");
        let connection_summary_status =
            Spi::get_one::<String>(&format!("SELECT status {connection_summary_from}"))
                .expect("local libpq connection summary status query should succeed")
                .expect("local libpq connection summary status should exist");
        let dispatch_count = Spi::get_one::<i64>(&format!("SELECT count(*) {dispatch_from}"))
            .expect("local libpq dispatch count query should succeed")
            .expect("local libpq dispatch count should exist");
        let dispatch_summary_count =
            Spi::get_one::<i64>(&format!("SELECT dispatch_count {dispatch_summary_from}"))
                .expect("local libpq dispatch summary count query should succeed")
                .expect("local libpq dispatch summary count should exist");
        let dispatch_summary_status =
            Spi::get_one::<String>(&format!("SELECT status {dispatch_summary_from}"))
                .expect("local libpq dispatch summary status query should succeed")
                .expect("local libpq dispatch summary status should exist");
        let executor_status = Spi::get_one::<String>(&format!("SELECT status {executor_from}"))
            .expect("local libpq executor status query should succeed")
            .expect("local libpq executor status should exist");
        let executor_next_step =
            Spi::get_one::<String>(&format!("SELECT next_executor_step {executor_from}"))
                .expect("local libpq executor step query should succeed")
                .expect("local libpq executor step should exist");
        let receive_summary_count =
            Spi::get_one::<i64>(&format!("SELECT receive_count {receive_summary_from}"))
                .expect("local receive summary count query should succeed")
                .expect("local receive summary count should exist");
        let receive_summary_status =
            Spi::get_one::<String>(&format!("SELECT status {receive_summary_from}"))
                .expect("local receive summary status query should succeed")
                .expect("local receive summary status should exist");

        assert_eq!(request_count, 0);
        assert_eq!(summary_request_count, 0);
        assert_eq!(connection_count, 0);
        assert_eq!(connection_summary_count, 0);
        assert_eq!(dispatch_count, 0);
        assert_eq!(dispatch_summary_count, 0);
        assert_eq!(summary_status, "ready");
        assert_eq!(connection_summary_status, "ready");
        assert_eq!(dispatch_summary_status, "ready");
        assert_eq!(executor_status, "ready");
        assert_eq!(executor_next_step, "none");
        assert_eq!(receive_summary_count, 0);
        assert_eq!(receive_summary_status, "ready");
    }

    #[pg_test]
    fn test_ec_spire_remote_search_receive_contract() {
        let parameter_contract_from = "FROM ec_spire_remote_search_libpq_parameter_contract()";
        let executor_contract_from = "FROM ec_spire_remote_search_libpq_executor_step_contract()";
        let result_contract_from = "FROM ec_spire_remote_search_libpq_result_contract()";
        let endpoint_contract_from = "FROM ec_spire_remote_search_endpoint_contract()";
        let parameter_count =
            Spi::get_one::<i64>(&format!("SELECT count(*) {parameter_contract_from}"))
                .expect("parameter contract count query should succeed")
                .expect("parameter contract count should exist");
        let first_parameter = Spi::get_one::<String>(&format!(
            "SELECT parameter_name {parameter_contract_from} WHERE parameter_ordinal = 1"
        ))
        .expect("parameter contract first parameter query should succeed")
        .expect("parameter contract first parameter should exist");
        let selected_pids_validator = Spi::get_one::<String>(&format!(
            "SELECT validator {parameter_contract_from} WHERE parameter_name = 'selected_pids'"
        ))
        .expect("parameter contract selected pids query should succeed")
        .expect("parameter contract selected pids validator should exist");
        let consistency_mode_role = Spi::get_one::<String>(&format!(
            "SELECT semantic_role {parameter_contract_from} WHERE parameter_name = 'consistency_mode'"
        ))
        .expect("parameter contract consistency mode query should succeed")
        .expect("parameter contract consistency mode role should exist");
        let column_count = Spi::get_one::<i64>(&format!("SELECT count(*) {result_contract_from}"))
            .expect("result contract count query should succeed")
            .expect("result contract count should exist");
        let first_column = Spi::get_one::<String>(&format!(
            "SELECT column_name {result_contract_from} WHERE column_ordinal = 1"
        ))
        .expect("result contract first column query should succeed")
        .expect("result contract first column should exist");
        let score_validator = Spi::get_one::<String>(&format!(
            "SELECT validator {result_contract_from} WHERE column_name = 'score'"
        ))
        .expect("result contract score validator query should succeed")
        .expect("result contract score validator should exist");
        let pid_validator = Spi::get_one::<String>(&format!(
            "SELECT validator {result_contract_from} WHERE column_name = 'pid'"
        ))
        .expect("result contract pid validator query should succeed")
        .expect("result contract pid validator should exist");
        let nullable_count = Spi::get_one::<i64>(&format!(
            "SELECT count(*) {result_contract_from} WHERE nullable"
        ))
        .expect("result contract nullable count query should succeed")
        .expect("result contract nullable count should exist");
        let executor_step_count =
            Spi::get_one::<i64>(&format!("SELECT count(*) {executor_contract_from}"))
                .expect("executor contract count query should succeed")
                .expect("executor contract count should exist");
        let first_executor_step = Spi::get_one::<String>(&format!(
            "SELECT step_name {executor_contract_from} WHERE step_ordinal = 1"
        ))
        .expect("executor contract first step query should succeed")
        .expect("executor contract first step should exist");
        let secret_step_action = Spi::get_one::<String>(&format!(
            "SELECT executor_action {executor_contract_from} \
             WHERE step_name = 'conninfo_secret_resolution'"
        ))
        .expect("executor contract secret step query should succeed")
        .expect("executor contract secret step action should exist");
        let budget_step_validator = Spi::get_one::<String>(&format!(
            "SELECT validator {executor_contract_from} \
             WHERE step_name = 'remote_executor_budget'"
        ))
        .expect("executor contract budget step query should succeed")
        .expect("executor contract budget step validator should exist");
        let merge_step_validator = Spi::get_one::<String>(&format!(
            "SELECT validator {executor_contract_from} \
             WHERE step_name = 'merge_validated_remote_search_candidate_batches'"
        ))
        .expect("executor contract merge step query should succeed")
        .expect("executor contract merge step validator should exist");
        let endpoint_count =
            Spi::get_one::<i64>(&format!("SELECT count(*) {endpoint_contract_from}"))
                .expect("endpoint contract count query should succeed")
                .expect("endpoint contract count should exist");
        let endpoint_protocol = Spi::get_one::<String>(&format!(
            "SELECT contract_value {endpoint_contract_from} \
             WHERE contract_item = 'protocol_version'"
        ))
        .expect("endpoint contract protocol query should succeed")
        .expect("endpoint contract protocol should exist");
        let endpoint_quantizer = Spi::get_one::<String>(&format!(
            "SELECT contract_value {endpoint_contract_from} \
             WHERE contract_item = 'quantizer_family'"
        ))
        .expect("endpoint contract quantizer query should succeed")
        .expect("endpoint contract quantizer should exist");
        let endpoint_tuple_transport_default = Spi::get_one::<String>(&format!(
            "SELECT contract_value {endpoint_contract_from} \
             WHERE contract_item = 'tuple_transport_default'"
        ))
        .expect("endpoint contract tuple transport query should succeed")
        .expect("endpoint contract tuple transport should exist");
        let endpoint_tuple_transport_validator = Spi::get_one::<String>(&format!(
            "SELECT validator {endpoint_contract_from} \
             WHERE contract_item = 'tuple_transport_capabilities'"
        ))
        .expect("endpoint contract tuple transport validator query should succeed")
        .expect("endpoint contract tuple transport validator should exist");
        let fingerprint_status = Spi::get_one::<String>(&format!(
            "SELECT status {endpoint_contract_from} \
             WHERE contract_item = 'quantizer_index_fingerprint_binding'"
        ))
        .expect("endpoint contract fingerprint query should succeed")
        .expect("endpoint contract fingerprint status should exist");
        let direct_sql_policy_validator = Spi::get_one::<String>(&format!(
            "SELECT validator {endpoint_contract_from} \
             WHERE contract_item = 'direct_sql_endpoint_status_policy'"
        ))
        .expect("endpoint contract direct SQL policy query should succeed")
        .expect("endpoint contract direct SQL policy should exist");
        let heap_preflight_validator = Spi::get_one::<String>(&format!(
            "SELECT validator {endpoint_contract_from} \
             WHERE contract_item = 'remote_heap_candidate_endpoint_identity_preflight'"
        ))
        .expect("endpoint contract heap preflight query should succeed")
        .expect("endpoint contract heap preflight should exist");
        let non_ready_endpoint_rows = Spi::get_one::<i64>(&format!(
            "SELECT count(*) {endpoint_contract_from} WHERE status <> 'ready'"
        ))
        .expect("endpoint contract non-ready query should succeed")
        .expect("endpoint contract non-ready count should exist");

        assert_eq!(parameter_count, 6);
        assert_eq!(first_parameter, "remote_index_oid");
        assert_eq!(
            selected_pids_validator,
            "must_be_nonempty_positive_unique_remote_leaf_pids_delta_rows_are_leaf_derived"
        );
        assert_eq!(consistency_mode_role, "strict_or_degraded_policy");
        assert_eq!(column_count, 18);
        assert_eq!(first_column, "served_epoch");
        assert_eq!(
            pid_validator,
            "must_be_selected_leaf_pid_or_leaf_derived_delta_pid"
        );
        assert_eq!(score_validator, "must_be_finite");
        assert_eq!(nullable_count, 0);
        assert_eq!(executor_step_count, 10);
        assert_eq!(first_executor_step, "remote_node_descriptor");
        assert_eq!(secret_step_action, "resolve_conninfo_secret_reference");
        assert_eq!(
            budget_step_validator,
            "must_block_over_budget_rows_before_secret_lookup_or_socket_open"
        );
        assert_eq!(merge_step_validator, "must_preserve_merge_order_contract");
        assert_eq!(endpoint_count, 15);
        assert_eq!(endpoint_protocol, "ec_spire_remote_search_v1");
        assert_eq!(endpoint_quantizer, "rabitq_only_pq_and_pqfastscan_reserved");
        assert_eq!(endpoint_tuple_transport_default, "pg_binary_attr_v1");
        assert_eq!(
            endpoint_tuple_transport_validator,
            "must_advertise_pg_binary_attr_v1_before_custom_scan_typed_receive"
        );
        assert_eq!(fingerprint_status, "requires_fingerprint_binding");
        assert_eq!(
            direct_sql_policy_validator,
            "must_not_treat_direct_sql_rows_as_mergeable_without_libpq_receive_validation"
        );
        assert_eq!(
            heap_preflight_validator,
            "must_validate_ready_endpoint_identity_before_remote_heap_candidate_merge"
        );
        assert_eq!(non_ready_endpoint_rows, 3);
    }

    #[pg_test]
    fn test_ec_spire_remote_search_endpoint_identity() {
        Spi::run(
            "CREATE TABLE ec_spire_endpoint_identity_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_endpoint_identity_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_endpoint_identity_default_idx \
             ON ec_spire_endpoint_identity_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("default ec_spire index creation should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_endpoint_identity_rabitq_idx \
             ON ec_spire_endpoint_identity_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) \
             WITH (nlists = 2, storage_format = 'rabitq')",
        )
        .expect("rabitq ec_spire index creation should succeed");

        let default_from =
            "FROM ec_spire_remote_search_endpoint_identity('ec_spire_endpoint_identity_default_idx'::regclass)";
        let rabitq_from =
            "FROM ec_spire_remote_search_endpoint_identity('ec_spire_endpoint_identity_rabitq_idx'::regclass)";
        let default_status = Spi::get_one::<String>(&format!("SELECT status {default_from}"))
            .expect("default identity status query should succeed")
            .expect("default identity status should exist");
        let default_assignment_payload =
            Spi::get_one::<String>(&format!("SELECT assignment_payload_format {default_from}"))
                .expect("default identity payload query should succeed")
                .expect("default identity payload should exist");
        let rabitq_status = Spi::get_one::<String>(&format!("SELECT status {rabitq_from}"))
            .expect("rabitq identity status query should succeed")
            .expect("rabitq identity status should exist");
        let protocol_version =
            Spi::get_one::<String>(&format!("SELECT protocol_version {rabitq_from}"))
                .expect("rabitq identity protocol query should succeed")
                .expect("rabitq identity protocol should exist");
        let extension_version =
            Spi::get_one::<String>(&format!("SELECT extension_version {rabitq_from}"))
                .expect("rabitq identity extension query should succeed")
                .expect("rabitq identity extension should exist");
        let opclass_identity =
            Spi::get_one::<String>(&format!("SELECT opclass_identity {rabitq_from}"))
                .expect("rabitq identity opclass query should succeed")
                .expect("rabitq identity opclass should exist");
        let storage_format =
            Spi::get_one::<String>(&format!("SELECT storage_format {rabitq_from}"))
                .expect("rabitq identity storage query should succeed")
                .expect("rabitq identity storage should exist");
        let assignment_payload =
            Spi::get_one::<String>(&format!("SELECT assignment_payload_format {rabitq_from}"))
                .expect("rabitq identity payload query should succeed")
                .expect("rabitq identity payload should exist");
        let quantizer_profile =
            Spi::get_one::<String>(&format!("SELECT quantizer_profile {rabitq_from}"))
                .expect("rabitq identity profile query should succeed")
                .expect("rabitq identity profile should exist");
        let scoring_profile =
            Spi::get_one::<String>(&format!("SELECT scoring_profile {rabitq_from}"))
                .expect("rabitq identity scoring query should succeed")
                .expect("rabitq identity scoring should exist");
        let tuple_transport_capabilities = Spi::get_one::<Vec<String>>(&format!(
            "SELECT tuple_transport_capabilities {rabitq_from}"
        ))
        .expect("rabitq tuple transport capabilities query should succeed")
        .expect("rabitq tuple transport capabilities should exist");
        let tuple_transport_default =
            Spi::get_one::<String>(&format!("SELECT tuple_transport_default {rabitq_from}"))
                .expect("rabitq tuple transport default query should succeed")
                .expect("rabitq tuple transport default should exist");
        let tuple_transport_status =
            Spi::get_one::<String>(&format!("SELECT tuple_transport_status {rabitq_from}"))
                .expect("rabitq tuple transport status query should succeed")
                .expect("rabitq tuple transport status should exist");
        let fingerprint_length =
            Spi::get_one::<i32>(&format!("SELECT length(profile_fingerprint) {rabitq_from}"))
                .expect("rabitq identity fingerprint query should succeed")
                .expect("rabitq identity fingerprint should exist");
        let fingerprint_before_reindex =
            Spi::get_one::<String>(&format!("SELECT profile_fingerprint {rabitq_from}"))
                .expect("rabitq identity fingerprint query should succeed")
                .expect("rabitq identity fingerprint should exist");

        Spi::run("REINDEX INDEX ec_spire_endpoint_identity_rabitq_idx")
            .expect("reindex should succeed");
        let fingerprint_after_reindex =
            Spi::get_one::<String>(&format!("SELECT profile_fingerprint {rabitq_from}"))
                .expect("reindexed rabitq identity fingerprint query should succeed")
                .expect("reindexed rabitq identity fingerprint should exist");

        assert_eq!(default_status, "requires_rabitq_storage_format");
        assert_eq!(default_assignment_payload, "turboquant");
        assert_eq!(rabitq_status, "ready");
        assert_eq!(protocol_version, "ec_spire_remote_search_v1");
        assert_eq!(extension_version, env!("CARGO_PKG_VERSION"));
        assert_eq!(opclass_identity, "ecvector_spire_ip_ops");
        assert_eq!(storage_format, "rabitq");
        assert_eq!(assignment_payload, "rabitq");
        assert_eq!(quantizer_profile, "rabitq_v1");
        assert_eq!(scoring_profile, "inner_product_score_v1");
        assert_eq!(tuple_transport_capabilities, vec!["pg_binary_attr_v1"]);
        assert_eq!(tuple_transport_default, "pg_binary_attr_v1");
        assert_eq!(tuple_transport_status, "ready");
        assert_eq!(fingerprint_length, 16);
        assert_ne!(fingerprint_before_reindex, fingerprint_after_reindex);
    }

    #[pg_test]
    fn test_ec_spire_remote_search_receive_plan_blocked() {
        Spi::run(
            "CREATE TABLE ec_spire_remote_receive_plan_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_remote_receive_plan_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_remote_receive_plan_sql_idx \
             ON ec_spire_remote_receive_plan_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("ec_spire index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_spire_remote_receive_plan_sql_idx'::regclass::oid",
        )
        .expect("index oid query should succeed")
        .expect("index oid should exist");
        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_remote_receive_plan_sql_idx'::regclass)",
        )
        .expect("hierarchy snapshot query should succeed")
        .expect("active epoch should exist");
        let selected_pids = Spi::get_one::<Vec<i64>>(
            "SELECT array_agg(leaf_pid ORDER BY leaf_pid) FROM \
             ec_spire_index_leaf_snapshot('ec_spire_remote_receive_plan_sql_idx'::regclass)",
        )
        .expect("leaf snapshot query should succeed")
        .expect("leaf pids should exist");
        assert_eq!(selected_pids.len(), 2);

        unsafe { am::debug_spire_rewrite_placement_node(index_oid, selected_pids[1] as u64, 2) };
        let receive_from = format!(
            "FROM ec_spire_remote_search_receive_plan(\
             'ec_spire_remote_receive_plan_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{}, {}]::bigint[], 3, 'strict')",
            selected_pids[0], selected_pids[1],
        );
        let receive_summary_from = format!(
            "FROM ec_spire_remote_search_receive_summary(\
             'ec_spire_remote_receive_plan_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{}, {}]::bigint[], 3, 'strict')",
            selected_pids[0], selected_pids[1],
        );
        let row_count = Spi::get_one::<i64>(&format!("SELECT count(*) {receive_from}"))
            .expect("receive plan count query should succeed")
            .expect("receive plan count should exist");
        let validator_function =
            Spi::get_one::<String>(&format!("SELECT validator_function {receive_from}"))
                .expect("receive plan validator query should succeed")
                .expect("receive plan validator should exist");
        let row_locator_policy =
            Spi::get_one::<String>(&format!("SELECT row_locator_policy {receive_from}"))
                .expect("receive plan locator policy query should succeed")
                .expect("receive plan locator policy should exist");
        let candidate_format =
            Spi::get_one::<String>(&format!("SELECT expected_candidate_format {receive_from}"))
                .expect("receive plan candidate format query should succeed")
                .expect("receive plan candidate format should exist");
        let status = Spi::get_one::<String>(&format!("SELECT status {receive_from}"))
            .expect("receive plan status query should succeed")
            .expect("receive plan status should exist");
        let summary_receive_count =
            Spi::get_one::<i64>(&format!("SELECT receive_count {receive_summary_from}"))
                .expect("receive summary count query should succeed")
                .expect("receive summary count should exist");
        let summary_ready_count = Spi::get_one::<i64>(&format!(
            "SELECT ready_receive_count {receive_summary_from}"
        ))
        .expect("receive summary ready count query should succeed")
        .expect("receive summary ready count should exist");
        let summary_blocked_count = Spi::get_one::<i64>(&format!(
            "SELECT blocked_receive_count {receive_summary_from}"
        ))
        .expect("receive summary blocked count query should succeed")
        .expect("receive summary blocked count should exist");
        let summary_blocked_pid_count =
            Spi::get_one::<i64>(&format!("SELECT blocked_pid_count {receive_summary_from}"))
                .expect("receive summary blocked pid query should succeed")
                .expect("receive summary blocked pid count should exist");
        let summary_status =
            Spi::get_one::<String>(&format!("SELECT status {receive_summary_from}"))
                .expect("receive summary status query should succeed")
                .expect("receive summary status should exist");

        assert_eq!(row_count, 1);
        assert_eq!(validator_function, "validate_remote_search_candidate_batch");
        assert_eq!(row_locator_policy, "opaque_origin_node_bytes");
        assert_eq!(candidate_format, "ec_spire_remote_search_v1");
        assert_eq!(status, "requires_remote_node_descriptor");
        assert_eq!(summary_receive_count, 1);
        assert_eq!(summary_ready_count, 0);
        assert_eq!(summary_blocked_count, 1);
        assert_eq!(summary_blocked_pid_count, 1);
        assert_eq!(summary_status, "requires_remote_node_descriptor");
    }

    #[pg_test]
    fn test_ec_spire_remote_search_receive_merge_summary() {
        Spi::run(
            "CREATE TABLE ec_spire_remote_receive_merge_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_remote_receive_merge_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_remote_receive_merge_sql_idx \
             ON ec_spire_remote_receive_merge_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("ec_spire index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_spire_remote_receive_merge_sql_idx'::regclass::oid",
        )
        .expect("index oid query should succeed")
        .expect("index oid should exist");
        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_remote_receive_merge_sql_idx'::regclass)",
        )
        .expect("hierarchy snapshot query should succeed")
        .expect("active epoch should exist");
        let selected_pids = Spi::get_one::<Vec<i64>>(
            "SELECT array_agg(leaf_pid ORDER BY leaf_pid) FROM \
             ec_spire_index_leaf_snapshot('ec_spire_remote_receive_merge_sql_idx'::regclass)",
        )
        .expect("leaf snapshot query should succeed")
        .expect("leaf pids should exist");
        assert_eq!(selected_pids.len(), 2);

        unsafe { am::debug_spire_rewrite_placement_node(index_oid, selected_pids[1] as u64, 2) };
        let summary_from = format!(
            "FROM ec_spire_remote_search_merge_input_summary(\
             'ec_spire_remote_receive_merge_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{}, {}]::bigint[], 3, 'strict')",
            selected_pids[0], selected_pids[1],
        );
        let remote_batch_count =
            Spi::get_one::<i64>(&format!("SELECT remote_batch_count {summary_from}"))
                .expect("merge input remote batch query should succeed")
                .expect("merge input remote batch count should exist");
        let local_batch_count =
            Spi::get_one::<i64>(&format!("SELECT local_batch_count {summary_from}"))
                .expect("merge input local batch query should succeed")
                .expect("merge input local batch count should exist");
        let blocked_batch_count =
            Spi::get_one::<i64>(&format!("SELECT blocked_batch_count {summary_from}"))
                .expect("merge input blocked batch query should succeed")
                .expect("merge input blocked batch count should exist");
        let merge_function =
            Spi::get_one::<String>(&format!("SELECT merge_function {summary_from}"))
                .expect("merge input merge function query should succeed")
                .expect("merge input merge function should exist");
        let dedupe_key = Spi::get_one::<String>(&format!("SELECT dedupe_key {summary_from}"))
            .expect("merge input dedupe key query should succeed")
            .expect("merge input dedupe key should exist");
        let tie_breaker = Spi::get_one::<String>(&format!("SELECT tie_breaker {summary_from}"))
            .expect("merge input tie-breaker query should succeed")
            .expect("merge input tie-breaker should exist");
        let status = Spi::get_one::<String>(&format!("SELECT status {summary_from}"))
            .expect("merge input status query should succeed")
            .expect("merge input status should exist");

        assert_eq!(remote_batch_count, 1);
        assert_eq!(local_batch_count, 1);
        assert_eq!(blocked_batch_count, 1);
        assert_eq!(
            merge_function,
            "merge_validated_remote_search_candidate_batches"
        );
        assert_eq!(dedupe_key, "global_vec_id_or_node_scoped_local_vec_id");
        assert_eq!(
            tie_breaker,
            "score_then_assignment_role_then_epoch_desc_then_node_pid_version_row_locator"
        );
        assert_eq!(status, "requires_remote_node_descriptor");
    }

    #[pg_test]
    fn test_ec_spire_remote_search_final_contract() {
        let locator_contract_from = "FROM ec_spire_remote_search_row_locator_contract()";
        let identity_contract_from = "FROM ec_spire_remote_search_vector_identity_contract()";
        let heap_contract_from = "FROM ec_spire_remote_search_heap_resolution_contract()";
        let row_count = Spi::get_one::<i64>(&format!("SELECT count(*) {locator_contract_from}"))
            .expect("row locator contract count query should succeed")
            .expect("row locator contract count should exist");
        let identity_count =
            Spi::get_one::<i64>(&format!("SELECT count(*) {identity_contract_from}"))
                .expect("vector identity contract count query should succeed")
                .expect("vector identity contract count should exist");
        let dedupe_key = Spi::get_one::<String>(&format!(
            "SELECT contract_value {identity_contract_from} \
             WHERE contract_item = 'remote_merge_dedupe_key'"
        ))
        .expect("vector identity dedupe key query should succeed")
        .expect("vector identity dedupe key should exist");
        let local_scope = Spi::get_one::<String>(&format!(
            "SELECT contract_value {identity_contract_from} \
             WHERE contract_item = 'local_vec_id_remote_scope'"
        ))
        .expect("vector identity local scope query should succeed")
        .expect("vector identity local scope should exist");
        let writer_global_source_identity = Spi::get_one::<String>(&format!(
            "SELECT contract_value {identity_contract_from} \
             WHERE contract_item = 'writer_global_source_identity'"
        ))
        .expect("vector identity writer source query should succeed")
        .expect("vector identity writer source should exist");
        let writer_global_base_storage_status = Spi::get_one::<String>(&format!(
            "SELECT status {identity_contract_from} \
             WHERE contract_item = 'writer_global_base_storage'"
        ))
        .expect("vector identity writer storage query should succeed")
        .expect("vector identity writer storage should exist");
        let interpretation = Spi::get_one::<String>(&format!(
            "SELECT contract_value {locator_contract_from} \
             WHERE contract_item = 'coordinator_interpretation'"
        ))
        .expect("row locator interpretation query should succeed")
        .expect("row locator interpretation should exist");
        let remote_resolution_status = Spi::get_one::<String>(&format!(
            "SELECT status {locator_contract_from} \
             WHERE contract_item = 'remote_heap_resolution'"
        ))
        .expect("row locator remote resolution status query should succeed")
        .expect("row locator remote resolution status should exist");
        let heap_resolution_count =
            Spi::get_one::<i64>(&format!("SELECT count(*) {heap_contract_from}"))
                .expect("heap resolution contract count query should succeed")
                .expect("heap resolution contract count should exist");
        let local_heap_owner = Spi::get_one::<String>(&format!(
            "SELECT heap_lookup_owner {heap_contract_from} WHERE resolution_scope = 'local'"
        ))
        .expect("local heap resolution owner query should succeed")
        .expect("local heap resolution owner should exist");
        let remote_heap_status = Spi::get_one::<String>(&format!(
            "SELECT status {heap_contract_from} WHERE resolution_scope = 'remote'"
        ))
        .expect("remote heap resolution status query should succeed")
        .expect("remote heap resolution status should exist");
        let remote_locator_policy = Spi::get_one::<String>(&format!(
            "SELECT row_locator_policy {heap_contract_from} WHERE resolution_scope = 'remote'"
        ))
        .expect("remote heap resolution locator query should succeed")
        .expect("remote heap resolution locator should exist");

        assert_eq!(row_count, 4);
        assert_eq!(identity_count, 10);
        assert_eq!(dedupe_key, "global_vec_id_or_node_scoped_local_vec_id");
        assert_eq!(local_scope, "node_id || local_vec_id_bytes");
        assert_eq!(
            writer_global_source_identity,
            "fixed_16_byte_source_identity_required_not_heap_tid"
        );
        assert_eq!(writer_global_base_storage_status, "phase11_2_landed");
        assert_eq!(interpretation, "opaque_bytes");
        assert_eq!(remote_resolution_status, "deferred_until_remote_heap_fetch");
        assert_eq!(heap_resolution_count, 2);
        assert_eq!(local_heap_owner, "coordinator_local_heap");
        assert_eq!(remote_heap_status, "requires_remote_heap_resolution");
        assert_eq!(remote_locator_policy, "opaque_origin_node_bytes");
    }

    #[pg_test]
    fn test_ec_spire_remote_search_local_heap_resolution_plan() {
        Spi::run(
            "CREATE TABLE ec_spire_remote_local_heap_res_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_remote_local_heap_res_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_remote_local_heap_res_sql_idx \
             ON ec_spire_remote_local_heap_res_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("ec_spire index creation should succeed");

        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_remote_local_heap_res_sql_idx'::regclass)",
        )
        .expect("hierarchy snapshot query should succeed")
        .expect("active epoch should exist");
        let selected_pids = Spi::get_one::<Vec<i64>>(
            "SELECT array_agg(leaf_pid ORDER BY leaf_pid) FROM \
             ec_spire_index_leaf_snapshot('ec_spire_remote_local_heap_res_sql_idx'::regclass)",
        )
        .expect("leaf snapshot query should succeed")
        .expect("leaf pids should exist");
        assert_eq!(selected_pids.len(), 2);

        let plan_from = format!(
            "FROM ec_spire_remote_search_local_heap_resolution_plan(\
             'ec_spire_remote_local_heap_res_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{}, {}]::bigint[], 2, 'strict')",
            selected_pids[0], selected_pids[1],
        );
        let summary_from = format!(
            "FROM ec_spire_remote_search_heap_resolution_summary(\
             'ec_spire_remote_local_heap_res_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{}, {}]::bigint[], 2, 'strict')",
            selected_pids[0], selected_pids[1],
        );
        let candidates_from = format!(
            "FROM ec_spire_remote_search_local_heap_candidates(\
             'ec_spire_remote_local_heap_res_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{}, {}]::bigint[], 2, 'strict')",
            selected_pids[0], selected_pids[1],
        );
        let candidate_summary_from = format!(
            "FROM ec_spire_remote_search_local_heap_candidate_summary(\
             'ec_spire_remote_local_heap_res_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{}, {}]::bigint[], 2, 'strict')",
            selected_pids[0], selected_pids[1],
        );
        let result_summary_from = format!(
            "FROM ec_spire_remote_search_coordinator_result_summary(\
             'ec_spire_remote_local_heap_res_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{}, {}]::bigint[], 2, 'strict')",
            selected_pids[0], selected_pids[1],
        );
        let row_count = Spi::get_one::<i64>(&format!("SELECT count(*) {plan_from}"))
            .expect("local heap resolution count query should succeed")
            .expect("local heap resolution count should exist");
        let ready_count = Spi::get_one::<i64>(&format!(
            "SELECT count(*) {plan_from} WHERE status = 'ready'"
        ))
        .expect("local heap resolution ready count query should succeed")
        .expect("local heap resolution ready count should exist");
        let owner_count = Spi::get_one::<i64>(&format!(
            "SELECT count(*) {plan_from} WHERE heap_lookup_owner = 'coordinator_local_heap'"
        ))
        .expect("local heap resolution owner count query should succeed")
        .expect("local heap resolution owner count should exist");
        let decoded_locator_count = Spi::get_one::<i64>(&format!(
            "SELECT count(*) {plan_from} \
             WHERE heap_block >= 0 AND heap_offset > 0 AND length(row_locator) = 6"
        ))
        .expect("local heap resolution locator query should succeed")
        .expect("local heap resolution locator count should exist");
        let node_count =
            Spi::get_one::<i64>(&format!("SELECT count(DISTINCT node_id) {plan_from}"))
                .expect("local heap resolution node count query should succeed")
                .expect("local heap resolution node count should exist");
        let summary_status = Spi::get_one::<String>(&format!("SELECT status {summary_from}"))
            .expect("heap resolution summary status query should succeed")
            .expect("heap resolution summary status should exist");
        let decoded_summary_count = Spi::get_one::<i64>(&format!(
            "SELECT decoded_local_locator_count {summary_from}"
        ))
        .expect("heap resolution summary decoded count query should succeed")
        .expect("heap resolution summary decoded count should exist");
        let local_resolution_status = Spi::get_one::<String>(&format!(
            "SELECT local_heap_resolution_status {summary_from}"
        ))
        .expect("heap resolution summary local status query should succeed")
        .expect("heap resolution summary local status should exist");
        let remote_resolution_status = Spi::get_one::<String>(&format!(
            "SELECT remote_heap_resolution_status {summary_from}"
        ))
        .expect("heap resolution summary remote status query should succeed")
        .expect("heap resolution summary remote status should exist");
        let candidate_count = Spi::get_one::<i64>(&format!("SELECT count(*) {candidates_from}"))
            .expect("local heap candidate count query should succeed")
            .expect("local heap candidate count should exist");
        let candidate_owner_count = Spi::get_one::<i64>(&format!(
            "SELECT count(*) {candidates_from} \
             WHERE heap_lookup_owner = 'coordinator_local_heap' AND status = 'ready'"
        ))
        .expect("local heap candidate owner query should succeed")
        .expect("local heap candidate owner count should exist");
        let candidate_locator_count = Spi::get_one::<i64>(&format!(
            "SELECT count(*) {candidates_from} \
             WHERE served_epoch = requested_epoch \
             AND heap_block >= 0 AND heap_offset > 0 \
             AND length(row_locator) = 6 AND score IS NOT NULL"
        ))
        .expect("local heap candidate locator query should succeed")
        .expect("local heap candidate locator count should exist");
        let returned_candidate_count = Spi::get_one::<i64>(&format!(
            "SELECT returned_candidate_count {candidate_summary_from}"
        ))
        .expect("local heap candidate summary return query should succeed")
        .expect("local heap candidate summary return count should exist");
        let candidate_summary_status =
            Spi::get_one::<String>(&format!("SELECT status {candidate_summary_from}"))
                .expect("local heap candidate summary status query should succeed")
                .expect("local heap candidate summary status should exist");
        let result_source =
            Spi::get_one::<String>(&format!("SELECT result_source {result_summary_from}"))
                .expect("coordinator result source query should succeed")
                .expect("coordinator result source should exist");
        let result_receive_count =
            Spi::get_one::<i64>(&format!("SELECT libpq_receive_count {result_summary_from}"))
                .expect("coordinator result receive count query should succeed")
                .expect("coordinator result receive count should exist");
        let result_receive_status = Spi::get_one::<String>(&format!(
            "SELECT libpq_receive_status {result_summary_from}"
        ))
        .expect("coordinator result receive status query should succeed")
        .expect("coordinator result receive status should exist");
        let result_status = Spi::get_one::<String>(&format!("SELECT status {result_summary_from}"))
            .expect("coordinator result status query should succeed")
            .expect("coordinator result status should exist");
        let result_returned_candidate_count = Spi::get_one::<i64>(&format!(
            "SELECT returned_candidate_count {result_summary_from}"
        ))
        .expect("coordinator result returned count query should succeed")
        .expect("coordinator result returned count should exist");
        let result_next_blocker =
            Spi::get_one::<String>(&format!("SELECT next_blocker {result_summary_from}"))
                .expect("coordinator result blocker query should succeed")
                .expect("coordinator result blocker should exist");

        assert_eq!(row_count, 2);
        assert_eq!(ready_count, row_count);
        assert_eq!(owner_count, row_count);
        assert_eq!(decoded_locator_count, row_count);
        assert_eq!(node_count, 1);
        assert_eq!(summary_status, "ready");
        assert_eq!(decoded_summary_count, row_count);
        assert_eq!(local_resolution_status, "ready");
        assert_eq!(remote_resolution_status, "none");
        assert_eq!(candidate_count, row_count);
        assert_eq!(candidate_owner_count, candidate_count);
        assert_eq!(candidate_locator_count, candidate_count);
        assert_eq!(returned_candidate_count, candidate_count);
        assert_eq!(candidate_summary_status, "ready");
        assert_eq!(result_source, "local_heap_candidates");
        assert_eq!(result_receive_count, 0);
        assert_eq!(result_receive_status, "ready");
        assert_eq!(result_status, "ready");
        assert_eq!(result_returned_candidate_count, candidate_count);
        assert_eq!(result_next_blocker, "none");

        Spi::run(
            "INSERT INTO ec_spire_remote_local_heap_res_sql (id, embedding) VALUES \
             (3, encode_to_ecvector(ARRAY[0.9, 0.1], 4, 42))",
        )
        .expect("post-build insert should publish a delta epoch");
        let delta_active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_remote_local_heap_res_sql_idx'::regclass)",
        )
        .expect("post-insert hierarchy snapshot query should succeed")
        .expect("post-insert active epoch should exist");
        let delta_selected_pids = Spi::get_one::<Vec<i64>>(
            "SELECT array_agg(leaf_pid ORDER BY leaf_pid) FROM \
             ec_spire_index_leaf_snapshot('ec_spire_remote_local_heap_res_sql_idx'::regclass)",
        )
        .expect("post-insert leaf snapshot query should succeed")
        .expect("post-insert leaf pids should exist");
        let delta_candidates_from = format!(
            "FROM ec_spire_remote_search_local_heap_candidates(\
             'ec_spire_remote_local_heap_res_sql_idx'::regclass, \
             {delta_active_epoch}, ARRAY[0.9, 0.1]::real[], \
             ARRAY[{}, {}]::bigint[], 3, 'strict')",
            delta_selected_pids[0], delta_selected_pids[1],
        );
        let post_insert_delta_object_count = ec_spire_active_snapshot_i64(
            "ec_spire_remote_local_heap_res_sql_idx",
            "delta_object_count",
        );
        let delta_candidate_count =
            Spi::get_one::<i64>(&format!("SELECT count(*) {delta_candidates_from}"))
                .expect("post-insert local heap candidate count query should succeed")
                .expect("post-insert local heap candidate count should exist");
        let delta_candidate_locator_count = Spi::get_one::<i64>(&format!(
            "SELECT count(*) {delta_candidates_from} \
             WHERE served_epoch = requested_epoch \
             AND heap_block >= 0 AND heap_offset > 0 \
             AND length(row_locator) = 6 AND score IS NOT NULL"
        ))
        .expect("post-insert local heap candidate locator query should succeed")
        .expect("post-insert local heap candidate locator count should exist");

        assert_eq!(delta_active_epoch, active_epoch + 1);
        assert_eq!(post_insert_delta_object_count, 1);
        assert_eq!(delta_candidate_count, 3);
        assert_eq!(delta_candidate_locator_count, delta_candidate_count);
    }
