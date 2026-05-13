fn failed_production_transport_probe_row(
    node_id: u32,
    batch_start: std::time::Instant,
    request_start: std::time::Instant,
    failure_category: &'static str,
) -> SpireRemoteProductionTransportProbeRow {
    SpireRemoteProductionTransportProbeRow {
        node_id,
        started_after_ms: elapsed_millis_u64(batch_start),
        completed_after_ms: elapsed_millis_u64(batch_start),
        elapsed_ms: elapsed_millis_u64(request_start),
        row_count: 0,
        status: SPIRE_REMOTE_STATUS_PRODUCTION_TRANSPORT_FAILED,
        failure_category,
    }
}

fn elapsed_millis_u64(start: std::time::Instant) -> u64 {
    u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX)
}

fn failed_production_candidate_receive_result(
    node_id: u32,
    batch_start: std::time::Instant,
    request_start: std::time::Instant,
    failure_category: &'static str,
) -> SpireRemoteProductionCandidateReceiveResult {
    SpireRemoteProductionCandidateReceiveResult {
        node_id,
        started_after_ms: elapsed_millis_u64(batch_start),
        completed_after_ms: elapsed_millis_u64(batch_start),
        elapsed_ms: elapsed_millis_u64(request_start),
        candidate_count: 0,
        status: SPIRE_REMOTE_STATUS_CANDIDATE_RECEIVE_FAILED,
        failure_category,
        batch: None,
    }
}

