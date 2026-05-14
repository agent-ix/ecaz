#[cfg(test)]
mod production_executor_state_tests {
    use super::*;

    fn planned_dispatch(node_id: u32, pid_count: u64) -> SpireRemoteSearchLibpqDispatchPlanRow {
        SpireRemoteSearchLibpqDispatchPlanRow {
            requested_epoch: 7,
            node_id,
            selected_pids: (0..pid_count).collect(),
            pid_count,
            query_dimension: 2,
            top_k: 10,
            consistency_mode: "strict",
            sql_template: SPIRE_REMOTE_SEARCH_LIBPQ_SQL_TEMPLATE,
            parameter_count: SPIRE_REMOTE_SEARCH_LIBPQ_PARAMETER_COUNT,
            result_column_count: remote_search_result_column_count(),
            conninfo_secret_name: format!("spire/remote/{node_id}"),
            remote_index_regclass: format!("ec_spire_remote_{node_id}_idx"),
            descriptor_generation: 1,
            remote_index_identity: vec![u8::try_from(node_id).expect("node id should fit u8")],
            pipeline_mode: SPIRE_REMOTE_TRANSPORT_LIBPQ_PIPELINE,
            dispatch_action: SPIRE_REMOTE_DISPATCH_PIPELINE_ACTION,
            receive_validator: SPIRE_REMOTE_SEARCH_RECEIVE_VALIDATOR,
            status: SPIRE_REMOTE_STATUS_READY,
        }
    }

    fn blocked_dispatch(
        node_id: u32,
        pid_count: u64,
        status: &'static str,
    ) -> SpireRemoteSearchLibpqDispatchPlanRow {
        let mut row = planned_dispatch(node_id, pid_count);
        row.pipeline_mode = SPIRE_REMOTE_NONE;
        row.dispatch_action = SPIRE_REMOTE_DISPATCH_BLOCKED_ACTION;
        row.status = status;
        row
    }

    fn ready_transport_row(
        node_id: u32,
        row_count: u64,
    ) -> SpireRemoteProductionTransportProbeRow {
        SpireRemoteProductionTransportProbeRow {
            node_id,
            started_after_ms: 1,
            completed_after_ms: 2,
            elapsed_ms: 1,
            row_count,
            status: SPIRE_REMOTE_STATUS_READY,
            failure_category: SPIRE_REMOTE_NONE,
        }
    }

    fn failed_transport_row(
        node_id: u32,
        failure_category: &'static str,
    ) -> SpireRemoteProductionTransportProbeRow {
        SpireRemoteProductionTransportProbeRow {
            node_id,
            started_after_ms: 1,
            completed_after_ms: 2,
            elapsed_ms: 1,
            row_count: 0,
            status: SPIRE_REMOTE_STATUS_PRODUCTION_TRANSPORT_FAILED,
            failure_category,
        }
    }

    fn candidate_for_state_test(
        node_id: u32,
        pid: u64,
        row_index: u32,
    ) -> SpireRemoteSearchCandidateRow {
        SpireRemoteSearchCandidateRow {
            served_epoch: 7,
            node_id,
            pid,
            object_version: 1,
            row_index,
            assignment_flags: storage::SPIRE_ASSIGNMENT_FLAG_PRIMARY,
            vec_id: storage::SpireVecId::local(
                (u64::from(node_id) << 32) | (u64::from(row_index) + 1),
            )
            .as_bytes()
            .to_vec(),
            row_locator: vec![row_index as u8 + 1],
            score: row_index as f32,
        }
    }

    fn ready_candidate_receive_result(
        node_id: u32,
        selected_pids: Vec<u64>,
        candidate_count: u32,
    ) -> SpireRemoteProductionCandidateReceiveResult {
        let pid = selected_pids
            .first()
            .copied()
            .expect("selected pid should exist");
        let candidates = (0..candidate_count)
            .map(|row_index| candidate_for_state_test(node_id, pid, row_index))
            .collect::<Vec<_>>();
        SpireRemoteProductionCandidateReceiveResult {
            node_id,
            started_after_ms: 2,
            completed_after_ms: 3,
            elapsed_ms: 1,
            candidate_count: u64::from(candidate_count),
            status: SPIRE_REMOTE_STATUS_READY,
            failure_category: SPIRE_REMOTE_NONE,
            batch: Some(SpireRemoteSearchCandidateBatch {
                node_id,
                selected_pids,
                candidates,
            }),
        }
    }

    fn failed_candidate_receive_result(
        node_id: u32,
        failure_category: &'static str,
    ) -> SpireRemoteProductionCandidateReceiveResult {
        SpireRemoteProductionCandidateReceiveResult {
            node_id,
            started_after_ms: 2,
            completed_after_ms: 3,
            elapsed_ms: 1,
            candidate_count: 0,
            status: SPIRE_REMOTE_STATUS_CANDIDATE_RECEIVE_FAILED,
            failure_category,
            batch: None,
        }
    }

    #[test]
    fn production_fault_matrix_covers_required_categories() {
        let rows = remote_search_production_fault_matrix_rows();
        let categories = rows
            .iter()
            .map(|row| row.failure_category)
            .collect::<std::collections::HashSet<_>>();
        let required = [
            SPIRE_REMOTE_PRODUCTION_TRANSPORT_CONNECT_FAILED,
            SPIRE_REMOTE_STATUS_EXECUTOR_OVERLOAD,
            SPIRE_REMOTE_STATUS_REQUIRES_SECRET,
            SPIRE_REMOTE_PRODUCTION_REMOTE_STATEMENT_TIMEOUT,
            SPIRE_REMOTE_PRODUCTION_LOCAL_STATEMENT_TIMEOUT,
            SPIRE_REMOTE_PRODUCTION_REMOTE_BACKEND_TERMINATED,
            SPIRE_REMOTE_PRODUCTION_TRANSPORT_REMOTE_QUERY_FAILED,
            SPIRE_REMOTE_PRODUCTION_REMOTE_QUERY_CANCELLED,
            SPIRE_REMOTE_PRODUCTION_LOCAL_QUERY_CANCELLED,
            SPIRE_REMOTE_PRODUCTION_CANDIDATE_VALIDATION_FAILED,
            SPIRE_REMOTE_STATUS_ENDPOINT_IDENTITY_MISMATCH,
            SPIRE_REMOTE_STATUS_TUPLE_TRANSPORT_RETIRED,
            SPIRE_REMOTE_STATUS_REMOTE_PAYLOAD_TOO_LARGE,
            SPIRE_REMOTE_STATUS_INCOMPATIBLE_EXTENSION_VERSION,
            SPIRE_REMOTE_PRODUCTION_EXTENSION_VERSION_MISMATCH,
            SPIRE_REMOTE_STATUS_STALE_EPOCH,
            SPIRE_REMOTE_PRODUCTION_SERVED_EPOCH_MISMATCH,
            SPIRE_REMOTE_STATUS_CONSISTENCY_MODE_MISMATCH,
            SPIRE_REMOTE_PRODUCTION_REMOTE_INDEX_UNAVAILABLE,
            SPIRE_REMOTE_PRODUCTION_REMOTE_HEAP_RESOLUTION_FAILED,
            SPIRE_REMOTE_PRODUCTION_REMOTE_HEAP_ROW_MISSING,
        ];

        assert_eq!(rows.len(), categories.len(), "matrix categories should be unique");
        for category in required {
            assert!(categories.contains(category), "missing category {category}");
        }
        let local_timeout = rows
            .iter()
            .find(|row| row.failure_category == SPIRE_REMOTE_PRODUCTION_LOCAL_STATEMENT_TIMEOUT)
            .expect("local timeout row should exist");
        let remote_timeout = rows
            .iter()
            .find(|row| row.failure_category == SPIRE_REMOTE_PRODUCTION_REMOTE_STATEMENT_TIMEOUT)
            .expect("remote timeout row should exist");
        let consistency_mismatch = rows
            .iter()
            .find(|row| row.failure_category == SPIRE_REMOTE_STATUS_CONSISTENCY_MODE_MISMATCH)
            .expect("consistency mismatch row should exist");

        assert_eq!(local_timeout.strict_action, "cancel_query");
        assert_eq!(local_timeout.degraded_action, "cancel_query");
        assert_eq!(remote_timeout.strict_action, "fail_closed");
        assert_eq!(remote_timeout.degraded_action, "skip_node");
        assert_eq!(consistency_mismatch.degraded_action, "fail_closed");
    }

