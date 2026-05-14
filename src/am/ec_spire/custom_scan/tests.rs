#[cfg(test)]
mod tests {
    use super::*;
    use crate::am::ec_spire::SpireDmlFrontdoorCustomScanMode;

    #[test]
    fn custom_scan_status_reports_executor_stream_tuple_payload_slots() {
        let row = custom_scan_status_row();

        assert_eq!(row.provider_name, "EcSpireDistributedScan");
        assert!(row.path_generation_enabled);
        assert!(row.exec_wiring_enabled);
        assert_eq!(row.next_step, "add ADR-069 write path");
    }

    #[test]
    fn custom_scan_eligibility_counts_remote_available_placements() {
        let row = SpireCustomScanIndexEligibilityRow {
            active_epoch: 7,
            local_placement_count: 1,
            remote_node_count: 1,
            remote_available_node_count: 1,
            remote_placement_count: 2,
            remote_available_placement_count: 2,
            remote_unavailable_placement_count: 0,
            all_remote_placements_available: true,
            eligible_for_custom_scan: true,
            status: "customscan_candidate",
            next_step:
                "planner path generation must also verify ORDER BY vector distance LIMIT query shape",
        };

        assert!(row.eligible_for_custom_scan);
        assert_eq!(row.status, "customscan_candidate");
        assert_eq!(row.remote_node_count, 1);
    }

    #[test]
    fn custom_scan_dml_modes_map_to_plan_private_values() {
        let cases = [
            (
                SpireCustomScanPlanMode::DmlPkSelectTuplePayload,
                CUSTOM_SCAN_PLAN_MODE_DML_PK_SELECT,
                SpireDmlFrontdoorCustomScanMode::CoordinatorPkSelectTuplePayload,
            ),
            (
                SpireCustomScanPlanMode::DmlUpdateTuplePayload,
                CUSTOM_SCAN_PLAN_MODE_DML_UPDATE,
                SpireDmlFrontdoorCustomScanMode::CoordinatorUpdateTuplePayload,
            ),
            (
                SpireCustomScanPlanMode::DmlDeleteTuplePayload,
                CUSTOM_SCAN_PLAN_MODE_DML_DELETE,
                SpireDmlFrontdoorCustomScanMode::CoordinatorDeleteTuplePayload,
            ),
        ];

        for (plan_mode, raw, dml_mode) in cases {
            assert!(plan_mode.is_dml());
            assert_eq!(plan_mode.raw(), raw);
            assert_eq!(custom_scan_mode_from_u32(raw), Some(plan_mode));
            assert_eq!(custom_scan_plan_mode_for_dml_mode(dml_mode), plan_mode);
        }
        assert!(!SpireCustomScanPlanMode::VectorOrderLimit.is_dml());
        assert_eq!(
            custom_scan_mode_from_u32(CUSTOM_SCAN_PLAN_MODE_VECTOR_ORDER_LIMIT),
            Some(SpireCustomScanPlanMode::VectorOrderLimit)
        );
    }

    #[test]
    fn custom_scan_dml_column_metadata_validates_by_mode() {
        let updated = vec!["title".to_owned()];
        let projected = vec!["id".to_owned(), "title".to_owned()];
        let empty = Vec::<String>::new();

        custom_scan_validate_dml_column_metadata(
            SpireCustomScanPlanMode::DmlUpdateTuplePayload,
            &updated,
            &empty,
        )
        .expect("UPDATE metadata should validate");
        custom_scan_validate_dml_column_metadata(
            SpireCustomScanPlanMode::DmlDeleteTuplePayload,
            &empty,
            &empty,
        )
        .expect("DELETE metadata should validate");
        custom_scan_validate_dml_column_metadata(
            SpireCustomScanPlanMode::DmlPkSelectTuplePayload,
            &empty,
            &projected,
        )
        .expect("PK SELECT metadata should validate");

        assert_eq!(
            custom_scan_validate_dml_column_metadata(
                SpireCustomScanPlanMode::DmlUpdateTuplePayload,
                &empty,
                &empty,
            )
            .expect_err("UPDATE without updated columns should fail"),
            "EcSpireDistributedScan DML UPDATE plan requires updated columns"
        );
        assert_eq!(
            custom_scan_validate_dml_column_metadata(
                SpireCustomScanPlanMode::DmlDeleteTuplePayload,
                &updated,
                &empty,
            )
            .expect_err("DELETE with updated columns should fail"),
            "EcSpireDistributedScan DML DELETE plan must not carry column payload metadata"
        );
    }

