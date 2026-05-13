#[derive(Debug, Clone, PartialEq, Eq)]
struct SpireCoordinatorPipeline {
    execution_summary: SpireRemoteSearchExecutionSummaryRow,
    dispatch_summary: SpireRemoteSearchLibpqDispatchSummaryRow,
    receive_rows: Vec<SpireRemoteSearchReceivePlanRow>,
    finalization_summary: SpireRemoteSearchFinalizationSummaryRow,
    executor_readiness: SpireRemoteSearchLibpqExecutorReadinessRow,
}

impl SpireCoordinatorPipeline {
    unsafe fn execute_once(
        index_relation: pg_sys::Relation,
        requested_epoch: u64,
        query: Vec<f32>,
        selected_pids: Vec<u64>,
        top_k: usize,
        consistency_mode: &str,
    ) -> Result<Self, String> {
        let top_k_for_empty_plan = u64::try_from(top_k)
            .map_err(|_| "ec_spire coordinator pipeline top_k exceeds u64")?;
        let query_for_summary_fallback = query.clone();
        let readiness_rows = unsafe {
            remote_search_request_readiness_rows(
                index_relation,
                requested_epoch,
                query.clone(),
                selected_pids,
                top_k,
                consistency_mode,
            )
        };
        let execution_rows = readiness_rows
            .into_iter()
            .map(remote_search_execution_plan_row_from_readiness)
            .collect::<Vec<_>>();
        let execution_summary = remote_search_execution_summary_from_plan_rows(
            requested_epoch,
            &execution_rows,
            query_for_summary_fallback.clone(),
            top_k_for_empty_plan,
            consistency_mode,
        )?;
        let request_rows = remote_search_libpq_request_plan_rows_from_execution(&execution_rows);
        let connection_rows = remote_search_libpq_connection_plan_rows_from_requests(
            unsafe { (*index_relation).rd_id },
            &request_rows,
        )?;
        let dispatch_rows = remote_search_libpq_dispatch_plan_rows_from_connections(&connection_rows);
        let dispatch_summary = remote_search_libpq_dispatch_summary_from_plan_rows(
            requested_epoch,
            &dispatch_rows,
            query_for_summary_fallback,
            top_k_for_empty_plan,
            consistency_mode,
        )?;
        let receive_rows = remote_search_receive_plan_rows_from_requests(&request_rows);
        let merge_summary = remote_search_merge_input_summary_from_execution(&execution_summary);
        let finalization_summary = remote_search_finalization_summary_from_merge(&merge_summary);
        let secret_rows = remote_search_libpq_secret_plan_rows_from_dispatch(&dispatch_rows);
        let secret_summary =
            remote_search_libpq_secret_summary_from_plan_rows(requested_epoch, &secret_rows)?;
        let executor_readiness = remote_search_libpq_executor_readiness_from_summaries(
            requested_epoch,
            &dispatch_summary,
            &secret_summary,
        );

        Ok(Self {
            execution_summary,
            dispatch_summary,
            receive_rows,
            finalization_summary,
            executor_readiness,
        })
    }
}