    #[test]
    fn stage_e_fault_matrix_covers_fixture_cases() {
        let rows = remote_search_stage_e_fault_matrix_rows();
        let cases = rows
            .iter()
            .map(|row| row.fault_case)
            .collect::<std::collections::HashSet<_>>();
        let required = [
            "epoch_mismatch",
            "version_skew",
            "fingerprint_mismatch",
            "connection_reset_mid_batch",
            "remote_backend_termination",
            "remote_statement_timeout",
            "local_statement_timeout",
            "local_cancel",
            "simulated_network_partition",
            "remote_oom",
            "missing_or_reindexed_remote_index",
        ];

        assert_eq!(rows.len(), cases.len(), "Stage E fault cases should be unique");
        for fault_case in required {
            assert!(cases.contains(fault_case), "missing Stage E case {fault_case}");
        }

        let local_cancel = rows
            .iter()
            .find(|row| row.fault_case == "local_cancel")
            .expect("local cancel case should exist");
        assert_eq!(local_cancel.strict_action, "cancel_query");
        assert_eq!(local_cancel.degraded_action, "cancel_query");
        assert!(local_cancel
            .counter_delta
            .contains("retained_candidate_batch_count=0"));

        let remote_oom = rows
            .iter()
            .find(|row| row.fault_case == "remote_oom")
            .expect("remote OOM case should exist");
        assert_eq!(
            remote_oom.failure_category,
            SPIRE_REMOTE_PRODUCTION_TRANSPORT_REMOTE_QUERY_FAILED
        );
        assert_eq!(remote_oom.degraded_action, "skip_node");

        let missing_index = rows
            .iter()
            .find(|row| row.fault_case == "missing_or_reindexed_remote_index")
            .expect("missing/reindexed index case should exist");
        assert_eq!(
            missing_index.failure_category,
            SPIRE_REMOTE_PRODUCTION_REMOTE_INDEX_UNAVAILABLE
        );
        assert_eq!(
            missing_index.next_executor_step,
            SPIRE_REMOTE_EXECUTOR_STEP_COMPACT_CANDIDATE_RECEIVE
        );
    }

    #[test]
    fn production_executor_state_moves_ready_transport_to_candidate_receive() {
        let dispatch_rows = vec![planned_dispatch(2, 1), planned_dispatch(3, 2)];
        let transport_rows = vec![ready_transport_row(2, 4), ready_transport_row(3, 5)];
        let row = remote_search_production_executor_state_summary_from_transport_probe_rows(
            7,
            &dispatch_rows,
            &transport_rows,
        )
        .expect("transport summary should succeed");

        assert_eq!(row.planned_dispatch_count, 2);
        assert_eq!(row.transport_pending_dispatch_count, 0);
        assert_eq!(row.transport_sent_dispatch_count, 2);
        assert_eq!(row.transport_ready_dispatch_count, 2);
        assert_eq!(row.transport_failed_dispatch_count, 0);
        assert_eq!(row.transport_row_count, 9);
        assert_eq!(row.next_executor_step, "compact_candidate_receive");
        assert_eq!(row.status, "requires_compact_candidate_receive");
        assert_eq!(row.first_transport_failure_category, "none");
    }

    #[test]
    fn production_executor_state_preserves_transport_failure_category() {
        let dispatch_rows = vec![planned_dispatch(2, 1), planned_dispatch(3, 1)];
        let transport_rows = vec![
            failed_transport_row(2, SPIRE_REMOTE_PRODUCTION_TRANSPORT_CONNECT_FAILED),
            ready_transport_row(3, 1),
        ];
        let row = remote_search_production_executor_state_summary_from_transport_probe_rows(
            7,
            &dispatch_rows,
            &transport_rows,
        )
        .expect("transport summary should succeed");

        assert_eq!(row.transport_sent_dispatch_count, 2);
        assert_eq!(row.transport_ready_dispatch_count, 1);
        assert_eq!(row.transport_failed_dispatch_count, 1);
        assert_eq!(row.next_executor_step, "production_transport_adapter");
        assert_eq!(row.status, "remote_transport_failed");
        assert_eq!(row.first_transport_failure_category, "connect_failed");
    }

    #[test]
    fn production_executor_degraded_transport_failure_skips_node() {
        let dispatch_rows = vec![planned_dispatch(2, 1), planned_dispatch(3, 1)];
        let transport_rows = vec![
            failed_transport_row(2, SPIRE_REMOTE_PRODUCTION_TRANSPORT_CONNECT_FAILED),
            ready_transport_row(3, 1),
        ];
        let row =
            remote_search_production_executor_state_summary_from_transport_probe_rows_with_consistency_mode(
                7,
                &dispatch_rows,
                &transport_rows,
                "degraded",
            )
            .expect("degraded transport summary should succeed");

        assert_eq!(row.transport_sent_dispatch_count, 1);
        assert_eq!(row.transport_failed_dispatch_count, 0);
        assert_eq!(row.degraded_skipped_dispatch_count, 1);
        assert_eq!(row.first_degraded_skip_category, "connect_failed");
        assert_eq!(row.candidate_receive_pending_dispatch_count, 1);
        assert_eq!(row.next_executor_step, "compact_candidate_receive");
        assert_eq!(row.status, "requires_compact_candidate_receive");
    }

