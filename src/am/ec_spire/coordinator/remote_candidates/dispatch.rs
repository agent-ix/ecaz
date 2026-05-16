fn remote_search_libpq_dispatch_budget_blocked(
    row: &SpireRemoteSearchLibpqConnectionPlanRow,
    limits: SpireRemoteSearchLibpqExecutorBudgetLimits,
    admitted_node_count: u64,
    admitted_pid_count: u64,
) -> bool {
    if row.pipeline_mode != SPIRE_REMOTE_TRANSPORT_LIBPQ_PIPELINE {
        return false;
    }
    if limits.has_pid_per_node_cap() && row.pid_count > limits.max_pids_per_node {
        return true;
    }
    if limits.has_node_cap() && admitted_node_count >= limits.max_nodes {
        return true;
    }
    if limits.has_pid_cap() && admitted_pid_count.saturating_add(row.pid_count) > limits.max_pids {
        return true;
    }
    false
}

pub(crate) unsafe fn remote_search_libpq_dispatch_summary_row(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    query: Vec<f32>,
    selected_pids: Vec<u64>,
    top_k: usize,
    consistency_mode: &str,
) -> SpireRemoteSearchLibpqDispatchSummaryRow {
    let result = (|| -> Result<SpireRemoteSearchLibpqDispatchSummaryRow, String> {
        let query_for_empty_plan = query.clone();
        let top_k_for_empty_plan = u64::try_from(top_k)
            .map_err(|_| "ec_spire remote search libpq dispatch summary top_k exceeds u64")?;
        let rows = unsafe {
            remote_search_libpq_dispatch_plan_rows(
                index_relation,
                requested_epoch,
                query,
                selected_pids,
                top_k,
                consistency_mode,
            )
        };
        remote_search_libpq_dispatch_summary_from_plan_rows(
            requested_epoch,
            &rows,
            query_for_empty_plan,
            top_k_for_empty_plan,
            consistency_mode,
        )
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

fn remote_search_libpq_dispatch_summary_from_plan_rows(
    requested_epoch: u64,
    rows: &[SpireRemoteSearchLibpqDispatchPlanRow],
    query_for_empty_plan: Vec<f32>,
    top_k_for_empty_plan: u64,
    consistency_mode: &str,
) -> Result<SpireRemoteSearchLibpqDispatchSummaryRow, String> {
        let mut rollup = SpireRemoteCountRollup::default();
        let mut pipeline_dispatch_count = 0_u64;
        let mut missing_descriptor_dispatch_count = 0_u64;
        let mut query_dimension = 0_u64;
        let mut top_k = 0_u64;
        let mut parsed_consistency_mode = "";

        for row in rows {
            query_dimension = row.query_dimension;
            top_k = row.top_k;
            parsed_consistency_mode = row.consistency_mode;
            rollup.record_remote_target(row.pid_count, "remote search libpq dispatch summary")?;
            rollup.record_status(row.status, row.pid_count, "remote search libpq dispatch summary")?;
            if row.dispatch_action == SPIRE_REMOTE_DISPATCH_PIPELINE_ACTION {
                pipeline_dispatch_count = pipeline_dispatch_count.checked_add(1).ok_or_else(|| {
                    "ec_spire remote search libpq dispatch summary pipeline count overflow".to_owned()
                })?;
            }
            if row.status == SPIRE_REMOTE_STATUS_REQUIRES_DESCRIPTOR {
                missing_descriptor_dispatch_count =
                    missing_descriptor_dispatch_count.checked_add(1).ok_or_else(|| {
                        "ec_spire remote search libpq dispatch summary missing descriptor count overflow"
                            .to_owned()
                    })?;
            }
        }

        if rows.is_empty() {
            let query = scan::SpireScanQuery::new(query_for_empty_plan)?;
            query_dimension = u64::try_from(query.values().len()).map_err(|_| {
                "ec_spire remote search libpq dispatch summary query dimension exceeds u64"
            })?;
            top_k = top_k_for_empty_plan;
            parsed_consistency_mode =
                consistency_mode_name(parse_remote_search_consistency_mode(consistency_mode)?);
        }

        let dispatch_count = u64::try_from(rows.len()).map_err(|_| {
            "ec_spire remote search libpq dispatch summary dispatch count exceeds u64"
        })?;
        let status = rollup.summary_status(top_k, SpireRemoteSummaryStatusMode::LibpqRequest);

        Ok(SpireRemoteSearchLibpqDispatchSummaryRow {
            requested_epoch,
            dispatch_count,
            pipeline_dispatch_count,
            missing_descriptor_dispatch_count,
            remote_pid_count: rollup.remote_pid_count,
            blocked_pid_count: rollup.blocked_pid_count,
            query_dimension,
            top_k,
            consistency_mode: parsed_consistency_mode,
            status,
        })
}

pub(crate) unsafe fn remote_search_libpq_executor_budget_summary_row(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    query: Vec<f32>,
    selected_pids: Vec<u64>,
    top_k: usize,
    consistency_mode: &str,
) -> SpireRemoteSearchLibpqExecutorBudgetSummaryRow {
    let result = (|| -> Result<SpireRemoteSearchLibpqExecutorBudgetSummaryRow, String> {
        let rows = unsafe {
            remote_search_libpq_dispatch_plan_rows(
                index_relation,
                requested_epoch,
                query,
                selected_pids,
                top_k,
                consistency_mode,
            )
        };
        remote_search_libpq_executor_budget_summary_from_dispatch_rows(requested_epoch, &rows)
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

fn remote_search_libpq_executor_budget_summary_from_dispatch_rows(
    requested_epoch: u64,
    rows: &[SpireRemoteSearchLibpqDispatchPlanRow],
) -> Result<SpireRemoteSearchLibpqExecutorBudgetSummaryRow, String> {
    let limits = SpireRemoteSearchLibpqExecutorBudgetLimits::from_session();
    let mut admitted_dispatch_count = 0_u64;
    let mut budget_blocked_dispatch_count = 0_u64;
    let mut remote_pid_count = 0_u64;
    let mut admitted_pid_count = 0_u64;
    let mut budget_blocked_pid_count = 0_u64;

    for row in rows {
        add_remote_count(
            &mut remote_pid_count,
            row.pid_count,
            "remote search libpq executor budget summary",
            "remote PID",
        )?;
        if row.status == SPIRE_REMOTE_STATUS_EXECUTOR_OVERLOAD {
            add_remote_count(
                &mut budget_blocked_dispatch_count,
                1,
                "remote search libpq executor budget summary",
                "budget-blocked dispatch",
            )?;
            add_remote_count(
                &mut budget_blocked_pid_count,
                row.pid_count,
                "remote search libpq executor budget summary",
                "budget-blocked PID",
            )?;
        } else if row.dispatch_action == SPIRE_REMOTE_DISPATCH_PIPELINE_ACTION {
            add_remote_count(
                &mut admitted_dispatch_count,
                1,
                "remote search libpq executor budget summary",
                "admitted dispatch",
            )?;
            add_remote_count(
                &mut admitted_pid_count,
                row.pid_count,
                "remote search libpq executor budget summary",
                "admitted PID",
            )?;
        }
    }

    let dispatch_count = u64::try_from(rows.len())
        .map_err(|_| "remote search libpq executor budget dispatch count exceeds u64")?;
    let (next_executor_step, status, recommendation) = if budget_blocked_dispatch_count > 0 {
        (
            SPIRE_REMOTE_EXECUTOR_STEP_BUDGET,
            SPIRE_REMOTE_STATUS_EXECUTOR_OVERLOAD,
            remote_search_pre_dispatch_blocker_recommendation(
                SPIRE_REMOTE_STATUS_EXECUTOR_OVERLOAD,
            ),
        )
    } else {
        (SPIRE_REMOTE_NONE, SPIRE_REMOTE_STATUS_READY, SPIRE_REMOTE_NONE)
    };

    Ok(SpireRemoteSearchLibpqExecutorBudgetSummaryRow {
        requested_epoch,
        dispatch_count,
        admitted_dispatch_count,
        budget_blocked_dispatch_count,
        remote_pid_count,
        admitted_pid_count,
        budget_blocked_pid_count,
        max_nodes: limits.max_nodes,
        max_pids: limits.max_pids,
        max_pids_per_node: limits.max_pids_per_node,
        max_concurrent_dispatches: limits.max_concurrent_dispatches,
        max_concurrent_dispatches_per_node: limits.max_concurrent_dispatches_per_node,
        connect_timeout_ms: limits.connect_timeout_ms,
        statement_timeout_ms: limits.statement_timeout_ms,
        next_executor_step,
        status,
        recommendation,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpireRemoteProductionTransportProbeRequest {
    pub(crate) node_id: u32,
    pub(crate) conninfo: String,
    pub(crate) sql: &'static str,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SpireRemoteProductionCandidateReceiveRequest {
    pub(crate) node_id: u32,
    pub(crate) conninfo: String,
    pub(crate) remote_index_regclass: String,
    pub(crate) remote_index_identity: Vec<u8>,
    pub(crate) requested_epoch: u64,
    pub(crate) query: Vec<f32>,
    pub(crate) selected_pids: Vec<u64>,
    pub(crate) top_k: usize,
    pub(crate) consistency_mode: String,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SpireRemoteProductionCandidateReceiveResult {
    pub(crate) node_id: u32,
    pub(crate) started_after_ms: u64,
    pub(crate) completed_after_ms: u64,
    pub(crate) elapsed_ms: u64,
    pub(crate) candidate_count: u64,
    pub(crate) status: &'static str,
    pub(crate) failure_category: &'static str,
    pub(crate) batch: Option<SpireRemoteSearchCandidateBatch>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SpireRemoteProductionHeapReceiveRequest {
    pub(crate) node_id: u32,
    pub(crate) conninfo: String,
    pub(crate) remote_index_regclass: String,
    pub(crate) remote_index_identity: Vec<u8>,
    pub(crate) requested_epoch: u64,
    pub(crate) query: Vec<f32>,
    pub(crate) selected_pids: Vec<u64>,
    pub(crate) top_k: usize,
    pub(crate) consistency_mode: String,
    pub(crate) tuple_payload_columns: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SpireRemoteProductionHeapReceiveResult {
    pub(crate) node_id: u32,
    pub(crate) started_after_ms: u64,
    pub(crate) completed_after_ms: u64,
    pub(crate) elapsed_ms: u64,
    pub(crate) candidate_count: u64,
    pub(crate) status: &'static str,
    pub(crate) failure_category: &'static str,
    pub(crate) candidates: Vec<SpireRemoteSearchLocalHeapCandidateRow>,
}

#[derive(Debug, Clone, PartialEq)]
struct SpireRemoteProductionCandidateAndHeapResult {
    candidate_results: Vec<SpireRemoteProductionCandidateReceiveResult>,
    heap_results: Vec<SpireRemoteProductionHeapReceiveResult>,
    metrics: SpireRemoteProductionReadMetrics,
}

struct SpireRemoteProductionCandidateSession {
    request: SpireRemoteProductionCandidateReceiveRequest,
    _governance_permit: SpireRemoteSearchLibpqGovernancePermit,
    client: tokio_postgres::Client,
    connection_task: tokio::task::JoinHandle<()>,
    tls_config: SpireRemoteTlsConfig,
    remote_index_oid: u32,
    endpoint_identity: SpireRemoteValidatedEndpointIdentity,
    selected_pids: Vec<i64>,
    requested_epoch: i64,
    top_k: i32,
    started_after_ms: u64,
    request_start: std::time::Instant,
}

struct SpireRemoteProductionCandidateSessionResult {
    candidate_result: SpireRemoteProductionCandidateReceiveResult,
    session: Option<SpireRemoteProductionCandidateSession>,
    metrics: SpireRemoteProductionReadMetrics,
}

struct SpireRemoteProductionHeapSessionResult {
    heap_result: SpireRemoteProductionHeapReceiveResult,
    metrics: SpireRemoteProductionReadMetrics,
}

#[derive(Debug, Clone, PartialEq)]
struct SpireCoordinatorInsertRemotePrepareRequest {
    node_id: u32,
    conninfo: String,
    remote_index_regclass: String,
    remote_sql: String,
    prepared_gid: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SpireCoordinatorInsertRemotePrepareResult {
    node_id: u32,
    conninfo: String,
    prepared_gid: String,
    remote_index_identity: Vec<u8>,
    remote_last_served_epoch: u64,
    remote_extension_version: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SpireCoordinatorInsertAsyncStep<T> {
    value: T,
    local_cancel_observed: bool,
}

struct SpireRemoteProductionTransportAdapter;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SpireRemoteLocalCancelSource {
    None,
    TestAfterMs(u64),
    PostgresInterruptPoll { poll_interval_ms: u64 },
}

const SPIRE_REMOTE_POSTGRES_INTERRUPT_POLL_MS: u64 = 5;

impl SpireRemoteLocalCancelSource {
    fn production() -> Self {
        Self::PostgresInterruptPoll {
            poll_interval_ms: SPIRE_REMOTE_POSTGRES_INTERRUPT_POLL_MS,
        }
    }

    fn test_after_ms(delay_ms: u64) -> Self {
        Self::TestAfterMs(delay_ms)
    }
}

impl SpireRemoteProductionTransportAdapter {
    fn run_probe_requests(
        requests: Vec<SpireRemoteProductionTransportProbeRequest>,
    ) -> Result<Vec<SpireRemoteProductionTransportProbeRow>, String> {
        Self::run_probe_requests_with_local_cancel_source(
            requests,
            SpireRemoteLocalCancelSource::production(),
        )
    }

    fn run_probe_requests_with_local_cancel(
        requests: Vec<SpireRemoteProductionTransportProbeRequest>,
        local_cancel_after_ms: Option<u64>,
    ) -> Result<Vec<SpireRemoteProductionTransportProbeRow>, String> {
        let cancel_source = local_cancel_after_ms
            .map(SpireRemoteLocalCancelSource::test_after_ms)
            .unwrap_or(SpireRemoteLocalCancelSource::None);
        Self::run_probe_requests_with_local_cancel_source(requests, cancel_source)
    }

    fn run_probe_requests_with_local_cancel_source(
        requests: Vec<SpireRemoteProductionTransportProbeRequest>,
        local_cancel_source: SpireRemoteLocalCancelSource,
    ) -> Result<Vec<SpireRemoteProductionTransportProbeRow>, String> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_io()
            .enable_time()
            .build()
            .map_err(|_| {
                "ec_spire production transport adapter failed to build runtime".to_owned()
            })?;

        runtime.block_on(async move {
            let batch_start = std::time::Instant::now();
            let futures = requests.into_iter().map(|request| async move {
                Self::run_one_probe_request(request, batch_start, local_cancel_source).await
            });
            Ok(futures_util::future::join_all(futures).await)
        })
    }

    fn run_candidate_receive_requests(
        requests: Vec<SpireRemoteProductionCandidateReceiveRequest>,
    ) -> Result<Vec<SpireRemoteProductionCandidateReceiveResult>, String> {
        Self::run_candidate_receive_requests_with_local_cancel_source(
            requests,
            SpireRemoteLocalCancelSource::production(),
        )
    }

    fn run_candidate_receive_requests_with_local_cancel(
        requests: Vec<SpireRemoteProductionCandidateReceiveRequest>,
        local_cancel_after_ms: Option<u64>,
    ) -> Result<Vec<SpireRemoteProductionCandidateReceiveResult>, String> {
        let cancel_source = local_cancel_after_ms
            .map(SpireRemoteLocalCancelSource::test_after_ms)
            .unwrap_or(SpireRemoteLocalCancelSource::None);
        Self::run_candidate_receive_requests_with_local_cancel_source(requests, cancel_source)
    }

    fn run_candidate_receive_requests_with_local_cancel_source(
        requests: Vec<SpireRemoteProductionCandidateReceiveRequest>,
        local_cancel_source: SpireRemoteLocalCancelSource,
    ) -> Result<Vec<SpireRemoteProductionCandidateReceiveResult>, String> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_io()
            .enable_time()
            .build()
            .map_err(|_| {
                "ec_spire production transport adapter failed to build runtime".to_owned()
            })?;

        runtime.block_on(async move {
            let batch_start = std::time::Instant::now();
            let futures = requests.into_iter().map(|request| async move {
                Self::run_one_candidate_receive_request(
                    request,
                    batch_start,
                    local_cancel_source,
                )
                .await
            });
            Ok(futures_util::future::join_all(futures).await)
        })
    }

    fn run_candidate_and_heap_receive_requests(
        requests: Vec<SpireRemoteProductionCandidateReceiveRequest>,
        tuple_payload_columns: Option<Vec<String>>,
        consistency_mode: &str,
    ) -> Result<SpireRemoteProductionCandidateAndHeapResult, String> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_io()
            .enable_time()
            .build()
            .map_err(|_| {
                "ec_spire production candidate/heap receive adapter failed to build runtime"
                    .to_owned()
            })?;
        let consistency_mode = consistency_mode.to_owned();

        runtime.block_on(async move {
            let batch_start = std::time::Instant::now();
            let futures = requests.into_iter().map(|request| async move {
                Self::run_one_candidate_session_request(
                    request,
                    batch_start,
                    SpireRemoteLocalCancelSource::production(),
                )
                .await
            });
            let mut session_results = futures_util::future::join_all(futures).await;
            let mut metrics = SpireRemoteProductionReadMetrics::default();
            let candidate_results = session_results
                .iter()
                .map(|result| result.candidate_result.clone())
                .collect::<Vec<_>>();
            for result in &session_results {
                metrics.add_transport_metrics(&result.metrics);
            }

            let allow_heap = Self::candidate_results_allow_heap(
                &candidate_results,
                consistency_mode.as_str(),
            )?;
            let mut heap_results = Vec::new();
            if allow_heap {
                let futures = session_results
                    .drain(..)
                    .filter_map(|session_result| session_result.session)
                    .map(|session| {
                        let tuple_payload_columns = tuple_payload_columns.clone();
                        let consistency_mode = consistency_mode.clone();
                        async move {
                            Self::run_heap_receive_on_candidate_session(
                                session,
                                tuple_payload_columns.as_deref(),
                                consistency_mode.as_str(),
                                batch_start,
                            )
                            .await
                        }
                    });
                for heap_result in futures_util::future::join_all(futures).await {
                    metrics.add_transport_metrics(&heap_result.metrics);
                    heap_results.push(heap_result.heap_result);
                }
            } else {
                for session_result in session_results.drain(..) {
                    if let Some(session) = session_result.session {
                        session.connection_task.abort();
                    }
                }
            }

            Ok(SpireRemoteProductionCandidateAndHeapResult {
                candidate_results,
                heap_results,
                metrics,
            })
        })
    }

    fn run_heap_receive_requests(
        requests: Vec<SpireRemoteProductionHeapReceiveRequest>,
    ) -> Result<Vec<SpireRemoteProductionHeapReceiveResult>, String> {
        Self::run_heap_receive_requests_with_local_cancel_source(
            requests,
            SpireRemoteLocalCancelSource::production(),
        )
    }

    fn run_heap_receive_requests_with_local_cancel_source(
        requests: Vec<SpireRemoteProductionHeapReceiveRequest>,
        local_cancel_source: SpireRemoteLocalCancelSource,
    ) -> Result<Vec<SpireRemoteProductionHeapReceiveResult>, String> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_io()
            .enable_time()
            .build()
            .map_err(|_| {
                "ec_spire production heap receive adapter failed to build runtime".to_owned()
            })?;

        runtime.block_on(async move {
            let batch_start = std::time::Instant::now();
            let futures = requests.into_iter().map(|request| async move {
                Self::run_one_heap_receive_request(request, batch_start, local_cancel_source).await
            });
            Ok(futures_util::future::join_all(futures).await)
        })
    }

    fn run_insert_prepare_requests(
        requests: Vec<SpireCoordinatorInsertRemotePrepareRequest>,
    ) -> Result<Vec<SpireCoordinatorInsertRemotePrepareResult>, String> {
        Self::run_insert_prepare_requests_with_local_cancel_source(
            requests,
            SpireRemoteLocalCancelSource::production(),
        )
    }

    fn run_insert_prepare_requests_with_local_cancel_source(
        requests: Vec<SpireCoordinatorInsertRemotePrepareRequest>,
        local_cancel_source: SpireRemoteLocalCancelSource,
    ) -> Result<Vec<SpireCoordinatorInsertRemotePrepareResult>, String> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_io()
            .enable_time()
            .build()
            .map_err(|_| {
                "ec_spire coordinator insert prepare adapter failed to build runtime".to_owned()
            })?;

        runtime.block_on(async move {
            let futures = requests.into_iter().map(|request| async move {
                Self::run_one_insert_prepare_request(request, local_cancel_source).await
            });
            let results = futures_util::future::join_all(futures).await;
            let mut prepared_rows = Vec::new();
            let mut first_error = None;
            for result in results {
                match result {
                    Ok(row) => prepared_rows.push(row),
                    Err(error) if first_error.is_none() => first_error = Some(error),
                    Err(_) => {}
                }
            }
            if let Some(error) = first_error {
                for row in &prepared_rows {
                    coordinator_insert_resolve_remote_prepared(
                        row.conninfo.clone(),
                        row.node_id,
                        row.prepared_gid.clone(),
                        false,
                    );
                }
                Err(error)
            } else {
                Ok(prepared_rows)
            }
        })
    }

    async fn run_one_probe_request(
        request: SpireRemoteProductionTransportProbeRequest,
        batch_start: std::time::Instant,
        local_cancel_source: SpireRemoteLocalCancelSource,
    ) -> SpireRemoteProductionTransportProbeRow {
        let started_after_ms = elapsed_millis_u64(batch_start);
        let request_start = std::time::Instant::now();
        let _governance_permit =
            match remote_search_libpq_executor_governance_permit_for_node(request.node_id) {
                Ok(permit) => permit,
                Err(error) => {
                    return failed_production_transport_probe_row(
                        request.node_id,
                        batch_start,
                        request_start,
                        production_governance_failure_category(&error),
                    );
                }
            };
        let limits = SpireRemoteSearchLibpqExecutorBudgetLimits::from_session();
        let SpireRemoteAsyncConnection {
            client,
            connection_task,
            tls_config,
        } = match remote_search_libpq_connect_async_with_session_timeouts(
            &request.conninfo,
            request.node_id,
            "production transport probe",
        )
        .await
        {
            Ok(connection) => connection,
            Err(error) => {
                return failed_production_transport_probe_row(
                    request.node_id,
                    batch_start,
                    request_start,
                    error.category,
                );
            }
        };

        let cancel_token = client.cancel_token();
        let query_result = Self::run_query_with_optional_local_cancel(
            cancel_token,
            tls_config,
            async {
                if limits.statement_timeout_ms > 0 {
                    client
                        .batch_execute(&format!(
                            "SET statement_timeout = {}",
                            limits.statement_timeout_ms
                        ))
                        .await
                        .map_err(|_| {
                            SPIRE_REMOTE_PRODUCTION_TRANSPORT_STATEMENT_TIMEOUT_SETUP_FAILED
                        })?;
                }
                client
                    .simple_query(request.sql)
                    .await
                    .map_err(|error| production_remote_query_failure_category(&error))
            },
            local_cancel_source,
        )
        .await;

        connection_task.abort();
        let completed_after_ms = elapsed_millis_u64(batch_start);
        let elapsed_ms = elapsed_millis_u64(request_start);
        match query_result {
            Ok(messages) => SpireRemoteProductionTransportProbeRow {
                node_id: request.node_id,
                started_after_ms,
                completed_after_ms,
                elapsed_ms,
                row_count: u64::try_from(messages.len()).unwrap_or(u64::MAX),
                status: SPIRE_REMOTE_STATUS_READY,
                failure_category: SPIRE_REMOTE_NONE,
            },
            Err(failure_category) => failed_production_transport_probe_row(
                request.node_id,
                batch_start,
                request_start,
                failure_category,
            ),
        }
    }

    fn candidate_results_allow_heap(
        results: &[SpireRemoteProductionCandidateReceiveResult],
        consistency_mode: &str,
    ) -> Result<bool, String> {
        if results.iter().any(|result| {
            is_local_cancellation_failure_category(result.failure_category)
        }) {
            return Ok(false);
        }
        let ready_count = results
            .iter()
            .filter(|result| result.status == SPIRE_REMOTE_STATUS_READY)
            .count();
        if ready_count == 0 {
            return Ok(false);
        }
        let failed_count = results
            .iter()
            .filter(|result| result.status != SPIRE_REMOTE_STATUS_READY)
            .count();
        let degraded =
            parse_remote_search_consistency_mode(consistency_mode)? == meta::SpireConsistencyMode::Degraded;
        Ok(degraded || failed_count == 0)
    }

    fn candidate_request_parameters(
        request: &SpireRemoteProductionCandidateReceiveRequest,
    ) -> Result<(Vec<i64>, i64, i32), &'static str> {
        validate_remote_payload_batch_row_count(
            request.selected_pids.len(),
            "remote candidate receive selected_pids",
        )
        .map_err(|_| SPIRE_REMOTE_STATUS_REMOTE_PAYLOAD_TOO_LARGE)?;
        let selected_pids = request
            .selected_pids
            .iter()
            .map(|pid| i64::try_from(*pid))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| SPIRE_REMOTE_PRODUCTION_CANDIDATE_INVALID_PARAMETERS)?;
        let requested_epoch = i64::try_from(request.requested_epoch)
            .map_err(|_| SPIRE_REMOTE_PRODUCTION_CANDIDATE_INVALID_PARAMETERS)?;
        let top_k = i32::try_from(request.top_k)
            .map_err(|_| SPIRE_REMOTE_PRODUCTION_CANDIDATE_INVALID_PARAMETERS)?;
        Ok((selected_pids, requested_epoch, top_k))
    }

    async fn run_one_candidate_session_request(
        request: SpireRemoteProductionCandidateReceiveRequest,
        batch_start: std::time::Instant,
        local_cancel_source: SpireRemoteLocalCancelSource,
    ) -> SpireRemoteProductionCandidateSessionResult {
        let mut metrics = SpireRemoteProductionReadMetrics::default();
        let started_after_ms = elapsed_millis_u64(batch_start);
        let request_start = std::time::Instant::now();
        let (selected_pids, requested_epoch, top_k) =
            match Self::candidate_request_parameters(&request) {
                Ok(parameters) => parameters,
                Err(failure_category) => {
                    metrics.record_failure_category(&request.consistency_mode, failure_category);
                    return SpireRemoteProductionCandidateSessionResult {
                        candidate_result: failed_production_candidate_receive_result(
                            request.node_id,
                            batch_start,
                            request_start,
                            failure_category,
                        ),
                        session: None,
                        metrics,
                    };
                }
            };
        let governance_permit =
            match remote_search_libpq_executor_governance_permit_for_node(request.node_id) {
                Ok(permit) => permit,
                Err(error) => {
                    let failure_category = production_governance_failure_category(&error);
                    metrics.record_failure_category(&request.consistency_mode, failure_category);
                    return SpireRemoteProductionCandidateSessionResult {
                        candidate_result: failed_production_candidate_receive_result(
                            request.node_id,
                            batch_start,
                            request_start,
                            failure_category,
                        ),
                        session: None,
                        metrics,
                    };
                }
            };
        let limits = SpireRemoteSearchLibpqExecutorBudgetLimits::from_session();
        let connect_start = std::time::Instant::now();
        let SpireRemoteAsyncConnection {
            client,
            connection_task,
            tls_config,
        } = match remote_search_libpq_connect_async_with_session_timeouts(
            &request.conninfo,
            request.node_id,
            "production candidate/heap receive",
        )
        .await
        {
            Ok(connection) => {
                add_profile_elapsed(&mut metrics.connect_elapsed_ms, connect_start);
                add_profile_count(&mut metrics.socket_open_count, 1);
                metrics.record_tls_config(&connection.tls_config);
                connection
            }
            Err(error) => {
                add_profile_elapsed(&mut metrics.connect_elapsed_ms, connect_start);
                metrics.record_failure_category(&request.consistency_mode, error.category);
                return SpireRemoteProductionCandidateSessionResult {
                    candidate_result: failed_production_candidate_receive_result(
                        request.node_id,
                        batch_start,
                        request_start,
                        error.category,
                    ),
                    session: None,
                    metrics,
                };
            }
        };

        let cancel_token = client.cancel_token();
        let cancel_tls_config = tls_config.clone();
        let result_rows = Self::run_query_with_optional_local_cancel(
            cancel_token,
            cancel_tls_config,
            async {
                let mut query_metrics = SpireRemoteProductionReadMetrics::default();
                if limits.statement_timeout_ms > 0 {
                    let timeout_start = std::time::Instant::now();
                    add_profile_count(&mut query_metrics.statement_timeout_setup_count, 1);
                    client
                        .batch_execute(&format!(
                            "SET statement_timeout = {}",
                            limits.statement_timeout_ms
                        ))
                        .await
                        .map_err(|_| {
                            SPIRE_REMOTE_PRODUCTION_TRANSPORT_STATEMENT_TIMEOUT_SETUP_FAILED
                        })?;
                    add_profile_elapsed(
                        &mut query_metrics.statement_timeout_setup_elapsed_ms,
                        timeout_start,
                    );
                }
                let regclass_start = std::time::Instant::now();
                add_profile_count(&mut query_metrics.regclass_probe_count, 1);
                let remote_index_oid = client
                    .query_one(
                        "SELECT to_regclass($1)::oid",
                        &[&request.remote_index_regclass.as_str()],
                    )
                    .await
                    .map_err(|error| {
                        let category = production_remote_query_failure_category(&error);
                        if category == SPIRE_REMOTE_PRODUCTION_TRANSPORT_REMOTE_QUERY_FAILED {
                            SPIRE_REMOTE_PRODUCTION_REMOTE_INDEX_UNAVAILABLE
                        } else {
                            category
                        }
                    })?
                    .try_get::<_, Option<u32>>(0)
                    .map_err(|_| SPIRE_REMOTE_PRODUCTION_REMOTE_INDEX_UNAVAILABLE)?
                    .ok_or(SPIRE_REMOTE_PRODUCTION_REMOTE_INDEX_UNAVAILABLE)?;
                add_profile_elapsed(&mut query_metrics.regclass_probe_elapsed_ms, regclass_start);

                let identity_start = std::time::Instant::now();
                add_profile_count(&mut query_metrics.endpoint_identity_query_count, 1);
                let endpoint_identity_row = client
                    .query_one(
                        SPIRE_REMOTE_SEARCH_ENDPOINT_IDENTITY_SQL_TEMPLATE,
                        &[&remote_index_oid],
                    )
                    .await
                    .map_err(|_| SPIRE_REMOTE_STATUS_ENDPOINT_IDENTITY_MISMATCH)?;
                let endpoint_identity =
                    validate_remote_search_endpoint_identity_row(&endpoint_identity_row)
                        .map_err(|_| SPIRE_REMOTE_STATUS_ENDPOINT_IDENTITY_MISMATCH)?;
                if endpoint_identity.profile_fingerprint_bytes.as_slice()
                    != request.remote_index_identity.as_slice()
                {
                    return Err(SPIRE_REMOTE_STATUS_ENDPOINT_IDENTITY_MISMATCH);
                }
                add_profile_elapsed(
                    &mut query_metrics.endpoint_identity_elapsed_ms,
                    identity_start,
                );

                let candidate_start = std::time::Instant::now();
                add_profile_count(&mut query_metrics.candidate_receive_query_count, 1);
                let result_rows = client
                    .query(
                        SPIRE_REMOTE_SEARCH_LIBPQ_SQL_TEMPLATE,
                        &[
                            &remote_index_oid,
                            &requested_epoch,
                            &request.query,
                            &selected_pids,
                            &top_k,
                            &request.consistency_mode,
                        ],
                    )
                    .await
                    .map_err(|error| production_remote_query_failure_category(&error))?;
                add_profile_elapsed(
                    &mut query_metrics.candidate_receive_elapsed_ms,
                    candidate_start,
                );

                Ok((result_rows, remote_index_oid, endpoint_identity, query_metrics))
            },
            local_cancel_source,
        )
        .await;

        let (result_rows, remote_index_oid, endpoint_identity, query_metrics) = match result_rows {
            Ok(value) => value,
            Err(failure_category) => {
                connection_task.abort();
                metrics.record_failure_category(&request.consistency_mode, failure_category);
                return SpireRemoteProductionCandidateSessionResult {
                    candidate_result: failed_production_candidate_receive_result(
                        request.node_id,
                        batch_start,
                        request_start,
                        failure_category,
                    ),
                    session: None,
                    metrics,
                };
            }
        };
        metrics.add_transport_metrics(&query_metrics);

        if validate_remote_payload_batch_row_count(
            result_rows.len(),
            "remote candidate receive result rows",
        )
        .is_err()
        {
            connection_task.abort();
            let failure_category = SPIRE_REMOTE_STATUS_REMOTE_PAYLOAD_TOO_LARGE;
            metrics.record_failure_category(&request.consistency_mode, failure_category);
            return SpireRemoteProductionCandidateSessionResult {
                candidate_result: failed_production_candidate_receive_result(
                    request.node_id,
                    batch_start,
                    request_start,
                    failure_category,
                ),
                session: None,
                metrics,
            };
        }
        let candidates = match result_rows
            .iter()
            .map(|candidate_row| {
                decode_remote_search_candidate_pg_row(
                    candidate_row,
                    request.node_id,
                    true,
                    Some(&request.remote_index_identity),
                )
            })
            .collect::<Result<Vec<_>, _>>()
        {
            Ok(candidates) => candidates,
            Err(error) => {
                connection_task.abort();
                let failure_category = production_candidate_decode_failure_category(&error);
                metrics.record_failure_category(&request.consistency_mode, failure_category);
                return SpireRemoteProductionCandidateSessionResult {
                    candidate_result: failed_production_candidate_receive_result(
                        request.node_id,
                        batch_start,
                        request_start,
                        failure_category,
                    ),
                    session: None,
                    metrics,
                };
            }
        };
        if let Err(error) = validate_remote_search_candidate_batch(
            request.requested_epoch,
            request.node_id,
            &request.selected_pids,
            &candidates,
        ) {
            connection_task.abort();
            let failure_category = production_candidate_validation_failure_category(&error);
            metrics.record_failure_category(&request.consistency_mode, failure_category);
            return SpireRemoteProductionCandidateSessionResult {
                candidate_result: failed_production_candidate_receive_result(
                    request.node_id,
                    batch_start,
                    request_start,
                    failure_category,
                ),
                session: None,
                metrics,
            };
        }
        let candidate_count = u64::try_from(candidates.len()).unwrap_or(u64::MAX);
        let candidate_result = SpireRemoteProductionCandidateReceiveResult {
            node_id: request.node_id,
            started_after_ms,
            completed_after_ms: elapsed_millis_u64(batch_start),
            elapsed_ms: elapsed_millis_u64(request_start),
            candidate_count,
            status: SPIRE_REMOTE_STATUS_READY,
            failure_category: SPIRE_REMOTE_NONE,
            batch: Some(SpireRemoteSearchCandidateBatch {
                node_id: request.node_id,
                selected_pids: request.selected_pids.clone(),
                candidates,
            }),
        };
        let session = SpireRemoteProductionCandidateSession {
            request,
            _governance_permit: governance_permit,
            client,
            connection_task,
            tls_config,
            remote_index_oid,
            endpoint_identity,
            selected_pids,
            requested_epoch,
            top_k,
            started_after_ms,
            request_start,
        };
        SpireRemoteProductionCandidateSessionResult {
            candidate_result,
            session: Some(session),
            metrics,
        }
    }

    async fn run_heap_receive_on_candidate_session(
        session: SpireRemoteProductionCandidateSession,
        tuple_payload_columns: Option<&[String]>,
        consistency_mode: &str,
        batch_start: std::time::Instant,
    ) -> SpireRemoteProductionHeapSessionResult {
        let mut metrics = SpireRemoteProductionReadMetrics::default();
        let SpireRemoteProductionCandidateSession {
            request,
            _governance_permit,
            client,
            connection_task,
            tls_config,
            remote_index_oid,
            endpoint_identity,
            selected_pids,
            requested_epoch,
            top_k,
            started_after_ms,
            request_start,
        } = session;
        let cancel_token = client.cancel_token();
        let cancel_tls_config = tls_config.clone();
        let result_rows = Self::run_query_with_optional_local_cancel(
            cancel_token,
            cancel_tls_config,
            async {
                let mut query_metrics = SpireRemoteProductionReadMetrics::default();
                let heap_start = std::time::Instant::now();
                add_profile_count(&mut query_metrics.heap_receive_query_count, 1);
                let result = match tuple_payload_columns {
                    Some(tuple_payload_columns) => {
                        let sql = remote_tuple_payload_production_sql(&endpoint_identity)?;
                        client
                            .query(
                                sql,
                                &[
                                    &remote_index_oid,
                                    &requested_epoch,
                                    &request.query,
                                    &selected_pids,
                                    &top_k,
                                    &request.consistency_mode,
                                    &tuple_payload_columns,
                                ],
                            )
                            .await
                            .map_err(|error| production_remote_query_failure_category(&error))
                    }
                    None => {
                        client
                            .query(
                                SPIRE_REMOTE_SEARCH_LIBPQ_HEAP_SQL_TEMPLATE,
                                &[
                                    &remote_index_oid,
                                    &requested_epoch,
                                    &request.query,
                                    &selected_pids,
                                    &top_k,
                                    &request.consistency_mode,
                                ],
                            )
                            .await
                            .map_err(|error| production_remote_query_failure_category(&error))
                    }
                }?;
                add_profile_elapsed(&mut query_metrics.heap_receive_elapsed_ms, heap_start);
                Ok((result, query_metrics))
            },
            SpireRemoteLocalCancelSource::production(),
        )
        .await;

        connection_task.abort();
        let result_rows = match result_rows {
            Ok((rows, query_metrics)) => {
                metrics.add_transport_metrics(&query_metrics);
                rows
            }
            Err(failure_category) => {
                metrics.record_failure_category(consistency_mode, failure_category);
                return SpireRemoteProductionHeapSessionResult {
                    heap_result: failed_production_heap_receive_result(
                        request.node_id,
                        batch_start,
                        request_start,
                        failure_category,
                    ),
                    metrics,
                };
            }
        };
        if validate_remote_payload_batch_row_count(result_rows.len(), "remote heap result rows")
            .is_err()
        {
            let failure_category = SPIRE_REMOTE_STATUS_REMOTE_PAYLOAD_TOO_LARGE;
            metrics.record_failure_category(consistency_mode, failure_category);
            return SpireRemoteProductionHeapSessionResult {
                heap_result: failed_production_heap_receive_result(
                    request.node_id,
                    batch_start,
                    request_start,
                    failure_category,
                ),
                metrics,
            };
        }
        let decode_start = std::time::Instant::now();
        let candidates = match result_rows
            .iter()
            .map(|candidate_row| {
                decode_remote_search_heap_candidate_pg_row(
                    candidate_row,
                    request.requested_epoch,
                    request.node_id,
                )
            })
            .collect::<Result<Vec<_>, _>>()
        {
            Ok(candidates) => candidates,
            Err(error) => {
                pgrx::warning!("ec_spire remote heap receive decode failed: {error}");
                let failure_category = production_remote_heap_decode_failure_category(&error);
                metrics.record_failure_category(consistency_mode, failure_category);
                return SpireRemoteProductionHeapSessionResult {
                    heap_result: failed_production_heap_receive_result(
                        request.node_id,
                        batch_start,
                        request_start,
                        failure_category,
                    ),
                    metrics,
                };
            }
        };
        add_profile_elapsed(&mut metrics.payload_decode_elapsed_ms, decode_start);
        add_profile_count(
            &mut metrics.payload_decode_row_count,
            u64::try_from(candidates.len()).unwrap_or(u64::MAX),
        );
        for candidate in &candidates {
            if let Some(payload) = candidate.typed_tuple_payload.as_ref() {
                let row_bytes = payload
                    .payload_values
                    .iter()
                    .map(Vec::len)
                    .try_fold(0_u64, |acc, len| {
                        u64::try_from(len)
                            .ok()
                            .and_then(|len| acc.checked_add(len))
                    })
                    .unwrap_or(u64::MAX);
                add_profile_count(&mut metrics.payload_decode_bytes, row_bytes);
            }
            if let Some(payload_json) = candidate.tuple_payload_json.as_ref() {
                add_profile_count(
                    &mut metrics.payload_decode_bytes,
                    u64::try_from(payload_json.len()).unwrap_or(u64::MAX),
                );
            }
        }
        let merge_candidates = candidates
            .iter()
            .map(|candidate| SpireRemoteSearchCandidateRow {
                served_epoch: candidate.served_epoch,
                node_id: candidate.node_id,
                pid: candidate.pid,
                object_version: candidate.object_version,
                row_index: candidate.row_index,
                assignment_flags: candidate.assignment_flags,
                vec_id: candidate.vec_id.clone(),
                row_locator: candidate.row_locator.clone(),
                score: candidate.score,
            })
            .collect::<Vec<_>>();
        if let Err(error) = validate_remote_search_candidate_batch(
            request.requested_epoch,
            request.node_id,
            &request.selected_pids,
            &merge_candidates,
        ) {
            let failure_category = production_remote_heap_decode_failure_category(&error);
            metrics.record_failure_category(consistency_mode, failure_category);
            return SpireRemoteProductionHeapSessionResult {
                heap_result: failed_production_heap_receive_result(
                    request.node_id,
                    batch_start,
                    request_start,
                    failure_category,
                ),
                metrics,
            };
        }
        let candidate_count = u64::try_from(candidates.len()).unwrap_or(u64::MAX);
        SpireRemoteProductionHeapSessionResult {
            heap_result: SpireRemoteProductionHeapReceiveResult {
                node_id: request.node_id,
                started_after_ms,
                completed_after_ms: elapsed_millis_u64(batch_start),
                elapsed_ms: elapsed_millis_u64(request_start),
                candidate_count,
                status: SPIRE_REMOTE_STATUS_READY,
                failure_category: SPIRE_REMOTE_NONE,
                candidates,
            },
            metrics,
        }
    }

    async fn run_one_candidate_receive_request(
        request: SpireRemoteProductionCandidateReceiveRequest,
        batch_start: std::time::Instant,
        local_cancel_source: SpireRemoteLocalCancelSource,
    ) -> SpireRemoteProductionCandidateReceiveResult {
        let started_after_ms = elapsed_millis_u64(batch_start);
        let request_start = std::time::Instant::now();
        if validate_remote_payload_batch_row_count(
            request.selected_pids.len(),
            "remote candidate receive selected_pids",
        )
        .is_err()
        {
            return failed_production_candidate_receive_result(
                request.node_id,
                batch_start,
                request_start,
                SPIRE_REMOTE_STATUS_REMOTE_PAYLOAD_TOO_LARGE,
            );
        }
        let selected_pids = match request
            .selected_pids
            .iter()
            .map(|pid| i64::try_from(*pid))
            .collect::<Result<Vec<_>, _>>()
        {
            Ok(selected_pids) => selected_pids,
            Err(_) => {
                return failed_production_candidate_receive_result(
                    request.node_id,
                    batch_start,
                    request_start,
                    SPIRE_REMOTE_PRODUCTION_CANDIDATE_INVALID_PARAMETERS,
                );
            }
        };
        let requested_epoch = match i64::try_from(request.requested_epoch) {
            Ok(requested_epoch) => requested_epoch,
            Err(_) => {
                return failed_production_candidate_receive_result(
                    request.node_id,
                    batch_start,
                    request_start,
                    SPIRE_REMOTE_PRODUCTION_CANDIDATE_INVALID_PARAMETERS,
                );
            }
        };
        let top_k = match i32::try_from(request.top_k) {
            Ok(top_k) => top_k,
            Err(_) => {
                return failed_production_candidate_receive_result(
                    request.node_id,
                    batch_start,
                    request_start,
                    SPIRE_REMOTE_PRODUCTION_CANDIDATE_INVALID_PARAMETERS,
                );
            }
        };
        let _governance_permit =
            match remote_search_libpq_executor_governance_permit_for_node(request.node_id) {
                Ok(permit) => permit,
                Err(error) => {
                    return failed_production_candidate_receive_result(
                        request.node_id,
                        batch_start,
                        request_start,
                        production_governance_failure_category(&error),
                    );
                }
            };
        let limits = SpireRemoteSearchLibpqExecutorBudgetLimits::from_session();
        let SpireRemoteAsyncConnection {
            client,
            connection_task,
            tls_config,
        } = match remote_search_libpq_connect_async_with_session_timeouts(
            &request.conninfo,
            request.node_id,
            "production candidate receive",
        )
        .await
        {
            Ok(connection) => connection,
            Err(error) => {
                return failed_production_candidate_receive_result(
                    request.node_id,
                    batch_start,
                    request_start,
                    error.category,
                );
            }
        };

        let cancel_token = client.cancel_token();
        let result_rows = Self::run_query_with_optional_local_cancel(
            cancel_token,
            tls_config,
            async {
                if limits.statement_timeout_ms > 0 {
                    client
                        .batch_execute(&format!(
                            "SET statement_timeout = {}",
                            limits.statement_timeout_ms
                        ))
                        .await
                        .map_err(|_| {
                            SPIRE_REMOTE_PRODUCTION_TRANSPORT_STATEMENT_TIMEOUT_SETUP_FAILED
                        })?;
                }
                let remote_index_oid = client
                    .query_one(
                        "SELECT to_regclass($1)::oid",
                        &[&request.remote_index_regclass.as_str()],
                    )
                    .await
                    .map_err(|error| {
                        let category = production_remote_query_failure_category(&error);
                        if category == SPIRE_REMOTE_PRODUCTION_TRANSPORT_REMOTE_QUERY_FAILED {
                            SPIRE_REMOTE_PRODUCTION_REMOTE_INDEX_UNAVAILABLE
                        } else {
                            category
                        }
                    })?
                    .try_get::<_, Option<u32>>(0)
                    .map_err(|_| SPIRE_REMOTE_PRODUCTION_REMOTE_INDEX_UNAVAILABLE)?
                    .ok_or(SPIRE_REMOTE_PRODUCTION_REMOTE_INDEX_UNAVAILABLE)?;
                let endpoint_identity_row = client
                    .query_one(
                        SPIRE_REMOTE_SEARCH_ENDPOINT_IDENTITY_SQL_TEMPLATE,
                        &[&remote_index_oid],
                    )
                    .await
                    .map_err(|_| SPIRE_REMOTE_STATUS_ENDPOINT_IDENTITY_MISMATCH)?;
                let endpoint_identity =
                    validate_remote_search_endpoint_identity_row(&endpoint_identity_row)
                        .map_err(|_| SPIRE_REMOTE_STATUS_ENDPOINT_IDENTITY_MISMATCH)?;
                if endpoint_identity.profile_fingerprint_bytes.as_slice()
                    != request.remote_index_identity.as_slice()
                {
                    return Err(SPIRE_REMOTE_STATUS_ENDPOINT_IDENTITY_MISMATCH);
                }
                client
                    .query(
                        SPIRE_REMOTE_SEARCH_LIBPQ_SQL_TEMPLATE,
                        &[
                            &remote_index_oid,
                            &requested_epoch,
                            &request.query,
                            &selected_pids,
                            &top_k,
                            &request.consistency_mode,
                        ],
                    )
                    .await
                    .map_err(|error| production_remote_query_failure_category(&error))
            },
            local_cancel_source,
        )
        .await;

        connection_task.abort();
        let result_rows = match result_rows {
            Ok(result_rows) => result_rows,
            Err(failure_category) => {
                return failed_production_candidate_receive_result(
                    request.node_id,
                    batch_start,
                    request_start,
                    failure_category,
                );
            }
        };
        if validate_remote_payload_batch_row_count(
            result_rows.len(),
            "remote candidate receive result rows",
        )
        .is_err()
        {
            return failed_production_candidate_receive_result(
                request.node_id,
                batch_start,
                request_start,
                SPIRE_REMOTE_STATUS_REMOTE_PAYLOAD_TOO_LARGE,
            );
        }
        let candidates = match result_rows
            .iter()
            .map(|candidate_row| {
                decode_remote_search_candidate_pg_row(
                    candidate_row,
                    request.node_id,
                    true,
                    Some(&request.remote_index_identity),
                )
            })
            .collect::<Result<Vec<_>, _>>()
        {
            Ok(candidates) => candidates,
            Err(error) => {
                return failed_production_candidate_receive_result(
                    request.node_id,
                    batch_start,
                    request_start,
                    production_candidate_decode_failure_category(&error),
                );
            }
        };
        if let Err(error) = validate_remote_search_candidate_batch(
            request.requested_epoch,
            request.node_id,
            &request.selected_pids,
            &candidates,
        ) {
            return failed_production_candidate_receive_result(
                request.node_id,
                batch_start,
                request_start,
                production_candidate_validation_failure_category(&error),
            );
        }
        let candidate_count = u64::try_from(candidates.len()).unwrap_or(u64::MAX);
        SpireRemoteProductionCandidateReceiveResult {
            node_id: request.node_id,
            started_after_ms,
            completed_after_ms: elapsed_millis_u64(batch_start),
            elapsed_ms: elapsed_millis_u64(request_start),
            candidate_count,
            status: SPIRE_REMOTE_STATUS_READY,
            failure_category: SPIRE_REMOTE_NONE,
            batch: Some(SpireRemoteSearchCandidateBatch {
                node_id: request.node_id,
                selected_pids: request.selected_pids,
                candidates,
            }),
        }
    }

    async fn run_one_heap_receive_request(
        request: SpireRemoteProductionHeapReceiveRequest,
        batch_start: std::time::Instant,
        local_cancel_source: SpireRemoteLocalCancelSource,
    ) -> SpireRemoteProductionHeapReceiveResult {
        let started_after_ms = elapsed_millis_u64(batch_start);
        let request_start = std::time::Instant::now();
        if validate_remote_payload_batch_row_count(
            request.selected_pids.len(),
            "remote heap receive selected_pids",
        )
        .is_err()
        {
            return failed_production_heap_receive_result(
                request.node_id,
                batch_start,
                request_start,
                SPIRE_REMOTE_STATUS_REMOTE_PAYLOAD_TOO_LARGE,
            );
        }
        let selected_pids = match request
            .selected_pids
            .iter()
            .map(|pid| i64::try_from(*pid))
            .collect::<Result<Vec<_>, _>>()
        {
            Ok(selected_pids) => selected_pids,
            Err(_) => {
                return failed_production_heap_receive_result(
                    request.node_id,
                    batch_start,
                    request_start,
                    SPIRE_REMOTE_PRODUCTION_CANDIDATE_INVALID_PARAMETERS,
                );
            }
        };
        let requested_epoch = match i64::try_from(request.requested_epoch) {
            Ok(requested_epoch) => requested_epoch,
            Err(_) => {
                return failed_production_heap_receive_result(
                    request.node_id,
                    batch_start,
                    request_start,
                    SPIRE_REMOTE_PRODUCTION_CANDIDATE_INVALID_PARAMETERS,
                );
            }
        };
        let top_k = match i32::try_from(request.top_k) {
            Ok(top_k) => top_k,
            Err(_) => {
                return failed_production_heap_receive_result(
                    request.node_id,
                    batch_start,
                    request_start,
                    SPIRE_REMOTE_PRODUCTION_CANDIDATE_INVALID_PARAMETERS,
                );
            }
        };
        let _governance_permit =
            match remote_search_libpq_executor_governance_permit_for_node(request.node_id) {
                Ok(permit) => permit,
                Err(error) => {
                    return failed_production_heap_receive_result(
                        request.node_id,
                        batch_start,
                        request_start,
                        production_governance_failure_category(&error),
                    );
                }
            };
        let limits = SpireRemoteSearchLibpqExecutorBudgetLimits::from_session();
        let SpireRemoteAsyncConnection {
            client,
            connection_task,
            tls_config,
        } = match remote_search_libpq_connect_async_with_session_timeouts(
            &request.conninfo,
            request.node_id,
            "production heap receive",
        )
        .await
        {
            Ok(connection) => connection,
            Err(error) => {
                return failed_production_heap_receive_result(
                    request.node_id,
                    batch_start,
                    request_start,
                    error.category,
                );
            }
        };

        let cancel_token = client.cancel_token();
        let result_rows = Self::run_query_with_optional_local_cancel(
            cancel_token,
            tls_config,
            async {
                if limits.statement_timeout_ms > 0 {
                    client
                        .batch_execute(&format!(
                            "SET statement_timeout = {}",
                            limits.statement_timeout_ms
                        ))
                        .await
                        .map_err(|_| {
                            SPIRE_REMOTE_PRODUCTION_TRANSPORT_STATEMENT_TIMEOUT_SETUP_FAILED
                        })?;
                }
                let remote_index_oid = client
                    .query_one(
                        "SELECT to_regclass($1)::oid",
                        &[&request.remote_index_regclass.as_str()],
                    )
                    .await
                    .map_err(|error| {
                        let category = production_remote_query_failure_category(&error);
                        if category == SPIRE_REMOTE_PRODUCTION_TRANSPORT_REMOTE_QUERY_FAILED {
                            SPIRE_REMOTE_PRODUCTION_REMOTE_INDEX_UNAVAILABLE
                        } else {
                            category
                        }
                    })?
                    .try_get::<_, Option<u32>>(0)
                    .map_err(|_| SPIRE_REMOTE_PRODUCTION_REMOTE_INDEX_UNAVAILABLE)?
                    .ok_or(SPIRE_REMOTE_PRODUCTION_REMOTE_INDEX_UNAVAILABLE)?;
                let endpoint_identity_row = client
                    .query_one(
                        SPIRE_REMOTE_SEARCH_ENDPOINT_IDENTITY_SQL_TEMPLATE,
                        &[&remote_index_oid],
                    )
                    .await
                    .map_err(|_| SPIRE_REMOTE_STATUS_ENDPOINT_IDENTITY_MISMATCH)?;
                let endpoint_identity =
                    validate_remote_search_endpoint_identity_row(&endpoint_identity_row)
                        .map_err(|_| SPIRE_REMOTE_STATUS_ENDPOINT_IDENTITY_MISMATCH)?;
                if endpoint_identity.profile_fingerprint_bytes.as_slice()
                    != request.remote_index_identity.as_slice()
                {
                    return Err(SPIRE_REMOTE_STATUS_ENDPOINT_IDENTITY_MISMATCH);
                }
                match request.tuple_payload_columns.as_ref() {
                    Some(tuple_payload_columns) => {
                        let sql = remote_tuple_payload_production_sql(&endpoint_identity)?;
                        client
                            .query(
                                sql,
                                &[
                                    &remote_index_oid,
                                    &requested_epoch,
                                    &request.query,
                                    &selected_pids,
                                    &top_k,
                                    &request.consistency_mode,
                                    tuple_payload_columns,
                                ],
                            )
                            .await
                            .map_err(|error| production_remote_query_failure_category(&error))
                    }
                    None => client
                        .query(
                            SPIRE_REMOTE_SEARCH_LIBPQ_HEAP_SQL_TEMPLATE,
                            &[
                                &remote_index_oid,
                                &requested_epoch,
                                &request.query,
                                &selected_pids,
                                &top_k,
                                &request.consistency_mode,
                            ],
                        )
                        .await
                        .map_err(|error| production_remote_query_failure_category(&error)),
                }
            },
            local_cancel_source,
        )
        .await;

        connection_task.abort();
        let result_rows = match result_rows {
            Ok(result_rows) => result_rows,
            Err(failure_category) => {
                return failed_production_heap_receive_result(
                    request.node_id,
                    batch_start,
                    request_start,
                    failure_category,
                );
            }
        };
        if validate_remote_payload_batch_row_count(result_rows.len(), "remote heap result rows")
            .is_err()
        {
            return failed_production_heap_receive_result(
                request.node_id,
                batch_start,
                request_start,
                SPIRE_REMOTE_STATUS_REMOTE_PAYLOAD_TOO_LARGE,
            );
        }
        let candidates = match result_rows
            .iter()
            .map(|candidate_row| {
                decode_remote_search_heap_candidate_pg_row(
                    candidate_row,
                    request.requested_epoch,
                    request.node_id,
                )
            })
            .collect::<Result<Vec<_>, _>>()
        {
            Ok(candidates) => candidates,
            Err(error) => {
                pgrx::warning!("ec_spire remote heap receive decode failed: {error}");
                return failed_production_heap_receive_result(
                    request.node_id,
                    batch_start,
                    request_start,
                    production_remote_heap_decode_failure_category(&error),
                );
            }
        };
        let merge_candidates = candidates
            .iter()
            .map(|candidate| SpireRemoteSearchCandidateRow {
                served_epoch: candidate.served_epoch,
                node_id: candidate.node_id,
                pid: candidate.pid,
                object_version: candidate.object_version,
                row_index: candidate.row_index,
                assignment_flags: candidate.assignment_flags,
                vec_id: candidate.vec_id.clone(),
                row_locator: candidate.row_locator.clone(),
                score: candidate.score,
            })
            .collect::<Vec<_>>();
        if let Err(error) = validate_remote_search_candidate_batch(
            request.requested_epoch,
            request.node_id,
            &request.selected_pids,
            &merge_candidates,
        ) {
            return failed_production_heap_receive_result(
                request.node_id,
                batch_start,
                request_start,
                production_remote_heap_decode_failure_category(&error),
            );
        }
        let candidate_count = u64::try_from(candidates.len()).unwrap_or(u64::MAX);
        SpireRemoteProductionHeapReceiveResult {
            node_id: request.node_id,
            started_after_ms,
            completed_after_ms: elapsed_millis_u64(batch_start),
            elapsed_ms: elapsed_millis_u64(request_start),
            candidate_count,
            status: SPIRE_REMOTE_STATUS_READY,
            failure_category: SPIRE_REMOTE_NONE,
            candidates,
        }
    }

    async fn run_one_insert_prepare_request(
        request: SpireCoordinatorInsertRemotePrepareRequest,
        local_cancel_source: SpireRemoteLocalCancelSource,
    ) -> Result<SpireCoordinatorInsertRemotePrepareResult, String> {
        let _governance_permit =
            remote_search_libpq_executor_governance_permit_for_node(request.node_id)?;
        let limits = SpireRemoteSearchLibpqExecutorBudgetLimits::from_session();
        let SpireRemoteAsyncConnection {
            client,
            connection_task,
            tls_config,
        } = remote_search_libpq_connect_async_with_session_timeouts(
            &request.conninfo,
            request.node_id,
            "coordinator insert remote prepare",
        )
            .await
            .map_err(|error| {
                format!("{}: {}", error.category, error.message)
            })?;
        let cancel_token = client.cancel_token();

        let result = async {
            if limits.statement_timeout_ms > 0 {
                client
                    .batch_execute(&format!(
                        "SET statement_timeout = {}",
                        limits.statement_timeout_ms
                    ))
                    .await
                    .map_err(|_| {
                        format!(
                            "ec_spire coordinator insert remote prepare failed to configure statement_timeout for node_id {}",
                            request.node_id
                        )
                    })?;
            }
            client.batch_execute("BEGIN").await.map_err(|_| {
                format!(
                    "ec_spire coordinator insert failed to begin remote transaction for node_id {}",
                    request.node_id
                )
            })?;

            let remote_sql_result = Self::run_insert_step_with_optional_local_cancel(
                cancel_token.clone(),
                tls_config.clone(),
                async {
                    client.batch_execute(&request.remote_sql).await.map_err(|error| {
                        format!(
                            "ec_spire coordinator insert remote SQL failed for node_id {}: {}",
                            request.node_id,
                            postgres_async_error_message_with_detail(&error)
                        )
                    })
                },
                local_cancel_source,
            )
            .await;
            if insert_step_observed_local_cancel(&remote_sql_result) {
                let _ = client.batch_execute("ROLLBACK").await;
                return Err(coordinator_remote_local_cancel_error(
                    "insert",
                    request.node_id,
                    postgres_local_cancel_failure_category(),
                ));
            }
            if let Err(error) = remote_sql_result.map(|_| ()) {
                let _ = client.batch_execute("ROLLBACK").await;
                return Err(error);
            }

            let metadata_result = Self::run_insert_step_with_optional_local_cancel(
                cancel_token.clone(),
                tls_config.clone(),
                async {
                    coordinator_insert_remote_descriptor_metadata_async(
                        &client,
                        request.node_id,
                        &request.remote_index_regclass,
                    )
                    .await
                },
                local_cancel_source,
            )
            .await;
            if insert_step_observed_local_cancel(&metadata_result) {
                let _ = client.batch_execute("ROLLBACK").await;
                return Err(coordinator_remote_local_cancel_error(
                    "insert",
                    request.node_id,
                    postgres_local_cancel_failure_category(),
                ));
            }
            let (remote_last_served_epoch, remote_index_identity, remote_extension_version) =
                match metadata_result.map(|step| step.value) {
                    Ok(metadata) => metadata,
                    Err(error) => {
                        let _ = client.batch_execute("ROLLBACK").await;
                        return Err(error);
                    }
                };

            let prepare_sql = format!(
                "PREPARE TRANSACTION {}",
                quote_sql_literal(&request.prepared_gid)
            );
            let prepare_result = Self::run_insert_step_with_optional_local_cancel(
                cancel_token,
                tls_config,
                async {
                    client.batch_execute(&prepare_sql).await.map_err(|error| {
                        spire_remote_prepare_transaction_async_error(
                            "insert",
                            request.node_id,
                            &error,
                        )
                    })
                },
                local_cancel_source,
            )
            .await;
            match prepare_result {
                Ok(step) if step.local_cancel_observed => {
                    coordinator_insert_resolve_remote_prepared(
                        request.conninfo.clone(),
                        request.node_id,
                        request.prepared_gid.clone(),
                        false,
                    );
                    Err(coordinator_remote_local_cancel_error(
                        "insert",
                        request.node_id,
                        postgres_local_cancel_failure_category(),
                    ))
                }
                Ok(_) => Ok(SpireCoordinatorInsertRemotePrepareResult {
                    node_id: request.node_id,
                    conninfo: request.conninfo,
                    prepared_gid: request.prepared_gid,
                    remote_index_identity,
                    remote_last_served_epoch,
                    remote_extension_version,
                }),
                Err(error) => {
                    let _ = client.batch_execute("ROLLBACK").await;
                    Err(error)
                }
            }
        }
        .await;

        connection_task.abort();
        result
    }

    async fn run_insert_step_with_optional_local_cancel<T, F>(
        cancel_token: tokio_postgres::CancelToken,
        tls_config: SpireRemoteTlsConfig,
        query_future: F,
        local_cancel_source: SpireRemoteLocalCancelSource,
    ) -> Result<SpireCoordinatorInsertAsyncStep<T>, String>
    where
        F: std::future::Future<Output = Result<T, String>>,
    {
        if local_cancel_source == SpireRemoteLocalCancelSource::None {
            return query_future.await.map(|value| SpireCoordinatorInsertAsyncStep {
                value,
                local_cancel_observed: false,
            });
        }
        let mut query_future = Box::pin(query_future);
        let mut cancel_signal = Box::pin(Self::local_cancel_signal(local_cancel_source));
        match futures_util::future::select(query_future.as_mut(), cancel_signal.as_mut()).await {
            futures_util::future::Either::Left((query_result, _)) => {
                query_result.map(|value| SpireCoordinatorInsertAsyncStep {
                    value,
                    local_cancel_observed: false,
                })
            }
            futures_util::future::Either::Right((failure_category, _)) => {
                remote_search_libpq_cancel_query(cancel_token, &tls_config).await;
                match query_future.await {
                    Ok(value) => Ok(SpireCoordinatorInsertAsyncStep {
                        value,
                        local_cancel_observed: true,
                    }),
                    Err(_) => Err(failure_category.to_owned()),
                }
            }
        }
    }

    async fn run_query_with_optional_local_cancel<T, F>(
        cancel_token: tokio_postgres::CancelToken,
        tls_config: SpireRemoteTlsConfig,
        query_future: F,
        local_cancel_source: SpireRemoteLocalCancelSource,
    ) -> Result<T, &'static str>
    where
        F: std::future::Future<Output = Result<T, &'static str>>,
    {
        if local_cancel_source == SpireRemoteLocalCancelSource::None {
            return query_future.await;
        }
        let cancel_signal = Self::local_cancel_signal(local_cancel_source);
        match futures_util::future::select(Box::pin(query_future), Box::pin(cancel_signal)).await {
            futures_util::future::Either::Left((query_result, _)) => query_result,
            futures_util::future::Either::Right((failure_category, _query_future)) => {
                remote_search_libpq_cancel_query(cancel_token, &tls_config).await;
                Err(failure_category)
            }
        }
    }

    async fn local_cancel_signal(
        local_cancel_source: SpireRemoteLocalCancelSource,
    ) -> &'static str {
        match local_cancel_source {
            SpireRemoteLocalCancelSource::None => std::future::pending::<&'static str>().await,
            SpireRemoteLocalCancelSource::TestAfterMs(delay_ms) => {
                tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
                SPIRE_REMOTE_PRODUCTION_LOCAL_QUERY_CANCELLED
            }
            SpireRemoteLocalCancelSource::PostgresInterruptPoll { poll_interval_ms } => {
                let poll_interval_ms = poll_interval_ms.max(1);
                loop {
                    if postgres_query_cancel_pending() {
                        return postgres_local_cancel_failure_category();
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(poll_interval_ms)).await;
                }
            }
        }
    }
}

unsafe extern "C" {
    fn dlsym(
        handle: *mut std::ffi::c_void,
        symbol: *const std::ffi::c_char,
    ) -> *mut std::ffi::c_void;
}

fn postgres_sig_atomic_flag(symbol_name: &'static [u8]) -> i32 {
    let ptr = unsafe { dlsym(std::ptr::null_mut(), symbol_name.as_ptr().cast()) };
    if ptr.is_null() {
        return 0;
    }
    unsafe { *(ptr.cast::<std::ffi::c_int>()) }
}

fn postgres_query_cancel_pending() -> bool {
    postgres_sig_atomic_flag(b"InterruptPending\0") != 0
        && postgres_sig_atomic_flag(b"QueryCancelPending\0") != 0
}

const POSTGRES_STATEMENT_TIMEOUT_ID: std::ffi::c_int = 3;

type PostgresGetTimeoutIndicator =
    unsafe extern "C" fn(std::ffi::c_int, bool) -> bool;

fn postgres_statement_timeout_pending() -> bool {
    let ptr = unsafe { dlsym(std::ptr::null_mut(), b"get_timeout_indicator\0".as_ptr().cast()) };
    if ptr.is_null() {
        return false;
    }
    let get_timeout_indicator: PostgresGetTimeoutIndicator = unsafe { std::mem::transmute(ptr) };
    unsafe { get_timeout_indicator(POSTGRES_STATEMENT_TIMEOUT_ID, false) }
}

fn postgres_local_cancel_failure_category() -> &'static str {
    if postgres_statement_timeout_pending() {
        SPIRE_REMOTE_PRODUCTION_LOCAL_STATEMENT_TIMEOUT
    } else {
        SPIRE_REMOTE_PRODUCTION_LOCAL_QUERY_CANCELLED
    }
}

fn is_local_cancellation_failure_category(failure_category: &str) -> bool {
    failure_category == SPIRE_REMOTE_PRODUCTION_LOCAL_QUERY_CANCELLED
        || failure_category == SPIRE_REMOTE_PRODUCTION_LOCAL_STATEMENT_TIMEOUT
}

fn coordinator_remote_local_cancel_error(
    operation: &str,
    node_id: u32,
    failure_category: &str,
) -> String {
    format!(
        "ec_spire coordinator {operation} remote prepare cancelled for node_id {node_id}: {failure_category}"
    )
}

fn production_remote_query_failure_category(error: &tokio_postgres::Error) -> &'static str {
    // `tokio-postgres` reports a backend terminated during an already-open
    // query as a closed connection, while pre-query connection failures are
    // classified earlier as connect failures.
    if error.is_closed() {
        return SPIRE_REMOTE_PRODUCTION_REMOTE_BACKEND_TERMINATED;
    }
    let Some(db_error) = error.as_db_error() else {
        return SPIRE_REMOTE_PRODUCTION_TRANSPORT_REMOTE_QUERY_FAILED;
    };
    match db_error.code().code() {
        // PostgreSQL uses SQLSTATE 57014 for query_canceled in general. The
        // statement-timeout message text is the stable PostgreSQL convention
        // that lets operators distinguish timeout remediation from cancellation
        // provenance.
        "57014" if db_error.message().contains("statement timeout") => {
            SPIRE_REMOTE_PRODUCTION_REMOTE_STATEMENT_TIMEOUT
        }
        "57014" => SPIRE_REMOTE_PRODUCTION_REMOTE_QUERY_CANCELLED,
        "57P01" | "57P02" | "57P03" => SPIRE_REMOTE_PRODUCTION_REMOTE_BACKEND_TERMINATED,
        _ => SPIRE_REMOTE_PRODUCTION_TRANSPORT_REMOTE_QUERY_FAILED,
    }
}

fn production_governance_failure_category(_error: &str) -> &'static str {
    SPIRE_REMOTE_STATUS_EXECUTOR_OVERLOAD
}

fn production_candidate_decode_failure_category(error: &str) -> &'static str {
    let status = remote_search_receive_attempt_failure_status(error);
    if status == SPIRE_REMOTE_STATUS_TUPLE_TRANSPORT_RETIRED {
        SPIRE_REMOTE_STATUS_TUPLE_TRANSPORT_RETIRED
    } else if status == SPIRE_REMOTE_STATUS_REMOTE_PAYLOAD_TOO_LARGE {
        SPIRE_REMOTE_STATUS_REMOTE_PAYLOAD_TOO_LARGE
    } else if status == SPIRE_REMOTE_STATUS_ENDPOINT_IDENTITY_MISMATCH
        || status == "protocol_version_mismatch"
        || status == "extension_version_mismatch"
    {
        SPIRE_REMOTE_STATUS_ENDPOINT_IDENTITY_MISMATCH
    } else {
        SPIRE_REMOTE_PRODUCTION_CANDIDATE_DECODE_FAILED
    }
}

fn remote_production_failure_hint(failure_category: &str) -> &'static str {
    match failure_category {
        SPIRE_REMOTE_STATUS_TUPLE_TRANSPORT_RETIRED => SPIRE_REMOTE_TUPLE_TRANSPORT_RETIRED_HINT,
        SPIRE_REMOTE_STATUS_REMOTE_PAYLOAD_TOO_LARGE => SPIRE_REMOTE_PAYLOAD_TOO_LARGE_HINT,
        _ => SPIRE_REMOTE_NONE,
    }
}

fn production_candidate_validation_failure_category(error: &str) -> &'static str {
    if remote_search_receive_attempt_failure_status(error)
        == SPIRE_REMOTE_PRODUCTION_SERVED_EPOCH_MISMATCH
    {
        SPIRE_REMOTE_PRODUCTION_SERVED_EPOCH_MISMATCH
    } else {
        SPIRE_REMOTE_PRODUCTION_CANDIDATE_VALIDATION_FAILED
    }
}