pub(crate) unsafe fn remote_search_coordinator_gate_summary_row(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    query: Vec<f32>,
    selected_pids: Vec<u64>,
    top_k: usize,
    consistency_mode: &str,
) -> SpireRemoteSearchCoordinatorGateSummaryRow {
    let pipeline = unsafe {
        SpireCoordinatorPipeline::execute_once(
            index_relation,
            requested_epoch,
            query,
            selected_pids,
            top_k,
            consistency_mode,
        )
    }
    .unwrap_or_else(|e| pgrx::error!("{e}"));
    let execution_summary = &pipeline.execution_summary;
    let dispatch_summary = &pipeline.dispatch_summary;
    let receive_rows = &pipeline.receive_rows;
    let finalization_summary = &pipeline.finalization_summary;
    let executor_readiness = &pipeline.executor_readiness;
    let libpq_receive_count =
        u64::try_from(receive_rows.len()).expect("receive row count should fit in u64");
    let libpq_receive_status = receive_rows
        .iter()
        .find(|row| row.status != SPIRE_REMOTE_STATUS_READY)
        .map(|row| row.status)
        .unwrap_or(SPIRE_REMOTE_STATUS_READY);

    let (next_blocker, status, recommendation) =
        if execution_summary.status == SPIRE_REMOTE_STATUS_REQUIRES_DESCRIPTOR {
            (
                "remote_node_descriptor",
                SPIRE_REMOTE_STATUS_REQUIRES_DESCRIPTOR,
                "register remote node descriptors before coordinator execution",
            )
        } else if matches!(
            execution_summary.status,
            SPIRE_REMOTE_STATUS_STALE_EPOCH
                | SPIRE_REMOTE_STATUS_RETENTION_GAP
                | SPIRE_REMOTE_STATUS_INCOMPATIBLE_EXTENSION_VERSION
        ) {
            (
                remote_search_pre_dispatch_blocker_step(execution_summary.status),
                execution_summary.status,
                remote_search_pre_dispatch_blocker_recommendation(execution_summary.status),
            )
        } else if executor_readiness.status == SPIRE_REMOTE_EXECUTOR_REQUIRED {
            (
                executor_readiness.next_executor_step,
                SPIRE_REMOTE_EXECUTOR_REQUIRED,
                executor_readiness.recommendation,
            )
        } else if execution_summary.status == SPIRE_REMOTE_STATUS_REQUIRES_LIBPQ {
            (
                "libpq_transport",
                SPIRE_REMOTE_STATUS_REQUIRES_LIBPQ,
                "add libpq pipeline execution before remote coordinator dispatch",
            )
        } else if finalization_summary.final_heap_fetch_status
            == SPIRE_REMOTE_FINAL_STATUS_REQUIRES_REMOTE_HEAP
        {
            (
                "remote_heap_resolution",
                SPIRE_REMOTE_FINAL_STATUS_REQUIRES_REMOTE_HEAP,
                "add origin-node row locator resolution before returning remote heap rows",
            )
        } else if finalization_summary.status == SPIRE_REMOTE_STATUS_EMPTY_TOP_K {
            (
                SPIRE_REMOTE_NONE,
                SPIRE_REMOTE_STATUS_EMPTY_TOP_K,
                SPIRE_REMOTE_NONE,
            )
        } else {
            (SPIRE_REMOTE_NONE, finalization_summary.status, SPIRE_REMOTE_NONE)
        };

    SpireRemoteSearchCoordinatorGateSummaryRow {
        requested_epoch,
        local_plan_count: execution_summary.local_plan_count,
        remote_plan_count: execution_summary.remote_plan_count,
        skipped_plan_count: execution_summary.skipped_plan_count,
        local_pid_count: execution_summary.local_pid_count,
        remote_pid_count: execution_summary.remote_pid_count,
        skipped_pid_count: execution_summary.skipped_pid_count,
        execution_status: execution_summary.status,
        libpq_dispatch_count: dispatch_summary.dispatch_count,
        libpq_dispatch_status: dispatch_summary.status,
        libpq_executor_status: executor_readiness.status,
        libpq_executor_next_step: executor_readiness.next_executor_step,
        libpq_receive_count,
        libpq_receive_status,
        merge_status: finalization_summary.merge_status,
        final_heap_fetch_status: finalization_summary.final_heap_fetch_status,
        next_blocker,
        status,
        recommendation,
    }
}