    #[test]
    fn custom_scan_dml_primitive_invocation_uses_plan_metadata() {
        let pk_value = [0, 0, 0, 0, 0, 0, 0, 5];
        let projected = vec!["id".to_owned(), "title".to_owned()];
        let invocation = custom_scan_dml_primitive_invocation_from_parts(
            pg_sys::Oid::from(42),
            SpireCustomScanPlanMode::DmlPkSelectTuplePayload,
            "id",
            pk_value,
            &[],
            &projected,
        )
        .expect("PK SELECT invocation should build");

        assert_eq!(invocation.index_oid, pg_sys::Oid::from(42));
        assert_eq!(
            invocation.mode,
            SpireDmlFrontdoorCustomScanMode::CoordinatorPkSelectTuplePayload
        );
        assert_eq!(
            invocation.primitive,
            "ec_spire_forward_coordinator_select_tuple_payload"
        );
        assert_eq!(invocation.pk_column, "id");
        assert_eq!(invocation.pk_value, pk_value);
        assert!(invocation.updated_columns.is_empty());
        assert_eq!(invocation.projected_columns, projected);
    }

    #[test]
    fn custom_scan_dml_primitive_invocation_rejects_incomplete_state() {
        let error = custom_scan_dml_primitive_invocation_from_parts(
            pg_sys::InvalidOid,
            SpireCustomScanPlanMode::DmlUpdateTuplePayload,
            "id",
            [0, 0, 0, 0, 0, 0, 0, 5],
            &["title".to_owned()],
            &[],
        )
        .expect_err("missing index OID should fail");
        assert_eq!(
            error,
            "EcSpireDistributedScan DML primitive invocation requires index OID"
        );

        let error = custom_scan_dml_primitive_invocation_from_parts(
            pg_sys::Oid::from(42),
            SpireCustomScanPlanMode::DmlUpdateTuplePayload,
            "id",
            [0, 0, 0, 0, 0, 0, 0, 5],
            &[],
            &[],
        )
        .expect_err("UPDATE without column metadata should fail");
        assert_eq!(
            error,
            "EcSpireDistributedScan DML UPDATE plan requires updated columns"
        );
    }

    #[test]
    fn custom_scan_dml_pk_select_requested_columns_include_pk_for_quals() {
        let invocation = custom_scan_dml_primitive_invocation_from_parts(
            pg_sys::Oid::from(42),
            SpireCustomScanPlanMode::DmlPkSelectTuplePayload,
            "id",
            [0, 0, 0, 0, 0, 0, 0, 5],
            &[],
            &["title".to_owned()],
        )
        .expect("PK SELECT invocation should build");

        assert_eq!(
            custom_scan_dml_pk_select_requested_columns(&invocation, &[]),
            vec!["id".to_owned(), "title".to_owned()]
        );
        assert_eq!(
            custom_scan_dml_pk_select_requested_columns(
                &invocation,
                &[
                    "id".to_owned(),
                    "title".to_owned(),
                    "source_identity".to_owned()
                ]
            ),
            vec![
                "id".to_owned(),
                "title".to_owned(),
                "source_identity".to_owned()
            ]
        );

        let invocation = custom_scan_dml_primitive_invocation_from_parts(
            pg_sys::Oid::from(42),
            SpireCustomScanPlanMode::DmlPkSelectTuplePayload,
            "id",
            [0, 0, 0, 0, 0, 0, 0, 5],
            &[],
            &["id".to_owned(), "title".to_owned()],
        )
        .expect("PK SELECT invocation should build");
        assert_eq!(
            custom_scan_dml_pk_select_requested_columns(&invocation, &[]),
            vec!["id".to_owned(), "title".to_owned()]
        );
    }

    #[test]
    fn custom_scan_default_state_starts_with_zero_progress_counters() {
        let state = custom_scan_default_exec_state();

        assert_eq!(state.mode, SpireCustomScanPlanMode::VectorOrderLimit);
        assert_eq!(state.index_oid, pg_sys::InvalidOid);
        assert_eq!(state.top_k, 0);
        assert!(state.query.is_empty());
        assert!(state.outputs.is_empty());
        assert_eq!(state.next_output, 0);
        assert!(!state.loaded_outputs);
        assert!(!state.dml_payload_loaded);
        assert!(!state.dml_payload_emitted);
        assert!(state.dml_tuple_payload_json.is_none());
    }

