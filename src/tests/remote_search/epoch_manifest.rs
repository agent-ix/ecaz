    #[pg_test]
    fn test_ec_spire_remote_epoch_publish_plan_missing() {
        Spi::run(
            "CREATE TABLE ec_spire_remote_epoch_plan_missing_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_remote_epoch_plan_missing_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_remote_epoch_plan_missing_sql_idx \
             ON ec_spire_remote_epoch_plan_missing_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("ec_spire index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_spire_remote_epoch_plan_missing_sql_idx'::regclass::oid",
        )
        .expect("index oid query should succeed")
        .expect("index oid should exist");
        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_remote_epoch_plan_missing_sql_idx'::regclass)",
        )
        .expect("hierarchy snapshot query should succeed")
        .expect("active epoch should exist");
        let selected_pid = Spi::get_one::<i64>(
            "SELECT min(leaf_pid) FROM \
             ec_spire_index_leaf_snapshot('ec_spire_remote_epoch_plan_missing_sql_idx'::regclass)",
        )
        .expect("leaf snapshot query should succeed")
        .expect("leaf pid should exist");

        unsafe { am::debug_spire_rewrite_placement_node(index_oid, selected_pid as u64, 2) };
        let plan_from = "FROM ec_spire_remote_epoch_publish_plan(\
             'ec_spire_remote_epoch_plan_missing_sql_idx'::regclass)";
        let row_count = Spi::get_one::<i64>(&format!("SELECT count(*) {plan_from}"))
            .expect("epoch publish plan count query should succeed")
            .expect("epoch publish plan count should exist");
        let descriptor_state =
            Spi::get_one::<String>(&format!("SELECT descriptor_state {plan_from}"))
                .expect("epoch publish plan descriptor query should succeed")
                .expect("epoch publish plan descriptor should exist");
        let status = Spi::get_one::<String>(&format!("SELECT status {plan_from}"))
            .expect("epoch publish plan status query should succeed")
            .expect("epoch publish plan status should exist");
        let epoch_window_status =
            Spi::get_one::<String>(&format!("SELECT epoch_window_status {plan_from}"))
                .expect("epoch publish plan epoch window query should succeed")
                .expect("epoch publish plan epoch window should exist");
        let required_last_served_epoch =
            Spi::get_one::<i64>(&format!("SELECT required_last_served_epoch {plan_from}"))
                .expect("epoch publish plan required served query should succeed")
                .expect("epoch publish plan required served should exist");
        let last_served_epoch =
            Spi::get_one::<i64>(&format!("SELECT last_served_epoch {plan_from}"))
                .expect("epoch publish plan served query should succeed")
                .expect("epoch publish plan served should exist");
        let placement_count = Spi::get_one::<i64>(&format!("SELECT placement_count {plan_from}"))
            .expect("epoch publish plan placement query should succeed")
            .expect("epoch publish plan placement count should exist");

        assert_eq!(row_count, 1);
        assert_eq!(descriptor_state, "missing");
        assert_eq!(status, "requires_remote_node_descriptor");
        assert_eq!(epoch_window_status, "missing_descriptor");
        assert_eq!(required_last_served_epoch, active_epoch);
        assert_eq!(last_served_epoch, 0);
        assert_eq!(placement_count, 1);
    }

    #[pg_test]
    fn test_ec_spire_remote_epoch_publish_manifest_stale_descriptor() {
        Spi::run(
            "CREATE TABLE ec_spire_remote_epoch_manifest_stale_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_remote_epoch_manifest_stale_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_remote_epoch_manifest_stale_sql_idx \
             ON ec_spire_remote_epoch_manifest_stale_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("ec_spire index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_spire_remote_epoch_manifest_stale_sql_idx'::regclass::oid",
        )
        .expect("index oid query should succeed")
        .expect("index oid should exist");
        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_remote_epoch_manifest_stale_sql_idx'::regclass)",
        )
        .expect("hierarchy snapshot query should succeed")
        .expect("active epoch should exist");
        let selected_pid = Spi::get_one::<i64>(
            "SELECT min(leaf_pid) FROM \
             ec_spire_index_leaf_snapshot('ec_spire_remote_epoch_manifest_stale_sql_idx'::regclass)",
        )
        .expect("leaf snapshot query should succeed")
        .expect("leaf pid should exist");
        let stale_served_epoch = active_epoch.saturating_sub(1);
        assert!(stale_served_epoch < active_epoch);

        unsafe { am::debug_spire_rewrite_placement_node(index_oid, selected_pid as u64, 2) };
        let register_result = Spi::get_one::<bool>(&format!(
            "SELECT ec_spire_register_remote_node_descriptor(\
                     '{}'::oid, 2, 8, 'spire/remote/stale', decode('02', 'hex'), \
                     'remote_spire_idx', 'active', {stale_served_epoch}, {active_epoch}, '{}', 'none')",
            u32::from(index_oid),
            env!("CARGO_PKG_VERSION")
        ))
        .expect("stale remote descriptor registration should succeed")
        .expect("stale remote descriptor registration result should exist");

        let plan_from = "FROM ec_spire_remote_epoch_publish_plan(\
             'ec_spire_remote_epoch_manifest_stale_sql_idx'::regclass)";
        let readiness_from = "FROM ec_spire_remote_epoch_publish_readiness(\
             'ec_spire_remote_epoch_manifest_stale_sql_idx'::regclass)";
        let gate_from = "FROM ec_spire_remote_epoch_publish_gate_summary(\
             'ec_spire_remote_epoch_manifest_stale_sql_idx'::regclass)";
        let manifest_from = "FROM ec_spire_remote_epoch_manifest_summary(\
             'ec_spire_remote_epoch_manifest_stale_sql_idx'::regclass)";

        let plan_status = Spi::get_one::<String>(&format!("SELECT status {plan_from}"))
            .expect("stale publish plan status query should succeed")
            .expect("stale publish plan status should exist");
        let epoch_window_status =
            Spi::get_one::<String>(&format!("SELECT epoch_window_status {plan_from}"))
                .expect("stale publish plan epoch window query should succeed")
                .expect("stale publish plan epoch window should exist");
        let readiness_status = Spi::get_one::<String>(&format!("SELECT status {readiness_from}"))
            .expect("stale publish readiness status query should succeed")
            .expect("stale publish readiness status should exist");
        let blocked_remote_node_count = Spi::get_one::<i64>(&format!(
            "SELECT blocked_remote_node_count {readiness_from}"
        ))
        .expect("stale publish readiness blocked count query should succeed")
        .expect("stale publish readiness blocked count should exist");
        let missing_descriptor_node_count = Spi::get_one::<i64>(&format!(
            "SELECT missing_descriptor_node_count {readiness_from}"
        ))
        .expect("stale publish readiness missing count query should succeed")
        .expect("stale publish readiness missing count should exist");
        let next_blocker = Spi::get_one::<String>(&format!("SELECT next_blocker {gate_from}"))
            .expect("stale publish gate blocker query should succeed")
            .expect("stale publish gate blocker should exist");
        let manifest_decision =
            Spi::get_one::<String>(&format!("SELECT manifest_decision {manifest_from}"))
                .expect("stale manifest decision query should succeed")
                .expect("stale manifest decision should exist");

        assert!(register_result);
        assert_eq!(plan_status, "stale_epoch");
        assert_eq!(epoch_window_status, "stale_epoch");
        assert_eq!(readiness_status, "remote_epoch_window");
        assert_eq!(blocked_remote_node_count, 1);
        assert_eq!(missing_descriptor_node_count, 0);
        assert_eq!(next_blocker, "remote_epoch_window");
        assert_eq!(manifest_decision, "block_manifest");
    }

    #[pg_test]
    fn test_ec_spire_remote_epoch_manifest_persist_ready() {
        Spi::run(
            "CREATE TABLE ec_spire_remote_manifest_persist_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_remote_manifest_persist_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_remote_manifest_persist_sql_idx \
             ON ec_spire_remote_manifest_persist_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("ec_spire index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_spire_remote_manifest_persist_sql_idx'::regclass::oid",
        )
        .expect("index oid query should succeed")
        .expect("index oid should exist");
        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_remote_manifest_persist_sql_idx'::regclass)",
        )
        .expect("hierarchy snapshot query should succeed")
        .expect("active epoch should exist");
        let selected_pid = Spi::get_one::<i64>(
            "SELECT min(leaf_pid) FROM \
             ec_spire_index_leaf_snapshot('ec_spire_remote_manifest_persist_sql_idx'::regclass)",
        )
        .expect("leaf snapshot query should succeed")
        .expect("leaf pid should exist");

        unsafe { am::debug_spire_rewrite_placement_node(index_oid, selected_pid as u64, 2) };
        let register_result = Spi::get_one::<bool>(&format!(
            "SELECT ec_spire_register_remote_node_descriptor(\
                     '{}'::oid, 2, 11, 'spire/remote/persist', decode('04', 'hex'), \
                     'remote_spire_idx', 'active', {active_epoch}, {active_epoch}, '{}', 'none')",
            u32::from(index_oid),
            env!("CARGO_PKG_VERSION")
        ))
        .expect("remote descriptor registration should succeed")
        .expect("remote descriptor registration result should exist");
        let persist_result = Spi::get_one::<bool>(
            "SELECT ec_spire_persist_remote_epoch_manifest(\
             'ec_spire_remote_manifest_persist_sql_idx'::regclass)",
        )
        .expect("remote manifest persist should succeed")
        .expect("remote manifest persist result should exist");

        let catalog_from = "FROM ec_spire_remote_epoch_manifest_catalog(\
             'ec_spire_remote_manifest_persist_sql_idx'::regclass)";
        let entry_from = "FROM ec_spire_remote_epoch_manifest_entry_catalog(\
             'ec_spire_remote_manifest_persist_sql_idx'::regclass)";
        let summary_from = "FROM ec_spire_remote_epoch_manifest_catalog_summary(\
             'ec_spire_remote_manifest_persist_sql_idx'::regclass)";
        let publication_from = "FROM ec_spire_remote_epoch_manifest_publication_plan(\
             'ec_spire_remote_manifest_persist_sql_idx'::regclass)";
        let freshness_from = "FROM ec_spire_remote_epoch_manifest_freshness(\
             'ec_spire_remote_manifest_persist_sql_idx'::regclass)";
        let publication_summary_from = "FROM ec_spire_remote_epoch_manifest_publication_summary(\
             'ec_spire_remote_manifest_persist_sql_idx'::regclass)";
        let manifest_libpq_request_from = "FROM ec_spire_remote_epoch_manifest_libpq_request_plan(\
             'ec_spire_remote_manifest_persist_sql_idx'::regclass)";
        let manifest_libpq_request_summary_from =
            "FROM ec_spire_remote_epoch_manifest_libpq_request_summary(\
             'ec_spire_remote_manifest_persist_sql_idx'::regclass)";
        let manifest_payload_from = "FROM ec_spire_remote_epoch_manifest_payload_plan(\
             'ec_spire_remote_manifest_persist_sql_idx'::regclass)";
        let manifest_payload_summary_from = "FROM ec_spire_remote_epoch_manifest_payload_summary(\
             'ec_spire_remote_manifest_persist_sql_idx'::regclass)";
        let manifest_dispatch_from = "FROM ec_spire_remote_epoch_manifest_libpq_dispatch_plan(\
             'ec_spire_remote_manifest_persist_sql_idx'::regclass)";
        let manifest_bind_from = "FROM ec_spire_remote_epoch_manifest_libpq_bind_plan(\
             'ec_spire_remote_manifest_persist_sql_idx'::regclass)";
        let manifest_bind_summary_from = "FROM ec_spire_remote_epoch_manifest_libpq_bind_summary(\
             'ec_spire_remote_manifest_persist_sql_idx'::regclass)";
        let manifest_work_from = "FROM ec_spire_remote_epoch_manifest_libpq_executor_work_plan(\
             'ec_spire_remote_manifest_persist_sql_idx'::regclass)";
        let manifest_work_summary_from =
            "FROM ec_spire_remote_epoch_manifest_libpq_executor_work_summary(\
             'ec_spire_remote_manifest_persist_sql_idx'::regclass)";
        let manifest_dispatch_summary_from =
            "FROM ec_spire_remote_epoch_manifest_libpq_dispatch_summary(\
             'ec_spire_remote_manifest_persist_sql_idx'::regclass)";
        let manifest_executor_readiness_from =
            "FROM ec_spire_remote_epoch_manifest_libpq_executor_readiness(\
             'ec_spire_remote_manifest_persist_sql_idx'::regclass)";
        let manifest_receive_from = "FROM ec_spire_remote_epoch_manifest_libpq_receive_plan(\
             'ec_spire_remote_manifest_persist_sql_idx'::regclass)";
        let manifest_receive_summary_from =
            "FROM ec_spire_remote_epoch_manifest_libpq_receive_summary(\
             'ec_spire_remote_manifest_persist_sql_idx'::regclass)";
        let manifest_publication_gate_from =
            "FROM ec_spire_remote_epoch_manifest_publication_gate_summary(\
             'ec_spire_remote_manifest_persist_sql_idx'::regclass)";
        let manifest_publication_result_from =
            "FROM ec_spire_remote_epoch_manifest_publication_result_summary(\
             'ec_spire_remote_manifest_persist_sql_idx'::regclass)";
        let catalog_count = Spi::get_one::<i64>(&format!("SELECT count(*) {catalog_from}"))
            .expect("manifest catalog count query should succeed")
            .expect("manifest catalog count should exist");
        let manifest_decision =
            Spi::get_one::<String>(&format!("SELECT manifest_decision {catalog_from}"))
                .expect("manifest catalog decision query should succeed")
                .expect("manifest catalog decision should exist");
        let manifest_entry_count =
            Spi::get_one::<i64>(&format!("SELECT manifest_entry_count {catalog_from}"))
                .expect("manifest catalog entry count query should succeed")
                .expect("manifest catalog entry count should exist");
        let included_remote_node_count =
            Spi::get_one::<i64>(&format!("SELECT included_remote_node_count {catalog_from}"))
                .expect("manifest catalog included node count query should succeed")
                .expect("manifest catalog included node count should exist");
        let persisted_at_micros =
            Spi::get_one::<i64>(&format!("SELECT persisted_at_micros {catalog_from}"))
                .expect("manifest catalog timestamp query should succeed")
                .expect("manifest catalog timestamp should exist");
        let entry_count = Spi::get_one::<i64>(&format!("SELECT count(*) {entry_from}"))
            .expect("manifest entry count query should succeed")
            .expect("manifest entry count should exist");
        let entry_node_id = Spi::get_one::<i64>(&format!("SELECT node_id {entry_from}"))
            .expect("manifest entry node query should succeed")
            .expect("manifest entry node should exist");
        let entry_action = Spi::get_one::<String>(&format!("SELECT manifest_action {entry_from}"))
            .expect("manifest entry action query should succeed")
            .expect("manifest entry action should exist");
        let entry_status = Spi::get_one::<String>(&format!("SELECT status {entry_from}"))
            .expect("manifest entry status query should succeed")
            .expect("manifest entry status should exist");
        let summary_status =
            Spi::get_one::<String>(&format!("SELECT catalog_status {summary_from}"))
                .expect("manifest catalog summary status query should succeed")
                .expect("manifest catalog summary status should exist");
        let summary_persisted_entry_count =
            Spi::get_one::<i64>(&format!("SELECT persisted_entry_count {summary_from}"))
                .expect("manifest catalog summary entry count query should succeed")
                .expect("manifest catalog summary entry count should exist");
        let summary_mismatch_count = Spi::get_one::<i64>(&format!(
            "SELECT persisted_entry_mismatch_count {summary_from}"
        ))
        .expect("manifest catalog summary mismatch count query should succeed")
        .expect("manifest catalog summary mismatch count should exist");
        let publication_action =
            Spi::get_one::<String>(&format!("SELECT publication_action {publication_from}"))
                .expect("manifest publication action query should succeed")
                .expect("manifest publication action should exist");
        let publication_transport =
            Spi::get_one::<String>(&format!("SELECT publication_transport {publication_from}"))
                .expect("manifest publication transport query should succeed")
                .expect("manifest publication transport should exist");
        let publication_status =
            Spi::get_one::<String>(&format!("SELECT status {publication_from}"))
                .expect("manifest publication status query should succeed")
                .expect("manifest publication status should exist");
        let publication_entry_matches = Spi::get_one::<bool>(&format!(
            "SELECT persisted_entry_matches {publication_from}"
        ))
        .expect("manifest publication match query should succeed")
        .expect("manifest publication match should exist");
        let freshness_status =
            Spi::get_one::<String>(&format!("SELECT freshness_status {freshness_from}"))
                .expect("manifest freshness status query should succeed")
                .expect("manifest freshness status should exist");
        let freshness_next_action =
            Spi::get_one::<String>(&format!("SELECT next_action {freshness_from}"))
                .expect("manifest freshness action query should succeed")
                .expect("manifest freshness action should exist");
        let freshness_entry_matches =
            Spi::get_one::<bool>(&format!("SELECT persisted_entry_matches {freshness_from}"))
                .expect("manifest freshness match query should succeed")
                .expect("manifest freshness match should exist");
        let publication_summary_decision = Spi::get_one::<String>(&format!(
            "SELECT publication_decision {publication_summary_from}"
        ))
        .expect("manifest publication summary decision query should succeed")
        .expect("manifest publication summary decision should exist");
        let publication_summary_ready_count = Spi::get_one::<i64>(&format!(
            "SELECT ready_publication_count {publication_summary_from}"
        ))
        .expect("manifest publication summary ready count query should succeed")
        .expect("manifest publication summary ready count should exist");
        let publication_summary_status =
            Spi::get_one::<String>(&format!("SELECT status {publication_summary_from}"))
                .expect("manifest publication summary status query should succeed")
                .expect("manifest publication summary status should exist");
        let publication_summary_executor_status = Spi::get_one::<String>(&format!(
            "SELECT publication_executor_status {publication_summary_from}"
        ))
        .expect("manifest publication summary executor status query should succeed")
        .expect("manifest publication summary executor status should exist");
        let publication_summary_executor_step = Spi::get_one::<String>(&format!(
            "SELECT publication_executor_next_step {publication_summary_from}"
        ))
        .expect("manifest publication summary executor step query should succeed")
        .expect("manifest publication summary executor step should exist");
        let libpq_request_action = Spi::get_one::<String>(&format!(
            "SELECT request_action {manifest_libpq_request_from}"
        ))
        .expect("manifest libpq request action query should succeed")
        .expect("manifest libpq request action should exist");
        let libpq_request_sql = Spi::get_one::<String>(&format!(
            "SELECT sql_template {manifest_libpq_request_from}"
        ))
        .expect("manifest libpq request SQL query should succeed")
        .expect("manifest libpq request SQL should exist");
        let libpq_request_parameter_count = Spi::get_one::<i64>(&format!(
            "SELECT parameter_count {manifest_libpq_request_from}"
        ))
        .expect("manifest libpq request parameter count query should succeed")
        .expect("manifest libpq request parameter count should exist");
        let libpq_request_executor_status = Spi::get_one::<String>(&format!(
            "SELECT executor_status {manifest_libpq_request_from}"
        ))
        .expect("manifest libpq request executor status query should succeed")
        .expect("manifest libpq request executor status should exist");
        let libpq_request_summary_count = Spi::get_one::<i64>(&format!(
            "SELECT request_count {manifest_libpq_request_summary_from}"
        ))
        .expect("manifest libpq request summary count query should succeed")
        .expect("manifest libpq request summary count should exist");
        let libpq_request_summary_ready_count = Spi::get_one::<i64>(&format!(
            "SELECT ready_request_count {manifest_libpq_request_summary_from}"
        ))
        .expect("manifest libpq request summary ready count query should succeed")
        .expect("manifest libpq request summary ready count should exist");
        let libpq_request_summary_result_columns = Spi::get_one::<i64>(&format!(
            "SELECT expected_result_column_count {manifest_libpq_request_summary_from}"
        ))
        .expect("manifest libpq request summary result count query should succeed")
        .expect("manifest libpq request summary result count should exist");
        let libpq_request_summary_status = Spi::get_one::<String>(&format!(
            "SELECT status {manifest_libpq_request_summary_from}"
        ))
        .expect("manifest libpq request summary status query should succeed")
        .expect("manifest libpq request summary status should exist");
        let manifest_payload_count =
            Spi::get_one::<i64>(&format!("SELECT count(*) {manifest_payload_from}"))
                .expect("manifest payload count query should succeed")
                .expect("manifest payload count should exist");
        let manifest_payload_format = Spi::get_one::<String>(&format!(
            "SELECT manifest_payload_format {manifest_payload_from}"
        ))
        .expect("manifest payload format query should succeed")
        .expect("manifest payload format should exist");
        let manifest_payload_status =
            Spi::get_one::<String>(&format!("SELECT status {manifest_payload_from}"))
                .expect("manifest payload status query should succeed")
                .expect("manifest payload status should exist");
        let manifest_payload_summary_ready_count = Spi::get_one::<i64>(&format!(
            "SELECT ready_payload_count {manifest_payload_summary_from}"
        ))
        .expect("manifest payload summary ready count query should succeed")
        .expect("manifest payload summary ready count should exist");
        let payload_validation_status = Spi::get_one::<String>(&format!(
            "SELECT status FROM ec_spire_validate_remote_epoch_manifest_payload(\
                 'ec_spire_remote_manifest_persist_sql_idx'::regclass, \
                 {active_epoch}, \
                 (SELECT manifest_payload {manifest_payload_from} WHERE node_id = 2))"
        ))
        .expect("remote manifest payload validation status query should succeed")
        .expect("remote manifest payload validation status should exist");
        let payload_validation_entry_count = Spi::get_one::<i64>(&format!(
            "SELECT validated_entry_count FROM ec_spire_validate_remote_epoch_manifest_payload(\
                 'ec_spire_remote_manifest_persist_sql_idx'::regclass, \
                 {active_epoch}, \
                 (SELECT manifest_payload {manifest_payload_from} WHERE node_id = 2))"
        ))
        .expect("remote manifest payload validation entry count query should succeed")
        .expect("remote manifest payload validation entry count should exist");
        let validation_epoch_mismatch_status = Spi::get_one::<String>(&format!(
            "SELECT status FROM ec_spire_validate_remote_epoch_manifest_payload(\
                 'ec_spire_remote_manifest_persist_sql_idx'::regclass, \
                 {active_epoch} + 1, \
                 (SELECT manifest_payload {manifest_payload_from} WHERE node_id = 2))"
        ))
        .expect("remote manifest payload validation mismatch query should succeed")
        .expect("remote manifest payload validation mismatch status should exist");
        let dispatch_action =
            Spi::get_one::<String>(&format!("SELECT dispatch_action {manifest_dispatch_from}"))
                .expect("manifest dispatch action query should succeed")
                .expect("manifest dispatch action should exist");
        let dispatch_validator = Spi::get_one::<String>(&format!(
            "SELECT receive_validator {manifest_dispatch_from}"
        ))
        .expect("manifest dispatch validator query should succeed")
        .expect("manifest dispatch validator should exist");
        let manifest_bind_count =
            Spi::get_one::<i64>(&format!("SELECT count(*) {manifest_bind_from}"))
                .expect("manifest bind count query should succeed")
                .expect("manifest bind count should exist");
        let manifest_bind_contract_mismatch_count = Spi::get_one::<i64>(&format!(
            "SELECT count(*) \
               FROM ec_spire_remote_epoch_manifest_libpq_parameter_contract() contract \
               LEFT JOIN (SELECT * {manifest_bind_from}) bind \
                 ON bind.parameter_ordinal = contract.parameter_ordinal \
                AND bind.parameter_name = contract.parameter_name \
                AND bind.pg_type = contract.pg_type \
              WHERE bind.parameter_ordinal IS NULL"
        ))
        .expect("manifest bind contract invariant query should succeed")
        .expect("manifest bind contract invariant count should exist");
        let manifest_bind_remote_index_preview = Spi::get_one::<String>(&format!(
            "SELECT value_preview {manifest_bind_from} WHERE parameter_name = 'remote_index_oid'"
        ))
        .expect("manifest bind remote index query should succeed")
        .expect("manifest bind remote index preview should exist");
        let manifest_bind_payload_element_count = Spi::get_one::<i64>(&format!(
            "SELECT element_count {manifest_bind_from} WHERE parameter_name = 'manifest_payload'"
        ))
        .expect("manifest bind payload element count query should succeed")
        .expect("manifest bind payload element count should exist");
        let manifest_bind_ready_count = Spi::get_one::<i64>(&format!(
            "SELECT count(*) {manifest_bind_from} WHERE value_status = 'ready'"
        ))
        .expect("manifest bind ready count query should succeed")
        .expect("manifest bind ready count should exist");
        let manifest_bind_summary_ready_count = Spi::get_one::<i64>(&format!(
            "SELECT ready_bind_count {manifest_bind_summary_from}"
        ))
        .expect("manifest bind summary ready count query should succeed")
        .expect("manifest bind summary ready count should exist");
        let manifest_bind_summary_entry_count = Spi::get_one::<i64>(&format!(
            "SELECT manifest_entry_count {manifest_bind_summary_from}"
        ))
        .expect("manifest bind summary entry count query should succeed")
        .expect("manifest bind summary entry count should exist");
        let manifest_bind_summary_status =
            Spi::get_one::<String>(&format!("SELECT status {manifest_bind_summary_from}"))
                .expect("manifest bind summary status query should succeed")
                .expect("manifest bind summary status should exist");
        let manifest_work_bind_status =
            Spi::get_one::<String>(&format!("SELECT bind_status {manifest_work_from}"))
                .expect("manifest work bind status query should succeed")
                .expect("manifest work bind status should exist");
        let manifest_work_action =
            Spi::get_one::<String>(&format!("SELECT work_action {manifest_work_from}"))
                .expect("manifest work action query should succeed")
                .expect("manifest work action should exist");
        let manifest_work_status =
            Spi::get_one::<String>(&format!("SELECT status {manifest_work_from}"))
                .expect("manifest work status query should succeed")
                .expect("manifest work status should exist");
        let manifest_work_summary_ready_count = Spi::get_one::<i64>(&format!(
            "SELECT ready_work_count {manifest_work_summary_from}"
        ))
        .expect("manifest work summary ready count query should succeed")
        .expect("manifest work summary ready count should exist");
        let manifest_work_summary_status =
            Spi::get_one::<String>(&format!("SELECT status {manifest_work_summary_from}"))
                .expect("manifest work summary status query should succeed")
                .expect("manifest work summary status should exist");
        let dispatch_executor_status =
            Spi::get_one::<String>(&format!("SELECT executor_status {manifest_dispatch_from}"))
                .expect("manifest dispatch executor status query should succeed")
                .expect("manifest dispatch executor status should exist");
        let dispatch_pipeline_count = Spi::get_one::<i64>(&format!(
            "SELECT pipeline_dispatch_count {manifest_dispatch_summary_from}"
        ))
        .expect("manifest dispatch summary pipeline count query should succeed")
        .expect("manifest dispatch summary pipeline count should exist");
        let dispatch_summary_status =
            Spi::get_one::<String>(&format!("SELECT status {manifest_dispatch_summary_from}"))
                .expect("manifest dispatch summary status query should succeed")
                .expect("manifest dispatch summary status should exist");
        let executor_readiness_status =
            Spi::get_one::<String>(&format!("SELECT status {manifest_executor_readiness_from}"))
                .expect("manifest executor readiness status query should succeed")
                .expect("manifest executor readiness status should exist");
        let executor_next_step = Spi::get_one::<String>(&format!(
            "SELECT next_executor_step {manifest_executor_readiness_from}"
        ))
        .expect("manifest executor readiness next step query should succeed")
        .expect("manifest executor readiness next step should exist");
        let executor_send_action = Spi::get_one::<String>(&format!(
            "SELECT send_action {manifest_executor_readiness_from}"
        ))
        .expect("manifest executor readiness send action query should succeed")
        .expect("manifest executor readiness send action should exist");
        let manifest_receive_count =
            Spi::get_one::<i64>(&format!("SELECT count(*) {manifest_receive_from}"))
                .expect("manifest receive count query should succeed")
                .expect("manifest receive count should exist");
        let manifest_receive_validator = Spi::get_one::<String>(&format!(
            "SELECT validator_function {manifest_receive_from}"
        ))
        .expect("manifest receive validator query should succeed")
        .expect("manifest receive validator should exist");
        let manifest_receive_action =
            Spi::get_one::<String>(&format!("SELECT result_action {manifest_receive_from}"))
                .expect("manifest receive action query should succeed")
                .expect("manifest receive action should exist");
        let manifest_receive_status =
            Spi::get_one::<String>(&format!("SELECT status {manifest_receive_from}"))
                .expect("manifest receive status query should succeed")
                .expect("manifest receive status should exist");
        let manifest_receive_summary_ready_count = Spi::get_one::<i64>(&format!(
            "SELECT ready_receive_count {manifest_receive_summary_from}"
        ))
        .expect("manifest receive summary ready count query should succeed")
        .expect("manifest receive summary ready count should exist");
        let manifest_receive_summary_result_columns = Spi::get_one::<i64>(&format!(
            "SELECT expected_result_column_count {manifest_receive_summary_from}"
        ))
        .expect("manifest receive summary result count query should succeed")
        .expect("manifest receive summary result count should exist");
        let manifest_receive_summary_status =
            Spi::get_one::<String>(&format!("SELECT status {manifest_receive_summary_from}"))
                .expect("manifest receive summary status query should succeed")
                .expect("manifest receive summary status should exist");
        let manifest_gate_request_count = Spi::get_one::<i64>(&format!(
            "SELECT libpq_request_count {manifest_publication_gate_from}"
        ))
        .expect("manifest publication gate request count query should succeed")
        .expect("manifest publication gate request count should exist");
        let manifest_gate_dispatch_count = Spi::get_one::<i64>(&format!(
            "SELECT libpq_dispatch_count {manifest_publication_gate_from}"
        ))
        .expect("manifest publication gate dispatch count query should succeed")
        .expect("manifest publication gate dispatch count should exist");
        let manifest_gate_receive_count = Spi::get_one::<i64>(&format!(
            "SELECT libpq_receive_count {manifest_publication_gate_from}"
        ))
        .expect("manifest publication gate receive count query should succeed")
        .expect("manifest publication gate receive count should exist");
        let manifest_gate_executor_status = Spi::get_one::<String>(&format!(
            "SELECT libpq_executor_status {manifest_publication_gate_from}"
        ))
        .expect("manifest publication gate executor status query should succeed")
        .expect("manifest publication gate executor status should exist");
        let manifest_gate_next_blocker = Spi::get_one::<String>(&format!(
            "SELECT next_blocker {manifest_publication_gate_from}"
        ))
        .expect("manifest publication gate blocker query should succeed")
        .expect("manifest publication gate blocker should exist");
        let manifest_gate_status =
            Spi::get_one::<String>(&format!("SELECT status {manifest_publication_gate_from}"))
                .expect("manifest publication gate status query should succeed")
                .expect("manifest publication gate status should exist");
        let manifest_result_source = Spi::get_one::<String>(&format!(
            "SELECT result_source {manifest_publication_result_from}"
        ))
        .expect("manifest publication result source query should succeed")
        .expect("manifest publication result source should exist");
        let manifest_result_receive_count = Spi::get_one::<i64>(&format!(
            "SELECT libpq_receive_count {manifest_publication_result_from}"
        ))
        .expect("manifest publication result receive count query should succeed")
        .expect("manifest publication result receive count should exist");
        let manifest_result_ready_receive_count = Spi::get_one::<i64>(&format!(
            "SELECT ready_receive_count {manifest_publication_result_from}"
        ))
        .expect("manifest publication result ready receive count query should succeed")
        .expect("manifest publication result ready receive count should exist");
        let manifest_result_validation_status = Spi::get_one::<String>(&format!(
            "SELECT validation_result_status {manifest_publication_result_from}"
        ))
        .expect("manifest publication result validation status query should succeed")
        .expect("manifest publication result validation status should exist");
        let manifest_result_next_blocker = Spi::get_one::<String>(&format!(
            "SELECT next_blocker {manifest_publication_result_from}"
        ))
        .expect("manifest publication result blocker query should succeed")
        .expect("manifest publication result blocker should exist");
        let manifest_result_status =
            Spi::get_one::<String>(&format!("SELECT status {manifest_publication_result_from}"))
                .expect("manifest publication result status query should succeed")
                .expect("manifest publication result status should exist");
        let executor_contract_mismatch_count = Spi::get_one::<i64>(&format!(
            "WITH readiness AS ( \
                 SELECT * {manifest_executor_readiness_from} \
             ), expected(step_name, readiness_action) AS ( \
                 VALUES \
                     ('conninfo_secret_resolution', \
                         (SELECT secret_resolution_action FROM readiness)), \
                     ('libpq_connection_open', \
                         (SELECT connection_action FROM readiness)), \
                     ('pipeline_mode_start', \
                         (SELECT pipeline_action FROM readiness)), \
                     ('send_manifest_request', \
                         (SELECT send_action FROM readiness)), \
                     ('receive_payload_validation_result', \
                         (SELECT receive_action FROM readiness)) \
             ) \
             SELECT count(*) \
               FROM expected \
               LEFT JOIN ec_spire_remote_epoch_manifest_libpq_executor_step_contract() contract \
                 ON contract.step_name = expected.step_name \
              WHERE contract.step_name IS NULL \
                 OR contract.executor_action <> expected.readiness_action"
        ))
        .expect("manifest executor contract invariant query should succeed")
        .expect("manifest executor contract invariant count should exist");

        assert!(register_result);
        assert!(persist_result);
        assert_eq!(catalog_count, 1);
        assert_eq!(manifest_decision, "emit_distributed_epoch_manifest");
        assert_eq!(manifest_entry_count, 1);
        assert_eq!(included_remote_node_count, 1);
        assert!(persisted_at_micros > 0);
        assert_eq!(entry_count, 1);
        assert_eq!(entry_node_id, 2);
        assert_eq!(entry_action, "include_remote_node");
        assert_eq!(entry_status, "ready");
        assert_eq!(summary_status, "ready");
        assert_eq!(summary_persisted_entry_count, 1);
        assert_eq!(summary_mismatch_count, 0);
        assert_eq!(publication_action, "publish_remote_epoch_manifest");
        assert_eq!(publication_transport, "libpq_pipeline");
        assert_eq!(publication_status, "ready");
        assert!(publication_entry_matches);
        assert_eq!(freshness_status, "ready");
        assert_eq!(freshness_next_action, "none");
        assert!(freshness_entry_matches);
        assert_eq!(
            publication_summary_decision,
            "publish_remote_epoch_manifest"
        );
        assert_eq!(publication_summary_ready_count, 1);
        assert_eq!(publication_summary_status, "ready");
        assert_eq!(
            publication_summary_executor_status,
            "requires_libpq_executor"
        );
        assert_eq!(
            publication_summary_executor_step,
            "conninfo_secret_resolution"
        );
        assert_eq!(libpq_request_action, "send_remote_epoch_manifest");
        assert!(libpq_request_sql.contains("ec_spire_apply_remote_epoch_manifest_payload"));
        assert_eq!(libpq_request_parameter_count, 3);
        assert_eq!(libpq_request_executor_status, "requires_libpq_executor");
        assert_eq!(libpq_request_summary_count, 1);
        assert_eq!(libpq_request_summary_ready_count, 1);
        assert_eq!(libpq_request_summary_result_columns, 3);
        assert_eq!(libpq_request_summary_status, "ready");
        assert_eq!(manifest_payload_count, 1);
        assert_eq!(manifest_payload_format, "ec_spire_remote_epoch_manifest_v1");
        assert_eq!(manifest_payload_status, "ready");
        assert_eq!(manifest_payload_summary_ready_count, 1);
        assert_eq!(payload_validation_status, "ready");
        assert_eq!(payload_validation_entry_count, 1);
        assert_eq!(validation_epoch_mismatch_status, "manifest_epoch_mismatch");
        assert_eq!(
            dispatch_action,
            "open_pipeline_and_send_remote_epoch_manifest"
        );
        assert_eq!(
            dispatch_validator,
            "ec_spire_remote_epoch_manifest_libpq_result_contract"
        );
        assert_eq!(manifest_bind_count, 3);
        assert_eq!(manifest_bind_contract_mismatch_count, 0);
        assert_eq!(manifest_bind_remote_index_preview, "remote_spire_idx");
        assert_eq!(manifest_bind_payload_element_count, 1);
        assert_eq!(manifest_bind_ready_count, 3);
        assert_eq!(manifest_bind_summary_ready_count, 3);
        assert_eq!(manifest_bind_summary_entry_count, 1);
        assert_eq!(manifest_bind_summary_status, "ready");
        assert_eq!(manifest_work_bind_status, "ready");
        assert_eq!(manifest_work_action, "resolve_conninfo_secret");
        assert_eq!(manifest_work_status, "requires_libpq_executor");
        assert_eq!(manifest_work_summary_ready_count, 1);
        assert_eq!(manifest_work_summary_status, "requires_libpq_executor");
        assert_eq!(dispatch_executor_status, "requires_libpq_executor");
        assert_eq!(dispatch_pipeline_count, 1);
        assert_eq!(dispatch_summary_status, "ready");
        assert_eq!(executor_readiness_status, "requires_libpq_executor");
        assert_eq!(executor_next_step, "conninfo_secret_resolution");
        assert_eq!(executor_send_action, "send_remote_epoch_manifest");
        assert_eq!(manifest_receive_count, 1);
        assert_eq!(
            manifest_receive_validator,
            "ec_spire_remote_epoch_manifest_libpq_result_contract"
        );
        assert_eq!(
            manifest_receive_action,
            "validate_remote_manifest_payload_result"
        );
        assert_eq!(manifest_receive_status, "requires_libpq_executor");
        assert_eq!(manifest_receive_summary_ready_count, 1);
        assert_eq!(manifest_receive_summary_result_columns, 3);
        assert_eq!(manifest_receive_summary_status, "requires_libpq_executor");
        assert_eq!(manifest_gate_request_count, 1);
        assert_eq!(manifest_gate_dispatch_count, 1);
        assert_eq!(manifest_gate_receive_count, 1);
        assert_eq!(manifest_gate_executor_status, "requires_libpq_executor");
        assert_eq!(manifest_gate_next_blocker, "conninfo_secret_resolution");
        assert_eq!(manifest_gate_status, "requires_libpq_executor");
        assert_eq!(manifest_result_source, "pending_libpq_executor");
        assert_eq!(manifest_result_receive_count, 1);
        assert_eq!(manifest_result_ready_receive_count, 1);
        assert_eq!(manifest_result_validation_status, "requires_libpq_executor");
        assert_eq!(manifest_result_next_blocker, "conninfo_secret_resolution");
        assert_eq!(manifest_result_status, "requires_libpq_executor");
        assert_eq!(executor_contract_mismatch_count, 0);

        Spi::run(&format!(
            "UPDATE ec_spire_remote_epoch_manifest_entry \
                SET last_served_epoch = last_served_epoch - 1 \
              WHERE coordinator_index_oid = '{}'::oid \
                AND active_epoch = {active_epoch} \
                AND node_id = 2",
            u32::from(index_oid)
        ))
        .expect("manifest entry drift update should succeed");
        let stale_summary_status =
            Spi::get_one::<String>(&format!("SELECT catalog_status {summary_from}"))
                .expect("stale manifest catalog summary status query should succeed")
                .expect("stale manifest catalog summary status should exist");
        let stale_mismatch_count = Spi::get_one::<i64>(&format!(
            "SELECT persisted_entry_mismatch_count {summary_from}"
        ))
        .expect("stale manifest catalog summary mismatch count query should succeed")
        .expect("stale manifest catalog summary mismatch count should exist");
        let stale_publication_action =
            Spi::get_one::<String>(&format!("SELECT publication_action {publication_from}"))
                .expect("stale manifest publication action query should succeed")
                .expect("stale manifest publication action should exist");
        let stale_publication_status =
            Spi::get_one::<String>(&format!("SELECT status {publication_from}"))
                .expect("stale manifest publication status query should succeed")
                .expect("stale manifest publication status should exist");
        let stale_publication_entry_matches = Spi::get_one::<bool>(&format!(
            "SELECT persisted_entry_matches {publication_from}"
        ))
        .expect("stale manifest publication match query should succeed")
        .expect("stale manifest publication match should exist");
        let stale_freshness_status =
            Spi::get_one::<String>(&format!("SELECT freshness_status {freshness_from}"))
                .expect("stale manifest freshness status query should succeed")
                .expect("stale manifest freshness status should exist");
        let stale_freshness_next_action =
            Spi::get_one::<String>(&format!("SELECT next_action {freshness_from}"))
                .expect("stale manifest freshness action query should succeed")
                .expect("stale manifest freshness action should exist");
        let stale_freshness_entry_matches =
            Spi::get_one::<bool>(&format!("SELECT persisted_entry_matches {freshness_from}"))
                .expect("stale manifest freshness match query should succeed")
                .expect("stale manifest freshness match should exist");
        let stale_publication_summary_decision = Spi::get_one::<String>(&format!(
            "SELECT publication_decision {publication_summary_from}"
        ))
        .expect("stale manifest publication summary decision query should succeed")
        .expect("stale manifest publication summary decision should exist");
        let stale_publication_summary_refresh_count = Spi::get_one::<i64>(&format!(
            "SELECT refresh_required_count {publication_summary_from}"
        ))
        .expect("stale manifest publication summary refresh count query should succeed")
        .expect("stale manifest publication summary refresh count should exist");
        let stale_publication_summary_next_blocker =
            Spi::get_one::<String>(&format!("SELECT next_blocker {publication_summary_from}"))
                .expect("stale manifest publication summary blocker query should succeed")
                .expect("stale manifest publication summary blocker should exist");
        assert_eq!(stale_summary_status, "stale_remote_epoch_manifest");
        assert_eq!(stale_mismatch_count, 1);
        assert_eq!(stale_publication_action, "refresh_remote_epoch_manifest");
        assert_eq!(stale_publication_status, "stale_remote_epoch_manifest");
        assert!(!stale_publication_entry_matches);
        assert_eq!(stale_freshness_status, "stale_remote_epoch_manifest");
        assert_eq!(stale_freshness_next_action, "refresh_remote_epoch_manifest");
        assert!(!stale_freshness_entry_matches);
        assert_eq!(
            stale_publication_summary_decision,
            "refresh_remote_epoch_manifest"
        );
        assert_eq!(stale_publication_summary_refresh_count, 1);
        assert_eq!(
            stale_publication_summary_next_blocker,
            "remote_epoch_manifest_refresh"
        );
    }

    #[pg_test]
    fn test_ec_spire_boundary_replica_manifest_freshness_sql() {
        Spi::run(
            "CREATE TABLE ec_spire_boundary_manifest_freshness_sql (\
               id bigint primary key, \
               source_identity uuid not null, \
               embedding ecvector\
             )",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_boundary_manifest_freshness_sql \
             (id, source_identity, embedding) VALUES \
             (1, '00000000-0000-0000-0000-000000000101', encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, '00000000-0000-0000-0000-000000000202', encode_to_ecvector(ARRAY[0.0, 1.0], 4, 42)), \
             (3, '00000000-0000-0000-0000-000000000303', encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42)), \
             (4, '00000000-0000-0000-0000-000000000404', encode_to_ecvector(ARRAY[0.0, -1.0], 4, 42))",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_boundary_manifest_freshness_idx \
             ON ec_spire_boundary_manifest_freshness_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) INCLUDE (source_identity) \
             WITH ( \
                 source_identity = 'include', \
                 nlists = 4, \
                 nprobe = 4, \
                 boundary_replica_count = 1 \
             )",
        )
        .expect("boundary replica index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_spire_boundary_manifest_freshness_idx'::regclass::oid",
        )
        .expect("index oid query should succeed")
        .expect("index oid should exist");
        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_boundary_manifest_freshness_idx'::regclass)",
        )
        .expect("hierarchy snapshot query should succeed")
        .expect("active epoch should exist");
        let remote_leaf_pid = Spi::get_one::<i64>(
            "SELECT pid FROM \
             ec_spire_index_object_snapshot('ec_spire_boundary_manifest_freshness_idx'::regclass) \
             WHERE object_kind = 'leaf' \
             ORDER BY pid \
             LIMIT 1",
        )
        .expect("leaf object query should succeed")
        .expect("leaf object should exist");

        unsafe { am::debug_spire_rewrite_placement_node(index_oid, remote_leaf_pid as u64, 2) };
        let register_result = Spi::get_one::<bool>(&format!(
            "SELECT ec_spire_register_remote_node_descriptor(\
                     '{}'::oid, 2, 22, 'spire/remote/boundary-freshness', decode('22', 'hex'), \
                     'remote_spire_idx', 'active', {active_epoch}, {active_epoch}, '{}', 'none')",
            u32::from(index_oid),
            env!("CARGO_PKG_VERSION")
        ))
        .expect("remote descriptor registration should succeed")
        .expect("remote descriptor registration result should exist");

        let freshness_from = "FROM ec_spire_remote_epoch_manifest_freshness(\
             'ec_spire_boundary_manifest_freshness_idx'::regclass)";
        let identity_from = "FROM ec_spire_index_boundary_replica_identity_snapshot(\
             'ec_spire_boundary_manifest_freshness_idx'::regclass)";

        let pre_persist_status =
            Spi::get_one::<String>(&format!("SELECT freshness_status {freshness_from}"))
                .expect("pre-persist freshness status query should succeed")
                .expect("pre-persist freshness status should exist");
        let pre_persist_action =
            Spi::get_one::<String>(&format!("SELECT next_action {freshness_from}"))
                .expect("pre-persist freshness action query should succeed")
                .expect("pre-persist freshness action should exist");
        let remote_identity_count = Spi::get_one::<i64>(&format!(
            "SELECT count(*) {identity_from} \
             WHERE vec_id_scope = 'global' \
               AND status = 'ready' \
               AND node_count = 2 \
               AND min_node_id = 0 \
               AND max_node_id = 2"
        ))
        .expect("remote boundary identity query should succeed")
        .expect("remote boundary identity count should exist");

        let persist_result = Spi::get_one::<bool>(
            "SELECT ec_spire_persist_remote_epoch_manifest(\
             'ec_spire_boundary_manifest_freshness_idx'::regclass)",
        )
        .expect("remote manifest persist should succeed")
        .expect("remote manifest persist result should exist");
        let ready_status =
            Spi::get_one::<String>(&format!("SELECT freshness_status {freshness_from}"))
                .expect("ready freshness status query should succeed")
                .expect("ready freshness status should exist");
        let ready_action = Spi::get_one::<String>(&format!("SELECT next_action {freshness_from}"))
            .expect("ready freshness action query should succeed")
            .expect("ready freshness action should exist");
        let ready_entry_matches =
            Spi::get_one::<bool>(&format!("SELECT persisted_entry_matches {freshness_from}"))
                .expect("ready freshness match query should succeed")
                .expect("ready freshness match should exist");

        Spi::run(&format!(
            "UPDATE ec_spire_remote_epoch_manifest_entry \
                SET last_served_epoch = last_served_epoch - 1 \
              WHERE coordinator_index_oid = '{}'::oid \
                AND active_epoch = {active_epoch} \
                AND node_id = 2",
            u32::from(index_oid)
        ))
        .expect("manifest entry drift update should succeed");
        let stale_status =
            Spi::get_one::<String>(&format!("SELECT freshness_status {freshness_from}"))
                .expect("stale freshness status query should succeed")
                .expect("stale freshness status should exist");
        let stale_action = Spi::get_one::<String>(&format!("SELECT next_action {freshness_from}"))
            .expect("stale freshness action query should succeed")
            .expect("stale freshness action should exist");
        let stale_entry_matches =
            Spi::get_one::<bool>(&format!("SELECT persisted_entry_matches {freshness_from}"))
                .expect("stale freshness match query should succeed")
                .expect("stale freshness match should exist");

        assert!(register_result);
        assert_eq!(
            pre_persist_status,
            "requires_remote_epoch_manifest_persistence"
        );
        assert_eq!(pre_persist_action, "persist_remote_epoch_manifest");
        assert!(remote_identity_count > 0);
        assert!(persist_result);
        assert_eq!(ready_status, "ready");
        assert_eq!(ready_action, "none");
        assert!(ready_entry_matches);
        assert_eq!(stale_status, "stale_remote_epoch_manifest");
        assert_eq!(stale_action, "refresh_remote_epoch_manifest");
        assert!(!stale_entry_matches);
    }

    #[pg_test]
    fn test_ec_spire_remote_epoch_manifest_libpq_executor_loopback() {
        let _env_lock = env_var_test_lock();
        let loopback_conninfo = current_pg_test_loopback_conninfo();
        let _conninfo_secret = ScopedEnvVar::set(
            "EC_SPIRE_REMOTE_CONNINFO_SPIRE_REMOTE_MANIFEST_LOOPBACK",
            &loopback_conninfo,
        );
        let mut loopback_client = postgres::Client::connect(&loopback_conninfo, postgres::NoTls)
            .expect("loopback client connection should succeed");
        loopback_client
            .batch_execute(
                "DROP TABLE IF EXISTS ec_spire_remote_manifest_executor_remote_sql; \
                 CREATE TABLE ec_spire_remote_manifest_executor_remote_sql \
                     (id bigint primary key, embedding ecvector); \
                 INSERT INTO ec_spire_remote_manifest_executor_remote_sql (id, embedding) VALUES \
                     (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
                     (2, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42)); \
                 CREATE INDEX ec_spire_remote_manifest_executor_remote_sql_idx \
                     ON ec_spire_remote_manifest_executor_remote_sql USING ec_spire \
                     (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
            )
            .expect("loopback remote manifest fixture should be created");

        Spi::run(
            "CREATE TABLE ec_spire_remote_manifest_executor_coord_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_remote_manifest_executor_coord_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_remote_manifest_executor_coord_sql_idx \
             ON ec_spire_remote_manifest_executor_coord_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("ec_spire index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_spire_remote_manifest_executor_coord_sql_idx'::regclass::oid",
        )
        .expect("index oid query should succeed")
        .expect("index oid should exist");
        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_remote_manifest_executor_coord_sql_idx'::regclass)",
        )
        .expect("hierarchy snapshot query should succeed")
        .expect("active epoch should exist");
        let selected_pid = Spi::get_one::<i64>(
            "SELECT min(leaf_pid) FROM \
             ec_spire_index_leaf_snapshot('ec_spire_remote_manifest_executor_coord_sql_idx'::regclass)",
        )
        .expect("leaf snapshot query should succeed")
        .expect("leaf pid should exist");

        unsafe { am::debug_spire_rewrite_placement_node(index_oid, selected_pid as u64, 2) };
        let register_result = Spi::get_one::<bool>(&format!(
            "SELECT ec_spire_register_remote_node_descriptor(\
                     '{}'::oid, 2, 12, 'spire/remote/manifest/loopback', decode('05', 'hex'), \
                     'ec_spire_remote_manifest_executor_remote_sql_idx', 'active', \
                     {active_epoch}, {active_epoch}, '{}', 'none')",
            u32::from(index_oid),
            env!("CARGO_PKG_VERSION")
        ))
        .expect("remote descriptor registration should succeed")
        .expect("remote descriptor registration result should exist");
        let persist_result = Spi::get_one::<bool>(
            "SELECT ec_spire_persist_remote_epoch_manifest(\
             'ec_spire_remote_manifest_executor_coord_sql_idx'::regclass)",
        )
        .expect("remote manifest persist should succeed")
        .expect("remote manifest persist result should exist");

        let executor_from = "FROM ec_spire_remote_epoch_manifest_libpq_executor_results(\
             'ec_spire_remote_manifest_executor_coord_sql_idx'::regclass)";
        let connection_attempted =
            Spi::get_one::<bool>(&format!("SELECT connection_attempted {executor_from}"))
                .expect("manifest executor connection attempted query should succeed")
                .expect("manifest executor connection attempted should exist");
        let connection_status =
            Spi::get_one::<String>(&format!("SELECT connection_status {executor_from}"))
                .expect("manifest executor connection status query should succeed")
                .expect("manifest executor connection status should exist");
        let validated_entry_count =
            Spi::get_one::<i64>(&format!("SELECT validated_entry_count {executor_from}"))
                .expect("manifest executor validated entry query should succeed")
                .expect("manifest executor validated entry should exist");
        let validation_status =
            Spi::get_one::<String>(&format!("SELECT validation_result_status {executor_from}"))
                .expect("manifest executor validation status query should succeed")
                .expect("manifest executor validation status should exist");
        let conninfo_lookup_kind =
            Spi::get_one::<String>(&format!("SELECT conninfo_lookup_kind {executor_from}"))
                .expect("manifest executor lookup kind query should succeed")
                .expect("manifest executor lookup kind should exist");
        let next_step =
            Spi::get_one::<String>(&format!("SELECT next_executor_step {executor_from}"))
                .expect("manifest executor next step query should succeed")
                .expect("manifest executor next step should exist");
        let status = Spi::get_one::<String>(&format!("SELECT status {executor_from}"))
            .expect("manifest executor status query should succeed")
            .expect("manifest executor status should exist");
        let remote_index_oid = loopback_client
            .query_one(
                "SELECT 'ec_spire_remote_manifest_executor_remote_sql_idx'::regclass::oid",
                &[],
            )
            .expect("remote index oid query should succeed")
            .try_get::<_, u32>(0)
            .expect("remote index oid should decode");
        let applied_count = loopback_client
            .query_one(
                "SELECT count(*)::bigint \
                   FROM ec_spire_remote_epoch_manifest_applied \
                  WHERE remote_index_oid = $1::oid AND active_epoch = $2::bigint",
                &[&remote_index_oid, &active_epoch],
            )
            .expect("remote applied manifest count query should succeed")
            .try_get::<_, i64>(0)
            .expect("remote applied manifest count should decode");
        let applied_entry_count = loopback_client
            .query_one(
                "SELECT count(*)::bigint \
                   FROM ec_spire_remote_epoch_manifest_applied_entry \
                  WHERE remote_index_oid = $1::oid AND active_epoch = $2::bigint",
                &[&remote_index_oid, &active_epoch],
            )
            .expect("remote applied manifest entry count query should succeed")
            .try_get::<_, i64>(0)
            .expect("remote applied manifest entry count should decode");

        assert!(register_result);
        assert!(persist_result);
        assert!(connection_attempted);
        assert_eq!(connection_status, "libpq_connection_opened");
        assert_eq!(validated_entry_count, 1);
        assert_eq!(validation_status, "ready");
        assert_eq!(conninfo_lookup_kind, "secret_provider");
        assert_eq!(next_step, "none");
        assert_eq!(status, "ready");
        assert_eq!(applied_count, 1);
        assert_eq!(applied_entry_count, 1);
    }

    #[pg_test]
    #[should_panic(
        expected = "ec_spire_persist_remote_epoch_manifest cannot persist remote epoch manifest"
    )]
    fn test_ec_spire_remote_epoch_manifest_persist_blocked() {
        Spi::run(
            "CREATE TABLE ec_spire_remote_manifest_blocked_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_remote_manifest_blocked_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_remote_manifest_blocked_sql_idx \
             ON ec_spire_remote_manifest_blocked_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("ec_spire index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_spire_remote_manifest_blocked_sql_idx'::regclass::oid",
        )
        .expect("index oid query should succeed")
        .expect("index oid should exist");
        let selected_pid = Spi::get_one::<i64>(
            "SELECT min(leaf_pid) FROM \
             ec_spire_index_leaf_snapshot('ec_spire_remote_manifest_blocked_sql_idx'::regclass)",
        )
        .expect("leaf snapshot query should succeed")
        .expect("leaf pid should exist");

        unsafe { am::debug_spire_rewrite_placement_node(index_oid, selected_pid as u64, 2) };
        let _ = Spi::get_one::<bool>(
            "SELECT ec_spire_persist_remote_epoch_manifest(\
             'ec_spire_remote_manifest_blocked_sql_idx'::regclass)",
        );
    }

    #[pg_test]
    fn test_ec_spire_remote_epoch_manifest_catalog_summary_missing() {
        Spi::run(
            "CREATE TABLE ec_spire_remote_manifest_summary_missing_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_remote_manifest_summary_missing_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_remote_manifest_summary_missing_sql_idx \
             ON ec_spire_remote_manifest_summary_missing_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("ec_spire index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_spire_remote_manifest_summary_missing_sql_idx'::regclass::oid",
        )
        .expect("index oid query should succeed")
        .expect("index oid should exist");
        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_remote_manifest_summary_missing_sql_idx'::regclass)",
        )
        .expect("hierarchy snapshot query should succeed")
        .expect("active epoch should exist");
        let selected_pid = Spi::get_one::<i64>(
            "SELECT min(leaf_pid) FROM \
             ec_spire_index_leaf_snapshot('ec_spire_remote_manifest_summary_missing_sql_idx'::regclass)",
        )
        .expect("leaf snapshot query should succeed")
        .expect("leaf pid should exist");

        unsafe { am::debug_spire_rewrite_placement_node(index_oid, selected_pid as u64, 2) };
        let register_result = Spi::get_one::<bool>(&format!(
            "SELECT ec_spire_register_remote_node_descriptor(\
                     '{}'::oid, 2, 12, 'spire/remote/summary-missing', decode('05', 'hex'), \
                     'remote_spire_idx', 'active', {active_epoch}, {active_epoch}, '{}', 'none')",
            u32::from(index_oid),
            env!("CARGO_PKG_VERSION")
        ))
        .expect("remote descriptor registration should succeed")
        .expect("remote descriptor registration result should exist");

        let summary_from = "FROM ec_spire_remote_epoch_manifest_catalog_summary(\
             'ec_spire_remote_manifest_summary_missing_sql_idx'::regclass)";
        let publication_summary_from = "FROM ec_spire_remote_epoch_manifest_publication_summary(\
             'ec_spire_remote_manifest_summary_missing_sql_idx'::regclass)";
        let publication_result_from =
            "FROM ec_spire_remote_epoch_manifest_publication_result_summary(\
             'ec_spire_remote_manifest_summary_missing_sql_idx'::regclass)";
        let manifest_decision =
            Spi::get_one::<String>(&format!("SELECT current_manifest_decision {summary_from}"))
                .expect("manifest summary decision query should succeed")
                .expect("manifest summary decision should exist");
        let catalog_status =
            Spi::get_one::<String>(&format!("SELECT catalog_status {summary_from}"))
                .expect("manifest summary status query should succeed")
                .expect("manifest summary status should exist");
        let persisted_manifest_count =
            Spi::get_one::<i64>(&format!("SELECT persisted_manifest_count {summary_from}"))
                .expect("manifest summary persisted count query should succeed")
                .expect("manifest summary persisted count should exist");
        let persisted_entry_count =
            Spi::get_one::<i64>(&format!("SELECT persisted_entry_count {summary_from}"))
                .expect("manifest summary entry count query should succeed")
                .expect("manifest summary entry count should exist");
        let persisted_entry_mismatch_count = Spi::get_one::<i64>(&format!(
            "SELECT persisted_entry_mismatch_count {summary_from}"
        ))
        .expect("manifest summary mismatch count query should succeed")
        .expect("manifest summary mismatch count should exist");
        let publication_decision = Spi::get_one::<String>(&format!(
            "SELECT publication_decision {publication_summary_from}"
        ))
        .expect("publication summary decision query should succeed")
        .expect("publication summary decision should exist");
        let persistence_required_count = Spi::get_one::<i64>(&format!(
            "SELECT persistence_required_count {publication_summary_from}"
        ))
        .expect("publication summary persistence count query should succeed")
        .expect("publication summary persistence count should exist");
        let publication_next_blocker =
            Spi::get_one::<String>(&format!("SELECT next_blocker {publication_summary_from}"))
                .expect("publication summary blocker query should succeed")
                .expect("publication summary blocker should exist");
        let publication_result_source =
            Spi::get_one::<String>(&format!("SELECT result_source {publication_result_from}"))
                .expect("publication result source query should succeed")
                .expect("publication result source should exist");
        let publication_result_receive_count = Spi::get_one::<i64>(&format!(
            "SELECT libpq_receive_count {publication_result_from}"
        ))
        .expect("publication result receive count query should succeed")
        .expect("publication result receive count should exist");
        let publication_result_status =
            Spi::get_one::<String>(&format!("SELECT status {publication_result_from}"))
                .expect("publication result status query should succeed")
                .expect("publication result status should exist");
        let publication_result_next_blocker =
            Spi::get_one::<String>(&format!("SELECT next_blocker {publication_result_from}"))
                .expect("publication result blocker query should succeed")
                .expect("publication result blocker should exist");

        assert!(register_result);
        assert_eq!(manifest_decision, "emit_distributed_epoch_manifest");
        assert_eq!(catalog_status, "requires_remote_epoch_manifest_persistence");
        assert_eq!(persisted_manifest_count, 0);
        assert_eq!(persisted_entry_count, 0);
        assert_eq!(persisted_entry_mismatch_count, 1);
        assert_eq!(publication_decision, "persist_remote_epoch_manifest");
        assert_eq!(persistence_required_count, 1);
        assert_eq!(
            publication_next_blocker,
            "remote_epoch_manifest_persistence"
        );
        assert_eq!(publication_result_source, "blocked");
        assert_eq!(publication_result_receive_count, 0);
        assert_eq!(
            publication_result_status,
            "requires_remote_epoch_manifest_persistence"
        );
        assert_eq!(
            publication_result_next_blocker,
            "remote_epoch_manifest_persistence"
        );
    }

    #[pg_test]
    fn test_ec_spire_remote_phase7_policy_contracts() {
        let degradation_from = "FROM ec_spire_remote_degradation_policy_contract()";
        let publication_from = "FROM ec_spire_remote_epoch_manifest_publication_contract()";
        let publication_result_from =
            "FROM ec_spire_remote_epoch_manifest_publication_result_contract()";
        let manifest_parameter_from =
            "FROM ec_spire_remote_epoch_manifest_libpq_parameter_contract()";
        let manifest_result_from = "FROM ec_spire_remote_epoch_manifest_libpq_result_contract()";
        let manifest_executor_step_from =
            "FROM ec_spire_remote_epoch_manifest_libpq_executor_step_contract()";
        let operator_entrypoint_from = "FROM ec_spire_remote_operator_entrypoint_contract()";
        let libpq_lifecycle_from = "FROM ec_spire_remote_libpq_connection_lifecycle_contract()";
        let secret_resolution_from = "FROM ec_spire_remote_conninfo_secret_resolution_contract()";
        let catalog_lifecycle_from = "FROM ec_spire_remote_catalog_lifecycle_contract()";
        let search_result_from = "FROM ec_spire_remote_search_coordinator_result_contract()";
        let merge_order_from = "FROM ec_spire_remote_search_merge_order_contract()";
        let identity_contract_from = "FROM ec_spire_remote_search_vector_identity_contract()";
        let degradation_count = Spi::get_one::<i64>(&format!("SELECT count(*) {degradation_from}"))
            .expect("degradation contract count query should succeed")
            .expect("degradation contract count should exist");
        let degraded_unavailable_action = Spi::get_one::<String>(&format!(
            "SELECT search_action {degradation_from} \
             WHERE consistency_mode = 'degraded' AND placement_state = 'unavailable'"
        ))
        .expect("degraded unavailable contract query should succeed")
        .expect("degraded unavailable contract should exist");
        let strict_unavailable_action = Spi::get_one::<String>(&format!(
            "SELECT search_action {degradation_from} \
             WHERE consistency_mode = 'strict' AND placement_state = 'unavailable'"
        ))
        .expect("strict unavailable contract query should succeed")
        .expect("strict unavailable contract should exist");
        let stale_degraded_status = Spi::get_one::<String>(&format!(
            "SELECT status {degradation_from} \
             WHERE consistency_mode = 'degraded' AND placement_state = 'stale'"
        ))
        .expect("degraded stale contract query should succeed")
        .expect("degraded stale contract should exist");
        let merge_order_count = Spi::get_one::<i64>(&format!("SELECT count(*) {merge_order_from}"))
            .expect("merge order contract count query should succeed")
            .expect("merge order contract count should exist");
        let first_order_key = Spi::get_one::<String>(&format!(
            "SELECT order_key {merge_order_from} WHERE order_ordinal = 1"
        ))
        .expect("merge first order query should succeed")
        .expect("merge first order should exist");
        let assignment_direction = Spi::get_one::<String>(&format!(
            "SELECT direction {merge_order_from} WHERE order_key = 'assignment_role'"
        ))
        .expect("merge assignment direction query should succeed")
        .expect("merge assignment direction should exist");
        let dedupe_order = Spi::get_one::<String>(&format!(
            "SELECT string_agg(order_key, ',' ORDER BY order_ordinal) {merge_order_from}"
        ))
        .expect("merge order aggregate query should succeed")
        .expect("merge order aggregate should exist");
        let remote_dedupe_key = Spi::get_one::<String>(&format!(
            "SELECT contract_value {identity_contract_from} \
             WHERE contract_item = 'remote_merge_dedupe_key'"
        ))
        .expect("remote vector identity dedupe key query should succeed")
        .expect("remote vector identity dedupe key should exist");
        let publication_step_count =
            Spi::get_one::<i64>(&format!("SELECT count(*) {publication_from}"))
                .expect("manifest publication contract count query should succeed")
                .expect("manifest publication contract count should exist");
        let persistence_action = Spi::get_one::<String>(&format!(
            "SELECT publication_action {publication_from} \
             WHERE failure_status = 'requires_remote_epoch_manifest_persistence'"
        ))
        .expect("manifest publication persistence query should succeed")
        .expect("manifest publication persistence action should exist");
        let stale_action = Spi::get_one::<String>(&format!(
            "SELECT publication_action {publication_from} \
             WHERE failure_status = 'stale_remote_epoch_manifest'"
        ))
        .expect("manifest publication stale query should succeed")
        .expect("manifest publication stale action should exist");
        let transport_validator = Spi::get_one::<String>(&format!(
            "SELECT validator {publication_from} \
             WHERE prerequisite = 'remote_epoch_manifest_transport'"
        ))
        .expect("manifest publication transport query should succeed")
        .expect("manifest publication transport validator should exist");
        let publication_result_count =
            Spi::get_one::<i64>(&format!("SELECT count(*) {publication_result_from}"))
                .expect("manifest publication result contract count query should succeed")
                .expect("manifest publication result contract count should exist");
        let pending_result_validator = Spi::get_one::<String>(&format!(
            "SELECT validator {publication_result_from} \
             WHERE result_source = 'pending_libpq_executor'"
        ))
        .expect("manifest publication pending result query should succeed")
        .expect("manifest publication pending result validator should exist");
        let validation_result_recommendation = Spi::get_one::<String>(&format!(
            "SELECT recommendation {publication_result_from} \
             WHERE result_source = 'remote_manifest_validation_result'"
        ))
        .expect("manifest publication validation result query should succeed")
        .expect("manifest publication validation result recommendation should exist");
        let manifest_parameter_count =
            Spi::get_one::<i64>(&format!("SELECT count(*) {manifest_parameter_from}"))
                .expect("manifest parameter contract count query should succeed")
                .expect("manifest parameter contract count should exist");
        let manifest_payload_type = Spi::get_one::<String>(&format!(
            "SELECT pg_type {manifest_parameter_from} \
             WHERE parameter_name = 'manifest_payload'"
        ))
        .expect("manifest payload parameter query should succeed")
        .expect("manifest payload parameter should exist");
        let manifest_result_count =
            Spi::get_one::<i64>(&format!("SELECT count(*) {manifest_result_from}"))
                .expect("manifest result contract count query should succeed")
                .expect("manifest result contract count should exist");
        let manifest_status_validator = Spi::get_one::<String>(&format!(
            "SELECT validator {manifest_result_from} WHERE column_name = 'status'"
        ))
        .expect("manifest result status query should succeed")
        .expect("manifest result status should exist");
        let manifest_executor_step_count =
            Spi::get_one::<i64>(&format!("SELECT count(*) {manifest_executor_step_from}"))
                .expect("manifest executor step count query should succeed")
                .expect("manifest executor step count should exist");
        let manifest_send_input = Spi::get_one::<String>(&format!(
            "SELECT input_contract {manifest_executor_step_from} \
             WHERE step_name = 'send_manifest_request'"
        ))
        .expect("manifest executor send input query should succeed")
        .expect("manifest executor send input should exist");
        let search_result_count =
            Spi::get_one::<i64>(&format!("SELECT count(*) {search_result_from}"))
                .expect("search result contract count query should succeed")
                .expect("search result contract count should exist");
        let search_blocked_validator = Spi::get_one::<String>(&format!(
            "SELECT validator {search_result_from} WHERE result_source = 'blocked'"
        ))
        .expect("search blocked result query should succeed")
        .expect("search blocked result should exist");
        let operator_entrypoint_count =
            Spi::get_one::<i64>(&format!("SELECT count(*) {operator_entrypoint_from}"))
                .expect("operator entrypoint count query should succeed")
                .expect("operator entrypoint count should exist");
        let operator_entrypoint_reachable_count = Spi::get_one::<i64>(&format!(
            "SELECT count(*) \
               FROM ec_spire_remote_operator_entrypoint_contract() contract \
              WHERE EXISTS ( \
                    SELECT 1 \
                      FROM pg_proc proc \
                     WHERE proc.proname = contract.entrypoint_name)"
        ))
        .expect("operator entrypoint reachability query should succeed")
        .expect("operator entrypoint reachability count should exist");
        let search_gate_next_action = Spi::get_one::<String>(&format!(
            "SELECT next_action {operator_entrypoint_from} \
             WHERE entrypoint_name = 'ec_spire_remote_search_coordinator_gate_summary'"
        ))
        .expect("operator search gate entrypoint query should succeed")
        .expect("operator search gate entrypoint should exist");
        let publication_result_use = Spi::get_one::<String>(&format!(
            "SELECT operator_use {operator_entrypoint_from} \
             WHERE entrypoint_name = 'ec_spire_remote_epoch_manifest_publication_result_summary'"
        ))
        .expect("operator publication result entrypoint query should succeed")
        .expect("operator publication result entrypoint should exist");
        let search_secret_next_action = Spi::get_one::<String>(&format!(
            "SELECT next_action {operator_entrypoint_from} \
             WHERE entrypoint_name = 'ec_spire_remote_search_libpq_secret_summary'"
        ))
        .expect("operator search secret entrypoint query should succeed")
        .expect("operator search secret entrypoint should exist");
        let single_secret_use = Spi::get_one::<String>(&format!(
            "SELECT operator_use {operator_entrypoint_from} \
             WHERE entrypoint_name = 'ec_spire_remote_conninfo_secret_resolution_status'"
        ))
        .expect("operator single secret entrypoint query should succeed")
        .expect("operator single secret entrypoint should exist");
        let production_state_use = Spi::get_one::<String>(&format!(
            "SELECT operator_use {operator_entrypoint_from} \
             WHERE entrypoint_name = 'ec_spire_remote_search_production_executor_state_summary'"
        ))
        .expect("operator production state entrypoint query should succeed")
        .expect("operator production state entrypoint should exist");
        let pipeline_steps_action = Spi::get_one::<String>(&format!(
            "SELECT next_action {operator_entrypoint_from} \
             WHERE entrypoint_name = 'ec_spire_remote_pipeline_steps'"
        ))
        .expect("operator pipeline steps entrypoint query should succeed")
        .expect("operator pipeline steps entrypoint should exist");
        let pipeline_steps_live_use = Spi::get_one::<String>(&format!(
            "SELECT operator_use {operator_entrypoint_from} \
             WHERE entrypoint_name = 'ec_spire_remote_pipeline_steps_live'"
        ))
        .expect("operator live pipeline steps entrypoint query should succeed")
        .expect("operator live pipeline steps entrypoint should exist");
        let receive_attempts_use = Spi::get_one::<String>(&format!(
            "SELECT operator_use {operator_entrypoint_from} \
             WHERE entrypoint_name = 'ec_spire_remote_search_libpq_executor_receive_attempts'"
        ))
        .expect("operator receive attempts entrypoint query should succeed")
        .expect("operator receive attempts entrypoint should exist");
        let budget_entrypoint_use = Spi::get_one::<String>(&format!(
            "SELECT operator_use {operator_entrypoint_from} \
             WHERE entrypoint_name = 'ec_spire_remote_search_libpq_executor_budget_summary'"
        ))
        .expect("operator budget entrypoint query should succeed")
        .expect("operator budget entrypoint should exist");
        let stage_e_fault_matrix_use = Spi::get_one::<String>(&format!(
            "SELECT operator_use {operator_entrypoint_from} \
             WHERE entrypoint_name = 'ec_spire_remote_search_stage_e_fault_matrix'"
        ))
        .expect("operator Stage E fault matrix entrypoint query should succeed")
        .expect("operator Stage E fault matrix entrypoint should exist");
        let operator_diagnostics_use = Spi::get_one::<String>(&format!(
            "SELECT operator_use {operator_entrypoint_from} \
             WHERE entrypoint_name = 'ec_spire_remote_search_operator_diagnostics'"
        ))
        .expect("operator diagnostics entrypoint query should succeed")
        .expect("operator diagnostics entrypoint should exist");
        let stage_e_lifecycle_matrix_use = Spi::get_one::<String>(&format!(
            "SELECT operator_use {operator_entrypoint_from} \
             WHERE entrypoint_name = 'ec_spire_remote_search_stage_e_lifecycle_matrix'"
        ))
        .expect("operator Stage E lifecycle matrix entrypoint query should succeed")
        .expect("operator Stage E lifecycle matrix entrypoint should exist");
        let manifest_freshness_use = Spi::get_one::<String>(&format!(
            "SELECT operator_use {operator_entrypoint_from} \
             WHERE entrypoint_name = 'ec_spire_remote_epoch_manifest_freshness'"
        ))
        .expect("operator manifest freshness entrypoint query should succeed")
        .expect("operator manifest freshness entrypoint should exist");
        let libpq_lifecycle_count =
            Spi::get_one::<i64>(&format!("SELECT count(*) {libpq_lifecycle_from}"))
                .expect("libpq lifecycle count query should succeed")
                .expect("libpq lifecycle count should exist");
        let search_connection_policy = Spi::get_one::<String>(&format!(
            "SELECT connection_lifecycle_policy {libpq_lifecycle_from} \
             WHERE surface = 'ec_spire_remote_search_libpq_executor'"
        ))
        .expect("search lifecycle policy query should succeed")
        .expect("search lifecycle policy should exist");
        let search_secret_policy = Spi::get_one::<String>(&format!(
            "SELECT secret_resolution_policy {libpq_lifecycle_from} \
             WHERE surface = 'ec_spire_remote_search_libpq_executor'"
        ))
        .expect("search lifecycle secret policy query should succeed")
        .expect("search lifecycle secret policy should exist");
        let manifest_conninfo_policy = Spi::get_one::<String>(&format!(
            "SELECT conninfo_exposure_policy {libpq_lifecycle_from} \
             WHERE surface = 'ec_spire_remote_epoch_manifest_publication_libpq_executor'"
        ))
        .expect("manifest lifecycle conninfo policy query should succeed")
        .expect("manifest lifecycle conninfo policy should exist");
        let secret_provider_count =
            Spi::get_one::<i64>(&format!("SELECT count(*) {secret_resolution_from}"))
                .expect("secret provider count query should succeed")
                .expect("secret provider count should exist");
        let selected_secret_provider = Spi::get_one::<String>(&format!(
            "SELECT provider_policy {secret_resolution_from} \
             WHERE provider_status = 'selected_v1'"
        ))
        .expect("selected secret provider query should succeed")
        .expect("selected secret provider should exist");
        let selected_raw_conninfo_allowed = Spi::get_one::<bool>(&format!(
            "SELECT raw_conninfo_allowed {secret_resolution_from} \
             WHERE provider_status = 'selected_v1'"
        ))
        .expect("selected raw conninfo query should succeed")
        .expect("selected raw conninfo should exist");
        let rejected_provider_storage = Spi::get_one::<String>(&format!(
            "SELECT sql_storage_policy {secret_resolution_from} \
             WHERE provider_policy = 'in_extension_conninfo_table'"
        ))
        .expect("rejected secret provider storage query should succeed")
        .expect("rejected secret provider storage should exist");
        let catalog_lifecycle_count =
            Spi::get_one::<i64>(&format!("SELECT count(*) {catalog_lifecycle_from}"))
                .expect("catalog lifecycle count query should succeed")
                .expect("catalog lifecycle count should exist");
        let dump_restore_status = Spi::get_one::<String>(&format!(
            "SELECT status {catalog_lifecycle_from} \
             WHERE lifecycle_event = 'pg_dump_restore'"
        ))
        .expect("dump restore lifecycle query should succeed")
        .expect("dump restore lifecycle should exist");
        let drop_index_cleanup_surface = Spi::get_one::<String>(&format!(
            "SELECT cleanup_surface {catalog_lifecycle_from} \
             WHERE lifecycle_event = 'drop_index'"
        ))
        .expect("drop index lifecycle query should succeed")
        .expect("drop index lifecycle should exist");
        let drop_index_migration_surface = Spi::get_one::<String>(&format!(
            "SELECT migration_surface {catalog_lifecycle_from} \
             WHERE lifecycle_event = 'drop_index'"
        ))
        .expect("drop index migration lifecycle query should succeed")
        .expect("drop index migration lifecycle should exist");
        let drop_index_status = Spi::get_one::<String>(&format!(
            "SELECT status {catalog_lifecycle_from} \
             WHERE lifecycle_event = 'drop_index'"
        ))
        .expect("drop index status lifecycle query should succeed")
        .expect("drop index status lifecycle should exist");
        let basebackup_status = Spi::get_one::<String>(&format!(
            "SELECT status {catalog_lifecycle_from} \
             WHERE lifecycle_event = 'basebackup_wal_replay'"
        ))
        .expect("basebackup lifecycle query should succeed")
        .expect("basebackup lifecycle should exist");
        let upgrade_migration_surface = Spi::get_one::<String>(&format!(
            "SELECT migration_surface {catalog_lifecycle_from} \
             WHERE lifecycle_event = 'extension_upgrade_0_1_0_to_0_1_1'"
        ))
        .expect("upgrade lifecycle query should succeed")
        .expect("upgrade lifecycle should exist");
        let upgrade_status = Spi::get_one::<String>(&format!(
            "SELECT status {catalog_lifecycle_from} \
             WHERE lifecycle_event = 'extension_upgrade_0_1_0_to_0_1_1'"
        ))
        .expect("upgrade lifecycle status query should succeed")
        .expect("upgrade lifecycle status should exist");

        assert_eq!(degradation_count, 8);
        assert_eq!(degraded_unavailable_action, "skip_and_report");
        assert_eq!(strict_unavailable_action, "fail_closed");
        assert_eq!(stale_degraded_status, "requires_fresh_epoch");
        assert_eq!(merge_order_count, 8);
        assert_eq!(first_order_key, "score");
        assert_eq!(assignment_direction, "primary_before_boundary_replica");
        assert_eq!(
            dedupe_order,
            "score,assignment_role,served_epoch,node_id,pid,object_version,row_index,row_locator"
        );
        assert_eq!(publication_step_count, 5);
        assert_eq!(persistence_action, "persist_remote_epoch_manifest");
        assert_eq!(stale_action, "refresh_remote_epoch_manifest");
        assert_eq!(
            transport_validator,
            "future_executor_must_use_libpq_pipeline"
        );
        assert_eq!(publication_result_count, 4);
        assert_eq!(pending_result_validator, "must_name_next_executor_step");
        assert!(validation_result_recommendation.contains("remote apply executor"));
        assert_eq!(manifest_parameter_count, 3);
        assert_eq!(manifest_payload_type, "jsonb");
        assert_eq!(manifest_result_count, 3);
        assert_eq!(manifest_status_validator, "must_report_ready_or_blocker");
        assert_eq!(manifest_executor_step_count, 5);
        assert_eq!(
            manifest_send_input,
            "ec_spire_remote_epoch_manifest_libpq_parameter_contract"
        );
        assert_eq!(search_result_count, 4);
        assert_eq!(search_blocked_validator, "must_preserve_next_blocker");
        assert_eq!(
            remote_dedupe_key,
            "global_vec_id_or_node_scoped_local_vec_id"
        );
        assert_eq!(operator_entrypoint_count, 23);
        assert_eq!(operator_entrypoint_reachable_count, 23);
        assert_eq!(
            search_gate_next_action,
            "resolve_reported_blocker_before_expect_result_rows"
        );
        assert_eq!(publication_result_use, "manifest_publication_result");
        assert_eq!(
            search_secret_next_action,
            "resolve_missing_conninfo_secrets_before_opening_libpq_connections"
        );
        assert_eq!(single_secret_use, "single_conninfo_secret_probe");
        assert_eq!(production_state_use, "production_executor_dry_state");
        assert_eq!(
            pipeline_steps_action,
            "inspect_first_non_ready_step_before_live_probe_or_narrow_surfaces"
        );
        assert_eq!(
            pipeline_steps_live_use,
            "consolidated_remote_pipeline_steps_live_probe"
        );
        assert_eq!(
            receive_attempts_use,
            "per_node_remote_receive_attempt_diagnostics"
        );
        assert_eq!(budget_entrypoint_use, "remote_executor_resource_governance");
        assert_eq!(
            stage_e_fault_matrix_use,
            "local_multi_instance_fault_fixture_contract"
        );
        assert_eq!(
            operator_diagnostics_use,
            "packet_friendly_production_readiness_rollup"
        );
        assert_eq!(
            stage_e_lifecycle_matrix_use,
            "local_multi_instance_lifecycle_fixture_contract"
        );
        assert_eq!(
            manifest_freshness_use,
            "stage_e_manifest_freshness_assertion"
        );
        assert_eq!(libpq_lifecycle_count, 2);
        assert_eq!(search_connection_policy, "per_query");
        assert_eq!(
            search_secret_policy,
            "conninfo_secret_name_resolved_by_executor"
        );
        assert_eq!(manifest_conninfo_policy, "never_expose_raw_conninfo_in_sql");
        assert_eq!(secret_provider_count, 3);
        assert_eq!(
            selected_secret_provider,
            "external_executor_secret_provider"
        );
        assert!(!selected_raw_conninfo_allowed);
        assert_eq!(
            rejected_provider_storage,
            "never_store_raw_conninfo_in_extension_catalog"
        );
        assert_eq!(catalog_lifecycle_count, 4);
        assert_eq!(dump_restore_status, "requires_operator_reregistration");
        assert_eq!(
            drop_index_cleanup_surface,
            "ec_spire_remote_catalog_index_cleanup,ec_spire_remote_catalog_orphan_cleanup"
        );
        assert_eq!(
            drop_index_migration_surface,
            "ec_spire_remote_catalog_drop_index_cleanup"
        );
        assert_eq!(drop_index_status, "automatic_event_trigger_cleanup");
        assert_eq!(basebackup_status, "supported");
        assert_eq!(upgrade_migration_surface, "ecaz--0.1.0--0.1.1.sql");
        assert_eq!(upgrade_status, "supported_after_upgrade_script");
    }