pub(crate) unsafe fn remote_search_heap_resolution_summary_row(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    query: Vec<f32>,
    selected_pids: Vec<u64>,
    top_k: usize,
    consistency_mode: &str,
) -> SpireRemoteSearchHeapResolutionSummaryRow {
    let gate = unsafe {
        remote_search_coordinator_gate_summary_row(
            index_relation,
            requested_epoch,
            query.clone(),
            selected_pids.clone(),
            top_k,
            consistency_mode,
        )
    };

    let decoded_local_locator_count = if gate.remote_plan_count == 0
        && gate.status != SPIRE_REMOTE_STATUS_EMPTY_TOP_K
    {
        let rows = unsafe {
            remote_search_local_heap_resolution_plan_rows(
                index_relation,
                requested_epoch,
                query,
                selected_pids,
                top_k,
                consistency_mode,
            )
        };
        u64::try_from(rows.len())
            .unwrap_or_else(|_| pgrx::error!("ec_spire local heap resolution row count overflow"))
    } else {
        0
    };

    let local_heap_resolution_status = if gate.local_plan_count == 0 {
        SPIRE_REMOTE_NONE
    } else if gate.status == SPIRE_REMOTE_STATUS_EMPTY_TOP_K {
        SPIRE_REMOTE_STATUS_EMPTY_TOP_K
    } else if gate.remote_plan_count == 0 && remote_search_status_allows_local_heap_rows(gate.status)
    {
        SPIRE_REMOTE_STATUS_READY
    } else {
        SPIRE_REMOTE_FINAL_STATUS_PLANNED
    };
    let remote_heap_resolution_status = if gate.remote_plan_count == 0 {
        SPIRE_REMOTE_NONE
    } else if gate.status == SPIRE_REMOTE_STATUS_REQUIRES_DESCRIPTOR
        || gate.status == SPIRE_REMOTE_STATUS_REQUIRES_LIBPQ
        || gate.status == SPIRE_REMOTE_STATUS_STALE_EPOCH
        || gate.status == SPIRE_REMOTE_STATUS_RETENTION_GAP
        || gate.status == SPIRE_REMOTE_STATUS_INCOMPATIBLE_EXTENSION_VERSION
    {
        gate.status
    } else {
        SPIRE_REMOTE_FINAL_STATUS_REQUIRES_REMOTE_HEAP
    };

    SpireRemoteSearchHeapResolutionSummaryRow {
        requested_epoch,
        local_plan_count: gate.local_plan_count,
        remote_plan_count: gate.remote_plan_count,
        skipped_plan_count: gate.skipped_plan_count,
        local_pid_count: gate.local_pid_count,
        remote_pid_count: gate.remote_pid_count,
        decoded_local_locator_count,
        local_heap_resolution_status,
        remote_heap_resolution_status,
        status: gate.status,
        recommendation: gate.recommendation,
    }
}

fn validate_remote_candidate_vec_id(
    candidate: &SpireRemoteSearchCandidateRow,
    context: &str,
) -> Result<storage::SpireVecId, String> {
    storage::SpireVecId::from_bytes(&candidate.vec_id).map_err(|e| {
        format!(
            "ec_spire {context} candidate PID {} row_index {} has invalid vec_id {}: {e}",
            candidate.pid,
            candidate.row_index,
            hex::encode(&candidate.vec_id)
        )
    })
}

fn remote_search_candidate_dedupe_key(
    candidate: &SpireRemoteSearchCandidateRow,
) -> Result<Vec<u8>, String> {
    let vec_id = validate_remote_candidate_vec_id(candidate, "remote candidate merge")?;
    let mut key = Vec::new();
    if vec_id.discriminator() == storage::SPIRE_GLOBAL_VEC_ID_DISCRIMINATOR {
        key.reserve_exact(1 + candidate.vec_id.len());
        key.push(SPIRE_REMOTE_VEC_ID_KEY_GLOBAL);
        key.extend_from_slice(&candidate.vec_id);
    } else {
        key.reserve_exact(1 + std::mem::size_of::<u32>() + candidate.vec_id.len());
        key.push(SPIRE_REMOTE_VEC_ID_KEY_NODE_LOCAL);
        key.extend_from_slice(&candidate.node_id.to_le_bytes());
        key.extend_from_slice(&candidate.vec_id);
    }
    Ok(key)
}