fn production_remote_heap_decode_failure_category(error: &str) -> &'static str {
    if error.contains(SPIRE_REMOTE_STATUS_REMOTE_PAYLOAD_TOO_LARGE) {
        SPIRE_REMOTE_STATUS_REMOTE_PAYLOAD_TOO_LARGE
    } else if error.contains(SPIRE_REMOTE_PRODUCTION_REMOTE_HEAP_ROW_MISSING) {
        SPIRE_REMOTE_PRODUCTION_REMOTE_HEAP_ROW_MISSING
    } else if error.contains(SPIRE_REMOTE_PRODUCTION_REMOTE_HEAP_ROW_DEAD) {
        SPIRE_REMOTE_PRODUCTION_REMOTE_HEAP_ROW_DEAD
    } else if error.contains(SPIRE_REMOTE_PRODUCTION_REMOTE_HEAP_ROW_STALE) {
        SPIRE_REMOTE_PRODUCTION_REMOTE_HEAP_ROW_STALE
    } else if remote_search_receive_attempt_failure_status(error)
        == SPIRE_REMOTE_PRODUCTION_SERVED_EPOCH_MISMATCH
    {
        SPIRE_REMOTE_PRODUCTION_SERVED_EPOCH_MISMATCH
    } else {
        SPIRE_REMOTE_PRODUCTION_REMOTE_HEAP_RESOLUTION_FAILED
    }
}
