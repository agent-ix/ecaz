pub(crate) unsafe fn remote_search_production_executor_state_summary_row(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    query: Vec<f32>,
    selected_pids: Vec<u64>,
    top_k: usize,
    consistency_mode: &str,
) -> SpireRemoteProductionExecutorStateSummaryRow {
    let result = (|| -> Result<SpireRemoteProductionExecutorStateSummaryRow, String> {
        let dispatch_rows = unsafe {
            remote_search_libpq_dispatch_plan_rows(
                index_relation,
                requested_epoch,
                query,
                selected_pids,
                top_k,
                consistency_mode,
            )
        };
        remote_search_production_executor_state_summary_from_dispatch_rows(
            requested_epoch,
            &dispatch_rows,
            "function_argument",
            consistency_mode,
        )
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

pub(crate) unsafe fn remote_search_production_degraded_skip_report_rows(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    query: Vec<f32>,
    selected_pids: Vec<u64>,
    top_k: usize,
    consistency_mode: &str,
) -> Vec<SpireRemoteProductionDegradedSkipReportRow> {
    let result = (|| -> Result<Vec<SpireRemoteProductionDegradedSkipReportRow>, String> {
        let dispatch_rows = unsafe {
            remote_search_libpq_dispatch_plan_rows(
                index_relation,
                requested_epoch,
                query,
                selected_pids,
                top_k,
                consistency_mode,
            )
        };
        remote_search_production_degraded_skip_report_from_dispatch_rows(
            requested_epoch,
            &dispatch_rows,
            consistency_mode,
        )
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

pub(crate) unsafe fn remote_search_production_executor_session_summary_row(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    query: Vec<f32>,
    selected_pids: Vec<u64>,
    top_k: usize,
) -> SpireRemoteProductionExecutorSessionSummaryRow {
    let consistency_mode = options::current_session_remote_search_consistency_mode_name();
    let summary = unsafe {
        remote_search_production_executor_state_summary_row(
            index_relation,
            requested_epoch,
            query,
            selected_pids,
            top_k,
            consistency_mode,
        )
    };
    SpireRemoteProductionExecutorSessionSummaryRow {
        requested_epoch: summary.requested_epoch,
        consistency_mode_source: "ec_spire.remote_search_consistency_mode",
        consistency_mode,
        dispatch_count: summary.dispatch_count,
        degraded_skipped_dispatch_count: summary.degraded_skipped_dispatch_count,
        first_degraded_skip_category: summary.first_degraded_skip_category,
        next_executor_step: summary.next_executor_step,
        status: summary.status,
        recommendation: summary.recommendation,
    }
}

pub(crate) unsafe fn remote_search_production_scan_handoff_summary_row(
    index_relation: pg_sys::Relation,
    query: Vec<f32>,
    top_k: usize,
) -> SpireRemoteProductionScanHandoffSummaryRow {
    let result = (|| -> Result<SpireRemoteProductionScanHandoffSummaryRow, String> {
        let query_for_scan = scan::SpireScanQuery::new(query.clone())?;
        let consistency_mode = options::current_session_remote_search_consistency_mode_name();
        let root_control = unsafe { page::read_root_control_page(index_relation) };
        if root_control.active_epoch == 0 {
            return Ok(SpireRemoteProductionScanHandoffSummaryRow {
                requested_epoch: 0,
                consistency_mode_source: "ec_spire.remote_search_consistency_mode",
                consistency_mode,
                effective_nprobe: 0,
                selected_pid_count: 0,
                local_pid_count: 0,
                remote_pid_count: 0,
                skipped_pid_count: 0,
                dispatch_count: 0,
                candidate_receive_ready_dispatch_count: 0,
                candidate_receive_failed_dispatch_count: 0,
                degraded_skipped_dispatch_count: 0,
                first_degraded_skip_category: SPIRE_REMOTE_NONE,
                candidate_row_count: 0,
                merged_candidate_count: 0,
                duplicate_vec_id_count: 0,
                final_heap_fetch_status: SPIRE_REMOTE_FINAL_STATUS_NO_BATCHES,
                next_blocker: SPIRE_REMOTE_NONE,
                status: "empty_index",
                recommendation: "publish an active SPIRE epoch before production scan handoff",
            });
        }

        let (epoch_manifest, object_manifest, placement_directory) = unsafe {
            load_relation_epoch_manifests_for_coordinator_fanout(index_relation, root_control)?
        };
        let snapshot = meta::SpirePublishedEpochSnapshot::new(
            &epoch_manifest,
            &object_manifest,
            &placement_directory,
        )?;
        let object_store = unsafe {
            storage::SpireRelationObjectStoreSet::for_index_relation_and_placements(
                index_relation,
                &placement_directory,
                pg_sys::AccessShareLock as pg_sys::LOCKMODE,
            )?
        };
        let relation_options = unsafe { options::relation_options(index_relation) };
        let top_graph_plan = relation_options.top_graph_plan()?;
        let leaf_count = scan::count_scan_plan_routable_leaf_pids(&snapshot, &object_store)?;
        let scan_plan = options::resolve_single_level_scan_plan(leaf_count, relation_options)?;
        let selected_leaf_pids = scan::collect_scan_plan_selected_leaf_pids(
            &snapshot,
            &object_store,
            &query_for_scan,
            scan_plan,
            top_graph_plan,
        )?;
        let selected_pid_count = u64::try_from(selected_leaf_pids.len())
            .map_err(|_| "ec_spire production scan handoff selected PID count exceeds u64")?;

        let execution_summary = unsafe {
            remote_search_execution_summary_row(
                index_relation,
                root_control.active_epoch,
                query.clone(),
                selected_leaf_pids.clone(),
                top_k,
                consistency_mode,
            )
        };
        if top_k == 0 {
            return Ok(SpireRemoteProductionScanHandoffSummaryRow {
                requested_epoch: root_control.active_epoch,
                consistency_mode_source: "ec_spire.remote_search_consistency_mode",
                consistency_mode,
                effective_nprobe: u64::from(scan_plan.nprobe),
                selected_pid_count,
                local_pid_count: execution_summary.local_pid_count,
                remote_pid_count: execution_summary.remote_pid_count,
                skipped_pid_count: execution_summary.skipped_pid_count,
                dispatch_count: 0,
                candidate_receive_ready_dispatch_count: 0,
                candidate_receive_failed_dispatch_count: 0,
                degraded_skipped_dispatch_count: 0,
                first_degraded_skip_category: SPIRE_REMOTE_NONE,
                candidate_row_count: 0,
                merged_candidate_count: 0,
                duplicate_vec_id_count: 0,
                final_heap_fetch_status: SPIRE_REMOTE_FINAL_STATUS_NO_BATCHES,
                next_blocker: SPIRE_REMOTE_NONE,
                status: SPIRE_REMOTE_STATUS_EMPTY_TOP_K,
                recommendation: SPIRE_REMOTE_NONE,
            });
        }

        let dispatch_rows = unsafe {
            remote_search_libpq_dispatch_plan_rows(
                index_relation,
                root_control.active_epoch,
                query.clone(),
                selected_leaf_pids,
                top_k,
                consistency_mode,
            )
        };
        let mut executor =
            SpireRemoteFanoutExecutor::from_libpq_dispatch_rows(root_control.active_epoch, &dispatch_rows);
        executor.mark_planned_dispatches_candidate_receive_ready();
        executor.run_compact_candidate_receive(&query, top_k, consistency_mode)?;
        let summary = executor.summary(
            "ec_spire.remote_search_consistency_mode",
            consistency_mode_name(parse_remote_search_consistency_mode(consistency_mode)?),
        )?;
        let should_merge = matches!(
            summary.status,
            SPIRE_REMOTE_FINAL_STATUS_REQUIRES_REMOTE_HEAP
                | SPIRE_REMOTE_STATUS_DEGRADED_READY
                | SPIRE_REMOTE_STATUS_DEGRADED_SKIPPED
                | SPIRE_REMOTE_STATUS_READY
        );
        let merge = if should_merge {
            executor.merge_ready_candidate_batches(Some(top_k))?
        } else {
            SpireRemoteSearchMergeResult {
                candidates: Vec::new(),
                input_count: 0,
                duplicate_vec_id_count: 0,
            }
        };
        let merged_candidate_count = u64::try_from(merge.candidates.len())
            .map_err(|_| "ec_spire production scan handoff merge count exceeds u64")?;
        let final_heap_fetch_status = if matches!(
            summary.status,
            SPIRE_REMOTE_FINAL_STATUS_REQUIRES_REMOTE_HEAP | SPIRE_REMOTE_STATUS_DEGRADED_READY
        ) {
            SPIRE_REMOTE_FINAL_STATUS_REQUIRES_REMOTE_HEAP
        } else if summary.status == SPIRE_REMOTE_STATUS_READY && execution_summary.remote_pid_count == 0
        {
            SPIRE_REMOTE_FINAL_STATUS_LOCAL_READY
        } else if summary.status == SPIRE_REMOTE_STATUS_READY
            || summary.status == SPIRE_REMOTE_STATUS_DEGRADED_SKIPPED
        {
            SPIRE_REMOTE_FINAL_STATUS_NO_BATCHES
        } else {
            SPIRE_REMOTE_FINAL_STATUS_BLOCKED
        };
        let next_blocker = if final_heap_fetch_status == SPIRE_REMOTE_FINAL_STATUS_REQUIRES_REMOTE_HEAP
        {
            SPIRE_REMOTE_EXECUTOR_STEP_REMOTE_HEAP_RESOLUTION
        } else if matches!(
            summary.status,
            SPIRE_REMOTE_STATUS_READY | SPIRE_REMOTE_STATUS_DEGRADED_SKIPPED
        ) {
            SPIRE_REMOTE_NONE
        } else {
            summary.next_executor_step
        };
        let (status, recommendation) =
            if final_heap_fetch_status == SPIRE_REMOTE_FINAL_STATUS_REQUIRES_REMOTE_HEAP {
                (
                    SPIRE_REMOTE_FINAL_STATUS_REQUIRES_REMOTE_HEAP,
                    "add origin-node row locator resolution before returning remote heap rows",
                )
            } else {
                (summary.status, summary.recommendation)
            };

        Ok(SpireRemoteProductionScanHandoffSummaryRow {
            requested_epoch: root_control.active_epoch,
            consistency_mode_source: "ec_spire.remote_search_consistency_mode",
            consistency_mode,
            effective_nprobe: u64::from(scan_plan.nprobe),
            selected_pid_count,
            local_pid_count: execution_summary.local_pid_count,
            remote_pid_count: execution_summary.remote_pid_count,
            skipped_pid_count: execution_summary.skipped_pid_count,
            dispatch_count: summary.dispatch_count,
            candidate_receive_ready_dispatch_count: summary.candidate_receive_ready_dispatch_count,
            candidate_receive_failed_dispatch_count: summary
                .candidate_receive_failed_dispatch_count,
            degraded_skipped_dispatch_count: summary.degraded_skipped_dispatch_count,
            first_degraded_skip_category: summary.first_degraded_skip_category,
            candidate_row_count: summary.candidate_row_count,
            merged_candidate_count,
            duplicate_vec_id_count: merge.duplicate_vec_id_count,
            final_heap_fetch_status,
            next_blocker,
            status,
            recommendation,
        })
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

fn production_scan_output_from_heap_candidate(
    candidate: &SpireRemoteSearchLocalHeapCandidateRow,
) -> SpireRemoteProductionScanOutputRow {
    SpireRemoteProductionScanOutputRow {
        requested_epoch: candidate.requested_epoch,
        served_epoch: candidate.served_epoch,
        node_id: candidate.node_id,
        heap_block: candidate.heap_block,
        heap_offset: candidate.heap_offset,
        score: candidate.score,
        heap_lookup_owner: candidate.heap_lookup_owner,
        vec_id: candidate.vec_id.clone(),
        row_locator: candidate.row_locator.clone(),
        tuple_payload_json: candidate.tuple_payload_json.clone(),
        typed_tuple_payload: candidate.typed_tuple_payload.clone(),
        tuple_payload_missing: candidate.tuple_payload_missing,
    }
}

fn production_scan_outputs_from_heap_candidates(
    candidates: &[SpireRemoteSearchLocalHeapCandidateRow],
) -> Vec<SpireRemoteProductionScanOutputRow> {
    candidates
        .iter()
        .map(production_scan_output_from_heap_candidate)
        .collect()
}

fn production_scan_output_is_local_heap_tid(output: &SpireRemoteProductionScanOutputRow) -> bool {
    matches!(
        output.heap_lookup_owner,
        SPIRE_REMOTE_LOCAL_HEAP_RESOLUTION
    )
}

fn production_scan_am_delivery_summary(
    summary: &SpireRemoteProductionScanHeapResolutionSummaryRow,
    outputs: &[SpireRemoteProductionScanOutputRow],
) -> Result<SpireRemoteProductionScanAmDeliverySummaryRow, String> {
    let output_count = u64::try_from(outputs.len())
        .map_err(|_| "ec_spire production scan output count exceeds u64")?;
    let local_heap_tid_output_count = u64::try_from(
        outputs
            .iter()
            .filter(|output| production_scan_output_is_local_heap_tid(output))
            .count(),
    )
    .map_err(|_| "ec_spire production scan local output count exceeds u64")?;
    let remote_origin_output_count = output_count
        .checked_sub(local_heap_tid_output_count)
        .ok_or_else(|| "ec_spire production scan output count underflow".to_owned())?;

    let (am_deliverable_output_count, status, next_blocker, recommendation) =
        if summary.next_blocker != SPIRE_REMOTE_NONE {
            (
                0,
                summary.status,
                summary.next_blocker,
                summary.recommendation,
            )
        } else if remote_origin_output_count > 0 {
            (
                0,
                SPIRE_REMOTE_FINAL_STATUS_REQUIRES_CUSTOM_SCAN_TUPLE_DELIVERY,
                SPIRE_REMOTE_EXECUTOR_STEP_CUSTOM_SCAN_TUPLE_DELIVERY,
                "route remote-origin rows through EcSpireDistributedScan tuple delivery instead of the index AM cursor",
            )
        } else if local_heap_tid_output_count > 0 {
            (
                local_heap_tid_output_count,
                SPIRE_REMOTE_STATUS_READY,
                SPIRE_REMOTE_NONE,
                SPIRE_REMOTE_NONE,
            )
        } else {
            (
                0,
                summary.status,
                summary.next_blocker,
                summary.recommendation,
            )
        };

    Ok(SpireRemoteProductionScanAmDeliverySummaryRow {
        requested_epoch: summary.requested_epoch,
        output_count,
        local_heap_tid_output_count,
        remote_origin_output_count,
        am_deliverable_output_count,
        status,
        next_blocker,
        recommendation,
    })
}

fn production_scan_result_stream(
    summary: SpireRemoteProductionScanHeapResolutionSummaryRow,
    outputs: Vec<SpireRemoteProductionScanOutputRow>,
) -> Result<SpireRemoteProductionScanResultStream, String> {
    let am_delivery = production_scan_am_delivery_summary(&summary, &outputs)?;
    Ok(SpireRemoteProductionScanResultStream {
        summary,
        am_delivery,
        outputs,
    })
}

#[derive(Debug, Clone, PartialEq)]
struct SpireRemoteProductionProfiledScanResult {
    stream: SpireRemoteProductionScanResultStream,
    metrics: SpireRemoteProductionReadMetrics,
}

fn production_profiled_scan_result_stream(
    summary: SpireRemoteProductionScanHeapResolutionSummaryRow,
    outputs: Vec<SpireRemoteProductionScanOutputRow>,
    metrics: SpireRemoteProductionReadMetrics,
) -> Result<SpireRemoteProductionProfiledScanResult, String> {
    Ok(SpireRemoteProductionProfiledScanResult {
        stream: production_scan_result_stream(summary, outputs)?,
        metrics,
    })
}

fn production_read_profile_row(
    summary: &SpireRemoteProductionScanHeapResolutionSummaryRow,
    metrics: &SpireRemoteProductionReadMetrics,
) -> SpireRemoteProductionReadProfileRow {
    SpireRemoteProductionReadProfileRow {
        requested_epoch: summary.requested_epoch,
        consistency_mode_source: summary.consistency_mode_source,
        consistency_mode: summary.consistency_mode,
        effective_nprobe: summary.effective_nprobe,
        selected_pid_count: summary.selected_pid_count,
        local_pid_count: summary.local_pid_count,
        remote_pid_count: summary.remote_pid_count,
        skipped_pid_count: summary.skipped_pid_count,
        dispatch_count: summary.dispatch_count,
        compact_candidate_count: summary.compact_candidate_count,
        remote_heap_ready_dispatch_count: summary.remote_heap_ready_dispatch_count,
        remote_heap_failed_dispatch_count: summary.remote_heap_failed_dispatch_count,
        remote_heap_candidate_count: summary.remote_heap_candidate_count,
        local_heap_candidate_count: summary.local_heap_candidate_count,
        returned_candidate_count: summary.returned_candidate_count,
        result_source: summary.result_source,
        final_heap_fetch_status: summary.final_heap_fetch_status,
        next_blocker: summary.next_blocker,
        status: summary.status,
        recommendation: summary.recommendation,
        planning_elapsed_ms: metrics.planning_elapsed_ms,
        fingerprint_guard_elapsed_ms: metrics.fingerprint_guard_elapsed_ms,
        conninfo_secret_lookup_elapsed_ms: metrics.conninfo_secret_lookup_elapsed_ms,
        connect_elapsed_ms: metrics.connect_elapsed_ms,
        statement_timeout_setup_elapsed_ms: metrics.statement_timeout_setup_elapsed_ms,
        regclass_probe_elapsed_ms: metrics.regclass_probe_elapsed_ms,
        endpoint_identity_elapsed_ms: metrics.endpoint_identity_elapsed_ms,
        candidate_receive_elapsed_ms: metrics.candidate_receive_elapsed_ms,
        heap_receive_elapsed_ms: metrics.heap_receive_elapsed_ms,
        payload_decode_elapsed_ms: metrics.payload_decode_elapsed_ms,
        merge_elapsed_ms: metrics.merge_elapsed_ms,
        total_elapsed_ms: metrics.total_elapsed_ms,
        conninfo_secret_lookup_count: metrics.conninfo_secret_lookup_count,
        socket_open_count: metrics.socket_open_count,
        tls_disable_count: metrics.tls_disable_count,
        tls_require_count: metrics.tls_require_count,
        tls_verify_full_count: metrics.tls_verify_full_count,
        statement_timeout_setup_count: metrics.statement_timeout_setup_count,
        regclass_probe_count: metrics.regclass_probe_count,
        endpoint_identity_query_count: metrics.endpoint_identity_query_count,
        candidate_receive_query_count: metrics.candidate_receive_query_count,
        heap_receive_query_count: metrics.heap_receive_query_count,
        payload_decode_row_count: metrics.payload_decode_row_count,
        payload_decode_bytes: metrics.payload_decode_bytes,
        merge_input_count: metrics.merge_input_count,
        merge_duplicate_vec_id_count: metrics.merge_duplicate_vec_id_count,
        merge_output_count: metrics.merge_output_count,
        strict_fail_count: metrics.strict_fail_count,
        remote_timeout_count: metrics.remote_timeout_count,
        remote_cancel_count: metrics.remote_cancel_count,
        degraded_skipped_dispatch_count: metrics.degraded_skipped_dispatch_count,
    }
}

unsafe fn remote_search_production_scan_heap_resolution_result_stream_impl(
    index_relation: pg_sys::Relation,
    query: Vec<f32>,
    top_k_override: Option<usize>,
    tuple_payload_columns: Option<&[String]>,
) -> Result<SpireRemoteProductionProfiledScanResult, String> {
        let total_start = std::time::Instant::now();
        let planning_start = std::time::Instant::now();
        let mut metrics = SpireRemoteProductionReadMetrics::default();
        let query_for_scan = scan::SpireScanQuery::new(query.clone())?;
        let consistency_mode = options::current_session_remote_search_consistency_mode_name();
        let root_control = unsafe { page::read_root_control_page(index_relation) };
        if root_control.active_epoch == 0 {
            add_profile_elapsed(&mut metrics.planning_elapsed_ms, planning_start);
            add_profile_elapsed(&mut metrics.total_elapsed_ms, total_start);
            return production_profiled_scan_result_stream(
                SpireRemoteProductionScanHeapResolutionSummaryRow {
                    requested_epoch: 0,
                    consistency_mode_source: "ec_spire.remote_search_consistency_mode",
                    consistency_mode,
                    effective_nprobe: 0,
                    selected_pid_count: 0,
                    local_pid_count: 0,
                    remote_pid_count: 0,
                    skipped_pid_count: 0,
                    dispatch_count: 0,
                    compact_candidate_count: 0,
                    remote_heap_ready_dispatch_count: 0,
                    remote_heap_failed_dispatch_count: 0,
                    remote_heap_candidate_count: 0,
                    local_heap_candidate_count: 0,
                    returned_candidate_count: 0,
                    result_source: SPIRE_REMOTE_NONE,
                    final_heap_fetch_status: SPIRE_REMOTE_FINAL_STATUS_NO_BATCHES,
                    next_blocker: SPIRE_REMOTE_NONE,
                    status: "empty_index",
                    recommendation: "publish an active SPIRE epoch before production scan heap resolution",
                },
                Vec::new(),
                metrics,
            );
        }

        let (epoch_manifest, object_manifest, placement_directory) = unsafe {
            load_relation_epoch_manifests_for_coordinator_fanout(index_relation, root_control)?
        };
        let snapshot = meta::SpirePublishedEpochSnapshot::new(
            &epoch_manifest,
            &object_manifest,
            &placement_directory,
        )?;
        let object_store = unsafe {
            storage::SpireRelationObjectStoreSet::for_index_relation_and_placements(
                index_relation,
                &placement_directory,
                pg_sys::AccessShareLock as pg_sys::LOCKMODE,
            )?
        };
        let relation_options = unsafe { options::relation_options(index_relation) };
        let top_graph_plan = relation_options.top_graph_plan()?;
        let leaf_count = scan::count_scan_plan_routable_leaf_pids(&snapshot, &object_store)?;
        let scan_plan = options::resolve_single_level_scan_plan(leaf_count, relation_options)?;
        let top_k = match top_k_override {
            Some(top_k) => top_k,
            None => scan_plan
                .candidate_limit
                .ok_or_else(|| "ec_spire production AM scan candidate limit is unavailable".to_owned())?,
        };
        let selected_leaf_pids = scan::collect_scan_plan_selected_leaf_pids(
            &snapshot,
            &object_store,
            &query_for_scan,
            scan_plan,
            top_graph_plan,
        )?;
        let selected_pid_count = u64::try_from(selected_leaf_pids.len())
            .map_err(|_| "ec_spire production scan heap selected PID count exceeds u64")?;
        let execution_summary = unsafe {
            remote_search_execution_summary_row(
                index_relation,
                root_control.active_epoch,
                query.clone(),
                selected_leaf_pids.clone(),
                top_k,
                consistency_mode,
            )
        };

        if top_k == 0 {
            add_profile_elapsed(&mut metrics.planning_elapsed_ms, planning_start);
            add_profile_elapsed(&mut metrics.total_elapsed_ms, total_start);
            return production_profiled_scan_result_stream(
                SpireRemoteProductionScanHeapResolutionSummaryRow {
                    requested_epoch: root_control.active_epoch,
                    consistency_mode_source: "ec_spire.remote_search_consistency_mode",
                    consistency_mode,
                    effective_nprobe: u64::from(scan_plan.nprobe),
                    selected_pid_count,
                    local_pid_count: execution_summary.local_pid_count,
                    remote_pid_count: execution_summary.remote_pid_count,
                    skipped_pid_count: execution_summary.skipped_pid_count,
                    dispatch_count: 0,
                    compact_candidate_count: 0,
                    remote_heap_ready_dispatch_count: 0,
                    remote_heap_failed_dispatch_count: 0,
                    remote_heap_candidate_count: 0,
                    local_heap_candidate_count: 0,
                    returned_candidate_count: 0,
                    result_source: SPIRE_REMOTE_NONE,
                    final_heap_fetch_status: SPIRE_REMOTE_FINAL_STATUS_NO_BATCHES,
                    next_blocker: SPIRE_REMOTE_NONE,
                    status: SPIRE_REMOTE_STATUS_EMPTY_TOP_K,
                    recommendation: SPIRE_REMOTE_NONE,
                },
                Vec::new(),
                metrics,
            );
        }

        let local_heap_rows = if execution_summary.local_pid_count > 0 {
            unsafe {
                remote_search_local_heap_candidate_rows_for_result_summary(
                    index_relation,
                    root_control.active_epoch,
                    query.clone(),
                    selected_leaf_pids.clone(),
                    top_k,
                    consistency_mode,
                )
            }
        } else {
            Vec::new()
        };
        let local_heap_candidate_count = u64::try_from(
            local_heap_rows
                .iter()
                .filter(|row| row.status == SPIRE_REMOTE_STATUS_READY)
                .count(),
        )
        .map_err(|_| "ec_spire production scan heap local candidate count exceeds u64")?;

        let fingerprint_start = std::time::Instant::now();
        let dispatch_rows = unsafe {
            remote_search_libpq_dispatch_plan_rows(
                index_relation,
                root_control.active_epoch,
                query.clone(),
                selected_leaf_pids,
                top_k,
                consistency_mode,
            )
        };
        add_profile_elapsed(&mut metrics.fingerprint_guard_elapsed_ms, fingerprint_start);
        add_profile_elapsed(&mut metrics.planning_elapsed_ms, planning_start);
        let dispatch_count = u64::try_from(dispatch_rows.len())
            .map_err(|_| "ec_spire production scan heap dispatch count exceeds u64")?;
        let mut executor =
            SpireRemoteFanoutExecutor::from_libpq_dispatch_rows(root_control.active_epoch, &dispatch_rows);
        executor.mark_planned_dispatches_candidate_receive_ready();
        executor.run_candidate_and_heap_receive_reusing_sessions(
            &query,
            top_k,
            consistency_mode,
            tuple_payload_columns,
            &mut metrics,
        )?;
        let executor_summary = executor.summary(
            "ec_spire.remote_search_consistency_mode",
            consistency_mode_name(parse_remote_search_consistency_mode(consistency_mode)?),
        );
        let executor_summary = executor_summary?;
        let (remote_heap_ready_dispatch_count, remote_heap_failed_dispatch_count, remote_heap_candidate_count) =
            executor.remote_heap_resolution_counts()?;
        let remote_heap_rows = if remote_heap_failed_dispatch_count == 0 {
            executor.ready_remote_heap_candidate_rows()?
        } else {
            Vec::new()
        };

        let mut heap_rows = local_heap_rows
            .into_iter()
            .filter(|row| row.status == SPIRE_REMOTE_STATUS_READY)
            .collect::<Vec<_>>();
        heap_rows.extend(remote_heap_rows);
        metrics.merge_input_count = u64::try_from(heap_rows.len()).unwrap_or(u64::MAX);
        let merge_start = std::time::Instant::now();
        let merge_result = merge_remote_search_heap_candidates_for_result_with_stats(heap_rows, top_k)?;
        add_profile_elapsed(&mut metrics.merge_elapsed_ms, merge_start);
        metrics.merge_duplicate_vec_id_count = merge_result.duplicate_vec_id_count;
        let merged = merge_result.candidates;
        metrics.merge_output_count = u64::try_from(merged.len()).unwrap_or(u64::MAX);
        let returned_candidate_count = u64::try_from(merged.len())
            .map_err(|_| "ec_spire production scan heap returned candidate count exceeds u64")?;

        let result_source = if remote_heap_candidate_count > 0 {
            SPIRE_REMOTE_RESULT_SOURCE_REMOTE_HEAP_CANDIDATES
        } else if local_heap_candidate_count > 0 {
            SPIRE_REMOTE_RESULT_SOURCE_LOCAL_HEAP_CANDIDATES
        } else if executor_summary.next_executor_step != SPIRE_REMOTE_NONE {
            SPIRE_REMOTE_RESULT_SOURCE_BLOCKED
        } else {
            SPIRE_REMOTE_NONE
        };
        let final_heap_fetch_status = if remote_heap_failed_dispatch_count > 0 {
            SPIRE_REMOTE_FINAL_STATUS_BLOCKED
        } else if remote_heap_candidate_count > 0 {
            SPIRE_REMOTE_FINAL_STATUS_REMOTE_READY
        } else if local_heap_candidate_count > 0 {
            SPIRE_REMOTE_FINAL_STATUS_LOCAL_READY
        } else {
            SPIRE_REMOTE_FINAL_STATUS_NO_BATCHES
        };
        let (next_blocker, status, recommendation) = if remote_heap_failed_dispatch_count > 0 {
            (
                SPIRE_REMOTE_EXECUTOR_STEP_REMOTE_HEAP_RESOLUTION,
                SPIRE_REMOTE_PRODUCTION_REMOTE_HEAP_RESOLUTION_FAILED,
                "inspect production remote heap failure category before final row delivery",
            )
        } else if executor_summary.next_executor_step != SPIRE_REMOTE_NONE {
            (
                executor_summary.next_executor_step,
                executor_summary.status,
                executor_summary.recommendation,
            )
        } else if returned_candidate_count > 0 {
            (
                SPIRE_REMOTE_NONE,
                if execution_summary.skipped_pid_count > 0 {
                    SPIRE_REMOTE_STATUS_DEGRADED_READY
                } else {
                    SPIRE_REMOTE_STATUS_READY
                },
                SPIRE_REMOTE_NONE,
            )
        } else {
            (
                SPIRE_REMOTE_NONE,
                SPIRE_REMOTE_FINAL_STATUS_NO_BATCHES,
                "inspect local and remote heap visibility for the selected candidates",
            )
        };

        metrics.degraded_skipped_dispatch_count = executor_summary.degraded_skipped_dispatch_count;
        add_profile_elapsed(&mut metrics.total_elapsed_ms, total_start);
        production_profiled_scan_result_stream(
            SpireRemoteProductionScanHeapResolutionSummaryRow {
                requested_epoch: root_control.active_epoch,
                consistency_mode_source: "ec_spire.remote_search_consistency_mode",
                consistency_mode,
                effective_nprobe: u64::from(scan_plan.nprobe),
                selected_pid_count,
                local_pid_count: execution_summary.local_pid_count,
                remote_pid_count: execution_summary.remote_pid_count,
                skipped_pid_count: execution_summary.skipped_pid_count,
                dispatch_count,
                compact_candidate_count: executor_summary.candidate_row_count,
                remote_heap_ready_dispatch_count,
                remote_heap_failed_dispatch_count,
                remote_heap_candidate_count,
                local_heap_candidate_count,
                returned_candidate_count,
                result_source,
                final_heap_fetch_status,
                next_blocker,
                status,
                recommendation,
            },
            production_scan_outputs_from_heap_candidates(&merged),
            metrics,
        )
}

pub(crate) unsafe fn remote_search_production_scan_heap_resolution_result_stream(
    index_relation: pg_sys::Relation,
    query: Vec<f32>,
    top_k: usize,
) -> SpireRemoteProductionScanResultStream {
    let result = unsafe {
        remote_search_production_scan_heap_resolution_result_stream_impl(
            index_relation,
            query,
            Some(top_k),
            None,
        )
    };
    result.unwrap_or_else(|e| pgrx::error!("{e}")).stream
}

pub(crate) unsafe fn remote_search_production_scan_tuple_payload_result_stream(
    index_relation: pg_sys::Relation,
    query: Vec<f32>,
    top_k: usize,
    tuple_payload_columns: &[String],
) -> SpireRemoteProductionScanResultStream {
    let result = unsafe {
        remote_search_production_scan_heap_resolution_result_stream_impl(
            index_relation,
            query,
            Some(top_k),
            Some(tuple_payload_columns),
        )
    };
    result.unwrap_or_else(|e| pgrx::error!("{e}")).stream
}

pub(crate) unsafe fn remote_search_production_scan_heap_resolution_am_result_stream(
    index_relation: pg_sys::Relation,
    heap_relation: pg_sys::Relation,
    snapshot: pg_sys::Snapshot,
    query: Vec<f32>,
) -> SpireRemoteProductionScanResultStream {
    let result = (|| -> Result<SpireRemoteProductionScanResultStream, String> {
        let stream = unsafe {
            remote_search_production_scan_heap_resolution_result_stream_impl(
                index_relation,
                query,
                None,
                None,
            )?
            .stream
        };
        let _ = (heap_relation, snapshot);
        Ok(stream)
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

pub(crate) unsafe fn remote_search_production_scan_heap_resolution_summary_row(
    index_relation: pg_sys::Relation,
    query: Vec<f32>,
    top_k: usize,
) -> SpireRemoteProductionScanHeapResolutionSummaryRow {
    unsafe {
        remote_search_production_scan_heap_resolution_result_stream(index_relation, query, top_k)
            .summary
    }
}

pub(crate) unsafe fn remote_search_production_read_profile_row(
    index_relation: pg_sys::Relation,
    query: Vec<f32>,
    top_k: usize,
) -> SpireRemoteProductionReadProfileRow {
    let result = unsafe {
        remote_search_production_scan_heap_resolution_result_stream_impl(
            index_relation,
            query,
            Some(top_k),
            None,
        )
    };
    let profiled = result.unwrap_or_else(|e| pgrx::error!("{e}"));
    production_read_profile_row(&profiled.stream.summary, &profiled.metrics)
}

pub(crate) unsafe fn remote_search_operator_diagnostics_row(
    index_relation: pg_sys::Relation,
    query: Vec<f32>,
    top_k: usize,
) -> SpireRemoteSearchOperatorDiagnosticsRow {
    let result = (|| -> Result<SpireRemoteSearchOperatorDiagnosticsRow, String> {
        let capability = unsafe { remote_node_capability_summary(index_relation) };
        let remote_snapshots = unsafe { remote_node_snapshot(index_relation) }
            .into_iter()
            .filter(|row| row.node_id != meta::SPIRE_LOCAL_NODE_ID)
            .collect::<Vec<_>>();
        let min_remote_last_served_epoch = remote_snapshots
            .iter()
            .map(|row| row.last_served_epoch)
            .min()
            .unwrap_or(0);
        let max_remote_last_served_epoch = remote_snapshots
            .iter()
            .map(|row| row.last_served_epoch)
            .max()
            .unwrap_or(0);
        let ready_remote_node_count = u64::try_from(
            remote_snapshots
                .iter()
                .filter(|row| row.status == SPIRE_REMOTE_STATUS_READY)
                .count(),
        )
        .map_err(|_| "ec_spire operator diagnostics ready remote node count exceeds u64")?;
        let blocked_remote_node_count = u64::try_from(remote_snapshots.len())
            .map_err(|_| "ec_spire operator diagnostics remote node count exceeds u64")?
            .checked_sub(ready_remote_node_count)
            .ok_or_else(|| "ec_spire operator diagnostics remote node count underflow".to_owned())?;
        let summary =
            unsafe { remote_search_production_scan_handoff_summary_row(index_relation, query, top_k) };

        let (next_blocker, status, recommendation) =
            if capability.remote_node_count > 0 && capability.status != SPIRE_REMOTE_STATUS_READY {
                (
                    "remote_node_capability",
                    capability.status,
                    capability.recommendation,
                )
            } else if summary.next_blocker != SPIRE_REMOTE_NONE {
                (summary.next_blocker, summary.status, summary.recommendation)
            } else {
                (summary.next_blocker, summary.status, summary.recommendation)
            };

        Ok(SpireRemoteSearchOperatorDiagnosticsRow {
            active_epoch: summary.requested_epoch,
            consistency_mode: summary.consistency_mode,
            remote_node_count: capability.remote_node_count,
            ready_remote_node_count,
            blocked_remote_node_count,
            min_remote_last_served_epoch,
            max_remote_last_served_epoch,
            remote_readiness_status: capability.status,
            effective_nprobe: summary.effective_nprobe,
            selected_pid_count: summary.selected_pid_count,
            local_pid_count: summary.local_pid_count,
            remote_pid_count: summary.remote_pid_count,
            skipped_pid_count: summary.skipped_pid_count,
            remote_fanout_count: summary.dispatch_count,
            candidate_batch_count: summary.dispatch_count,
            candidate_row_count: summary.candidate_row_count,
            remote_heap_ready_dispatch_count: 0,
            remote_heap_failed_dispatch_count: 0,
            remote_heap_candidate_count: 0,
            local_heap_candidate_count: 0,
            returned_candidate_count: 0,
            result_source: SPIRE_REMOTE_RESULT_SOURCE_BLOCKED,
            final_heap_fetch_status: summary.final_heap_fetch_status,
            merge_status: summary.status,
            am_delivery_status: summary.status,
            am_deliverable_output_count: 0,
            remote_origin_output_count: 0,
            next_blocker,
            status,
            recommendation,
        })
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}