/// Validates one target-scoped remote candidate receive batch.
///
/// The batch must match the requested epoch, expected node, selected leaf PID
/// set, visible assignment flags, valid vec_id, nonempty opaque row_locator,
/// and finite score contract before candidates can enter the merge path.
///
/// Delta insert rows are leaf-derived: the storage endpoint is selected by
/// leaf PID, but it returns the delta object PID for row/version tie-breaking.
/// The coordinator can envelope-validate those rows by their delta flag; the
/// origin storage endpoint remains responsible for binding each delta row to a
/// selected parent leaf.
pub(crate) fn validate_remote_search_candidate_batch(
    requested_epoch: u64,
    expected_node_id: u32,
    selected_pids: &[u64],
    candidates: &[SpireRemoteSearchCandidateRow],
) -> Result<(), String> {
    if requested_epoch == 0 {
        return Err(
            "ec_spire remote candidate batch requested_epoch must be greater than 0".to_owned(),
        );
    }

    let mut selected = HashSet::new();
    for &pid in selected_pids {
        if pid == 0 {
            return Err("ec_spire remote candidate batch selected PID 0 is invalid".to_owned());
        }
        if !selected.insert(pid) {
            return Err(format!(
                "ec_spire remote candidate batch selected PID {pid} appears more than once"
            ));
        }
    }

    for candidate in candidates {
        if candidate.served_epoch != requested_epoch {
            return Err(format!(
                "ec_spire remote candidate batch served epoch {} does not match requested epoch {requested_epoch}",
                candidate.served_epoch
            ));
        }
        if candidate.node_id != expected_node_id {
            return Err(format!(
                "ec_spire remote candidate batch node_id {} does not match expected node_id {expected_node_id}",
                candidate.node_id
            ));
        }
        if candidate.pid == 0 {
            return Err("ec_spire remote candidate batch candidate PID 0 is invalid".to_owned());
        }
        if !selected.contains(&candidate.pid)
            && candidate.assignment_flags & storage::SPIRE_ASSIGNMENT_FLAG_DELTA_INSERT == 0
        {
            return Err(format!(
                "ec_spire remote candidate batch candidate PID {} was not selected for node_id {expected_node_id} and is not a delta insert candidate",
                candidate.pid
            ));
        }
        if candidate.object_version == 0 {
            return Err(format!(
                "ec_spire remote candidate batch candidate PID {} has object_version 0",
                candidate.pid
            ));
        }
        if !storage::is_visible_scored_assignment_flags(candidate.assignment_flags) {
            return Err(format!(
                "ec_spire remote candidate batch candidate PID {} has non-visible assignment_flags {}",
                candidate.pid, candidate.assignment_flags
            ));
        }
        validate_remote_candidate_vec_id(candidate, "remote candidate batch")?;
        if candidate.row_locator.is_empty() {
            return Err(format!(
                "ec_spire remote candidate batch candidate PID {} has empty row_locator",
                candidate.pid
            ));
        }
        if !candidate.score.is_finite() {
            return Err("ec_spire remote candidate batch received non-finite score".to_owned());
        }
    }

    Ok(())
}

/// Merges candidates using globally comparable vec-id bytes when available and
/// node-scoped local vec-id bytes for compatibility with existing local-only
/// indexes.
pub(crate) fn merge_remote_search_candidates<I>(
    candidates: I,
    limit: Option<usize>,
) -> Result<SpireRemoteSearchMergeResult, String>
where
    I: IntoIterator<Item = SpireRemoteSearchCandidateRow>,
{
    let mut input_count = 0_u64;
    let mut duplicate_vec_id_count = 0_u64;
    let mut best_by_vec_id: HashMap<Vec<u8>, SpireRemoteSearchCandidateRow> = HashMap::new();

    for candidate in candidates {
        input_count = input_count
            .checked_add(1)
            .ok_or_else(|| "ec_spire remote candidate merge input count overflow".to_owned())?;
        if !candidate.score.is_finite() {
            return Err("ec_spire remote candidate merge received non-finite score".to_owned());
        }
        let dedupe_key = remote_search_candidate_dedupe_key(&candidate)?;

        match best_by_vec_id.entry(dedupe_key) {
            std::collections::hash_map::Entry::Occupied(mut entry) => {
                duplicate_vec_id_count =
                    duplicate_vec_id_count.checked_add(1).ok_or_else(|| {
                        "ec_spire remote candidate merge duplicate count overflow".to_owned()
                    })?;
                if remote_search_candidate_cmp(&candidate, entry.get()).is_lt() {
                    *entry.get_mut() = candidate;
                }
            }
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert(candidate);
            }
        }
    }

    let mut candidates = best_by_vec_id.into_values().collect::<Vec<_>>();
    candidates.sort_by(remote_search_candidate_cmp);
    if let Some(limit) = limit {
        candidates.truncate(limit);
    }

    Ok(SpireRemoteSearchMergeResult {
        candidates,
        input_count,
        duplicate_vec_id_count,
    })
}

/// Validates each target-scoped receive batch before global candidate merge.
///
/// Global vec-id bytes dedupe across nodes. Local vec-id bytes dedupe only
/// within their origin node as a compatibility fallback.
pub(crate) fn merge_validated_remote_search_candidate_batches(
    requested_epoch: u64,
    batches: Vec<SpireRemoteSearchCandidateBatch>,
    limit: Option<usize>,
) -> Result<SpireRemoteSearchMergeResult, String> {
    for batch in &batches {
        validate_remote_search_candidate_batch(
            requested_epoch,
            batch.node_id,
            &batch.selected_pids,
            &batch.candidates,
        )?;
    }

    merge_remote_search_candidates(
        batches.into_iter().flat_map(|batch| batch.candidates),
        limit,
    )
}