fn failed_production_heap_receive_result(
    node_id: u32,
    batch_start: std::time::Instant,
    request_start: std::time::Instant,
    failure_category: &'static str,
) -> SpireRemoteProductionHeapReceiveResult {
    SpireRemoteProductionHeapReceiveResult {
        node_id,
        started_after_ms: elapsed_millis_u64(batch_start),
        completed_after_ms: elapsed_millis_u64(batch_start),
        elapsed_ms: elapsed_millis_u64(request_start),
        candidate_count: 0,
        status: SPIRE_REMOTE_PRODUCTION_REMOTE_HEAP_RESOLUTION_FAILED,
        failure_category,
        candidates: Vec::new(),
    }
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) fn remote_search_production_transport_probe_for_test(
    requests: Vec<SpireRemoteProductionTransportProbeRequest>,
) -> Vec<SpireRemoteProductionTransportProbeRow> {
    SpireRemoteProductionTransportAdapter::run_probe_requests(requests)
        .unwrap_or_else(|e| pgrx::error!("{e}"))
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) fn remote_search_production_transport_probe_summary_for_test(
    requests: Vec<SpireRemoteProductionTransportProbeRequest>,
    consistency_mode: &str,
) -> SpireRemoteProductionExecutorStateSummaryRow {
    let result = (|| -> Result<SpireRemoteProductionExecutorStateSummaryRow, String> {
        let consistency_mode_name =
            consistency_mode_name(parse_remote_search_consistency_mode(consistency_mode)?);
        let dispatch_rows = requests
            .iter()
            .map(|request| SpireRemoteSearchLibpqDispatchPlanRow {
                requested_epoch: 1,
                node_id: request.node_id,
                selected_pids: vec![u64::from(request.node_id)],
                pid_count: 1,
                query_dimension: 2,
                top_k: 1,
                consistency_mode: consistency_mode_name,
                sql_template: "SELECT 1",
                parameter_count: 0,
                result_column_count: 1,
                conninfo_secret_name: format!("tests/node/{}", request.node_id),
                remote_index_regclass: "tests.ec_spire_transport_probe_idx".to_owned(),
                descriptor_generation: 1,
                remote_index_identity: Vec::new(),
                pipeline_mode: SPIRE_REMOTE_TRANSPORT_LIBPQ_PIPELINE,
                dispatch_action: SPIRE_REMOTE_DISPATCH_PIPELINE_ACTION,
                receive_validator: "test_transport_probe",
                status: SPIRE_REMOTE_STATUS_READY,
            })
            .collect::<Vec<_>>();
        let transport_rows = SpireRemoteProductionTransportAdapter::run_probe_requests(requests)?;
        remote_search_production_executor_state_summary_from_transport_probe_rows_with_consistency_mode(
            1,
            &dispatch_rows,
            &transport_rows,
            consistency_mode,
        )
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) fn remote_search_production_transport_probe_with_local_cancel_for_test(
    requests: Vec<SpireRemoteProductionTransportProbeRequest>,
    local_cancel_after_ms: u64,
) -> Vec<SpireRemoteProductionTransportProbeRow> {
    SpireRemoteProductionTransportAdapter::run_probe_requests_with_local_cancel(
        requests,
        Some(local_cancel_after_ms),
    )
    .unwrap_or_else(|e| pgrx::error!("{e}"))
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) fn remote_search_production_transport_probe_with_local_cancel_summary_for_test(
    requests: Vec<SpireRemoteProductionTransportProbeRequest>,
    local_cancel_after_ms: u64,
    consistency_mode: &str,
) -> SpireRemoteProductionExecutorStateSummaryRow {
    let result = (|| -> Result<SpireRemoteProductionExecutorStateSummaryRow, String> {
        let consistency_mode_name =
            consistency_mode_name(parse_remote_search_consistency_mode(consistency_mode)?);
        let dispatch_rows = requests
            .iter()
            .map(|request| SpireRemoteSearchLibpqDispatchPlanRow {
                requested_epoch: 1,
                node_id: request.node_id,
                selected_pids: vec![u64::from(request.node_id)],
                pid_count: 1,
                query_dimension: 2,
                top_k: 1,
                consistency_mode: consistency_mode_name,
                sql_template: "SELECT 1",
                parameter_count: 0,
                result_column_count: 1,
                conninfo_secret_name: format!("tests/node/{}", request.node_id),
                remote_index_regclass: "tests.ec_spire_transport_probe_idx".to_owned(),
                descriptor_generation: 1,
                remote_index_identity: Vec::new(),
                pipeline_mode: SPIRE_REMOTE_TRANSPORT_LIBPQ_PIPELINE,
                dispatch_action: SPIRE_REMOTE_DISPATCH_PIPELINE_ACTION,
                receive_validator: "test_transport_probe",
                status: SPIRE_REMOTE_STATUS_READY,
            })
            .collect::<Vec<_>>();
        let transport_rows =
            SpireRemoteProductionTransportAdapter::run_probe_requests_with_local_cancel(
                requests,
                Some(local_cancel_after_ms),
            )?;
        remote_search_production_executor_state_summary_from_transport_probe_rows_with_consistency_mode(
            1,
            &dispatch_rows,
            &transport_rows,
            consistency_mode,
        )
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) fn remote_search_production_candidate_receive_for_test(
    requests: Vec<SpireRemoteProductionCandidateReceiveRequest>,
) -> Vec<SpireRemoteProductionCandidateReceiveResult> {
    SpireRemoteProductionTransportAdapter::run_candidate_receive_requests(requests)
        .unwrap_or_else(|e| pgrx::error!("{e}"))
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) fn remote_search_production_candidate_receive_summary_for_test(
    requests: Vec<SpireRemoteProductionCandidateReceiveRequest>,
    consistency_mode: &str,
) -> SpireRemoteProductionExecutorStateSummaryRow {
    let result = (|| -> Result<SpireRemoteProductionExecutorStateSummaryRow, String> {
        let consistency_mode_name =
            consistency_mode_name(parse_remote_search_consistency_mode(consistency_mode)?);
        let requested_epoch = requests
            .first()
            .map(|request| request.requested_epoch)
            .unwrap_or(1);
        let dispatch_rows = requests
            .iter()
            .map(|request| SpireRemoteSearchLibpqDispatchPlanRow {
                requested_epoch,
                node_id: request.node_id,
                selected_pids: request.selected_pids.clone(),
                pid_count: u64::try_from(request.selected_pids.len()).unwrap_or(u64::MAX),
                query_dimension: u64::try_from(request.query.len()).unwrap_or(u64::MAX),
                top_k: u64::try_from(request.top_k).unwrap_or(u64::MAX),
                consistency_mode: consistency_mode_name,
                sql_template: SPIRE_REMOTE_SEARCH_LIBPQ_SQL_TEMPLATE,
                parameter_count: 6,
                result_column_count: 11,
                conninfo_secret_name: format!("tests/node/{}", request.node_id),
                remote_index_regclass: request.remote_index_regclass.clone(),
                descriptor_generation: 1,
                remote_index_identity: request.remote_index_identity.clone(),
                pipeline_mode: SPIRE_REMOTE_TRANSPORT_LIBPQ_PIPELINE,
                dispatch_action: SPIRE_REMOTE_DISPATCH_PIPELINE_ACTION,
                receive_validator: "test_candidate_receive",
                status: SPIRE_REMOTE_STATUS_READY,
            })
            .collect::<Vec<_>>();
        let transport_rows = requests
            .iter()
            .map(|request| SpireRemoteProductionTransportProbeRow {
                node_id: request.node_id,
                started_after_ms: 0,
                completed_after_ms: 0,
                elapsed_ms: 0,
                row_count: 1,
                status: SPIRE_REMOTE_STATUS_READY,
                failure_category: SPIRE_REMOTE_NONE,
            })
            .collect::<Vec<_>>();
        let receive_results =
            SpireRemoteProductionTransportAdapter::run_candidate_receive_requests(requests)?;
        remote_search_production_executor_state_summary_from_candidate_receive_results_with_consistency_mode(
            requested_epoch,
            &dispatch_rows,
            &transport_rows,
            &receive_results,
            consistency_mode,
        )
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) fn remote_search_production_candidate_receive_with_local_cancel_for_test(
    requests: Vec<SpireRemoteProductionCandidateReceiveRequest>,
    local_cancel_after_ms: u64,
) -> Vec<SpireRemoteProductionCandidateReceiveResult> {
    SpireRemoteProductionTransportAdapter::run_candidate_receive_requests_with_local_cancel(
        requests,
        Some(local_cancel_after_ms),
    )
    .unwrap_or_else(|e| pgrx::error!("{e}"))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SpireRemoteProductionDispatchState {
    Planned,
    BlockedBeforeDispatch,
    TransportReady,
    TransportFailed,
    CandidateReceiveReady,
    CandidateReceiveFailed,
    RemoteHeapReady,
    RemoteHeapFailed,
    DegradedSkipped,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq)]
struct SpireRemoteProductionDispatch {
    node_id: u32,
    selected_pids: Vec<u64>,
    pid_count: u64,
    conninfo_secret_name: String,
    remote_index_regclass: String,
    remote_index_identity: Vec<u8>,
    state: SpireRemoteProductionDispatchState,
    status: &'static str,
    next_executor_step: &'static str,
    transport_row_count: u64,
    transport_failure_category: &'static str,
    candidate_count: u64,
    candidate_failure_category: &'static str,
    degraded_skip_category: &'static str,
    candidate_batch: Option<SpireRemoteSearchCandidateBatch>,
    remote_heap_candidate_count: u64,
    remote_heap_failure_category: &'static str,
    remote_heap_candidates: Vec<SpireRemoteSearchLocalHeapCandidateRow>,
}

impl SpireRemoteProductionDispatch {
    fn from_libpq_dispatch(row: &SpireRemoteSearchLibpqDispatchPlanRow) -> Self {
        if row.dispatch_action == SPIRE_REMOTE_DISPATCH_PIPELINE_ACTION {
            Self {
                node_id: row.node_id,
                selected_pids: row.selected_pids.clone(),
                pid_count: row.pid_count,
                conninfo_secret_name: row.conninfo_secret_name.clone(),
                remote_index_regclass: row.remote_index_regclass.clone(),
                remote_index_identity: row.remote_index_identity.clone(),
                state: SpireRemoteProductionDispatchState::Planned,
                status: SPIRE_REMOTE_STATUS_REQUIRES_PRODUCTION_TRANSPORT,
                next_executor_step: SPIRE_REMOTE_EXECUTOR_STEP_PRODUCTION_TRANSPORT,
                transport_row_count: 0,
                transport_failure_category: SPIRE_REMOTE_NONE,
                candidate_count: 0,
                candidate_failure_category: SPIRE_REMOTE_NONE,
                degraded_skip_category: SPIRE_REMOTE_NONE,
                candidate_batch: None,
                remote_heap_candidate_count: 0,
                remote_heap_failure_category: SPIRE_REMOTE_NONE,
                remote_heap_candidates: Vec::new(),
            }
        } else {
            Self {
                node_id: row.node_id,
                selected_pids: row.selected_pids.clone(),
                pid_count: row.pid_count,
                conninfo_secret_name: row.conninfo_secret_name.clone(),
                remote_index_regclass: row.remote_index_regclass.clone(),
                remote_index_identity: row.remote_index_identity.clone(),
                state: SpireRemoteProductionDispatchState::BlockedBeforeDispatch,
                status: row.status,
                next_executor_step: remote_search_pre_dispatch_blocker_step(row.status),
                transport_row_count: 0,
                transport_failure_category: SPIRE_REMOTE_NONE,
                candidate_count: 0,
                candidate_failure_category: SPIRE_REMOTE_NONE,
                degraded_skip_category: SPIRE_REMOTE_NONE,
                candidate_batch: None,
                remote_heap_candidate_count: 0,
                remote_heap_failure_category: SPIRE_REMOTE_NONE,
                remote_heap_candidates: Vec::new(),
            }
        }
    }

    fn apply_transport_probe_row(
        &mut self,
        row: &SpireRemoteProductionTransportProbeRow,
    ) -> Result<(), String> {
        if self.state != SpireRemoteProductionDispatchState::Planned {
            return Err(format!(
                "ec_spire production executor transport outcome for node_id {} cannot apply to dispatch state {:?}",
                row.node_id, self.state
            ));
        }

        self.transport_row_count = row.row_count;
        self.transport_failure_category = row.failure_category;
        if row.status == SPIRE_REMOTE_STATUS_READY {
            self.state = SpireRemoteProductionDispatchState::TransportReady;
            self.status = SPIRE_REMOTE_STATUS_REQUIRES_COMPACT_CANDIDATE_RECEIVE;
            self.next_executor_step = SPIRE_REMOTE_EXECUTOR_STEP_COMPACT_CANDIDATE_RECEIVE;
        } else {
            self.state = SpireRemoteProductionDispatchState::TransportFailed;
            self.status = row.status;
            self.next_executor_step = SPIRE_REMOTE_EXECUTOR_STEP_PRODUCTION_TRANSPORT;
        }
        Ok(())
    }

    fn apply_transport_degraded_skip(
        &mut self,
        row: &SpireRemoteProductionTransportProbeRow,
    ) -> Result<(), String> {
        if self.state != SpireRemoteProductionDispatchState::Planned {
            return Err(format!(
                "ec_spire production executor transport outcome for node_id {} cannot apply to dispatch state {:?}",
                row.node_id, self.state
            ));
        }

        self.transport_row_count = row.row_count;
        self.transport_failure_category = row.failure_category;
        self.apply_degraded_skip(row.failure_category);
        Ok(())
    }

    fn apply_candidate_receive_result(
        &mut self,
        result: &SpireRemoteProductionCandidateReceiveResult,
    ) -> Result<(), String> {
        if self.state != SpireRemoteProductionDispatchState::TransportReady {
            return Err(format!(
                "ec_spire production executor candidate receive outcome for node_id {} cannot apply to dispatch state {:?}",
                result.node_id, self.state
            ));
        }

        self.candidate_failure_category = result.failure_category;
        if result.status == SPIRE_REMOTE_STATUS_READY {
            let Some(batch) = result.batch.as_ref() else {
                return Err(format!(
                    "ec_spire production executor candidate receive outcome for node_id {} is ready without a candidate batch",
                    result.node_id
                ));
            };
            let batch_candidate_count = u64::try_from(batch.candidates.len()).map_err(|_| {
                "ec_spire production executor candidate receive batch count exceeds u64"
                    .to_owned()
            })?;
            if result.candidate_count != batch_candidate_count {
                return Err(format!(
                    "ec_spire production executor candidate receive outcome for node_id {} reports {} candidates but batch contains {}",
                    result.node_id, result.candidate_count, batch_candidate_count
                ));
            }
            self.candidate_count = result.candidate_count;
            self.candidate_batch = Some(batch.clone());
            self.remote_heap_candidate_count = 0;
            self.remote_heap_failure_category = SPIRE_REMOTE_NONE;
            self.remote_heap_candidates.clear();
            self.state = SpireRemoteProductionDispatchState::CandidateReceiveReady;
            self.status = SPIRE_REMOTE_FINAL_STATUS_REQUIRES_REMOTE_HEAP;
            self.next_executor_step = SPIRE_REMOTE_EXECUTOR_STEP_REMOTE_HEAP_RESOLUTION;
        } else {
            self.candidate_count = 0;
            self.candidate_batch = None;
            self.remote_heap_candidate_count = 0;
            self.remote_heap_failure_category = SPIRE_REMOTE_NONE;
            self.remote_heap_candidates.clear();
            self.state = SpireRemoteProductionDispatchState::CandidateReceiveFailed;
            self.status = SPIRE_REMOTE_STATUS_CANDIDATE_RECEIVE_FAILED;
            self.next_executor_step = SPIRE_REMOTE_EXECUTOR_STEP_COMPACT_CANDIDATE_RECEIVE;
        }
        Ok(())
    }

    fn apply_candidate_receive_degraded_skip(
        &mut self,
        result: &SpireRemoteProductionCandidateReceiveResult,
    ) -> Result<(), String> {
        if self.state != SpireRemoteProductionDispatchState::TransportReady {
            return Err(format!(
                "ec_spire production executor candidate receive outcome for node_id {} cannot apply to dispatch state {:?}",
                result.node_id, self.state
            ));
        }

        self.candidate_failure_category = result.failure_category;
        self.apply_degraded_skip(result.failure_category);
        Ok(())
    }

    fn apply_candidate_receive_failure(&mut self, failure_category: &'static str) {
        self.candidate_count = 0;
        self.candidate_failure_category = failure_category;
        self.candidate_batch = None;
        self.remote_heap_candidate_count = 0;
        self.remote_heap_failure_category = SPIRE_REMOTE_NONE;
        self.remote_heap_candidates.clear();
        self.state = SpireRemoteProductionDispatchState::CandidateReceiveFailed;
        self.status = SPIRE_REMOTE_STATUS_CANDIDATE_RECEIVE_FAILED;
        self.next_executor_step = SPIRE_REMOTE_EXECUTOR_STEP_COMPACT_CANDIDATE_RECEIVE;
    }

    fn apply_remote_heap_receive_result(
        &mut self,
        result: &SpireRemoteProductionHeapReceiveResult,
    ) -> Result<(), String> {
        if self.state != SpireRemoteProductionDispatchState::CandidateReceiveReady {
            return Err(format!(
                "ec_spire production executor remote heap outcome for node_id {} cannot apply to dispatch state {:?}",
                result.node_id, self.state
            ));
        }

        self.remote_heap_failure_category = result.failure_category;
        if result.status == SPIRE_REMOTE_STATUS_READY {
            let candidate_count = u64::try_from(result.candidates.len()).map_err(|_| {
                "ec_spire production executor remote heap batch count exceeds u64".to_owned()
            })?;
            if result.candidate_count != candidate_count {
                return Err(format!(
                    "ec_spire production executor remote heap outcome for node_id {} reports {} candidates but batch contains {}",
                    result.node_id, result.candidate_count, candidate_count
                ));
            }
            self.remote_heap_candidate_count = result.candidate_count;
            self.remote_heap_candidates = result.candidates.clone();
            self.state = SpireRemoteProductionDispatchState::RemoteHeapReady;
            self.status = SPIRE_REMOTE_STATUS_READY;
            self.next_executor_step = SPIRE_REMOTE_NONE;
        } else {
            self.remote_heap_candidate_count = 0;
            self.remote_heap_candidates.clear();
            self.state = SpireRemoteProductionDispatchState::RemoteHeapFailed;
            self.status = SPIRE_REMOTE_PRODUCTION_REMOTE_HEAP_RESOLUTION_FAILED;
            self.next_executor_step = SPIRE_REMOTE_EXECUTOR_STEP_REMOTE_HEAP_RESOLUTION;
        }
        Ok(())
    }

    fn apply_remote_heap_receive_degraded_skip(
        &mut self,
        result: &SpireRemoteProductionHeapReceiveResult,
    ) -> Result<(), String> {
        if self.state != SpireRemoteProductionDispatchState::CandidateReceiveReady {
            return Err(format!(
                "ec_spire production executor remote heap outcome for node_id {} cannot apply to dispatch state {:?}",
                result.node_id, self.state
            ));
        }

        self.remote_heap_failure_category = result.failure_category;
        self.apply_degraded_skip(result.failure_category);
        Ok(())
    }

    fn apply_degraded_skip(&mut self, failure_category: &'static str) {
        self.candidate_count = 0;
        self.degraded_skip_category = failure_category;
        self.candidate_batch = None;
        self.remote_heap_candidate_count = 0;
        self.remote_heap_failure_category = SPIRE_REMOTE_NONE;
        self.remote_heap_candidates.clear();
        self.state = SpireRemoteProductionDispatchState::DegradedSkipped;
        self.status = SPIRE_REMOTE_STATUS_DEGRADED_SKIPPED;
        self.next_executor_step = SPIRE_REMOTE_EXECUTOR_STEP_REMOTE_HEAP_RESOLUTION;
    }

    fn apply_local_query_cancel(&mut self, failure_category: &'static str) {
        self.candidate_count = 0;
        self.candidate_failure_category = failure_category;
        self.degraded_skip_category = SPIRE_REMOTE_NONE;
        self.candidate_batch = None;
        self.remote_heap_candidate_count = 0;
        self.remote_heap_failure_category = SPIRE_REMOTE_NONE;
        self.remote_heap_candidates.clear();
        self.state = SpireRemoteProductionDispatchState::Cancelled;
        self.status = SPIRE_REMOTE_STATUS_EXECUTOR_CANCELLED;
        self.next_executor_step = SPIRE_REMOTE_EXECUTOR_STEP_CANCELLATION;
    }
}

#[derive(Debug, Clone, PartialEq)]
struct SpireRemoteFanoutExecutor {
    requested_epoch: u64,
    dispatches: Vec<SpireRemoteProductionDispatch>,
    conninfo_secret_lookup_count: u64,
    socket_open_count: u64,
    endpoint_identity_query_count: u64,
}

impl SpireRemoteFanoutExecutor {
    fn from_libpq_dispatch_rows(
        requested_epoch: u64,
        rows: &[SpireRemoteSearchLibpqDispatchPlanRow],
    ) -> Self {
        Self {
            requested_epoch,
            dispatches: rows
                .iter()
                .map(SpireRemoteProductionDispatch::from_libpq_dispatch)
                .collect(),
            conninfo_secret_lookup_count: 0,
            socket_open_count: 0,
            endpoint_identity_query_count: 0,
        }
    }

    fn apply_transport_probe_rows(
        &mut self,
        rows: &[SpireRemoteProductionTransportProbeRow],
    ) -> Result<(), String> {
        self.apply_transport_probe_rows_with_consistency_mode(rows, "strict")
    }

    fn apply_transport_probe_rows_with_consistency_mode(
        &mut self,
        rows: &[SpireRemoteProductionTransportProbeRow],
        consistency_mode: &str,
    ) -> Result<(), String> {
        let degraded =
            parse_remote_search_consistency_mode(consistency_mode)? == meta::SpireConsistencyMode::Degraded;
        if let Some(cancelled_row) = rows
            .iter()
            .find(|row| is_local_cancellation_failure_category(row.failure_category))
        {
            if !self.dispatches.iter().any(|dispatch| {
                dispatch.node_id == cancelled_row.node_id
                    && dispatch.state == SpireRemoteProductionDispatchState::Planned
            }) {
                return Err(format!(
                    "ec_spire production executor transport outcome for node_id {} does not match a planned dispatch",
                    cancelled_row.node_id
                ));
            }
            self.apply_local_query_cancel(cancelled_row.failure_category);
            return Ok(());
        }
        for row in rows {
            let dispatch = self
                .dispatches
                .iter_mut()
                .find(|dispatch| {
                    dispatch.node_id == row.node_id
                        && dispatch.state == SpireRemoteProductionDispatchState::Planned
                })
                .ok_or_else(|| {
                    format!(
                        "ec_spire production executor transport outcome for node_id {} does not match a planned dispatch",
                        row.node_id
                    )
                })?;
            if degraded && row.status != SPIRE_REMOTE_STATUS_READY {
                dispatch.apply_transport_degraded_skip(row)?;
            } else {
                dispatch.apply_transport_probe_row(row)?;
            }
        }
        Ok(())
    }

    fn apply_candidate_receive_results(
        &mut self,
        results: &[SpireRemoteProductionCandidateReceiveResult],
    ) -> Result<(), String> {
        self.apply_candidate_receive_results_with_consistency_mode(results, "strict")
    }

    fn apply_candidate_receive_results_with_consistency_mode(
        &mut self,
        results: &[SpireRemoteProductionCandidateReceiveResult],
        consistency_mode: &str,
    ) -> Result<(), String> {
        let degraded =
            parse_remote_search_consistency_mode(consistency_mode)? == meta::SpireConsistencyMode::Degraded;
        if let Some(cancelled_result) = results
            .iter()
            .find(|result| is_local_cancellation_failure_category(result.failure_category))
        {
            if !self.dispatches.iter().any(|dispatch| {
                dispatch.node_id == cancelled_result.node_id
                    && dispatch.state == SpireRemoteProductionDispatchState::TransportReady
            }) {
                return Err(format!(
                    "ec_spire production executor candidate receive outcome for node_id {} does not match a transport-ready dispatch",
                    cancelled_result.node_id
                ));
            }
            self.apply_local_query_cancel(cancelled_result.failure_category);
            return Ok(());
        }
        for result in results {
            let dispatch = self
                .dispatches
                .iter_mut()
                .find(|dispatch| {
                    dispatch.node_id == result.node_id
                        && dispatch.state == SpireRemoteProductionDispatchState::TransportReady
                })
                .ok_or_else(|| {
                    format!(
                        "ec_spire production executor candidate receive outcome for node_id {} does not match a transport-ready dispatch",
                        result.node_id
                    )
                })?;
            if degraded && result.status != SPIRE_REMOTE_STATUS_READY {
                dispatch.apply_candidate_receive_degraded_skip(result)?;
            } else {
                dispatch.apply_candidate_receive_result(result)?;
            }
        }
        Ok(())
    }

    fn apply_remote_heap_receive_results_with_consistency_mode(
        &mut self,
        results: &[SpireRemoteProductionHeapReceiveResult],
        consistency_mode: &str,
    ) -> Result<(), String> {
        let degraded =
            parse_remote_search_consistency_mode(consistency_mode)? == meta::SpireConsistencyMode::Degraded;
        if let Some(cancelled_result) = results
            .iter()
            .find(|result| is_local_cancellation_failure_category(result.failure_category))
        {
            if !self.dispatches.iter().any(|dispatch| {
                dispatch.node_id == cancelled_result.node_id
                    && dispatch.state == SpireRemoteProductionDispatchState::CandidateReceiveReady
            }) {
                return Err(format!(
                    "ec_spire production executor remote heap outcome for node_id {} does not match a candidate-ready dispatch",
                    cancelled_result.node_id
                ));
            }
            self.apply_local_query_cancel(cancelled_result.failure_category);
            return Ok(());
        }
        for result in results {
            let dispatch = self
                .dispatches
                .iter_mut()
                .find(|dispatch| {
                    dispatch.node_id == result.node_id
                        && dispatch.state == SpireRemoteProductionDispatchState::CandidateReceiveReady
                })
                .ok_or_else(|| {
                    format!(
                        "ec_spire production executor remote heap outcome for node_id {} does not match a candidate-ready dispatch",
                        result.node_id
                    )
                })?;
            if degraded && result.status != SPIRE_REMOTE_STATUS_READY {
                dispatch.apply_remote_heap_receive_degraded_skip(result)?;
            } else {
                dispatch.apply_remote_heap_receive_result(result)?;
            }
        }
        Ok(())
    }

    fn apply_local_query_cancel(&mut self, failure_category: &'static str) {
        for dispatch in &mut self.dispatches {
            dispatch.apply_local_query_cancel(failure_category);
        }
    }

    fn apply_blocked_before_dispatch_degraded_skips(&mut self) {
        for dispatch in &mut self.dispatches {
            if dispatch.state == SpireRemoteProductionDispatchState::BlockedBeforeDispatch {
                dispatch.apply_degraded_skip(dispatch.status);
            }
        }
    }

    fn mark_planned_dispatches_candidate_receive_ready(&mut self) {
        for dispatch in &mut self.dispatches {
            if dispatch.state == SpireRemoteProductionDispatchState::Planned {
                dispatch.state = SpireRemoteProductionDispatchState::TransportReady;
                dispatch.status = SPIRE_REMOTE_STATUS_REQUIRES_COMPACT_CANDIDATE_RECEIVE;
                dispatch.next_executor_step =
                    SPIRE_REMOTE_EXECUTOR_STEP_COMPACT_CANDIDATE_RECEIVE;
            }
        }
    }

    fn compact_candidate_receive_requests(
        &mut self,
        query: &[f32],
        top_k: usize,
        consistency_mode: &str,
    ) -> Result<Vec<SpireRemoteProductionCandidateReceiveRequest>, String> {
        let degraded =
            parse_remote_search_consistency_mode(consistency_mode)? == meta::SpireConsistencyMode::Degraded;
        let mut requests = Vec::new();
        let mut secret_lookup_count = self.conninfo_secret_lookup_count;
        for dispatch in &mut self.dispatches {
            if dispatch.state != SpireRemoteProductionDispatchState::TransportReady {
                continue;
            }
            add_remote_count(
                &mut secret_lookup_count,
                1,
                "remote production executor compact receive request build",
                "conninfo secret lookup",
            )?;
            match remote_conninfo_secret_value(&dispatch.conninfo_secret_name) {
                Ok(conninfo) => requests.push(SpireRemoteProductionCandidateReceiveRequest {
                    node_id: dispatch.node_id,
                    conninfo,
                    remote_index_regclass: dispatch.remote_index_regclass.clone(),
                    remote_index_identity: dispatch.remote_index_identity.clone(),
                    requested_epoch: self.requested_epoch,
                    query: query.to_vec(),
                    selected_pids: dispatch.selected_pids.clone(),
                    top_k,
                    consistency_mode: consistency_mode.to_owned(),
                }),
                Err(_) if degraded => dispatch.apply_degraded_skip(SPIRE_REMOTE_STATUS_REQUIRES_SECRET),
                Err(_) => dispatch.apply_candidate_receive_failure(SPIRE_REMOTE_STATUS_REQUIRES_SECRET),
            }
        }
        self.conninfo_secret_lookup_count = secret_lookup_count;
        Ok(requests)
    }

    fn run_compact_candidate_receive(
        &mut self,
        query: &[f32],
        top_k: usize,
        consistency_mode: &str,
    ) -> Result<(), String> {
        let requests = self.compact_candidate_receive_requests(query, top_k, consistency_mode)?;
        if requests.is_empty() {
            return Ok(());
        }
        let results =
            SpireRemoteProductionTransportAdapter::run_candidate_receive_requests(requests)?;
        self.apply_candidate_receive_results_with_consistency_mode(&results, consistency_mode)
    }

    fn remote_heap_receive_requests(
        &mut self,
        query: &[f32],
        top_k: usize,
        consistency_mode: &str,
        tuple_payload_columns: Option<&[String]>,
    ) -> Result<Vec<SpireRemoteProductionHeapReceiveRequest>, String> {
        let degraded =
            parse_remote_search_consistency_mode(consistency_mode)? == meta::SpireConsistencyMode::Degraded;
        let mut requests = Vec::new();
        let mut secret_lookup_count = self.conninfo_secret_lookup_count;
        for dispatch in &mut self.dispatches {
            if dispatch.state != SpireRemoteProductionDispatchState::CandidateReceiveReady {
                continue;
            }
            add_remote_count(
                &mut secret_lookup_count,
                1,
                "remote production executor heap receive request build",
                "conninfo secret lookup",
            )?;
            match remote_conninfo_secret_value(&dispatch.conninfo_secret_name) {
                Ok(conninfo) => requests.push(SpireRemoteProductionHeapReceiveRequest {
                    node_id: dispatch.node_id,
                    conninfo,
                    remote_index_regclass: dispatch.remote_index_regclass.clone(),
                    remote_index_identity: dispatch.remote_index_identity.clone(),
                    requested_epoch: self.requested_epoch,
                    query: query.to_vec(),
                    selected_pids: dispatch.selected_pids.clone(),
                    top_k,
                    consistency_mode: consistency_mode.to_owned(),
                    tuple_payload_columns: tuple_payload_columns.map(<[String]>::to_vec),
                }),
                Err(_) if degraded => dispatch.apply_degraded_skip(SPIRE_REMOTE_STATUS_REQUIRES_SECRET),
                Err(_) => {
                    let now = std::time::Instant::now();
                    let result = failed_production_heap_receive_result(
                        dispatch.node_id,
                        now,
                        now,
                        SPIRE_REMOTE_STATUS_REQUIRES_SECRET,
                    );
                    dispatch.apply_remote_heap_receive_result(&result)?;
                }
            }
        }
        self.conninfo_secret_lookup_count = secret_lookup_count;
        Ok(requests)
    }

    fn run_remote_heap_receive(
        &mut self,
        query: &[f32],
        top_k: usize,
        consistency_mode: &str,
        tuple_payload_columns: Option<&[String]>,
    ) -> Result<(), String> {
        let requests =
            self.remote_heap_receive_requests(query, top_k, consistency_mode, tuple_payload_columns)?;
        if requests.is_empty() {
            return Ok(());
        }
        let results = SpireRemoteProductionTransportAdapter::run_heap_receive_requests(requests)?;
        self.apply_remote_heap_receive_results_with_consistency_mode(&results, consistency_mode)
    }

    fn ready_candidate_batches(&self) -> Result<Vec<SpireRemoteSearchCandidateBatch>, String> {
        let mut batches = Vec::new();
        for dispatch in &self.dispatches {
            match dispatch.state {
                SpireRemoteProductionDispatchState::CandidateReceiveReady => {
                    let batch = dispatch.candidate_batch.clone().ok_or_else(|| {
                        format!(
                            "ec_spire production executor candidate receive ready dispatch for node_id {} is missing its candidate batch",
                            dispatch.node_id
                        )
                    })?;
                    batches.push(batch);
                }
                SpireRemoteProductionDispatchState::BlockedBeforeDispatch
                | SpireRemoteProductionDispatchState::Planned
                | SpireRemoteProductionDispatchState::TransportReady
                | SpireRemoteProductionDispatchState::TransportFailed
                | SpireRemoteProductionDispatchState::CandidateReceiveFailed
                | SpireRemoteProductionDispatchState::RemoteHeapReady
                | SpireRemoteProductionDispatchState::RemoteHeapFailed
                | SpireRemoteProductionDispatchState::Cancelled => {
                    return Err(format!(
                        "ec_spire production executor cannot merge compact candidates while node_id {} is in state {:?} with status {}",
                        dispatch.node_id, dispatch.state, dispatch.status
                    ));
                }
                SpireRemoteProductionDispatchState::DegradedSkipped => {}
            }
        }
        Ok(batches)
    }

    fn merge_ready_candidate_batches(
        &self,
        limit: Option<usize>,
    ) -> Result<SpireRemoteSearchMergeResult, String> {
        merge_validated_remote_search_candidate_batches(
            self.requested_epoch,
            self.ready_candidate_batches()?,
            limit,
        )
    }

    fn ready_remote_heap_candidate_rows(
        &self,
    ) -> Result<Vec<SpireRemoteSearchLocalHeapCandidateRow>, String> {
        let mut candidates = Vec::new();
        for dispatch in &self.dispatches {
            match dispatch.state {
                SpireRemoteProductionDispatchState::RemoteHeapReady => {
                    candidates.extend(dispatch.remote_heap_candidates.clone());
                }
                SpireRemoteProductionDispatchState::DegradedSkipped => {}
                SpireRemoteProductionDispatchState::BlockedBeforeDispatch
                | SpireRemoteProductionDispatchState::Planned
                | SpireRemoteProductionDispatchState::TransportReady
                | SpireRemoteProductionDispatchState::TransportFailed
                | SpireRemoteProductionDispatchState::CandidateReceiveReady
                | SpireRemoteProductionDispatchState::CandidateReceiveFailed
                | SpireRemoteProductionDispatchState::RemoteHeapFailed
                | SpireRemoteProductionDispatchState::Cancelled => {
                    return Err(format!(
                        "ec_spire production executor cannot merge remote heap candidates while node_id {} is in state {:?} with status {}",
                        dispatch.node_id, dispatch.state, dispatch.status
                    ));
                }
            }
        }
        Ok(candidates)
    }

    fn remote_heap_resolution_counts(&self) -> Result<(u64, u64, u64), String> {
        let mut ready_dispatch_count = 0_u64;
        let mut failed_dispatch_count = 0_u64;
        let mut candidate_count = 0_u64;
        for dispatch in &self.dispatches {
            match dispatch.state {
                SpireRemoteProductionDispatchState::RemoteHeapReady => {
                    add_remote_count(
                        &mut ready_dispatch_count,
                        1,
                        "remote production executor heap resolution counts",
                        "ready dispatch",
                    )?;
                    add_remote_count(
                        &mut candidate_count,
                        dispatch.remote_heap_candidate_count,
                        "remote production executor heap resolution counts",
                        "remote heap candidate",
                    )?;
                }
                SpireRemoteProductionDispatchState::RemoteHeapFailed => {
                    add_remote_count(
                        &mut failed_dispatch_count,
                        1,
                        "remote production executor heap resolution counts",
                        "failed dispatch",
                    )?;
                }
                _ => {}
            }
        }
        Ok((ready_dispatch_count, failed_dispatch_count, candidate_count))
    }

    fn degraded_skip_report(
        &self,
    ) -> Result<Vec<SpireRemoteProductionDegradedSkipReportRow>, String> {
        let mut rows = Vec::new();
        for dispatch in &self.dispatches {
            if dispatch.state != SpireRemoteProductionDispatchState::DegradedSkipped {
                continue;
            }
            rows.push(SpireRemoteProductionDegradedSkipReportRow {
                requested_epoch: self.requested_epoch,
                node_id: dispatch.node_id,
                skipped_pid_count: dispatch.pid_count,
                first_skip_category: dispatch.degraded_skip_category,
                first_skip_hint: remote_production_failure_hint(dispatch.degraded_skip_category),
                status: dispatch.status,
            });
        }
        Ok(rows)
    }

    fn summary(
        &self,
        consistency_mode_source: &'static str,
        consistency_mode: &'static str,
    ) -> Result<SpireRemoteProductionExecutorStateSummaryRow, String> {
        let mut planned_dispatch_count = 0_u64;
        let mut blocked_before_dispatch_count = 0_u64;
        let mut remote_pid_count = 0_u64;
        let mut planned_pid_count = 0_u64;
        let mut blocked_pid_count = 0_u64;
        let mut transport_pending_dispatch_count = 0_u64;
        let mut transport_sent_dispatch_count = 0_u64;
        let mut transport_ready_dispatch_count = 0_u64;
        let mut transport_failed_dispatch_count = 0_u64;
        let mut transport_row_count = 0_u64;
        let mut first_transport_failure_category = SPIRE_REMOTE_NONE;
        let mut candidate_receive_pending_dispatch_count = 0_u64;
        let mut candidate_receive_sent_dispatch_count = 0_u64;
        let mut candidate_receive_ready_dispatch_count = 0_u64;
        let mut candidate_receive_failed_dispatch_count = 0_u64;
        let mut candidate_row_count = 0_u64;
        let mut first_candidate_receive_failure_category = SPIRE_REMOTE_NONE;
        let mut degraded_skipped_dispatch_count = 0_u64;
        let mut first_degraded_skip_category = SPIRE_REMOTE_NONE;
        let mut cancelled_dispatch_count = 0_u64;
        let mut first_cancellation_category = SPIRE_REMOTE_NONE;
        let mut first_blocked_status = SPIRE_REMOTE_STATUS_READY;
        let mut first_blocked_step = SPIRE_REMOTE_NONE;
        let mut remote_heap_ready_dispatch_count = 0_u64;
        let mut remote_heap_failed_dispatch_count = 0_u64;

        for dispatch in &self.dispatches {
            add_remote_count(
                &mut remote_pid_count,
                dispatch.pid_count,
                "remote production executor state summary",
                "remote PID",
            )?;
            match dispatch.state {
                SpireRemoteProductionDispatchState::Planned => {
                    add_remote_count(
                        &mut planned_dispatch_count,
                        1,
                        "remote production executor state summary",
                        "planned dispatch",
                    )?;
                    add_remote_count(
                        &mut planned_pid_count,
                        dispatch.pid_count,
                        "remote production executor state summary",
                        "planned PID",
                    )?;
                    add_remote_count(
                        &mut transport_pending_dispatch_count,
                        1,
                        "remote production executor state summary",
                        "transport-pending dispatch",
                    )?;
                }
                SpireRemoteProductionDispatchState::BlockedBeforeDispatch => {
                    if blocked_before_dispatch_count == 0 {
                        first_blocked_status = dispatch.status;
                        first_blocked_step = dispatch.next_executor_step;
                    }
                    add_remote_count(
                        &mut blocked_before_dispatch_count,
                        1,
                        "remote production executor state summary",
                        "blocked dispatch",
                    )?;
                    add_remote_count(
                        &mut blocked_pid_count,
                        dispatch.pid_count,
                        "remote production executor state summary",
                        "blocked PID",
                    )?;
                }
                SpireRemoteProductionDispatchState::TransportReady => {
                    add_remote_count(
                        &mut planned_dispatch_count,
                        1,
                        "remote production executor state summary",
                        "planned dispatch",
                    )?;
                    add_remote_count(
                        &mut planned_pid_count,
                        dispatch.pid_count,
                        "remote production executor state summary",
                        "planned PID",
                    )?;
                    add_remote_count(
                        &mut transport_sent_dispatch_count,
                        1,
                        "remote production executor state summary",
                        "transport-sent dispatch",
                    )?;
                    add_remote_count(
                        &mut transport_ready_dispatch_count,
                        1,
                        "remote production executor state summary",
                        "transport-ready dispatch",
                    )?;
                    add_remote_count(
                        &mut transport_row_count,
                        dispatch.transport_row_count,
                        "remote production executor state summary",
                        "transport row",
                    )?;
                    add_remote_count(
                        &mut candidate_receive_pending_dispatch_count,
                        1,
                        "remote production executor state summary",
                        "candidate-receive-pending dispatch",
                    )?;
                }
                SpireRemoteProductionDispatchState::TransportFailed => {
                    if transport_failed_dispatch_count == 0 {
                        first_transport_failure_category = dispatch.transport_failure_category;
                    }
                    add_remote_count(
                        &mut planned_dispatch_count,
                        1,
                        "remote production executor state summary",
                        "planned dispatch",
                    )?;
                    add_remote_count(
                        &mut planned_pid_count,
                        dispatch.pid_count,
                        "remote production executor state summary",
                        "planned PID",
                    )?;
                    add_remote_count(
                        &mut transport_sent_dispatch_count,
                        1,
                        "remote production executor state summary",
                        "transport-sent dispatch",
                    )?;
                    add_remote_count(
                        &mut transport_failed_dispatch_count,
                        1,
                        "remote production executor state summary",
                        "transport-failed dispatch",
                    )?;
                }
                SpireRemoteProductionDispatchState::CandidateReceiveReady => {
                    add_remote_count(
                        &mut planned_dispatch_count,
                        1,
                        "remote production executor state summary",
                        "planned dispatch",
                    )?;
                    add_remote_count(
                        &mut planned_pid_count,
                        dispatch.pid_count,
                        "remote production executor state summary",
                        "planned PID",
                    )?;
                    add_remote_count(
                        &mut transport_sent_dispatch_count,
                        1,
                        "remote production executor state summary",
                        "transport-sent dispatch",
                    )?;
                    add_remote_count(
                        &mut transport_ready_dispatch_count,
                        1,
                        "remote production executor state summary",
                        "transport-ready dispatch",
                    )?;
                    add_remote_count(
                        &mut transport_row_count,
                        dispatch.transport_row_count,
                        "remote production executor state summary",
                        "transport row",
                    )?;
                    add_remote_count(
                        &mut candidate_receive_sent_dispatch_count,
                        1,
                        "remote production executor state summary",
                        "candidate-receive-sent dispatch",
                    )?;
                    add_remote_count(
                        &mut candidate_receive_ready_dispatch_count,
                        1,
                        "remote production executor state summary",
                        "candidate-receive-ready dispatch",
                    )?;
                    add_remote_count(
                        &mut candidate_row_count,
                        dispatch.candidate_count,
                        "remote production executor state summary",
                        "candidate row",
                    )?;
                }
                SpireRemoteProductionDispatchState::CandidateReceiveFailed => {
                    if candidate_receive_failed_dispatch_count == 0 {
                        first_candidate_receive_failure_category =
                            dispatch.candidate_failure_category;
                    }
                    add_remote_count(
                        &mut planned_dispatch_count,
                        1,
                        "remote production executor state summary",
                        "planned dispatch",
                    )?;
                    add_remote_count(
                        &mut planned_pid_count,
                        dispatch.pid_count,
                        "remote production executor state summary",
                        "planned PID",
                    )?;
                    add_remote_count(
                        &mut transport_sent_dispatch_count,
                        1,
                        "remote production executor state summary",
                        "transport-sent dispatch",
                    )?;
                    add_remote_count(
                        &mut transport_ready_dispatch_count,
                        1,
                        "remote production executor state summary",
                        "transport-ready dispatch",
                    )?;
                    add_remote_count(
                        &mut transport_row_count,
                        dispatch.transport_row_count,
                        "remote production executor state summary",
                        "transport row",
                    )?;
                    add_remote_count(
                        &mut candidate_receive_sent_dispatch_count,
                        1,
                        "remote production executor state summary",
                        "candidate-receive-sent dispatch",
                    )?;
                    add_remote_count(
                        &mut candidate_receive_failed_dispatch_count,
                        1,
                        "remote production executor state summary",
                        "candidate-receive-failed dispatch",
                    )?;
                }
                SpireRemoteProductionDispatchState::RemoteHeapReady => {
                    add_remote_count(
                        &mut planned_dispatch_count,
                        1,
                        "remote production executor state summary",
                        "planned dispatch",
                    )?;
                    add_remote_count(
                        &mut planned_pid_count,
                        dispatch.pid_count,
                        "remote production executor state summary",
                        "planned PID",
                    )?;
                    add_remote_count(
                        &mut transport_sent_dispatch_count,
                        1,
                        "remote production executor state summary",
                        "transport-sent dispatch",
                    )?;
                    add_remote_count(
                        &mut transport_ready_dispatch_count,
                        1,
                        "remote production executor state summary",
                        "transport-ready dispatch",
                    )?;
                    add_remote_count(
                        &mut transport_row_count,
                        dispatch.transport_row_count,
                        "remote production executor state summary",
                        "transport row",
                    )?;
                    add_remote_count(
                        &mut candidate_receive_sent_dispatch_count,
                        1,
                        "remote production executor state summary",
                        "candidate-receive-sent dispatch",
                    )?;
                    add_remote_count(
                        &mut candidate_receive_ready_dispatch_count,
                        1,
                        "remote production executor state summary",
                        "candidate-receive-ready dispatch",
                    )?;
                    add_remote_count(
                        &mut candidate_row_count,
                        dispatch.candidate_count,
                        "remote production executor state summary",
                        "candidate row",
                    )?;
                    add_remote_count(
                        &mut remote_heap_ready_dispatch_count,
                        1,
                        "remote production executor state summary",
                        "remote-heap-ready dispatch",
                    )?;
                }
                SpireRemoteProductionDispatchState::RemoteHeapFailed => {
                    add_remote_count(
                        &mut planned_dispatch_count,
                        1,
                        "remote production executor state summary",
                        "planned dispatch",
                    )?;
                    add_remote_count(
                        &mut planned_pid_count,
                        dispatch.pid_count,
                        "remote production executor state summary",
                        "planned PID",
                    )?;
                    add_remote_count(
                        &mut transport_sent_dispatch_count,
                        1,
                        "remote production executor state summary",
                        "transport-sent dispatch",
                    )?;
                    add_remote_count(
                        &mut transport_ready_dispatch_count,
                        1,
                        "remote production executor state summary",
                        "transport-ready dispatch",
                    )?;
                    add_remote_count(
                        &mut transport_row_count,
                        dispatch.transport_row_count,
                        "remote production executor state summary",
                        "transport row",
                    )?;
                    add_remote_count(
                        &mut candidate_receive_sent_dispatch_count,
                        1,
                        "remote production executor state summary",
                        "candidate-receive-sent dispatch",
                    )?;
                    add_remote_count(
                        &mut candidate_receive_ready_dispatch_count,
                        1,
                        "remote production executor state summary",
                        "candidate-receive-ready dispatch",
                    )?;
                    add_remote_count(
                        &mut candidate_row_count,
                        dispatch.candidate_count,
                        "remote production executor state summary",
                        "candidate row",
                    )?;
                    add_remote_count(
                        &mut remote_heap_failed_dispatch_count,
                        1,
                        "remote production executor state summary",
                        "remote-heap-failed dispatch",
                    )?;
                }
                SpireRemoteProductionDispatchState::DegradedSkipped => {
                    if degraded_skipped_dispatch_count == 0 {
                        first_degraded_skip_category = dispatch.degraded_skip_category;
                    }
                    add_remote_count(
                        &mut planned_dispatch_count,
                        1,
                        "remote production executor state summary",
                        "planned dispatch",
                    )?;
                    add_remote_count(
                        &mut planned_pid_count,
                        dispatch.pid_count,
                        "remote production executor state summary",
                        "planned PID",
                    )?;
                    add_remote_count(
                        &mut degraded_skipped_dispatch_count,
                        1,
                        "remote production executor state summary",
                        "degraded-skipped dispatch",
                    )?;
                }
                SpireRemoteProductionDispatchState::Cancelled => {
                    if cancelled_dispatch_count == 0 {
                        first_cancellation_category = dispatch.candidate_failure_category;
                    }
                    add_remote_count(
                        &mut cancelled_dispatch_count,
                        1,
                        "remote production executor state summary",
                        "cancelled dispatch",
                    )?;
                }
            }
        }

        let dispatch_count = u64::try_from(self.dispatches.len()).map_err(|_| {
            "ec_spire remote production executor dispatch count exceeds u64".to_owned()
        })?;
        let (next_executor_step, status, recommendation) = if cancelled_dispatch_count > 0 {
            (
                SPIRE_REMOTE_EXECUTOR_STEP_CANCELLATION,
                SPIRE_REMOTE_STATUS_EXECUTOR_CANCELLED,
                "release governance and discard retained remote batches after local cancellation",
            )
        } else if blocked_before_dispatch_count > 0 {
            (
                first_blocked_step,
                first_blocked_status,
                remote_search_pre_dispatch_blocker_recommendation(first_blocked_status),
            )
        } else if transport_failed_dispatch_count > 0 {
            (
                SPIRE_REMOTE_EXECUTOR_STEP_PRODUCTION_TRANSPORT,
                SPIRE_REMOTE_STATUS_PRODUCTION_TRANSPORT_FAILED,
                "inspect production transport failure category before compact candidate receive",
            )
        } else if transport_pending_dispatch_count > 0 {
            (
                SPIRE_REMOTE_EXECUTOR_STEP_PRODUCTION_TRANSPORT,
                SPIRE_REMOTE_STATUS_REQUIRES_PRODUCTION_TRANSPORT,
                "implement production async or libpq pipeline transport before remote fanout execution",
            )
        } else if candidate_receive_failed_dispatch_count > 0 {
            (
                SPIRE_REMOTE_EXECUTOR_STEP_COMPACT_CANDIDATE_RECEIVE,
                SPIRE_REMOTE_STATUS_CANDIDATE_RECEIVE_FAILED,
                "inspect production candidate receive failure category before merge",
            )
        } else if candidate_receive_pending_dispatch_count > 0 {
            (
                SPIRE_REMOTE_EXECUTOR_STEP_COMPACT_CANDIDATE_RECEIVE,
                SPIRE_REMOTE_STATUS_REQUIRES_COMPACT_CANDIDATE_RECEIVE,
                "wire production compact candidate receive before AM scan merge",
            )
        } else if remote_heap_failed_dispatch_count > 0 {
            (
                SPIRE_REMOTE_EXECUTOR_STEP_REMOTE_HEAP_RESOLUTION,
                SPIRE_REMOTE_PRODUCTION_REMOTE_HEAP_RESOLUTION_FAILED,
                "inspect production remote heap failure category before final row delivery",
            )
        } else if remote_heap_ready_dispatch_count > 0 && degraded_skipped_dispatch_count > 0 {
            (
                SPIRE_REMOTE_NONE,
                SPIRE_REMOTE_STATUS_DEGRADED_READY,
                "return ready heap-resolved rows and report degraded skipped dispatches",
            )
        } else if remote_heap_ready_dispatch_count > 0 {
            (SPIRE_REMOTE_NONE, SPIRE_REMOTE_STATUS_READY, SPIRE_REMOTE_NONE)
        } else if candidate_receive_ready_dispatch_count > 0 && degraded_skipped_dispatch_count > 0
        {
            (
                SPIRE_REMOTE_EXECUTOR_STEP_REMOTE_HEAP_RESOLUTION,
                SPIRE_REMOTE_STATUS_DEGRADED_READY,
                "continue with ready remote batches and report degraded skipped dispatches",
            )
        } else if degraded_skipped_dispatch_count > 0 {
            (
                SPIRE_REMOTE_EXECUTOR_STEP_REMOTE_HEAP_RESOLUTION,
                SPIRE_REMOTE_STATUS_DEGRADED_SKIPPED,
                "continue without skipped remote dispatches in degraded mode",
            )
        } else if transport_ready_dispatch_count > 0 {
            (
                SPIRE_REMOTE_EXECUTOR_STEP_REMOTE_HEAP_RESOLUTION,
                SPIRE_REMOTE_FINAL_STATUS_REQUIRES_REMOTE_HEAP,
                "wire origin-node remote heap resolution before returning SQL rows",
            )
        } else {
            (SPIRE_REMOTE_NONE, SPIRE_REMOTE_STATUS_READY, SPIRE_REMOTE_NONE)
        };

        Ok(SpireRemoteProductionExecutorStateSummaryRow {
            requested_epoch: self.requested_epoch,
            state_model: SPIRE_REMOTE_PRODUCTION_STATE_MODEL,
            transport_mode: SPIRE_REMOTE_PRODUCTION_TRANSPORT_PENDING,
            consistency_mode_source,
            consistency_mode,
            dispatch_count,
            planned_dispatch_count,
            blocked_before_dispatch_count,
            remote_pid_count,
            planned_pid_count,
            blocked_pid_count,
            conninfo_secret_lookup_count: self.conninfo_secret_lookup_count,
            socket_open_count: self.socket_open_count,
            endpoint_identity_query_count: self.endpoint_identity_query_count,
            transport_pending_dispatch_count,
            transport_sent_dispatch_count,
            transport_ready_dispatch_count,
            transport_failed_dispatch_count,
            transport_row_count,
            first_transport_failure_category,
            candidate_receive_pending_dispatch_count,
            candidate_receive_sent_dispatch_count,
            candidate_receive_ready_dispatch_count,
            candidate_receive_failed_dispatch_count,
            candidate_row_count,
            first_candidate_receive_failure_category,
            degraded_skipped_dispatch_count,
            first_degraded_skip_category,
            cancelled_dispatch_count,
            first_cancellation_category,
            next_executor_step,
            status,
            recommendation,
        })
    }
}

fn remote_search_production_executor_state_summary_from_dispatch_rows(
    requested_epoch: u64,
    rows: &[SpireRemoteSearchLibpqDispatchPlanRow],
    consistency_mode_source: &'static str,
    consistency_mode: &str,
) -> Result<SpireRemoteProductionExecutorStateSummaryRow, String> {
    let parsed_consistency_mode = parse_remote_search_consistency_mode(consistency_mode)?;
    let consistency_mode = consistency_mode_name(parsed_consistency_mode);
    let mut executor = SpireRemoteFanoutExecutor::from_libpq_dispatch_rows(requested_epoch, rows);
    if parsed_consistency_mode == meta::SpireConsistencyMode::Degraded {
        executor.apply_blocked_before_dispatch_degraded_skips();
    }
    executor.summary(consistency_mode_source, consistency_mode)
}

fn remote_search_production_degraded_skip_report_from_dispatch_rows(
    requested_epoch: u64,
    rows: &[SpireRemoteSearchLibpqDispatchPlanRow],
    consistency_mode: &str,
) -> Result<Vec<SpireRemoteProductionDegradedSkipReportRow>, String> {
    let parsed_consistency_mode = parse_remote_search_consistency_mode(consistency_mode)?;
    let mut executor = SpireRemoteFanoutExecutor::from_libpq_dispatch_rows(requested_epoch, rows);
    if parsed_consistency_mode == meta::SpireConsistencyMode::Degraded {
        executor.apply_blocked_before_dispatch_degraded_skips();
    }
    executor.degraded_skip_report()
}

#[cfg(any(test, feature = "pg_test"))]
fn remote_search_production_executor_state_summary_from_transport_probe_rows(
    requested_epoch: u64,
    dispatch_rows: &[SpireRemoteSearchLibpqDispatchPlanRow],
    transport_rows: &[SpireRemoteProductionTransportProbeRow],
) -> Result<SpireRemoteProductionExecutorStateSummaryRow, String> {
    remote_search_production_executor_state_summary_from_transport_probe_rows_with_consistency_mode(
        requested_epoch,
        dispatch_rows,
        transport_rows,
        "strict",
    )
}

#[cfg(any(test, feature = "pg_test"))]
fn remote_search_production_executor_state_summary_from_transport_probe_rows_with_consistency_mode(
    requested_epoch: u64,
    dispatch_rows: &[SpireRemoteSearchLibpqDispatchPlanRow],
    transport_rows: &[SpireRemoteProductionTransportProbeRow],
    consistency_mode: &str,
) -> Result<SpireRemoteProductionExecutorStateSummaryRow, String> {
    let mut executor =
        SpireRemoteFanoutExecutor::from_libpq_dispatch_rows(requested_epoch, dispatch_rows);
    executor.apply_transport_probe_rows_with_consistency_mode(transport_rows, consistency_mode)?;
    executor.summary(
        "function_argument",
        consistency_mode_name(parse_remote_search_consistency_mode(consistency_mode)?),
    )
}

#[cfg(any(test, feature = "pg_test"))]
fn remote_search_production_executor_state_summary_from_candidate_receive_results(
    requested_epoch: u64,
    dispatch_rows: &[SpireRemoteSearchLibpqDispatchPlanRow],
    transport_rows: &[SpireRemoteProductionTransportProbeRow],
    candidate_receive_results: &[SpireRemoteProductionCandidateReceiveResult],
) -> Result<SpireRemoteProductionExecutorStateSummaryRow, String> {
    remote_search_production_executor_state_summary_from_candidate_receive_results_with_consistency_mode(
        requested_epoch,
        dispatch_rows,
        transport_rows,
        candidate_receive_results,
        "strict",
    )
}

#[cfg(any(test, feature = "pg_test"))]
fn remote_search_production_executor_state_summary_from_candidate_receive_results_with_consistency_mode(
    requested_epoch: u64,
    dispatch_rows: &[SpireRemoteSearchLibpqDispatchPlanRow],
    transport_rows: &[SpireRemoteProductionTransportProbeRow],
    candidate_receive_results: &[SpireRemoteProductionCandidateReceiveResult],
    consistency_mode: &str,
) -> Result<SpireRemoteProductionExecutorStateSummaryRow, String> {
    let mut executor =
        SpireRemoteFanoutExecutor::from_libpq_dispatch_rows(requested_epoch, dispatch_rows);
    executor.apply_transport_probe_rows_with_consistency_mode(transport_rows, consistency_mode)?;
    executor.apply_candidate_receive_results_with_consistency_mode(
        candidate_receive_results,
        consistency_mode,
    )?;
    executor.summary(
        "function_argument",
        consistency_mode_name(parse_remote_search_consistency_mode(consistency_mode)?),
    )
}

#[cfg(any(test, feature = "pg_test"))]
fn remote_search_production_degraded_skip_report_from_candidate_receive_results_with_consistency_mode(
    requested_epoch: u64,
    dispatch_rows: &[SpireRemoteSearchLibpqDispatchPlanRow],
    transport_rows: &[SpireRemoteProductionTransportProbeRow],
    candidate_receive_results: &[SpireRemoteProductionCandidateReceiveResult],
    consistency_mode: &str,
) -> Result<Vec<SpireRemoteProductionDegradedSkipReportRow>, String> {
    let mut executor =
        SpireRemoteFanoutExecutor::from_libpq_dispatch_rows(requested_epoch, dispatch_rows);
    executor.apply_transport_probe_rows_with_consistency_mode(transport_rows, consistency_mode)?;
    executor.apply_candidate_receive_results_with_consistency_mode(
        candidate_receive_results,
        consistency_mode,
    )?;
    executor.degraded_skip_report()
}

#[cfg(any(test, feature = "pg_test"))]
fn remote_search_production_compact_merge_from_candidate_receive_results(
    requested_epoch: u64,
    dispatch_rows: &[SpireRemoteSearchLibpqDispatchPlanRow],
    transport_rows: &[SpireRemoteProductionTransportProbeRow],
    candidate_receive_results: &[SpireRemoteProductionCandidateReceiveResult],
    limit: Option<usize>,
) -> Result<SpireRemoteSearchMergeResult, String> {
    remote_search_production_compact_merge_from_candidate_receive_results_with_consistency_mode(
        requested_epoch,
        dispatch_rows,
        transport_rows,
        candidate_receive_results,
        limit,
        "strict",
    )
}

#[cfg(any(test, feature = "pg_test"))]
fn remote_search_production_compact_merge_from_candidate_receive_results_with_consistency_mode(
    requested_epoch: u64,
    dispatch_rows: &[SpireRemoteSearchLibpqDispatchPlanRow],
    transport_rows: &[SpireRemoteProductionTransportProbeRow],
    candidate_receive_results: &[SpireRemoteProductionCandidateReceiveResult],
    limit: Option<usize>,
    consistency_mode: &str,
) -> Result<SpireRemoteSearchMergeResult, String> {
    let mut executor =
        SpireRemoteFanoutExecutor::from_libpq_dispatch_rows(requested_epoch, dispatch_rows);
    executor.apply_transport_probe_rows_with_consistency_mode(transport_rows, consistency_mode)?;
    executor.apply_candidate_receive_results_with_consistency_mode(
        candidate_receive_results,
        consistency_mode,
    )?;
    executor.merge_ready_candidate_batches(limit)
}

