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

pub(crate) fn remote_search_libpq_dispatch_summary_row(
    index: SpireLiveIndexRelation,
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
        let rows = remote_search_libpq_dispatch_plan_rows(
            index,
            requested_epoch,
            query,
            selected_pids,
            top_k,
            consistency_mode,
        );
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
        rollup.record_status(
            row.status,
            row.pid_count,
            "remote search libpq dispatch summary",
        )?;
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

    let dispatch_count = u64::try_from(rows.len())
        .map_err(|_| "ec_spire remote search libpq dispatch summary dispatch count exceeds u64")?;
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

pub(crate) fn remote_search_libpq_executor_budget_summary_row(
    index: SpireLiveIndexRelation,
    requested_epoch: u64,
    query: Vec<f32>,
    selected_pids: Vec<u64>,
    top_k: usize,
    consistency_mode: &str,
) -> SpireRemoteSearchLibpqExecutorBudgetSummaryRow {
    let result = (|| -> Result<SpireRemoteSearchLibpqExecutorBudgetSummaryRow, String> {
        let rows = remote_search_libpq_dispatch_plan_rows(
            index,
            requested_epoch,
            query,
            selected_pids,
            top_k,
            consistency_mode,
        );
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
        (
            SPIRE_REMOTE_NONE,
            SPIRE_REMOTE_STATUS_READY,
            SPIRE_REMOTE_NONE,
        )
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
struct SpireRemotePooledConnectionKey {
    node_id: u32,
    descriptor_generation: u64,
    conninfo_secret_name: String,
    remote_index_regclass: String,
    remote_index_identity: Vec<u8>,
    tls_mode: &'static str,
    user: String,
    dbname: String,
    statement_timeout_ms: i32,
    conninfo_fingerprint: u64,
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
    pub(crate) conninfo_secret_name: String,
    pub(crate) conninfo: String,
    pub(crate) remote_index_regclass: String,
    pub(crate) descriptor_generation: u64,
    pub(crate) remote_index_identity: Vec<u8>,
    pub(crate) requested_epoch: u64,
    pub(crate) query: Vec<f32>,
    pub(crate) selected_pids: Vec<u64>,
    pub(crate) top_k: usize,
    pub(crate) effective_rerank_width: i32,
    pub(crate) consistency_mode: String,
    pub(crate) initial_threshold_score: Option<f32>,
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
    pub(crate) conninfo_secret_name: String,
    pub(crate) conninfo: String,
    pub(crate) remote_index_regclass: String,
    pub(crate) descriptor_generation: u64,
    pub(crate) remote_index_identity: Vec<u8>,
    pub(crate) requested_epoch: u64,
    pub(crate) query: Vec<f32>,
    pub(crate) selected_pids: Vec<u64>,
    pub(crate) top_k: usize,
    pub(crate) effective_rerank_width: i32,
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
    pub(crate) payload_decode_elapsed_us: u64,
    pub(crate) payload_decode_row_count: u64,
    pub(crate) payload_decode_bytes: u64,
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

struct SpireRemoteProductionCandidateAndHeapExecution {
    result: SpireRemoteProductionCandidateAndHeapResult,
    reusable_connections: Vec<SpireRemotePooledConnection>,
}

struct SpireRemoteProductionCandidateSession {
    request: SpireRemoteProductionCandidateReceiveRequest,
    connection: SpireRemotePooledConnection,
    remote_index_oid: u32,
    endpoint_identity: SpireRemoteValidatedEndpointIdentity,
    selected_pids: Vec<i64>,
    requested_epoch: i64,
    top_k: i32,
    effective_rerank_width: i32,
    started_after_ms: u64,
    request_start: std::time::Instant,
    global_heap_candidates: Option<Vec<SpireRemoteSearchCandidateRow>>,
}

struct SpireRemoteProductionCandidateSessionResult {
    candidate_result: SpireRemoteProductionCandidateReceiveResult,
    session: Option<SpireRemoteProductionCandidateSession>,
    metrics: SpireRemoteProductionReadMetrics,
}

struct SpireRemoteProductionHeapSessionResult {
    heap_result: SpireRemoteProductionHeapReceiveResult,
    metrics: SpireRemoteProductionReadMetrics,
    reusable_connection: Option<SpireRemotePooledConnection>,
}

fn remote_heap_payload_decode_bytes(candidates: &[SpireRemoteSearchLocalHeapCandidateRow]) -> u64 {
    candidates
        .iter()
        .map(|candidate| {
            let typed_bytes = candidate
                .typed_tuple_payload
                .as_ref()
                .map(|payload| {
                    payload
                        .payload_values
                        .iter()
                        .map(Vec::len)
                        .try_fold(0_u64, |acc, len| {
                            u64::try_from(len).ok().and_then(|len| acc.checked_add(len))
                        })
                        .unwrap_or(u64::MAX)
                })
                .unwrap_or(0);
            let json_bytes = candidate
                .tuple_payload_json
                .as_ref()
                .map(|payload_json| u64::try_from(payload_json.len()).unwrap_or(u64::MAX))
                .unwrap_or(0);
            typed_bytes.saturating_add(json_bytes)
        })
        .fold(0_u64, u64::saturating_add)
}

struct SpireRemoteExplicitHeapCandidateParameters {
    served_epochs: Vec<i64>,
    pids: Vec<i64>,
    object_versions: Vec<i64>,
    row_indices: Vec<i64>,
    assignment_flags: Vec<i16>,
    vec_id_hex_values: Vec<String>,
    row_locator_hex_values: Vec<String>,
    scores: Vec<f32>,
}

fn explicit_heap_candidate_parameters(
    candidates: &[SpireRemoteSearchCandidateRow],
) -> Result<SpireRemoteExplicitHeapCandidateParameters, &'static str> {
    let mut served_epochs = Vec::with_capacity(candidates.len());
    let mut pids = Vec::with_capacity(candidates.len());
    let mut object_versions = Vec::with_capacity(candidates.len());
    let mut row_indices = Vec::with_capacity(candidates.len());
    let mut assignment_flags = Vec::with_capacity(candidates.len());
    let mut vec_id_hex_values = Vec::with_capacity(candidates.len());
    let mut row_locator_hex_values = Vec::with_capacity(candidates.len());
    let mut scores = Vec::with_capacity(candidates.len());

    for candidate in candidates {
        served_epochs.push(
            i64::try_from(candidate.served_epoch)
                .map_err(|_| SPIRE_REMOTE_PRODUCTION_CANDIDATE_INVALID_PARAMETERS)?,
        );
        pids.push(
            i64::try_from(candidate.pid)
                .map_err(|_| SPIRE_REMOTE_PRODUCTION_CANDIDATE_INVALID_PARAMETERS)?,
        );
        object_versions.push(
            i64::try_from(candidate.object_version)
                .map_err(|_| SPIRE_REMOTE_PRODUCTION_CANDIDATE_INVALID_PARAMETERS)?,
        );
        row_indices.push(i64::from(candidate.row_index));
        assignment_flags.push(
            i16::try_from(candidate.assignment_flags)
                .map_err(|_| SPIRE_REMOTE_PRODUCTION_CANDIDATE_INVALID_PARAMETERS)?,
        );
        vec_id_hex_values.push(hex::encode(&candidate.vec_id));
        row_locator_hex_values.push(hex::encode(&candidate.row_locator));
        scores.push(candidate.score);
    }

    Ok(SpireRemoteExplicitHeapCandidateParameters {
        served_epochs,
        pids,
        object_versions,
        row_indices,
        assignment_flags,
        vec_id_hex_values,
        row_locator_hex_values,
        scores,
    })
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

struct SpireRemotePooledConnection {
    key: SpireRemotePooledConnectionKey,
    client: tokio_postgres::Client,
    connection_task: tokio::task::JoinHandle<()>,
    tls_config: SpireRemoteTlsConfig,
    _governance_permit: SpireRemoteSearchLibpqGovernancePermit,
    validated_remote_index_oid: Option<u32>,
    validated_endpoint_identity: Option<SpireRemoteValidatedEndpointIdentity>,
    candidate_statement: Option<tokio_postgres::Statement>,
    candidate_initial_threshold_statement: Option<tokio_postgres::Statement>,
    heap_statement: Option<tokio_postgres::Statement>,
    explicit_heap_statement: Option<tokio_postgres::Statement>,
    typed_tuple_payload_statement: Option<tokio_postgres::Statement>,
}

impl Drop for SpireRemotePooledConnection {
    fn drop(&mut self) {
        self.connection_task.abort();
    }
}

struct SpireRemoteBackendTransportState {
    runtime: tokio::runtime::Runtime,
    idle_connections: VecDeque<SpireRemotePooledConnection>,
}

thread_local! {
    static SPIRE_REMOTE_BACKEND_TRANSPORT_STATE: RefCell<Option<SpireRemoteBackendTransportState>> =
        const { RefCell::new(None) };
}

impl SpireRemotePooledConnectionKey {
    fn from_request_fields(
        node_id: u32,
        conninfo_secret_name: &str,
        conninfo: &str,
        remote_index_regclass: &str,
        descriptor_generation: u64,
        remote_index_identity: &[u8],
    ) -> Result<Self, String> {
        let parsed = spire_remote_parse_conninfo(conninfo).map_err(|error| {
            format!("ec_spire remote connection pool conninfo parse failed for node_id {node_id}: {error}")
        })?;
        let config = parsed
            .base_conninfo
            .parse::<tokio_postgres::Config>()
            .map_err(|_| {
                format!(
                    "ec_spire remote connection pool conninfo parse failed for node_id {node_id}"
                )
            })?;
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        conninfo.hash(&mut hasher);

        Ok(Self {
            node_id,
            descriptor_generation,
            conninfo_secret_name: conninfo_secret_name.to_owned(),
            remote_index_regclass: remote_index_regclass.to_owned(),
            remote_index_identity: remote_index_identity.to_vec(),
            tls_mode: parsed.tls_config.sslmode_name(),
            user: config.get_user().unwrap_or("").to_owned(),
            dbname: config.get_dbname().unwrap_or("").to_owned(),
            statement_timeout_ms: options::current_session_remote_search_statement_timeout_ms(),
            conninfo_fingerprint: hasher.finish(),
        })
    }
}

impl SpireRemoteBackendTransportState {
    fn new() -> Result<Self, String> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_io()
            .enable_time()
            .build()
            .map_err(|_| {
                "ec_spire production transport adapter failed to build pooled runtime".to_owned()
            })?;
        Ok(Self {
            runtime,
            idle_connections: VecDeque::new(),
        })
    }

    fn pool_limit() -> usize {
        usize::try_from(options::current_session_remote_search_connection_pool_size()).unwrap_or(0)
    }

    fn take_connection(
        &mut self,
        request: &SpireRemoteProductionCandidateReceiveRequest,
    ) -> Option<SpireRemotePooledConnection> {
        if Self::pool_limit() == 0 {
            self.idle_connections.clear();
            return None;
        }
        let key = SpireRemotePooledConnectionKey::from_request_fields(
            request.node_id,
            &request.conninfo_secret_name,
            &request.conninfo,
            &request.remote_index_regclass,
            request.descriptor_generation,
            &request.remote_index_identity,
        )
        .ok()?;
        self.idle_connections
            .iter()
            .position(|connection| {
                connection.key == key && !connection.connection_task.is_finished()
            })
            .and_then(|position| self.idle_connections.remove(position))
    }

    fn put_connection(&mut self, connection: SpireRemotePooledConnection) {
        let limit = Self::pool_limit();
        if limit == 0 || connection.connection_task.is_finished() {
            return;
        }
        self.idle_connections
            .retain(|idle| idle.key != connection.key && !idle.connection_task.is_finished());
        while self.idle_connections.len() >= limit {
            self.idle_connections.pop_front();
        }
        self.idle_connections.push_back(connection);
    }
}

fn cached_production_endpoint_identity(
    validated_remote_index_oid: Option<u32>,
    validated_endpoint_identity: Option<&SpireRemoteValidatedEndpointIdentity>,
    remote_index_identity: &[u8],
) -> Option<(u32, SpireRemoteValidatedEndpointIdentity)> {
    let remote_index_oid = validated_remote_index_oid?;
    let endpoint_identity = validated_endpoint_identity?;
    if endpoint_identity.profile_fingerprint_bytes.as_slice() != remote_index_identity {
        return None;
    }
    Some((remote_index_oid, endpoint_identity.clone()))
}

async fn production_pooled_statement<'a>(
    client: &tokio_postgres::Client,
    slot: &'a mut Option<tokio_postgres::Statement>,
    sql: &'static str,
) -> Result<&'a tokio_postgres::Statement, &'static str> {
    if slot.is_none() {
        let statement = client
            .prepare(sql)
            .await
            .map_err(|error| production_remote_query_failure_category(&error))?;
        *slot = Some(statement);
    }
    Ok(slot
        .as_ref()
        .expect("SPIRE production pooled statement initialized"))
}

fn with_spire_remote_backend_transport_state<T>(
    f: impl FnOnce(&mut SpireRemoteBackendTransportState) -> Result<T, String>,
) -> Result<T, String> {
    SPIRE_REMOTE_BACKEND_TRANSPORT_STATE.with(|cell| {
        let mut state = cell.borrow_mut();
        if state.is_none() {
            *state = Some(SpireRemoteBackendTransportState::new()?);
        }
        f(state
            .as_mut()
            .expect("SPIRE remote backend transport state initialized"))
    })
}

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
                Self::run_one_candidate_receive_request(request, batch_start, local_cancel_source)
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
        let consistency_mode = consistency_mode.to_owned();

        with_spire_remote_backend_transport_state(|state| {
            let requests = requests
                .into_iter()
                .map(|request| {
                    let pooled_connection = state.take_connection(&request);
                    (request, pooled_connection)
                })
                .collect::<Vec<_>>();
            let execution = state.runtime.block_on(async move {
                let batch_start = std::time::Instant::now();
                let mut metrics = SpireRemoteProductionReadMetrics::default();
                let mut heap_results = Vec::new();
                let mut reusable_connections = Vec::new();
                let global_pre_heap_merge = options::remote_search_global_pre_heap_merge_enabled()
                    && tuple_payload_columns.is_none();
                let mut candidate_results = Vec::new();

                if global_pre_heap_merge {
                    let futures =
                        requests
                            .into_iter()
                            .map(|(request, pooled_connection)| async move {
                                Self::run_one_candidate_session_request(
                                    request,
                                    pooled_connection,
                                    batch_start,
                                    SpireRemoteLocalCancelSource::production(),
                                )
                                .await
                            });
                    let mut session_results = futures_util::future::join_all(futures).await;
                    candidate_results = session_results
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
                    if allow_heap {
                        Self::assign_global_heap_candidate_subsets(
                            &mut session_results,
                            &mut metrics,
                        )?;
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
                        for mut heap_result in futures_util::future::join_all(futures).await {
                            metrics.add_transport_metrics(&heap_result.metrics);
                            if let Some(connection) = heap_result.reusable_connection.take() {
                                reusable_connections.push(connection);
                            }
                            heap_results.push(heap_result.heap_result);
                        }
                    }
                } else {
                    let mut candidate_futures = requests
                        .into_iter()
                        .map(|(request, pooled_connection)| {
                            Self::run_one_candidate_session_request(
                                request,
                                pooled_connection,
                                batch_start,
                                SpireRemoteLocalCancelSource::production(),
                            )
                        })
                        .collect::<futures_util::stream::FuturesUnordered<_>>();
                    let mut heap_futures = futures_util::stream::FuturesUnordered::<
                        futures_util::future::BoxFuture<
                            'static,
                            SpireRemoteProductionHeapSessionResult,
                        >,
                    >::new();
                    let degraded = parse_remote_search_consistency_mode(consistency_mode.as_str())?
                        == meta::SpireConsistencyMode::Degraded;
                    let mut launch_heap_for_ready_candidates = true;
                    macro_rules! process_candidate_result {
                        ($candidate_result:expr) => {{
                            let candidate_result = $candidate_result;
                            metrics.add_transport_metrics(&candidate_result.metrics);
                            let status = candidate_result.candidate_result.status;
                            let failure_category =
                                candidate_result.candidate_result.failure_category;
                            let session = candidate_result.session;
                            candidate_results.push(candidate_result.candidate_result);
                            if is_local_cancellation_failure_category(failure_category) {
                                launch_heap_for_ready_candidates = false;
                            } else if status != SPIRE_REMOTE_STATUS_READY && !degraded {
                                launch_heap_for_ready_candidates = false;
                            }
                            if launch_heap_for_ready_candidates
                                && status == SPIRE_REMOTE_STATUS_READY
                            {
                                if let Some(session) = session {
                                    let tuple_payload_columns = tuple_payload_columns.clone();
                                    let consistency_mode = consistency_mode.clone();
                                    heap_futures.push(Box::pin(async move {
                                        Self::run_heap_receive_on_candidate_session(
                                            session,
                                            tuple_payload_columns.as_deref(),
                                            consistency_mode.as_str(),
                                            batch_start,
                                        )
                                        .await
                                    }));
                                }
                            }
                        }};
                    }
                    macro_rules! process_heap_result {
                        ($heap_result:expr) => {{
                            let mut heap_result = $heap_result;
                            metrics.add_transport_metrics(&heap_result.metrics);
                            if let Some(connection) = heap_result.reusable_connection.take() {
                                reusable_connections.push(connection);
                            }
                            heap_results.push(heap_result.heap_result);
                        }};
                    }
                    while !candidate_futures.is_empty() || !heap_futures.is_empty() {
                        if candidate_futures.is_empty() {
                            if let Some(heap_result) =
                                futures_util::StreamExt::next(&mut heap_futures).await
                            {
                                process_heap_result!(heap_result);
                            }
                        } else if heap_futures.is_empty() {
                            if let Some(candidate_result) =
                                futures_util::StreamExt::next(&mut candidate_futures).await
                            {
                                process_candidate_result!(candidate_result);
                            }
                        } else {
                            match futures_util::future::select(
                                futures_util::StreamExt::next(&mut candidate_futures),
                                futures_util::StreamExt::next(&mut heap_futures),
                            )
                            .await
                            {
                                futures_util::future::Either::Left((Some(candidate_result), _)) => {
                                    process_candidate_result!(candidate_result);
                                }
                                futures_util::future::Either::Right((Some(heap_result), _)) => {
                                    process_heap_result!(heap_result);
                                }
                                futures_util::future::Either::Left((None, _))
                                | futures_util::future::Either::Right((None, _)) => {}
                            }
                        }
                    }
                }

                Ok::<SpireRemoteProductionCandidateAndHeapExecution, String>(
                    SpireRemoteProductionCandidateAndHeapExecution {
                        result: SpireRemoteProductionCandidateAndHeapResult {
                            candidate_results,
                            heap_results,
                            metrics,
                        },
                        reusable_connections,
                    },
                )
            })?;
            for connection in execution.reusable_connections {
                state.put_connection(connection);
            }
            Ok(execution.result)
        })
    }

    fn assign_global_heap_candidate_subsets(
        session_results: &mut [SpireRemoteProductionCandidateSessionResult],
        metrics: &mut SpireRemoteProductionReadMetrics,
    ) -> Result<(), String> {
        let batches = session_results
            .iter()
            .filter_map(|result| result.candidate_result.batch.clone())
            .collect::<Vec<_>>();
        if batches.is_empty() {
            return Ok(());
        }
        let session = session_results
            .iter()
            .find_map(|result| result.session.as_ref())
            .ok_or_else(|| {
                "ec_spire production read global pre-heap merge has candidates without session metadata"
                    .to_owned()
            })?;
        let requested_epoch = session.request.requested_epoch;
        let merged = merge_validated_remote_search_candidate_batches(
            requested_epoch,
            batches,
            Some(session.request.top_k),
        )?;
        metrics.global_pre_heap_input_count = merged.input_count;
        metrics.global_pre_heap_candidate_count =
            u64::try_from(merged.candidates.len()).map_err(|_| {
                "ec_spire production read global pre-heap candidate count exceeds u64".to_owned()
            })?;
        metrics.global_pre_heap_duplicate_vec_id_count = merged.duplicate_vec_id_count;
        metrics.global_pre_heap_pruned_candidate_count = merged
            .input_count
            .saturating_sub(metrics.global_pre_heap_candidate_count);

        let mut by_node = BTreeMap::<u32, Vec<SpireRemoteSearchCandidateRow>>::new();
        for candidate in merged.candidates {
            by_node
                .entry(candidate.node_id)
                .or_default()
                .push(candidate);
        }
        for result in session_results {
            if let Some(session) = result.session.as_mut() {
                session.global_heap_candidates =
                    Some(by_node.remove(&session.request.node_id).unwrap_or_default());
            }
        }
        Ok(())
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
        if results
            .iter()
            .any(|result| is_local_cancellation_failure_category(result.failure_category))
        {
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
        let degraded = parse_remote_search_consistency_mode(consistency_mode)?
            == meta::SpireConsistencyMode::Degraded;
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
        pooled_connection: Option<SpireRemotePooledConnection>,
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
        let limits = SpireRemoteSearchLibpqExecutorBudgetLimits::from_session();
        let mut connection = match pooled_connection {
            Some(connection) => {
                add_profile_count(&mut metrics.connection_pool_hit_count, 1);
                connection
            }
            None => {
                add_profile_count(&mut metrics.connection_pool_miss_count, 1);
                let governance_permit =
                    match remote_search_libpq_executor_governance_permit_for_node(request.node_id) {
                        Ok(permit) => permit,
                        Err(error) => {
                            let failure_category = production_governance_failure_category(&error);
                            metrics.record_failure_category(
                                &request.consistency_mode,
                                failure_category,
                            );
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
                let connect_start = std::time::Instant::now();
                match remote_search_libpq_connect_async_with_session_timeouts(
                    &request.conninfo,
                    request.node_id,
                    "production candidate/heap receive",
                )
                .await
                {
                    Ok(connection) => {
                        add_profile_elapsed(&mut metrics.connect_elapsed_us, connect_start);
                        add_profile_count(&mut metrics.socket_open_count, 1);
                        metrics.record_tls_config(&connection.tls_config);
                        let key = match SpireRemotePooledConnectionKey::from_request_fields(
                            request.node_id,
                            &request.conninfo_secret_name,
                            &request.conninfo,
                            &request.remote_index_regclass,
                            request.descriptor_generation,
                            &request.remote_index_identity,
                        ) {
                            Ok(key) => key,
                            Err(error) => {
                                metrics.record_failure_category(
                                    &request.consistency_mode,
                                    SPIRE_REMOTE_PRODUCTION_TRANSPORT_CONNINFO_PARSE_FAILED,
                                );
                                pgrx::warning!("{error}");
                                return SpireRemoteProductionCandidateSessionResult {
                                    candidate_result: failed_production_candidate_receive_result(
                                        request.node_id,
                                        batch_start,
                                        request_start,
                                        SPIRE_REMOTE_PRODUCTION_TRANSPORT_CONNINFO_PARSE_FAILED,
                                    ),
                                    session: None,
                                    metrics,
                                };
                            }
                        };
                        SpireRemotePooledConnection {
                            key,
                            client: connection.client,
                            connection_task: connection.connection_task,
                            tls_config: connection.tls_config,
                            _governance_permit: governance_permit,
                            validated_remote_index_oid: None,
                            validated_endpoint_identity: None,
                            candidate_statement: None,
                            candidate_initial_threshold_statement: None,
                            heap_statement: None,
                            explicit_heap_statement: None,
                            typed_tuple_payload_statement: None,
                        }
                    }
                    Err(error) => {
                        add_profile_elapsed(&mut metrics.connect_elapsed_us, connect_start);
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
                }
            }
        };

        let cancel_token = connection.client.cancel_token();
        let cancel_tls_config = connection.tls_config.clone();
        let result_rows = Self::run_query_with_optional_local_cancel(
            cancel_token,
            cancel_tls_config,
            async {
                let mut query_metrics = SpireRemoteProductionReadMetrics::default();
                if limits.statement_timeout_ms > 0 {
                    let timeout_start = std::time::Instant::now();
                    add_profile_count(&mut query_metrics.statement_timeout_setup_count, 1);
                    connection
                        .client
                        .batch_execute(&format!(
                            "SET statement_timeout = {}",
                            limits.statement_timeout_ms
                        ))
                        .await
                        .map_err(|_| {
                            SPIRE_REMOTE_PRODUCTION_TRANSPORT_STATEMENT_TIMEOUT_SETUP_FAILED
                        })?;
                    add_profile_elapsed(
                        &mut query_metrics.statement_timeout_setup_elapsed_us,
                        timeout_start,
                    );
                }
                let (remote_index_oid, endpoint_identity) =
                    match cached_production_endpoint_identity(
                        connection.validated_remote_index_oid,
                        connection.validated_endpoint_identity.as_ref(),
                        &request.remote_index_identity,
                    ) {
                        Some((remote_index_oid, endpoint_identity)) => {
                            (remote_index_oid, endpoint_identity)
                        }
                        None => {
                            let regclass_start = std::time::Instant::now();
                            add_profile_count(&mut query_metrics.regclass_probe_count, 1);
                            let remote_index_oid = connection
                                .client
                                .query_one(
                                    "SELECT to_regclass($1)::oid",
                                    &[&request.remote_index_regclass.as_str()],
                                )
                                .await
                                .map_err(|error| {
                                    let category = production_remote_query_failure_category(&error);
                                    if category
                                        == SPIRE_REMOTE_PRODUCTION_TRANSPORT_REMOTE_QUERY_FAILED
                                    {
                                        SPIRE_REMOTE_PRODUCTION_REMOTE_INDEX_UNAVAILABLE
                                    } else {
                                        category
                                    }
                                })?
                                .try_get::<_, Option<u32>>(0)
                                .map_err(|_| SPIRE_REMOTE_PRODUCTION_REMOTE_INDEX_UNAVAILABLE)?
                                .ok_or(SPIRE_REMOTE_PRODUCTION_REMOTE_INDEX_UNAVAILABLE)?;
                            add_profile_elapsed(
                                &mut query_metrics.regclass_probe_elapsed_us,
                                regclass_start,
                            );

                            let identity_start = std::time::Instant::now();
                            add_profile_count(&mut query_metrics.endpoint_identity_query_count, 1);
                            let endpoint_identity_row = connection
                                .client
                                .query_one(
                                    SPIRE_REMOTE_SEARCH_ENDPOINT_IDENTITY_SQL_TEMPLATE,
                                    &[&remote_index_oid],
                                )
                                .await
                                .map_err(|_| SPIRE_REMOTE_STATUS_ENDPOINT_IDENTITY_MISMATCH)?;
                            let endpoint_identity = validate_remote_search_endpoint_identity_row(
                                &endpoint_identity_row,
                            )
                            .map_err(|_| SPIRE_REMOTE_STATUS_ENDPOINT_IDENTITY_MISMATCH)?;
                            if endpoint_identity.profile_fingerprint_bytes.as_slice()
                                != request.remote_index_identity.as_slice()
                            {
                                return Err(SPIRE_REMOTE_STATUS_ENDPOINT_IDENTITY_MISMATCH);
                            }
                            add_profile_elapsed(
                                &mut query_metrics.endpoint_identity_elapsed_us,
                                identity_start,
                            );
                            connection.validated_remote_index_oid = Some(remote_index_oid);
                            connection.validated_endpoint_identity =
                                Some(endpoint_identity.clone());
                            (remote_index_oid, endpoint_identity)
                        }
                    };

                let candidate_start = std::time::Instant::now();
                add_profile_count(&mut query_metrics.candidate_receive_query_count, 1);
                let result_rows =
                    if let Some(initial_threshold_score) = request.initial_threshold_score {
                        let statement = production_pooled_statement(
                            &connection.client,
                            &mut connection.candidate_initial_threshold_statement,
                            SPIRE_REMOTE_SEARCH_LIBPQ_INITIAL_THRESHOLD_SQL_TEMPLATE,
                        )
                        .await?;
                        connection
                            .client
                            .query(
                                statement,
                                &[
                                    &remote_index_oid,
                                    &requested_epoch,
                                    &request.query,
                                    &selected_pids,
                                    &top_k,
                                    &request.consistency_mode,
                                    &initial_threshold_score,
                                ],
                            )
                            .await
                            .map_err(|error| production_remote_query_failure_category(&error))?
                    } else {
                        let statement = production_pooled_statement(
                            &connection.client,
                            &mut connection.candidate_statement,
                            SPIRE_REMOTE_SEARCH_LIBPQ_SQL_TEMPLATE,
                        )
                        .await?;
                        connection
                            .client
                            .query(
                                statement,
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
                            .map_err(|error| production_remote_query_failure_category(&error))?
                    };
                add_profile_elapsed(
                    &mut query_metrics.candidate_receive_elapsed_us,
                    candidate_start,
                );

                Ok((
                    result_rows,
                    remote_index_oid,
                    endpoint_identity,
                    query_metrics,
                ))
            },
            local_cancel_source,
        )
        .await;

        let (result_rows, remote_index_oid, endpoint_identity, query_metrics) = match result_rows {
            Ok(value) => value,
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
        metrics.add_transport_metrics(&query_metrics);

        if validate_remote_payload_batch_row_count(
            result_rows.len(),
            "remote candidate receive result rows",
        )
        .is_err()
        {
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
        let candidate_decode_start = std::time::Instant::now();
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
                add_profile_elapsed(
                    &mut metrics.candidate_decode_elapsed_us,
                    candidate_decode_start,
                );
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
        add_profile_elapsed(
            &mut metrics.candidate_decode_elapsed_us,
            candidate_decode_start,
        );
        if let Err(error) = validate_remote_search_candidate_batch(
            request.requested_epoch,
            request.node_id,
            &request.selected_pids,
            &candidates,
        ) {
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
        let effective_rerank_width = request.effective_rerank_width;
        let session = SpireRemoteProductionCandidateSession {
            request,
            connection,
            remote_index_oid,
            endpoint_identity,
            selected_pids,
            requested_epoch,
            top_k,
            effective_rerank_width,
            started_after_ms,
            request_start,
            global_heap_candidates: None,
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
            mut connection,
            remote_index_oid,
            endpoint_identity,
            selected_pids,
            requested_epoch,
            top_k,
            effective_rerank_width,
            started_after_ms: _candidate_started_after_ms,
            request_start: _candidate_request_start,
            global_heap_candidates,
        } = session;
        if matches!(global_heap_candidates.as_ref(), Some(candidates) if candidates.is_empty()) {
            let completed_after_ms = elapsed_millis_u64(batch_start);
            return SpireRemoteProductionHeapSessionResult {
                heap_result: SpireRemoteProductionHeapReceiveResult {
                    node_id: request.node_id,
                    started_after_ms: completed_after_ms,
                    completed_after_ms,
                    elapsed_ms: 0,
                    candidate_count: 0,
                    payload_decode_elapsed_us: 0,
                    payload_decode_row_count: 0,
                    payload_decode_bytes: 0,
                    status: SPIRE_REMOTE_STATUS_READY,
                    failure_category: SPIRE_REMOTE_NONE,
                    candidates: Vec::new(),
                },
                metrics,
                reusable_connection: Some(connection),
            };
        }
        let cancel_token = connection.client.cancel_token();
        let cancel_tls_config = connection.tls_config.clone();
        let heap_start = std::time::Instant::now();
        let heap_started_after_ms = elapsed_millis_u64(batch_start);
        let result_rows = Self::run_query_with_optional_local_cancel(
            cancel_token,
            cancel_tls_config,
            async {
                let mut query_metrics = SpireRemoteProductionReadMetrics::default();
                add_profile_count(&mut query_metrics.heap_receive_query_count, 1);
                let remote_rerank_width = effective_rerank_width.to_string();
                connection
                    .client
                    .execute(
                        "SELECT set_config('ec_spire.rerank_width', $1, false)",
                        &[&remote_rerank_width],
                    )
                    .await
                    .map_err(|error| production_remote_query_failure_category(&error))?;
                let result = match tuple_payload_columns {
                    None if global_heap_candidates.is_some() => {
                        let candidates = global_heap_candidates.as_ref().expect("checked is_some");
                        let parameters = explicit_heap_candidate_parameters(candidates)?;
                        let statement = production_pooled_statement(
                            &connection.client,
                            &mut connection.explicit_heap_statement,
                            SPIRE_REMOTE_SEARCH_LIBPQ_EXPLICIT_HEAP_SQL_TEMPLATE,
                        )
                        .await?;
                        connection
                            .client
                            .query(
                                statement,
                                &[
                                    &remote_index_oid,
                                    &requested_epoch,
                                    &request.query,
                                    &parameters.served_epochs,
                                    &parameters.pids,
                                    &parameters.object_versions,
                                    &parameters.row_indices,
                                    &parameters.assignment_flags,
                                    &parameters.vec_id_hex_values,
                                    &parameters.row_locator_hex_values,
                                    &parameters.scores,
                                ],
                            )
                            .await
                            .map_err(|error| production_remote_query_failure_category(&error))
                    }
                    Some(tuple_payload_columns) => {
                        let sql = remote_tuple_payload_production_sql(&endpoint_identity)?;
                        let statement = production_pooled_statement(
                            &connection.client,
                            &mut connection.typed_tuple_payload_statement,
                            sql,
                        )
                        .await?;
                        connection
                            .client
                            .query(
                                statement,
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
                        let statement = production_pooled_statement(
                            &connection.client,
                            &mut connection.heap_statement,
                            SPIRE_REMOTE_SEARCH_LIBPQ_HEAP_SQL_TEMPLATE,
                        )
                        .await?;
                        connection
                            .client
                            .query(
                                statement,
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
                add_profile_elapsed(&mut query_metrics.heap_receive_elapsed_us, heap_start);
                Ok((result, query_metrics))
            },
            SpireRemoteLocalCancelSource::production(),
        )
        .await;

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
                        heap_start,
                        failure_category,
                    ),
                    metrics,
                    reusable_connection: None,
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
                    heap_start,
                    failure_category,
                ),
                metrics,
                reusable_connection: None,
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
                        heap_start,
                        failure_category,
                    ),
                    metrics,
                    reusable_connection: None,
                };
            }
        };
        add_profile_elapsed(&mut metrics.payload_decode_elapsed_us, decode_start);
        add_profile_count(
            &mut metrics.payload_decode_row_count,
            u64::try_from(candidates.len()).unwrap_or(u64::MAX),
        );
        add_profile_count(
            &mut metrics.payload_decode_bytes,
            remote_heap_payload_decode_bytes(&candidates),
        );
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
                    heap_start,
                    failure_category,
                ),
                metrics,
                reusable_connection: None,
            };
        }
        let candidate_count = u64::try_from(candidates.len()).unwrap_or(u64::MAX);
        SpireRemoteProductionHeapSessionResult {
            heap_result: SpireRemoteProductionHeapReceiveResult {
                node_id: request.node_id,
                started_after_ms: heap_started_after_ms,
                completed_after_ms: elapsed_millis_u64(batch_start),
                elapsed_ms: elapsed_millis_u64(heap_start),
                candidate_count,
                payload_decode_elapsed_us: metrics.payload_decode_elapsed_us,
                payload_decode_row_count: metrics.payload_decode_row_count,
                payload_decode_bytes: metrics.payload_decode_bytes,
                status: SPIRE_REMOTE_STATUS_READY,
                failure_category: SPIRE_REMOTE_NONE,
                candidates,
            },
            metrics,
            reusable_connection: Some(connection),
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
                let remote_rerank_width = request.effective_rerank_width.to_string();
                client
                    .execute(
                        "SELECT set_config('ec_spire.rerank_width', $1, false)",
                        &[&remote_rerank_width],
                    )
                    .await
                    .map_err(|error| production_remote_query_failure_category(&error))?;
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
                return failed_production_heap_receive_result(
                    request.node_id,
                    batch_start,
                    request_start,
                    production_remote_heap_decode_failure_category(&error),
                );
            }
        };
        let payload_decode_elapsed_us = elapsed_micros_u64(decode_start);
        let payload_decode_row_count = u64::try_from(candidates.len()).unwrap_or(u64::MAX);
        let payload_decode_bytes = remote_heap_payload_decode_bytes(&candidates);
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
            payload_decode_elapsed_us,
            payload_decode_row_count,
            payload_decode_bytes,
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
        .map_err(|error| format!("{}: {}", error.category, error.message))?;
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
            return query_future
                .await
                .map(|value| SpireCoordinatorInsertAsyncStep {
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
    // SAFETY: symbol_name is a static NUL-terminated PostgreSQL global name and
    // dlsym with a null handle searches the current backend process image.
    let ptr = unsafe { dlsym(std::ptr::null_mut(), symbol_name.as_ptr().cast()) };
    if ptr.is_null() {
        return 0;
    }
    // SAFETY: callers pass names of PostgreSQL sig_atomic_t-compatible globals;
    // a null lookup was rejected above and this only performs a single read.
    unsafe { *(ptr.cast::<std::ffi::c_int>()) }
}

fn postgres_query_cancel_pending() -> bool {
    postgres_sig_atomic_flag(b"InterruptPending\0") != 0
        && postgres_sig_atomic_flag(b"QueryCancelPending\0") != 0
}

const POSTGRES_STATEMENT_TIMEOUT_ID: std::ffi::c_int = 3;

type PostgresGetTimeoutIndicator = unsafe extern "C" fn(std::ffi::c_int, bool) -> bool;

fn postgres_statement_timeout_pending() -> bool {
    // SAFETY: the symbol name is static and NUL-terminated, and dlsym only
    // returns a raw address that is checked for null before use.
    let ptr = unsafe {
        dlsym(
            std::ptr::null_mut(),
            b"get_timeout_indicator\0".as_ptr().cast(),
        )
    };
    if ptr.is_null() {
        return false;
    }
    // SAFETY: PostgreSQL exports get_timeout_indicator with this backend-local
    // ABI; the resolved address is non-null and is invoked read-only for the
    // statement-timeout indicator.
    let get_timeout_indicator: PostgresGetTimeoutIndicator = unsafe { std::mem::transmute(ptr) };
    // SAFETY: the function pointer was resolved and typed above, and this call
    // only reads PostgreSQL's statement-timeout indicator for the current backend.
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