    #[test]
    fn production_executor_degraded_pre_dispatch_block_skips_node() {
        let dispatch_rows = vec![
            blocked_dispatch(2, 1, SPIRE_REMOTE_STATUS_INCOMPATIBLE_EXTENSION_VERSION),
            planned_dispatch(3, 1),
        ];
        let row = remote_search_production_executor_state_summary_from_dispatch_rows(
            7,
            &dispatch_rows,
            "function_argument",
            "degraded",
        )
        .expect("degraded pre-dispatch summary should succeed");

        assert_eq!(row.dispatch_count, 2);
        assert_eq!(row.blocked_before_dispatch_count, 0);
        assert_eq!(row.degraded_skipped_dispatch_count, 1);
        assert_eq!(
            row.first_degraded_skip_category,
            "incompatible_extension_version"
        );
        assert_eq!(row.transport_pending_dispatch_count, 1);
        assert_eq!(row.next_executor_step, "production_transport_adapter");
        assert_eq!(row.status, "requires_production_transport_adapter");
    }

    #[test]
    fn degraded_skip_report_lists_each_skipped_node() {
        let dispatch_rows = vec![
            blocked_dispatch(2, 3, SPIRE_REMOTE_STATUS_STALE_EPOCH),
            blocked_dispatch(4, 2, SPIRE_REMOTE_STATUS_INCOMPATIBLE_EXTENSION_VERSION),
            planned_dispatch(5, 1),
        ];
        let rows = remote_search_production_degraded_skip_report_from_dispatch_rows(
            7,
            &dispatch_rows,
            "degraded",
        )
        .expect("degraded skip report should succeed");

        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].requested_epoch, 7);
        assert_eq!(rows[0].node_id, 2);
        assert_eq!(rows[0].skipped_pid_count, 3);
        assert_eq!(rows[0].first_skip_category, "stale_epoch");
        assert_eq!(rows[0].first_skip_hint, "none");
        assert_eq!(rows[0].status, "degraded_skipped");
        assert_eq!(rows[1].node_id, 4);
        assert_eq!(rows[1].skipped_pid_count, 2);
        assert_eq!(rows[1].first_skip_category, "incompatible_extension_version");
        assert_eq!(rows[1].first_skip_hint, "none");
        assert_eq!(rows[1].status, "degraded_skipped");
    }

    #[test]
    fn degraded_skip_report_hints_retired_tuple_transport_upgrade() {
        let dispatch_rows = vec![planned_dispatch(2, 1), planned_dispatch(3, 1)];
        let transport_rows = vec![ready_transport_row(2, 1), ready_transport_row(3, 1)];
        let receive_results = vec![
            failed_candidate_receive_result(2, SPIRE_REMOTE_STATUS_TUPLE_TRANSPORT_RETIRED),
            ready_candidate_receive_result(3, vec![30], 1),
        ];
        let rows =
            remote_search_production_degraded_skip_report_from_candidate_receive_results_with_consistency_mode(
                7,
                &dispatch_rows,
                &transport_rows,
                &receive_results,
                "degraded",
            )
            .expect("degraded skip report should include retired tuple transport");

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].first_skip_category, SPIRE_REMOTE_STATUS_TUPLE_TRANSPORT_RETIRED);
        assert_eq!(rows[0].first_skip_hint, SPIRE_REMOTE_TUPLE_TRANSPORT_RETIRED_HINT);
    }

    #[test]
    fn remote_payload_caps_reject_oversized_rows_and_batches() {
        let row_error =
            validate_remote_payload_row_bytes_with_limit(17, 16, "remote typed tuple payload")
                .expect_err("row over cap should fail");
        assert!(row_error.contains(SPIRE_REMOTE_STATUS_REMOTE_PAYLOAD_TOO_LARGE));
        assert!(row_error.contains("ec_spire.max_remote_payload_bytes_per_row"));

        let batch_error =
            validate_remote_payload_batch_row_count_with_limit(65, 64, "remote heap result rows")
                .expect_err("batch over cap should fail");
        assert!(batch_error.contains(SPIRE_REMOTE_STATUS_REMOTE_PAYLOAD_TOO_LARGE));
        assert!(batch_error.contains("ec_spire.max_remote_payload_rows_per_batch"));

        let payload_values = vec!["0a0b".to_owned(), "ff".to_owned()];
        assert_eq!(typed_payload_hex_decoded_bytes(&payload_values).unwrap(), 3);
    }

    #[test]
    fn production_receive_adapters_reject_selected_pid_batches_before_connection() {
        let oversized_pids = (0_u64..65).collect::<Vec<_>>();
        let candidate_results =
            SpireRemoteProductionTransportAdapter::run_candidate_receive_requests(vec![
                SpireRemoteProductionCandidateReceiveRequest {
                    node_id: 2,
                    conninfo: "host=/conninfo/should/not/be/used".to_owned(),
                    remote_index_regclass: "ec_spire_remote_2_idx".to_owned(),
                    remote_index_identity: vec![2],
                    requested_epoch: 7,
                    query: vec![1.0, 0.0],
                    selected_pids: oversized_pids.clone(),
                    top_k: 1,
                    consistency_mode: "strict".to_owned(),
                },
            ])
            .expect("candidate receive cap check should not need a live connection");
        assert_eq!(candidate_results.len(), 1);
        assert_eq!(
            candidate_results[0].status,
            SPIRE_REMOTE_STATUS_CANDIDATE_RECEIVE_FAILED
        );
        assert_eq!(
            candidate_results[0].failure_category,
            SPIRE_REMOTE_STATUS_REMOTE_PAYLOAD_TOO_LARGE
        );
        assert!(candidate_results[0].batch.is_none());

        let heap_results = SpireRemoteProductionTransportAdapter::run_heap_receive_requests(vec![
            SpireRemoteProductionHeapReceiveRequest {
                node_id: 2,
                conninfo: "host=/conninfo/should/not/be/used".to_owned(),
                remote_index_regclass: "ec_spire_remote_2_idx".to_owned(),
                remote_index_identity: vec![2],
                requested_epoch: 7,
                query: vec![1.0, 0.0],
                selected_pids: oversized_pids,
                top_k: 1,
                consistency_mode: "strict".to_owned(),
                tuple_payload_columns: Some(vec!["id".to_owned()]),
            },
        ])
        .expect("heap receive cap check should not need a live connection");
        assert_eq!(heap_results.len(), 1);
        assert_eq!(
            heap_results[0].status,
            SPIRE_REMOTE_PRODUCTION_REMOTE_HEAP_RESOLUTION_FAILED
        );
        assert_eq!(
            heap_results[0].failure_category,
            SPIRE_REMOTE_STATUS_REMOTE_PAYLOAD_TOO_LARGE
        );
        assert!(heap_results[0].candidates.is_empty());
    }

    #[test]
    fn degraded_skip_report_hints_remote_payload_cap() {
        let dispatch_rows = vec![planned_dispatch(2, 1), planned_dispatch(3, 1)];
        let transport_rows = vec![ready_transport_row(2, 1), ready_transport_row(3, 1)];
        let receive_results = vec![
            failed_candidate_receive_result(2, SPIRE_REMOTE_STATUS_REMOTE_PAYLOAD_TOO_LARGE),
            ready_candidate_receive_result(3, vec![30], 1),
        ];
        let rows =
            remote_search_production_degraded_skip_report_from_candidate_receive_results_with_consistency_mode(
                7,
                &dispatch_rows,
                &transport_rows,
                &receive_results,
                "degraded",
            )
            .expect("degraded skip report should include remote payload cap");

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].first_skip_category, SPIRE_REMOTE_STATUS_REMOTE_PAYLOAD_TOO_LARGE);
        assert_eq!(rows[0].first_skip_hint, SPIRE_REMOTE_PAYLOAD_TOO_LARGE_HINT);
    }

    #[test]
    fn production_executor_strict_candidate_receive_preserves_12a_failure_categories() {
        for failure_category in [
            SPIRE_REMOTE_STATUS_TUPLE_TRANSPORT_RETIRED,
            SPIRE_REMOTE_STATUS_REMOTE_PAYLOAD_TOO_LARGE,
        ] {
            let dispatch_rows = vec![planned_dispatch(2, 1), planned_dispatch(3, 1)];
            let transport_rows = vec![ready_transport_row(2, 1), ready_transport_row(3, 1)];
            let receive_results = vec![
                failed_candidate_receive_result(2, failure_category),
                ready_candidate_receive_result(3, vec![30], 1),
            ];
            let row = remote_search_production_executor_state_summary_from_candidate_receive_results(
                7,
                &dispatch_rows,
                &transport_rows,
                &receive_results,
            )
            .expect("strict candidate receive summary should preserve 12a category");

            assert_eq!(row.candidate_receive_sent_dispatch_count, 2);
            assert_eq!(row.candidate_receive_ready_dispatch_count, 1);
            assert_eq!(row.candidate_receive_failed_dispatch_count, 1);
            assert_eq!(row.first_candidate_receive_failure_category, failure_category);
            assert_eq!(row.degraded_skipped_dispatch_count, 0);
            assert_eq!(row.next_executor_step, "compact_candidate_receive");
            assert_eq!(row.status, "remote_candidate_receive_failed");
        }
    }

    #[test]
    fn production_executor_state_rejects_unplanned_transport_result() {
        let dispatch_rows = vec![planned_dispatch(2, 1)];
        let transport_rows = vec![ready_transport_row(3, 1)];
        let error = remote_search_production_executor_state_summary_from_transport_probe_rows(
            7,
            &dispatch_rows,
            &transport_rows,
        )
        .expect_err("unplanned transport row should fail");

        assert!(
            error.contains("does not match a planned dispatch"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn production_executor_state_moves_ready_receive_to_remote_heap_resolution() {
        let dispatch_rows = vec![planned_dispatch(2, 1), planned_dispatch(3, 1)];
        let transport_rows = vec![ready_transport_row(2, 1), ready_transport_row(3, 1)];
        let receive_results = vec![
            ready_candidate_receive_result(2, vec![0], 2),
            ready_candidate_receive_result(3, vec![0], 1),
        ];
        let row = remote_search_production_executor_state_summary_from_candidate_receive_results(
            7,
            &dispatch_rows,
            &transport_rows,
            &receive_results,
        )
        .expect("candidate receive summary should succeed");

        assert_eq!(row.candidate_receive_pending_dispatch_count, 0);
        assert_eq!(row.candidate_receive_sent_dispatch_count, 2);
        assert_eq!(row.candidate_receive_ready_dispatch_count, 2);
        assert_eq!(row.candidate_receive_failed_dispatch_count, 0);
        assert_eq!(row.candidate_row_count, 3);
        assert_eq!(row.next_executor_step, "remote_heap_resolution");
        assert_eq!(row.status, "requires_remote_heap_resolution");
        assert_eq!(row.first_candidate_receive_failure_category, "none");
    }

    #[test]
    fn production_executor_heap_receive_requests_carry_tuple_payload_columns() {
        let dispatch_rows = vec![planned_dispatch(82, 1)];
        let transport_rows = vec![ready_transport_row(82, 1)];
        let receive_results = vec![ready_candidate_receive_result(82, vec![0], 1)];
        let secret_key = remote_conninfo_secret_provider_lookup_key("spire/remote/82")
            .expect("secret key should build");
        std::env::set_var(&secret_key, "host=127.0.0.1 port=1");
        let mut executor =
            SpireRemoteFanoutExecutor::from_libpq_dispatch_rows(7, &dispatch_rows);
        executor
            .apply_transport_probe_rows(&transport_rows)
            .expect("transport rows should apply");
        executor
            .apply_candidate_receive_results(&receive_results)
            .expect("receive rows should apply");
        let requested_columns = vec!["id".to_owned(), "title".to_owned()];

        let requests = executor
            .remote_heap_receive_requests(&[1.0, 0.0], 1, "strict", Some(&requested_columns))
            .expect("heap receive requests should build");
        std::env::remove_var(secret_key);

        assert_eq!(requests.len(), 1);
        assert_eq!(
            requests[0].tuple_payload_columns.as_deref(),
            Some(requested_columns.as_slice())
        );
    }

    #[test]
    fn production_executor_state_preserves_candidate_receive_failure_category() {
        let dispatch_rows = vec![planned_dispatch(2, 1), planned_dispatch(3, 1)];
        let transport_rows = vec![ready_transport_row(2, 1), ready_transport_row(3, 1)];
        let receive_results = vec![
            failed_candidate_receive_result(2, SPIRE_REMOTE_PRODUCTION_CANDIDATE_DECODE_FAILED),
            ready_candidate_receive_result(3, vec![0], 1),
        ];
        let row = remote_search_production_executor_state_summary_from_candidate_receive_results(
            7,
            &dispatch_rows,
            &transport_rows,
            &receive_results,
        )
        .expect("candidate receive summary should succeed");

        assert_eq!(row.candidate_receive_sent_dispatch_count, 2);
        assert_eq!(row.candidate_receive_ready_dispatch_count, 1);
        assert_eq!(row.candidate_receive_failed_dispatch_count, 1);
        assert_eq!(row.candidate_row_count, 1);
        assert_eq!(row.next_executor_step, "compact_candidate_receive");
        assert_eq!(row.status, "remote_candidate_receive_failed");
        assert_eq!(row.first_candidate_receive_failure_category, "candidate_decode_failed");
    }

    #[test]
    fn production_executor_degraded_receive_failure_allows_ready_merge() {
        let dispatch_rows = vec![planned_dispatch(2, 1), planned_dispatch(3, 1)];
        let transport_rows = vec![ready_transport_row(2, 1), ready_transport_row(3, 1)];
        let receive_results = vec![
            failed_candidate_receive_result(2, SPIRE_REMOTE_STATUS_ENDPOINT_IDENTITY_MISMATCH),
            ready_candidate_receive_result(3, vec![30], 1),
        ];
        let row =
            remote_search_production_executor_state_summary_from_candidate_receive_results_with_consistency_mode(
                7,
                &dispatch_rows,
                &transport_rows,
                &receive_results,
                "degraded",
            )
            .expect("degraded candidate receive summary should succeed");
        let merged =
            remote_search_production_compact_merge_from_candidate_receive_results_with_consistency_mode(
                7,
                &dispatch_rows,
                &transport_rows,
                &receive_results,
                Some(10),
                "degraded",
            )
            .expect("degraded candidate receive should merge ready batches");

        assert_eq!(row.candidate_receive_ready_dispatch_count, 1);
        assert_eq!(row.candidate_receive_failed_dispatch_count, 0);
        assert_eq!(row.degraded_skipped_dispatch_count, 1);
        assert_eq!(row.first_degraded_skip_category, "endpoint_identity_mismatch");
        assert_eq!(row.next_executor_step, "remote_heap_resolution");
        assert_eq!(row.status, "degraded_ready");
        assert_eq!(merged.input_count, 1);
        assert_eq!(merged.candidates.len(), 1);
        assert_eq!(merged.candidates[0].node_id, 3);
    }

    #[test]
    fn production_executor_state_rejects_receive_without_ready_transport() {
        let dispatch_rows = vec![planned_dispatch(2, 1)];
        let transport_rows = Vec::new();
        let receive_results = vec![ready_candidate_receive_result(2, vec![0], 1)];
        let error = remote_search_production_executor_state_summary_from_candidate_receive_results(
            7,
            &dispatch_rows,
            &transport_rows,
            &receive_results,
        )
        .expect_err("receive before transport should fail");

        assert!(
            error.contains("does not match a transport-ready dispatch"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn production_executor_compact_merge_uses_ready_candidate_batches() {
        let dispatch_rows = vec![planned_dispatch(2, 1), planned_dispatch(3, 1)];
        let transport_rows = vec![ready_transport_row(2, 1), ready_transport_row(3, 1)];
        let mut node_two = ready_candidate_receive_result(2, vec![10], 1);
        let mut node_three = ready_candidate_receive_result(3, vec![20], 1);
        let shared = storage::SpireVecId::global(b"shared")
            .expect("test global vec_id should build")
            .as_bytes()
            .to_vec();
        node_two
            .batch
            .as_mut()
            .expect("node two batch should exist")
            .candidates[0]
            .vec_id = shared.clone();
        node_two
            .batch
            .as_mut()
            .expect("node two batch should exist")
            .candidates[0]
            .score = 0.4;
        node_three
            .batch
            .as_mut()
            .expect("node three batch should exist")
            .candidates[0]
            .vec_id = shared;
        node_three
            .batch
            .as_mut()
            .expect("node three batch should exist")
            .candidates[0]
            .score = 0.2;

        let merged = remote_search_production_compact_merge_from_candidate_receive_results(
            7,
            &dispatch_rows,
            &transport_rows,
            &[node_two, node_three],
            Some(1),
        )
        .expect("ready candidate batches should merge");

        assert_eq!(merged.input_count, 2);
        assert_eq!(merged.duplicate_vec_id_count, 1);
        assert_eq!(merged.candidates.len(), 1);
        assert_eq!(merged.candidates[0].node_id, 3);
        assert_eq!(merged.candidates[0].score, 0.2);
    }

    #[test]
    fn production_executor_compact_merge_rejects_failed_receive() {
        let dispatch_rows = vec![planned_dispatch(2, 1), planned_dispatch(3, 1)];
        let transport_rows = vec![ready_transport_row(2, 1), ready_transport_row(3, 1)];
        let receive_results = vec![
            failed_candidate_receive_result(2, SPIRE_REMOTE_PRODUCTION_CANDIDATE_DECODE_FAILED),
            ready_candidate_receive_result(3, vec![20], 1),
        ];
        let error = remote_search_production_compact_merge_from_candidate_receive_results(
            7,
            &dispatch_rows,
            &transport_rows,
            &receive_results,
            Some(1),
        )
        .expect_err("failed receive should block compact merge");

        assert!(error.contains("remote_candidate_receive_failed"));
    }

    #[test]
    fn production_executor_local_cancel_clears_ready_candidate_batches() {
        let dispatch_rows = vec![planned_dispatch(2, 1), planned_dispatch(3, 1)];
        let transport_rows = vec![ready_transport_row(2, 1), ready_transport_row(3, 1)];
        let receive_results = vec![
            ready_candidate_receive_result(2, vec![10], 1),
            ready_candidate_receive_result(3, vec![20], 1),
        ];
        let mut executor =
            SpireRemoteFanoutExecutor::from_libpq_dispatch_rows(7, &dispatch_rows);
        executor
            .apply_transport_probe_rows(&transport_rows)
            .expect("transport rows should apply");
        executor
            .apply_candidate_receive_results(&receive_results)
            .expect("receive rows should apply");
        assert_eq!(
            executor
                .ready_candidate_batches()
                .expect("ready batches should exist before cancel")
                .len(),
            2
        );

        executor.apply_local_query_cancel(SPIRE_REMOTE_PRODUCTION_LOCAL_QUERY_CANCELLED);
        let row = executor
            .summary("function_argument", "strict")
            .expect("summary should succeed");
        assert_eq!(row.cancelled_dispatch_count, 2);
        assert_eq!(row.first_cancellation_category, "local_query_cancelled");
        assert_eq!(row.candidate_receive_ready_dispatch_count, 0);
        assert_eq!(row.candidate_row_count, 0);
        assert_eq!(row.next_executor_step, "remote_executor_cancellation");
        assert_eq!(row.status, "remote_executor_cancelled");
        assert!(executor
            .dispatches
            .iter()
            .all(|dispatch| dispatch.candidate_batch.is_none()));

        let error = executor
            .merge_ready_candidate_batches(Some(1))
            .expect_err("cancelled batches should not merge");
        assert!(error.contains("remote_executor_cancelled"));
    }

    #[test]
    fn production_executor_transport_local_cancel_result_cancels_all_dispatches() {
        let dispatch_rows = vec![planned_dispatch(2, 1), planned_dispatch(3, 1)];
        let mut executor =
            SpireRemoteFanoutExecutor::from_libpq_dispatch_rows(7, &dispatch_rows);
        executor
            .apply_transport_probe_rows(&[failed_transport_row(
                2,
                SPIRE_REMOTE_PRODUCTION_LOCAL_QUERY_CANCELLED,
            )])
            .expect("transport local cancel should apply globally");

        let row = executor
            .summary("function_argument", "strict")
            .expect("summary should succeed");
        assert_eq!(row.cancelled_dispatch_count, 2);
        assert_eq!(row.first_cancellation_category, "local_query_cancelled");
        assert_eq!(row.transport_failed_dispatch_count, 0);
        assert_eq!(row.next_executor_step, "remote_executor_cancellation");
        assert_eq!(row.status, "remote_executor_cancelled");
        assert!(executor.dispatches.iter().all(|dispatch| {
            dispatch.state == SpireRemoteProductionDispatchState::Cancelled
                && dispatch.candidate_batch.is_none()
        }));
    }

    #[test]
    fn production_executor_transport_local_statement_timeout_cancels_all_dispatches() {
        let dispatch_rows = vec![planned_dispatch(2, 1), planned_dispatch(3, 1)];
        let mut executor =
            SpireRemoteFanoutExecutor::from_libpq_dispatch_rows(7, &dispatch_rows);
        executor
            .apply_transport_probe_rows(&[failed_transport_row(
                2,
                SPIRE_REMOTE_PRODUCTION_LOCAL_STATEMENT_TIMEOUT,
            )])
            .expect("transport local statement timeout should apply globally");

        let row = executor
            .summary("function_argument", "strict")
            .expect("summary should succeed");
        assert_eq!(row.cancelled_dispatch_count, 2);
        assert_eq!(row.first_cancellation_category, "local_statement_timeout");
        assert_eq!(row.transport_failed_dispatch_count, 0);
        assert_eq!(row.next_executor_step, "remote_executor_cancellation");
        assert_eq!(row.status, "remote_executor_cancelled");
        assert!(executor.dispatches.iter().all(|dispatch| {
            dispatch.state == SpireRemoteProductionDispatchState::Cancelled
                && dispatch.candidate_batch.is_none()
        }));
    }

    #[test]
    fn production_executor_receive_local_cancel_result_cancels_all_dispatches() {
        let dispatch_rows = vec![planned_dispatch(2, 1), planned_dispatch(3, 1)];
        let transport_rows = vec![ready_transport_row(2, 1), ready_transport_row(3, 1)];
        let receive_results = vec![failed_candidate_receive_result(
            2,
            SPIRE_REMOTE_PRODUCTION_LOCAL_QUERY_CANCELLED,
        )];
        let mut executor =
            SpireRemoteFanoutExecutor::from_libpq_dispatch_rows(7, &dispatch_rows);
        executor
            .apply_transport_probe_rows(&transport_rows)
            .expect("transport rows should apply");
        executor
            .apply_candidate_receive_results(&receive_results)
            .expect("receive local cancel should apply globally");

        let row = executor
            .summary("function_argument", "strict")
            .expect("summary should succeed");
        assert_eq!(row.cancelled_dispatch_count, 2);
        assert_eq!(row.first_cancellation_category, "local_query_cancelled");
        assert_eq!(row.candidate_receive_failed_dispatch_count, 0);
        assert_eq!(row.candidate_row_count, 0);
        assert_eq!(row.next_executor_step, "remote_executor_cancellation");
        assert_eq!(row.status, "remote_executor_cancelled");
        assert!(executor.dispatches.iter().all(|dispatch| {
            dispatch.state == SpireRemoteProductionDispatchState::Cancelled
                && dispatch.candidate_batch.is_none()
        }));
    }

    #[test]
    fn production_executor_compact_merge_rejects_every_non_ready_state() {
        let mut blocked_row = planned_dispatch(2, 1);
        blocked_row.dispatch_action = SPIRE_REMOTE_DISPATCH_BLOCKED_ACTION;
        blocked_row.status = SPIRE_REMOTE_STATUS_EXECUTOR_OVERLOAD;
        let blocked_executor =
            SpireRemoteFanoutExecutor::from_libpq_dispatch_rows(7, &[blocked_row]);
        assert!(blocked_executor
            .merge_ready_candidate_batches(None)
            .expect_err("blocked dispatch should not merge")
            .contains("remote_executor_overload"));

        let planned_executor =
            SpireRemoteFanoutExecutor::from_libpq_dispatch_rows(7, &[planned_dispatch(2, 1)]);
        assert!(planned_executor
            .merge_ready_candidate_batches(None)
            .expect_err("planned dispatch should not merge")
            .contains("requires_production_transport_adapter"));

        let mut transport_ready_executor =
            SpireRemoteFanoutExecutor::from_libpq_dispatch_rows(7, &[planned_dispatch(2, 1)]);
        transport_ready_executor
            .apply_transport_probe_rows(&[ready_transport_row(2, 1)])
            .expect("transport row should apply");
        assert!(transport_ready_executor
            .merge_ready_candidate_batches(None)
            .expect_err("transport-ready dispatch should not merge")
            .contains("requires_compact_candidate_receive"));

        let mut transport_failed_executor =
            SpireRemoteFanoutExecutor::from_libpq_dispatch_rows(7, &[planned_dispatch(2, 1)]);
        transport_failed_executor
            .apply_transport_probe_rows(&[failed_transport_row(
                2,
                SPIRE_REMOTE_PRODUCTION_TRANSPORT_CONNECT_FAILED,
            )])
            .expect("failed transport row should apply");
        assert!(transport_failed_executor
            .merge_ready_candidate_batches(None)
            .expect_err("transport-failed dispatch should not merge")
            .contains("remote_transport_failed"));

        let mut receive_failed_executor =
            SpireRemoteFanoutExecutor::from_libpq_dispatch_rows(7, &[planned_dispatch(2, 1)]);
        receive_failed_executor
            .apply_transport_probe_rows(&[ready_transport_row(2, 1)])
            .expect("transport row should apply");
        receive_failed_executor
            .apply_candidate_receive_results(&[failed_candidate_receive_result(
                2,
                SPIRE_REMOTE_PRODUCTION_CANDIDATE_DECODE_FAILED,
            )])
            .expect("failed receive row should apply");
        assert!(receive_failed_executor
            .merge_ready_candidate_batches(None)
            .expect_err("receive-failed dispatch should not merge")
            .contains("remote_candidate_receive_failed"));

        let mut cancelled_executor =
            SpireRemoteFanoutExecutor::from_libpq_dispatch_rows(7, &[planned_dispatch(2, 1)]);
        cancelled_executor.apply_local_query_cancel(SPIRE_REMOTE_PRODUCTION_LOCAL_QUERY_CANCELLED);
        assert!(cancelled_executor
            .merge_ready_candidate_batches(None)
            .expect_err("cancelled dispatch should not merge")
            .contains("remote_executor_cancelled"));
    }

    #[test]
    fn production_executor_compact_receive_requests_use_dispatch_state() {
        let secret_42 =
            remote_conninfo_secret_provider_lookup_key("spire/remote/42").expect("key should build");
        let secret_43 =
            remote_conninfo_secret_provider_lookup_key("spire/remote/43").expect("key should build");
        std::env::set_var(&secret_42, "host=/tmp dbname=postgres");
        std::env::set_var(&secret_43, "host=/tmp dbname=postgres");

        let dispatch_rows = vec![planned_dispatch(42, 2), planned_dispatch(43, 1)];
        let transport_rows = vec![ready_transport_row(42, 1), ready_transport_row(43, 1)];
        let mut executor =
            SpireRemoteFanoutExecutor::from_libpq_dispatch_rows(7, &dispatch_rows);
        executor
            .apply_transport_probe_rows(&transport_rows)
            .expect("transport rows should apply");
        let requests = executor
            .compact_candidate_receive_requests(&[1.0, 0.0], 4, "strict")
            .expect("request build should succeed");

        std::env::remove_var(&secret_42);
        std::env::remove_var(&secret_43);

        assert_eq!(requests.len(), 2);
        assert_eq!(executor.conninfo_secret_lookup_count, 2);
        assert!(executor
            .dispatches
            .iter()
            .all(|dispatch| dispatch.state == SpireRemoteProductionDispatchState::TransportReady));
        let node_42 = requests
            .iter()
            .find(|request| request.node_id == 42)
            .expect("node 42 request should exist");
        assert_eq!(node_42.remote_index_regclass, "ec_spire_remote_42_idx");
        assert_eq!(node_42.remote_index_identity, vec![42]);
        assert_eq!(node_42.selected_pids, vec![0, 1]);
        assert_eq!(node_42.requested_epoch, 7);
        assert_eq!(node_42.query, vec![1.0, 0.0]);
        assert_eq!(node_42.top_k, 4);
        assert_eq!(node_42.consistency_mode, "strict");
    }

    #[test]
    fn production_executor_compact_receive_request_build_isolates_missing_secret() {
        let secret_52 =
            remote_conninfo_secret_provider_lookup_key("spire/remote/52").expect("key should build");
        let secret_53 =
            remote_conninfo_secret_provider_lookup_key("spire/remote/53").expect("key should build");
        std::env::set_var(&secret_52, "host=/tmp dbname=postgres");
        std::env::remove_var(&secret_53);

        let dispatch_rows = vec![planned_dispatch(52, 1), planned_dispatch(53, 1)];
        let transport_rows = vec![ready_transport_row(52, 1), ready_transport_row(53, 1)];
        let mut executor =
            SpireRemoteFanoutExecutor::from_libpq_dispatch_rows(7, &dispatch_rows);
        executor
            .apply_transport_probe_rows(&transport_rows)
            .expect("transport rows should apply");
        let requests = executor
            .compact_candidate_receive_requests(&[1.0, 0.0], 3, "strict")
            .expect("request build should isolate missing secrets");

        std::env::remove_var(&secret_52);

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].node_id, 52);
        assert_eq!(executor.conninfo_secret_lookup_count, 2);
        let row = executor
            .summary("function_argument", "strict")
            .expect("summary should succeed");
        assert_eq!(row.candidate_receive_pending_dispatch_count, 1);
        assert_eq!(row.candidate_receive_failed_dispatch_count, 1);
        assert_eq!(
            row.first_candidate_receive_failure_category,
            "requires_conninfo_secret_resolution"
        );
        assert_eq!(row.next_executor_step, "compact_candidate_receive");
        assert_eq!(row.status, "remote_candidate_receive_failed");
    }

    #[test]
    fn production_executor_degraded_missing_secret_skips_receive_request() {
        let secret_72 =
            remote_conninfo_secret_provider_lookup_key("spire/remote/72").expect("key should build");
        let secret_73 =
            remote_conninfo_secret_provider_lookup_key("spire/remote/73").expect("key should build");
        std::env::set_var(&secret_72, "host=/tmp dbname=postgres");
        std::env::remove_var(&secret_73);

        let dispatch_rows = vec![planned_dispatch(72, 1), planned_dispatch(73, 1)];
        let transport_rows = vec![ready_transport_row(72, 1), ready_transport_row(73, 1)];
        let mut executor =
            SpireRemoteFanoutExecutor::from_libpq_dispatch_rows(7, &dispatch_rows);
        executor
            .apply_transport_probe_rows(&transport_rows)
            .expect("transport rows should apply");
        let requests = executor
            .compact_candidate_receive_requests(&[1.0, 0.0], 3, "degraded")
            .expect("degraded request build should isolate missing secrets");

        std::env::remove_var(&secret_72);

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].node_id, 72);
        assert_eq!(executor.conninfo_secret_lookup_count, 2);
        let row = executor
            .summary("function_argument", "degraded")
            .expect("summary should succeed");
        assert_eq!(row.candidate_receive_pending_dispatch_count, 1);
        assert_eq!(row.candidate_receive_failed_dispatch_count, 0);
        assert_eq!(row.degraded_skipped_dispatch_count, 1);
        assert_eq!(
            row.first_degraded_skip_category,
            "requires_conninfo_secret_resolution"
        );
        assert_eq!(row.next_executor_step, "compact_candidate_receive");
        assert_eq!(row.status, "requires_compact_candidate_receive");
    }

    #[test]
    fn production_executor_compact_receive_run_applies_adapter_failure() {
        let secret_62 =
            remote_conninfo_secret_provider_lookup_key("spire/remote/62").expect("key should build");
        std::env::set_var(&secret_62, "port=not-a-number dbname=postgres");

        let dispatch_rows = vec![planned_dispatch(62, 1)];
        let transport_rows = vec![ready_transport_row(62, 1)];
        let mut executor =
            SpireRemoteFanoutExecutor::from_libpq_dispatch_rows(7, &dispatch_rows);
        executor
            .apply_transport_probe_rows(&transport_rows)
            .expect("transport row should apply");
        executor
            .run_compact_candidate_receive(&[1.0, 0.0], 3, "strict")
            .expect("adapter failure should stay isolated in executor state");

        std::env::remove_var(&secret_62);

        let row = executor
            .summary("function_argument", "strict")
            .expect("summary should succeed");
        assert_eq!(executor.conninfo_secret_lookup_count, 1);
        assert_eq!(row.candidate_receive_pending_dispatch_count, 0);
        assert_eq!(row.candidate_receive_failed_dispatch_count, 1);
        assert_eq!(row.candidate_row_count, 0);
        assert_eq!(
            row.first_candidate_receive_failure_category,
            "conninfo_parse_failed"
        );
        assert_eq!(row.status, "remote_candidate_receive_failed");
    }

    #[test]
    fn prepared_transaction_gid_parser_extracts_reaper_identity() {
        let parts = parse_spire_prepared_gid("ec_spire_insert_123_4_567_890")
            .expect("valid SPIRE prepared gid should parse");
        assert_eq!(parts.index_oid, 123);
        assert_eq!(parts.node_id, 4);
        assert_eq!(parts.served_epoch, 567);
        assert_eq!(parts.xid, 890);

        assert!(parse_spire_prepared_gid("ec_spire_delete_123_4_567_890").is_none());
        assert!(parse_spire_prepared_gid("ec_spire_insert_123_4_567").is_none());
        assert!(parse_spire_prepared_gid("ec_spire_insert_123_4_567_890_extra").is_none());
        assert!(parse_spire_prepared_gid("ec_spire_insert_123_node_567_890").is_none());
    }

    #[test]
    fn prepared_transaction_intent_state_validator_matches_catalog_contract() {
        assert!(coordinator_prepared_xact_intent_state_is_valid(
            "prepare_requested"
        ));
        assert!(coordinator_prepared_xact_intent_state_is_valid("prepare_acked"));
        assert!(coordinator_prepared_xact_intent_state_is_valid("commit_local"));
        assert!(coordinator_prepared_xact_intent_state_is_valid("rollback_local"));
        assert!(!coordinator_prepared_xact_intent_state_is_valid("prepared"));
    }

    #[test]
    fn prepared_transaction_intent_transitions_cannot_bypass_prepare_ack() {
        assert!(coordinator_prepared_xact_intent_transition_is_valid(
            SPIRE_PREPARED_XACT_INTENT_PREPARE_REQUESTED,
            SPIRE_PREPARED_XACT_INTENT_PREPARE_ACKED,
            SpirePreparedXactIntentTransitionContext::RemotePrepareAck,
        ));
        assert!(coordinator_prepared_xact_intent_transition_is_valid(
            SPIRE_PREPARED_XACT_INTENT_PREPARE_ACKED,
            SPIRE_PREPARED_XACT_INTENT_COMMIT_LOCAL,
            SpirePreparedXactIntentTransitionContext::LocalCommitRecorded,
        ));
        assert!(coordinator_prepared_xact_intent_transition_is_valid(
            SPIRE_PREPARED_XACT_INTENT_PREPARE_REQUESTED,
            SPIRE_PREPARED_XACT_INTENT_ROLLBACK_LOCAL,
            SpirePreparedXactIntentTransitionContext::ReaperRollback,
        ));
        assert!(coordinator_prepared_xact_intent_transition_is_valid(
            SPIRE_PREPARED_XACT_INTENT_PREPARE_ACKED,
            SPIRE_PREPARED_XACT_INTENT_ROLLBACK_LOCAL,
            SpirePreparedXactIntentTransitionContext::ReaperRollback,
        ));

        assert!(!coordinator_prepared_xact_intent_transition_is_valid(
            SPIRE_PREPARED_XACT_INTENT_PREPARE_REQUESTED,
            SPIRE_PREPARED_XACT_INTENT_COMMIT_LOCAL,
            SpirePreparedXactIntentTransitionContext::LocalCommitRecorded,
        ));
        assert!(!coordinator_prepared_xact_intent_transition_is_valid(
            SPIRE_PREPARED_XACT_INTENT_PREPARE_REQUESTED,
            SPIRE_PREPARED_XACT_INTENT_ROLLBACK_LOCAL,
            SpirePreparedXactIntentTransitionContext::LocalCommitRecorded,
        ));
        assert!(!coordinator_prepared_xact_intent_transition_is_valid(
            SPIRE_PREPARED_XACT_INTENT_PREPARE_ACKED,
            SPIRE_PREPARED_XACT_INTENT_ROLLBACK_LOCAL,
            SpirePreparedXactIntentTransitionContext::LocalCommitRecorded,
        ));
        assert!(!coordinator_prepared_xact_intent_transition_is_valid(
            SPIRE_PREPARED_XACT_INTENT_COMMIT_LOCAL,
            SPIRE_PREPARED_XACT_INTENT_ROLLBACK_LOCAL,
            SpirePreparedXactIntentTransitionContext::ReaperRollback,
        ));
    }

    #[test]
    fn prepare_transaction_capacity_classifier_matches_postgres_errors() {
        assert!(postgres_prepare_transaction_capacity_failure(
            Some("55000"),
            "prepared transactions are disabled"
        ));
        assert!(postgres_prepare_transaction_capacity_failure(
            Some("55000"),
            "object not in prerequisite state"
        ));
        assert!(postgres_prepare_transaction_capacity_failure(
            Some("53300"),
            "maximum number of prepared transactions reached"
        ));
        assert!(postgres_prepare_transaction_capacity_failure(
            Some("53400"),
            "max_prepared_transactions must be increased"
        ));
        assert!(postgres_prepare_transaction_capacity_failure(
            None,
            "maximum number of prepared transactions reached"
        ));
        assert!(!postgres_prepare_transaction_capacity_failure(
            Some("40P01"),
            "deadlock detected"
        ));
        assert!(!postgres_prepare_transaction_capacity_failure(
            Some("53300"),
            "remaining connection slots are reserved"
        ));
    }

    #[test]
    fn prepared_transaction_registration_warning_handles_unresolved_secret() {
        let missing_secret = "spire/tests/prepared-warning/missing";
        let missing_key = remote_conninfo_secret_provider_lookup_key(missing_secret)
            .expect("missing secret lookup key should build");
        std::env::remove_var(&missing_key);
        let missing_warning =
            remote_prepared_transaction_registration_warning(missing_secret, 2)
                .expect("missing secret should warn");
        assert!(missing_warning.contains("max_prepared_transactions preflight"));
        assert!(missing_warning.contains("conninfo_secret_missing"));

        let empty_secret = "spire/tests/prepared-warning/empty";
        let empty_key = remote_conninfo_secret_provider_lookup_key(empty_secret)
            .expect("empty secret lookup key should build");
        std::env::set_var(&empty_key, "");
        let empty_warning =
            remote_prepared_transaction_registration_warning(empty_secret, 3)
                .expect("empty secret should warn");
        std::env::remove_var(&empty_key);
        assert!(empty_warning.contains("max_prepared_transactions preflight"));
        assert!(empty_warning.contains("conninfo_secret_empty"));
    }
}