    #[test]
    fn custom_scan_begin_vector_order_limit_state_initializes_plan_parts() {
        let mut state = custom_scan_default_exec_state();
        state.outputs = vec![remote_output_row(50, 3, -1.25)];
        state.next_output = 1;
        state.loaded_outputs = true;
        state.dml_payload_loaded = true;
        state.dml_payload_emitted = true;
        state.dml_tuple_payload_json = Some(r#"{"id":5}"#.to_owned());

        custom_scan_init_vector_order_limit_exec_state(
            &mut state,
            pg_sys::Oid::from(42),
            3,
            vec![0.25, 0.5, 0.75],
            vec!["id".to_owned(), "title".to_owned()],
            Vec::new(),
        );

        assert_eq!(state.mode, SpireCustomScanPlanMode::VectorOrderLimit);
        assert_eq!(state.index_oid, pg_sys::Oid::from(42));
        assert_eq!(state.top_k, 3);
        assert_eq!(state.query, vec![0.25, 0.5, 0.75]);
        assert_eq!(
            state.tuple_payload_columns,
            vec!["id".to_owned(), "title".to_owned()]
        );
        assert_eq!(state.tuple_payload_columns.len(), 2);
        assert!(state.tuple_payload_inputs.is_empty());
        assert!(state.outputs.is_empty());
        assert_eq!(state.next_output, 0);
        assert!(!state.loaded_outputs);
        assert!(!state.dml_payload_loaded);
        assert!(!state.dml_payload_emitted);
        assert!(state.dml_tuple_payload_json.is_none());
    }

    #[test]
    fn custom_scan_end_release_drops_reachable_rust_state() {
        let mut state = custom_scan_default_exec_state();
        state.index_oid = pg_sys::Oid::from(42);
        state.top_k = 3;
        state.query = vec![1.0, 2.0, 3.0];
        state.dml_pk_column = "id".to_owned();
        state.dml_pk_value = [0, 0, 0, 0, 0, 0, 0, 5];
        state.dml_updated_columns = vec!["title".to_owned()];
        state.dml_projected_columns = vec!["id".to_owned(), "title".to_owned()];
        state.dml_update_value_exprs = vec![std::ptr::null_mut()];
        state.tuple_payload_columns = vec!["id".to_owned(), "title".to_owned()];
        state.outputs = vec![remote_output_row(50, 3, -1.25), remote_output_row(51, 4, -1.0)];
        state.next_output = 2;
        state.loaded_outputs = true;
        state.dml_payload_loaded = true;
        state.dml_payload_emitted = true;
        state.dml_tuple_payload_json = Some(r#"{"id":5}"#.to_owned());

        custom_scan_release_exec_state_for_end(&mut state);

        assert_eq!(state.index_oid, pg_sys::InvalidOid);
        assert_eq!(state.top_k, 0);
        assert_eq!(state.query.capacity(), 0);
        assert_eq!(state.dml_pk_column.capacity(), 0);
        assert_eq!(state.dml_pk_value, [0; 8]);
        assert_eq!(state.dml_updated_columns.capacity(), 0);
        assert_eq!(state.dml_projected_columns.capacity(), 0);
        assert_eq!(state.dml_update_value_exprs.capacity(), 0);
        assert_eq!(state.tuple_payload_columns.capacity(), 0);
        assert_eq!(state.tuple_payload_inputs.capacity(), 0);
        assert_eq!(state.outputs.capacity(), 0);
        assert_eq!(state.next_output, 0);
        assert!(!state.loaded_outputs);
        assert!(!state.dml_payload_loaded);
        assert!(!state.dml_payload_emitted);
        assert!(state.dml_tuple_payload_json.is_none());
    }

    #[test]
    fn custom_scan_rescan_resets_output_progress_and_allows_second_pass() {
        let expected = vec![remote_output_row(50, 3, -1.25), remote_output_row(51, 4, -1.0)];
        let mut state = custom_scan_default_exec_state();
        state.outputs = expected.clone();
        state.loaded_outputs = true;
        state.dml_payload_loaded = true;
        state.dml_payload_emitted = true;
        state.dml_tuple_payload_json = Some(r#"{"id":5}"#.to_owned());

        let mut first_pass = Vec::new();
        while let Some(output_index) = custom_scan_next_output_index(&mut state) {
            first_pass.push(state.outputs[output_index].clone());
        }
        assert_eq!(first_pass, expected);
        assert_eq!(state.next_output, expected.len());

        custom_scan_reset_exec_state_for_rescan(&mut state);

        assert!(state.outputs.is_empty());
        assert_eq!(state.next_output, 0);
        assert!(!state.loaded_outputs);
        assert!(!state.dml_payload_loaded);
        assert!(!state.dml_payload_emitted);
        assert!(state.dml_tuple_payload_json.is_none());

        state.outputs = expected.clone();
        state.loaded_outputs = true;
        let mut second_pass = Vec::new();
        while let Some(output_index) = custom_scan_next_output_index(&mut state) {
            second_pass.push(state.outputs[output_index].clone());
        }
        assert_eq!(second_pass, expected);
    }

    #[test]
    fn custom_scan_cost_increases_with_remote_fanout() {
        let mut low_fanout = eligible_cost_row();
        low_fanout.remote_available_node_count = 1;
        low_fanout.remote_available_placement_count = 4;
        let mut high_fanout = low_fanout;
        high_fanout.remote_available_node_count = 4;
        high_fanout.remote_available_placement_count = 16;

        let low = estimate_custom_scan_cost_with_constants(
            10.0,
            1_000.0,
            64.0,
            &low_fanout,
            default_cost_constants(),
            0.01,
        );
        let high = estimate_custom_scan_cost_with_constants(
            10.0,
            1_000.0,
            64.0,
            &high_fanout,
            default_cost_constants(),
            0.01,
        );

        assert!(low.total_cost.is_finite());
        assert!(high.startup_cost > low.startup_cost);
        assert!(high.total_cost > low.total_cost);
    }

    #[test]
    fn custom_scan_cost_increases_with_output_rows() {
        let eligibility = eligible_cost_row();
        let small = estimate_custom_scan_cost_with_constants(
            1.0,
            1_000.0,
            64.0,
            &eligibility,
            default_cost_constants(),
            0.01,
        );
        let large = estimate_custom_scan_cost_with_constants(
            100.0,
            1_000.0,
            64.0,
            &eligibility,
            default_cost_constants(),
            0.01,
        );

        assert!(large.total_cost > small.total_cost);
        assert_eq!(large.startup_cost, small.startup_cost);
    }

    #[test]
    fn custom_scan_cost_accounts_for_projected_tuple_width() {
        let eligibility = eligible_cost_row();
        let narrow = estimate_custom_scan_cost_with_constants(
            100.0,
            1_000.0,
            8.0,
            &eligibility,
            default_cost_constants(),
            0.01,
        );
        let wide = estimate_custom_scan_cost_with_constants(
            100.0,
            1_000.0,
            512.0,
            &eligibility,
            default_cost_constants(),
            0.01,
        );

        assert!(wide.total_cost > narrow.total_cost);
        assert_eq!(wide.startup_cost, narrow.startup_cost);
    }

    fn eligible_cost_row() -> SpireCustomScanIndexEligibilityRow {
        SpireCustomScanIndexEligibilityRow {
            active_epoch: 7,
            local_placement_count: 0,
            remote_node_count: 2,
            remote_available_node_count: 2,
            remote_placement_count: 8,
            remote_available_placement_count: 8,
            remote_unavailable_placement_count: 0,
            all_remote_placements_available: true,
            eligible_for_custom_scan: true,
            status: "customscan_candidate",
            next_step:
                "planner path generation must also verify ORDER BY vector distance LIMIT query shape",
        }
    }

    fn default_cost_constants() -> PlannerCostConstants {
        PlannerCostConstants {
            random_page_cost: 4.0,
            seq_page_cost: 1.0,
            cpu_operator_cost: 0.0025,
        }
    }

    fn remote_output_row(
        heap_block: u32,
        heap_offset: u16,
        score: f32,
    ) -> super::super::SpireRemoteProductionScanOutputRow {
        super::super::SpireRemoteProductionScanOutputRow {
            requested_epoch: 1,
            served_epoch: 1,
            node_id: super::super::meta::SPIRE_LOCAL_NODE_ID,
            heap_block,
            heap_offset,
            score,
            heap_lookup_owner: super::super::SPIRE_REMOTE_LOCAL_HEAP_RESOLUTION,
            vec_id: vec![heap_block as u8],
            row_locator: vec![heap_offset as u8],
            tuple_payload_json: None,
            typed_tuple_payload: None,
            tuple_payload_missing: false,
        }
    }
}
