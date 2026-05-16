fn remote_search_libpq_executor_candidates_for_dispatch(
    index_relid: pg_sys::Oid,
    row: &SpireRemoteSearchLibpqDispatchPlanRow,
    query: &[f32],
    top_k: usize,
    consistency_mode: &str,
    executor_state: &mut SpireRemoteSearchLibpqExecutorState,
) -> Result<Vec<SpireRemoteSearchCandidateRow>, String> {
    if row.dispatch_action != SPIRE_REMOTE_DISPATCH_PIPELINE_ACTION {
        return Err(format!(
            "ec_spire remote search libpq executor dispatch for node_id {} is blocked with status {}",
            row.node_id, row.status
        ));
    }

    let _governance_permit = remote_search_libpq_executor_governance_permit(row)?;
    let conninfo = remote_conninfo_secret_value(&row.conninfo_secret_name).map_err(|status| {
        format!(
            "ec_spire remote search libpq executor conninfo secret for node_id {} is not resolved: {status}",
            row.node_id
        )
    })?;
    let mut client = remote_search_libpq_connect_with_session_timeouts(
        &conninfo,
        row.node_id,
        "remote search libpq executor",
    )?;
    let remote_index_oid = client
        .query_one(
            "SELECT to_regclass($1)::oid",
            &[&row.remote_index_regclass.as_str()],
        )
        .map_err(|_| {
            format!(
                "ec_spire remote search libpq executor failed to resolve remote index for node_id {}",
                row.node_id
            )
        })?
        .try_get::<_, Option<u32>>(0)
        .map_err(|_| {
            format!(
                "ec_spire remote search libpq executor remote index oid decode failed for node_id {}",
                row.node_id
            )
        })?
        .ok_or_else(|| {
            format!(
                "ec_spire remote search libpq executor remote index is missing for node_id {}",
                row.node_id
            )
        })?;
    validate_remote_search_libpq_endpoint_identity_for_dispatch(
        &mut client,
        index_relid,
        remote_index_oid,
        row,
        executor_state,
    )?;
    let requested_epoch = i64::try_from(row.requested_epoch)
        .map_err(|_| "ec_spire remote search libpq executor requested_epoch exceeds i64")?;
    let selected_pids = row
        .selected_pids
        .iter()
        .map(|pid| {
            i64::try_from(*pid)
                .map_err(|_| "ec_spire remote search libpq executor selected PID exceeds i64")
        })
        .collect::<Result<Vec<_>, _>>()?;
    let top_k = i32::try_from(top_k)
        .map_err(|_| "ec_spire remote search libpq executor top_k exceeds i32")?;

    let result_rows = client
        .query(
            SPIRE_REMOTE_SEARCH_LIBPQ_SQL_TEMPLATE,
            &[
                &remote_index_oid,
                &requested_epoch,
                &query,
                &selected_pids,
                &top_k,
                &consistency_mode,
            ],
        )
        .map_err(|_| {
            format!(
                "ec_spire remote search libpq executor remote search query failed for node_id {}",
                row.node_id
            )
        })?;
    let candidates = result_rows
        .iter()
        .map(|candidate_row| {
            decode_remote_search_candidate_pg_row(
                candidate_row,
                row.node_id,
                true,
                Some(&row.remote_index_identity),
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    validate_remote_search_candidate_batch(
        row.requested_epoch,
        row.node_id,
        &row.selected_pids,
        &candidates,
    )?;

    Ok(candidates)
}

fn remote_search_dispatch_blocked_status(error: &str) -> Option<&str> {
    error
        .strip_prefix("ec_spire remote search libpq executor dispatch for node_id ")
        .and_then(|value| value.rsplit_once(" is blocked with status "))
        .map(|(_, status)| status)
}

fn remote_search_receive_attempt_failure_status(error: &str) -> String {
    if let Some(status) = remote_search_dispatch_blocked_status(error) {
        return status.to_owned();
    }

    let endpoint_status_prefix = "ec_spire remote search executor endpoint_status ";
    let endpoint_status_suffix = " is not ready";
    if let Some(status) = error
        .strip_prefix(endpoint_status_prefix)
        .and_then(|value| value.strip_suffix(endpoint_status_suffix))
    {
        return status.to_owned();
    }

    if error.contains("protocol_version") {
        "protocol_version_mismatch".to_owned()
    } else if error.contains("extension_version") {
        "extension_version_mismatch".to_owned()
    } else if error.contains("served epoch") {
        "served_epoch_mismatch".to_owned()
    } else if error.contains("opclass_identity")
        || error.contains("storage_format")
        || error.contains("assignment_payload_format")
        || error.contains("quantizer_profile")
        || error.contains("scoring_profile")
        || error.contains("profile_fingerprint")
        || error.contains("remote_index_identity")
        || error.contains("endpoint identity")
    {
        SPIRE_REMOTE_STATUS_ENDPOINT_IDENTITY_MISMATCH.to_owned()
    } else if error.contains(SPIRE_REMOTE_STATUS_REMOTE_PAYLOAD_TOO_LARGE) {
        SPIRE_REMOTE_STATUS_REMOTE_PAYLOAD_TOO_LARGE.to_owned()
    } else if error.contains(SPIRE_REMOTE_STATUS_TUPLE_TRANSPORT_RETIRED) {
        SPIRE_REMOTE_STATUS_TUPLE_TRANSPORT_RETIRED.to_owned()
    } else if error.contains(SPIRE_REMOTE_STATUS_EXECUTOR_OVERLOAD) {
        SPIRE_REMOTE_STATUS_EXECUTOR_OVERLOAD.to_owned()
    } else if error.contains("conninfo secret") {
        SPIRE_REMOTE_STATUS_REQUIRES_SECRET.to_owned()
    } else if error.contains("failed to open connection") {
        "libpq_connection_failed".to_owned()
    } else if error.contains("remote index") {
        "remote_index_unavailable".to_owned()
    } else {
        "remote_candidate_batch_rejected".to_owned()
    }
}

fn remote_search_receive_attempt_next_blocker(error: &str) -> String {
    if let Some(status) = remote_search_dispatch_blocked_status(error) {
        return remote_search_pre_dispatch_blocker_step(status).to_owned();
    }

    if error.contains("endpoint_status")
        || error.contains("protocol_version")
        || error.contains("extension_version")
        || error.contains("opclass_identity")
        || error.contains("storage_format")
        || error.contains("assignment_payload_format")
        || error.contains("quantizer_profile")
        || error.contains("scoring_profile")
        || error.contains("profile_fingerprint")
        || error.contains("remote_index_identity")
        || error.contains("endpoint identity")
    {
        "remote_endpoint_identity".to_owned()
    } else if error.contains("served epoch") {
        "served_epoch".to_owned()
    } else if error.contains(SPIRE_REMOTE_STATUS_EXECUTOR_OVERLOAD) {
        SPIRE_REMOTE_EXECUTOR_STEP_GOVERNANCE.to_owned()
    } else if error.contains("conninfo secret") {
        "conninfo_secret_resolution".to_owned()
    } else if error.contains("failed to open connection") {
        "open_libpq_connection".to_owned()
    } else if error.contains("remote index") {
        "remote_index_regclass".to_owned()
    } else {
        "remote_search_candidate_batch".to_owned()
    }
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn remote_search_libpq_identity_cache_contract_probe_counts(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    query: Vec<f32>,
    selected_pids: Vec<u64>,
    top_k: usize,
    consistency_mode: &str,
) -> (u64, u64, u64, u64, String) {
    let result = (|| -> Result<(u64, u64, u64, u64, String), String> {
        let index_relid = unsafe { (*index_relation).rd_id };
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
        let row = dispatch_rows
            .iter()
            .find(|row| row.dispatch_action == SPIRE_REMOTE_DISPATCH_PIPELINE_ACTION)
            .ok_or_else(|| {
                "ec_spire remote search libpq identity cache contract probe found no ready dispatch"
                    .to_owned()
            })?;
        let conninfo = remote_conninfo_secret_value(&row.conninfo_secret_name).map_err(|status| {
            format!(
                "ec_spire remote search libpq identity cache contract probe conninfo secret for node_id {} is not resolved: {status}",
                row.node_id
            )
        })?;
        let mut client = remote_search_libpq_connect_with_session_timeouts(
            &conninfo,
            row.node_id,
            "remote search libpq identity cache contract probe",
        )?;
        let remote_index_oid = client
            .query_one(
                "SELECT to_regclass($1)::oid",
                &[&row.remote_index_regclass.as_str()],
            )
            .map_err(|_| {
                format!(
                    "ec_spire remote search libpq identity cache contract probe failed to resolve remote index for node_id {}",
                    row.node_id
                )
            })?
            .try_get::<_, Option<u32>>(0)
            .map_err(|_| {
                format!(
                    "ec_spire remote search libpq identity cache contract probe remote index oid decode failed for node_id {}",
                    row.node_id
                )
            })?
            .ok_or_else(|| {
                format!(
                    "ec_spire remote search libpq identity cache contract probe remote index is missing for node_id {}",
                    row.node_id
                )
            })?;

        let mut executor_state = SpireRemoteSearchLibpqExecutorState::default();
        validate_remote_search_libpq_endpoint_identity_for_dispatch(
            &mut client,
            index_relid,
            remote_index_oid,
            row,
            &mut executor_state,
        )?;
        validate_remote_search_libpq_endpoint_identity_for_dispatch(
            &mut client,
            index_relid,
            remote_index_oid,
            row,
            &mut executor_state,
        )?;

        let mut generation_row = row.clone();
        generation_row.descriptor_generation =
            generation_row.descriptor_generation.checked_add(1).ok_or_else(|| {
                "ec_spire remote search libpq identity cache contract probe descriptor generation overflow"
                    .to_owned()
            })?;
        validate_remote_search_libpq_endpoint_identity_for_dispatch(
            &mut client,
            index_relid,
            remote_index_oid,
            &generation_row,
            &mut executor_state,
        )?;

        let mut served_epoch_row = row.clone();
        served_epoch_row.requested_epoch =
            served_epoch_row.requested_epoch.checked_add(1).ok_or_else(|| {
                "ec_spire remote search libpq identity cache contract probe served epoch overflow"
                    .to_owned()
            })?;
        validate_remote_search_libpq_endpoint_identity_for_dispatch(
            &mut client,
            index_relid,
            remote_index_oid,
            &served_epoch_row,
            &mut executor_state,
        )?;

        let mut identity_row = row.clone();
        identity_row.remote_index_identity = if identity_row.remote_index_identity.as_slice() == &[0xff] {
            vec![0x00]
        } else {
            vec![0xff]
        };
        let mismatch_status = match validate_remote_search_libpq_endpoint_identity_for_dispatch(
            &mut client,
            index_relid,
            remote_index_oid,
            &identity_row,
            &mut executor_state,
        ) {
            Ok(()) => SPIRE_REMOTE_STATUS_READY.to_owned(),
            Err(error) => remote_search_receive_attempt_failure_status(&error),
        };

        Ok((
            executor_state.endpoint_identity_cache_entry_count()?,
            executor_state.endpoint_identity_query_count(),
            executor_state.endpoint_identity_cache_hit_count(),
            executor_state.endpoint_identity_cache_miss_count(),
            mismatch_status,
        ))
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

#[cfg(any(test, feature = "pg_test"))]
#[allow(clippy::type_complexity)]
pub(crate) fn remote_search_libpq_executor_budget_contract_probe_counts(
) -> (u64, u64, u64, u64, u64, u64, &'static str, &'static str) {
    let connection_rows = vec![
        SpireRemoteSearchLibpqConnectionPlanRow {
            requested_epoch: 7,
            node_id: 2,
            selected_pids: vec![10],
            pid_count: 1,
            query_dimension: 2,
            top_k: 1,
            consistency_mode: "strict",
            execution_transport: SPIRE_REMOTE_TRANSPORT_LIBPQ_PIPELINE,
            conninfo_secret_name: "spire/remote/budget/2".to_owned(),
            remote_index_regclass: "remote_spire_idx".to_owned(),
            descriptor_generation: 21,
            remote_index_identity: vec![10],
            remote_index_identity_bytes: 1,
            conninfo_resolution: SPIRE_REMOTE_CONNINFO_READY,
            pipeline_mode: SPIRE_REMOTE_TRANSPORT_LIBPQ_PIPELINE,
            status: SPIRE_REMOTE_STATUS_REQUIRES_LIBPQ,
        },
        SpireRemoteSearchLibpqConnectionPlanRow {
            requested_epoch: 7,
            node_id: 3,
            selected_pids: vec![20],
            pid_count: 1,
            query_dimension: 2,
            top_k: 1,
            consistency_mode: "strict",
            execution_transport: SPIRE_REMOTE_TRANSPORT_LIBPQ_PIPELINE,
            conninfo_secret_name: "spire/remote/budget/3".to_owned(),
            remote_index_regclass: "remote_spire_idx".to_owned(),
            descriptor_generation: 22,
            remote_index_identity: vec![11],
            remote_index_identity_bytes: 1,
            conninfo_resolution: SPIRE_REMOTE_CONNINFO_READY,
            pipeline_mode: SPIRE_REMOTE_TRANSPORT_LIBPQ_PIPELINE,
            status: SPIRE_REMOTE_STATUS_REQUIRES_LIBPQ,
        },
    ];
    let dispatch_rows = remote_search_libpq_dispatch_plan_rows_from_connections(&connection_rows);
    let budget_summary =
        remote_search_libpq_executor_budget_summary_from_dispatch_rows(7, &dispatch_rows)
            .unwrap_or_else(|e| pgrx::error!("{e}"));
    let secret_rows = remote_search_libpq_secret_plan_rows_from_dispatch(&dispatch_rows);
    let secret_summary = remote_search_libpq_secret_summary_from_plan_rows(7, &secret_rows)
        .unwrap_or_else(|e| pgrx::error!("{e}"));
    let dispatch_count = u64::try_from(dispatch_rows.len())
        .unwrap_or_else(|_| pgrx::error!("budget probe dispatch count overflow"));
    let secret_budget_blocked_count = u64::try_from(
        secret_rows
            .iter()
            .filter(|row| {
                row.status == SPIRE_REMOTE_STATUS_EXECUTOR_OVERLOAD
                    && row.provider_lookup_key == SPIRE_REMOTE_NONE
                    && row.next_executor_step == SPIRE_REMOTE_EXECUTOR_STEP_BUDGET
            })
            .count(),
    )
    .unwrap_or_else(|_| pgrx::error!("budget probe secret blocked count overflow"));

    (
        dispatch_count,
        budget_summary.admitted_dispatch_count,
        budget_summary.budget_blocked_dispatch_count,
        budget_summary.admitted_pid_count,
        budget_summary.budget_blocked_pid_count,
        secret_budget_blocked_count,
        budget_summary.status,
        secret_summary.next_executor_step,
    )
}

pub(crate) unsafe fn remote_search_libpq_executor_receive_attempt_rows(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    query: Vec<f32>,
    selected_pids: Vec<u64>,
    top_k: usize,
    consistency_mode: &str,
) -> Vec<SpireRemoteSearchLibpqReceiveAttemptRow> {
    let result = (|| -> Result<Vec<SpireRemoteSearchLibpqReceiveAttemptRow>, String> {
        let requested_consistency_mode = parse_remote_search_consistency_mode(consistency_mode)?;
        let index_relid = unsafe { (*index_relation).rd_id };
        let dispatch_rows = unsafe {
            remote_search_libpq_dispatch_plan_rows(
                index_relation,
                requested_epoch,
                query.clone(),
                selected_pids,
                top_k,
                consistency_mode,
            )
        };
        let mut executor_state = SpireRemoteSearchLibpqExecutorState::default();
        let mut rows = Vec::with_capacity(dispatch_rows.len());
        for row in &dispatch_rows {
            match remote_search_libpq_executor_candidates_for_dispatch(
                index_relid,
                row,
                &query,
                top_k,
                consistency_mode,
                &mut executor_state,
            ) {
                Ok(candidates) => {
                    rows.push(SpireRemoteSearchLibpqReceiveAttemptRow {
                        requested_epoch: row.requested_epoch,
                        node_id: row.node_id,
                        selected_pids: row.selected_pids.clone(),
                        pid_count: row.pid_count,
                        candidate_count: u64::try_from(candidates.len()).map_err(|_| {
                            "ec_spire remote receive attempt candidate count exceeds u64"
                                .to_owned()
                        })?,
                        status: SPIRE_REMOTE_STATUS_READY.to_owned(),
                        next_blocker: SPIRE_REMOTE_NONE.to_owned(),
                        failure_action: SPIRE_REMOTE_NONE.to_owned(),
                        failure_reason: SPIRE_REMOTE_NONE.to_owned(),
                        recommendation: SPIRE_REMOTE_NONE.to_owned(),
                    });
                }
                Err(error) => {
                    let degraded =
                        requested_consistency_mode == meta::SpireConsistencyMode::Degraded;
                    let failure_action = if degraded {
                        "skip_node"
                    } else {
                        "fail_closed"
                    };
                    let recommendation = if degraded {
                        format!(
                            "skip node_id {} in degraded mode before merge: {error}",
                            row.node_id
                        )
                    } else {
                        format!("strict mode fails closed before merge: {error}")
                    };
                    rows.push(SpireRemoteSearchLibpqReceiveAttemptRow {
                        requested_epoch: row.requested_epoch,
                        node_id: row.node_id,
                        selected_pids: row.selected_pids.clone(),
                        pid_count: row.pid_count,
                        candidate_count: 0,
                        status: remote_search_receive_attempt_failure_status(&error),
                        next_blocker: remote_search_receive_attempt_next_blocker(&error),
                        failure_action: failure_action.to_owned(),
                        failure_reason: error,
                        recommendation,
                    });
                }
            }
        }
        Ok(rows)
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

fn remote_search_libpq_executor_heap_candidates_for_dispatch(
    index_relid: pg_sys::Oid,
    row: &SpireRemoteSearchLibpqDispatchPlanRow,
    query: &[f32],
    top_k: usize,
    consistency_mode: &str,
    executor_state: &mut SpireRemoteSearchLibpqExecutorState,
) -> Result<Vec<SpireRemoteSearchLocalHeapCandidateRow>, String> {
    if row.dispatch_action != SPIRE_REMOTE_DISPATCH_PIPELINE_ACTION {
        return Err(format!(
            "ec_spire remote heap executor dispatch for node_id {} is blocked with status {}",
            row.node_id, row.status
        ));
    }

    let _governance_permit = remote_search_libpq_executor_governance_permit(row)?;
    let conninfo = remote_conninfo_secret_value(&row.conninfo_secret_name).map_err(|status| {
        format!(
            "ec_spire remote heap executor conninfo secret for node_id {} is not resolved: {status}",
            row.node_id
        )
    })?;
    let mut client = remote_search_libpq_connect_with_session_timeouts(
        &conninfo,
        row.node_id,
        "remote heap executor",
    )?;
    let remote_index_oid = client
        .query_one(
            "SELECT to_regclass($1)::oid",
            &[&row.remote_index_regclass.as_str()],
        )
        .map_err(|_| {
            format!(
                "ec_spire remote heap executor failed to resolve remote index for node_id {}",
                row.node_id
            )
        })?
        .try_get::<_, Option<u32>>(0)
        .map_err(|_| {
            format!(
                "ec_spire remote heap executor remote index oid decode failed for node_id {}",
                row.node_id
            )
        })?
        .ok_or_else(|| {
            format!(
                "ec_spire remote heap executor remote index is missing for node_id {}",
                row.node_id
            )
        })?;
    validate_remote_search_libpq_endpoint_identity_for_dispatch(
        &mut client,
        index_relid,
        remote_index_oid,
        row,
        executor_state,
    )?;
    let requested_epoch = i64::try_from(row.requested_epoch)
        .map_err(|_| "ec_spire remote heap executor requested_epoch exceeds i64")?;
    let selected_pids = row
        .selected_pids
        .iter()
        .map(|pid| {
            i64::try_from(*pid)
                .map_err(|_| "ec_spire remote heap executor selected PID exceeds i64")
        })
        .collect::<Result<Vec<_>, _>>()?;
    let top_k =
        i32::try_from(top_k).map_err(|_| "ec_spire remote heap executor top_k exceeds i32")?;

    let result_rows = client
        .query(
            SPIRE_REMOTE_SEARCH_LIBPQ_HEAP_SQL_TEMPLATE,
            &[
                &remote_index_oid,
                &requested_epoch,
                &query,
                &selected_pids,
                &top_k,
                &consistency_mode,
            ],
        )
        .map_err(|_| {
            format!(
                "ec_spire remote heap executor remote heap query failed for node_id {}",
                row.node_id
            )
        })?;
    let candidates = result_rows
        .iter()
        .map(|candidate_row| {
            decode_remote_search_heap_candidate_pg_row(
                candidate_row,
                row.requested_epoch,
                row.node_id,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
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
    validate_remote_search_candidate_batch(
        row.requested_epoch,
        row.node_id,
        &row.selected_pids,
        &merge_candidates,
    )?;

    Ok(candidates)
}

fn remote_search_libpq_executor_candidates_from_dispatch_rows_with_state(
    index_relid: pg_sys::Oid,
    dispatch_rows: &[SpireRemoteSearchLibpqDispatchPlanRow],
    query: &[f32],
    top_k: usize,
    consistency_mode: &str,
    executor_state: &mut SpireRemoteSearchLibpqExecutorState,
) -> Result<Vec<SpireRemoteSearchCandidateRow>, String> {
    let mut candidates = Vec::new();
    for row in dispatch_rows {
        candidates.extend(remote_search_libpq_executor_candidates_for_dispatch(
            index_relid,
            row,
            query,
            top_k,
            consistency_mode,
            executor_state,
        )?);
    }
    Ok(candidates)
}

fn remote_search_libpq_executor_candidate_rows_with_state(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    query: Vec<f32>,
    selected_pids: Vec<u64>,
    top_k: usize,
    consistency_mode: &str,
    executor_state: &mut SpireRemoteSearchLibpqExecutorState,
) -> Result<Vec<SpireRemoteSearchCandidateRow>, String> {
    let dispatch_rows = unsafe {
        remote_search_libpq_dispatch_plan_rows(
            index_relation,
            requested_epoch,
            query.clone(),
            selected_pids,
            top_k,
            consistency_mode,
        )
    };
    remote_search_libpq_executor_candidates_from_dispatch_rows_with_state(
        unsafe { (*index_relation).rd_id },
        &dispatch_rows,
        &query,
        top_k,
        consistency_mode,
        executor_state,
    )
}

pub(crate) unsafe fn remote_search_libpq_executor_candidate_rows(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    query: Vec<f32>,
    selected_pids: Vec<u64>,
    top_k: usize,
    consistency_mode: &str,
) -> Vec<SpireRemoteSearchCandidateRow> {
    let result = (|| -> Result<Vec<SpireRemoteSearchCandidateRow>, String> {
        let mut executor_state = SpireRemoteSearchLibpqExecutorState::default();
        remote_search_libpq_executor_candidate_rows_with_state(
            index_relation,
            requested_epoch,
            query,
            selected_pids,
            top_k,
            consistency_mode,
            &mut executor_state,
        )
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

fn remote_search_libpq_executor_heap_candidates_from_dispatch_rows_with_state(
    index_relid: pg_sys::Oid,
    dispatch_rows: &[SpireRemoteSearchLibpqDispatchPlanRow],
    query: &[f32],
    top_k: usize,
    consistency_mode: &str,
    executor_state: &mut SpireRemoteSearchLibpqExecutorState,
) -> Result<Vec<SpireRemoteSearchLocalHeapCandidateRow>, String> {
    let mut candidates = Vec::new();
    for row in dispatch_rows {
        candidates.extend(remote_search_libpq_executor_heap_candidates_for_dispatch(
            index_relid,
            row,
            query,
            top_k,
            consistency_mode,
            executor_state,
        )?);
    }
    candidates.sort_by(|left, right| {
        remote_search_candidate_cmp(
            &SpireRemoteSearchCandidateRow {
                served_epoch: left.served_epoch,
                node_id: left.node_id,
                pid: left.pid,
                object_version: left.object_version,
                row_index: left.row_index,
                assignment_flags: left.assignment_flags,
                vec_id: left.vec_id.clone(),
                row_locator: left.row_locator.clone(),
                score: left.score,
            },
            &SpireRemoteSearchCandidateRow {
                served_epoch: right.served_epoch,
                node_id: right.node_id,
                pid: right.pid,
                object_version: right.object_version,
                row_index: right.row_index,
                assignment_flags: right.assignment_flags,
                vec_id: right.vec_id.clone(),
                row_locator: right.row_locator.clone(),
                score: right.score,
            },
        )
    });
    candidates.truncate(top_k);
    Ok(candidates)
}

fn remote_search_libpq_executor_heap_candidate_rows_with_state(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    query: Vec<f32>,
    selected_pids: Vec<u64>,
    top_k: usize,
    consistency_mode: &str,
    executor_state: &mut SpireRemoteSearchLibpqExecutorState,
) -> Result<Vec<SpireRemoteSearchLocalHeapCandidateRow>, String> {
    let dispatch_rows = unsafe {
        remote_search_libpq_dispatch_plan_rows(
            index_relation,
            requested_epoch,
            query.clone(),
            selected_pids,
            top_k,
            consistency_mode,
        )
    };
    remote_search_libpq_executor_heap_candidates_from_dispatch_rows_with_state(
        unsafe { (*index_relation).rd_id },
        &dispatch_rows,
        &query,
        top_k,
        consistency_mode,
        executor_state,
    )
}

pub(crate) unsafe fn remote_search_libpq_executor_heap_candidate_rows(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    query: Vec<f32>,
    selected_pids: Vec<u64>,
    top_k: usize,
    consistency_mode: &str,
) -> Vec<SpireRemoteSearchLocalHeapCandidateRow> {
    let result = (|| -> Result<Vec<SpireRemoteSearchLocalHeapCandidateRow>, String> {
        let mut executor_state = SpireRemoteSearchLibpqExecutorState::default();
        remote_search_libpq_executor_heap_candidate_rows_with_state(
            index_relation,
            requested_epoch,
            query,
            selected_pids,
            top_k,
            consistency_mode,
            &mut executor_state,
        )
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

pub(crate) unsafe fn remote_search_libpq_identity_cache_summary_row(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    query: Vec<f32>,
    selected_pids: Vec<u64>,
    top_k: usize,
    consistency_mode: &str,
) -> SpireRemoteSearchLibpqIdentityCacheSummaryRow {
    let result = (|| -> Result<SpireRemoteSearchLibpqIdentityCacheSummaryRow, String> {
        let index_relid = unsafe { (*index_relation).rd_id };
        let dispatch_rows = unsafe {
            remote_search_libpq_dispatch_plan_rows(
                index_relation,
                requested_epoch,
                query.clone(),
                selected_pids,
                top_k,
                consistency_mode,
            )
        };
        let dispatch_count = u64::try_from(dispatch_rows.len()).map_err(|_| {
            "ec_spire remote search libpq identity cache dispatch count exceeds u64".to_owned()
        })?;
        let first_blocked_status = dispatch_rows
            .iter()
            .find(|row| row.dispatch_action != SPIRE_REMOTE_DISPATCH_PIPELINE_ACTION)
            .map(|row| row.status);
        let mut executor_state = SpireRemoteSearchLibpqExecutorState::default();
        let (compact_candidate_count, heap_candidate_count, status) = if top_k == 0 {
            (0_u64, 0_u64, SPIRE_REMOTE_STATUS_EMPTY_TOP_K)
        } else if let Some(status) = first_blocked_status {
            (0_u64, 0_u64, status)
        } else {
            match remote_search_libpq_executor_candidates_from_dispatch_rows_with_state(
                index_relid,
                &dispatch_rows,
                &query,
                top_k,
                consistency_mode,
                &mut executor_state,
            ) {
                Ok(compact_candidates) => {
                    let heap_candidates =
                        remote_search_libpq_executor_heap_candidates_from_dispatch_rows_with_state(
                            index_relid,
                            &dispatch_rows,
                            &query,
                            top_k,
                            consistency_mode,
                            &mut executor_state,
                        )?;
                    (
                        u64::try_from(compact_candidates.len()).map_err(|_| {
                            "ec_spire remote search libpq identity cache compact candidate count exceeds u64"
                                .to_owned()
                        })?,
                        u64::try_from(heap_candidates.len()).map_err(|_| {
                            "ec_spire remote search libpq identity cache heap candidate count exceeds u64"
                                .to_owned()
                        })?,
                        SPIRE_REMOTE_STATUS_READY,
                    )
                }
                Err(error) if remote_search_receive_attempt_failure_status(&error)
                    == SPIRE_REMOTE_STATUS_ENDPOINT_IDENTITY_MISMATCH =>
                {
                    (0_u64, 0_u64, SPIRE_REMOTE_STATUS_ENDPOINT_IDENTITY_MISMATCH)
                }
                Err(error) => return Err(error),
            }
        };

        Ok(SpireRemoteSearchLibpqIdentityCacheSummaryRow {
            requested_epoch,
            dispatch_count,
            compact_candidate_count,
            heap_candidate_count,
            endpoint_identity_cache_entry_count: executor_state
                .endpoint_identity_cache_entry_count()?,
            endpoint_identity_query_count: executor_state.endpoint_identity_query_count(),
            endpoint_identity_cache_hit_count: executor_state.endpoint_identity_cache_hit_count(),
            endpoint_identity_cache_miss_count: executor_state.endpoint_identity_cache_miss_count(),
            raw_conninfo_cached: false,
            status,
        })
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

pub(crate) unsafe fn remote_search_receive_plan_rows(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    query: Vec<f32>,
    selected_pids: Vec<u64>,
    top_k: usize,
    consistency_mode: &str,
) -> Vec<SpireRemoteSearchReceivePlanRow> {
    let rows = unsafe {
        remote_search_libpq_request_plan_rows(
            index_relation,
            requested_epoch,
            query,
            selected_pids,
            top_k,
            consistency_mode,
        )
    };
    remote_search_receive_plan_rows_from_requests(&rows)
}

fn remote_search_receive_plan_rows_from_requests(
    rows: &[SpireRemoteSearchLibpqRequestPlanRow],
) -> Vec<SpireRemoteSearchReceivePlanRow> {
    rows.iter()
        .map(|row| SpireRemoteSearchReceivePlanRow {
            requested_epoch: row.requested_epoch,
            node_id: row.node_id,
            selected_pids: row.selected_pids.clone(),
            pid_count: row.pid_count,
            expected_candidate_format: row.candidate_format,
            expected_result_column_count: row.result_column_count,
            validator_function: SPIRE_REMOTE_SEARCH_RECEIVE_VALIDATOR,
            row_locator_policy: SPIRE_REMOTE_ROW_LOCATOR_POLICY,
            status: row.status,
        })
        .collect()
}

pub(crate) unsafe fn remote_search_merge_input_summary_row(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    query: Vec<f32>,
    selected_pids: Vec<u64>,
    top_k: usize,
    consistency_mode: &str,
) -> SpireRemoteSearchMergeInputSummaryRow {
    let result = (|| -> Result<SpireRemoteSearchMergeInputSummaryRow, String> {
        let execution_summary = unsafe {
            remote_search_execution_summary_row(
                index_relation,
                requested_epoch,
                query,
                selected_pids,
                top_k,
                consistency_mode,
            )
        };
        Ok(remote_search_merge_input_summary_from_execution(
            &execution_summary,
        ))
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

fn remote_search_merge_input_summary_from_execution(
    execution_summary: &SpireRemoteSearchExecutionSummaryRow,
) -> SpireRemoteSearchMergeInputSummaryRow {
    let remote_batch_count = execution_summary.remote_plan_count;
    let local_batch_count = execution_summary.local_plan_count;
    let skipped_batch_count = execution_summary.skipped_plan_count;
    let ready_batch_count = execution_summary.ready_plan_count;
    let blocked_batch_count = execution_summary.blocked_plan_count;
    let status = if execution_summary.top_k == 0 {
        SPIRE_REMOTE_STATUS_EMPTY_TOP_K
    } else if blocked_batch_count > 0 {
        execution_summary.status
    } else if execution_summary.degraded_skipped_plan_count > 0 {
        SPIRE_REMOTE_STATUS_DEGRADED_READY
    } else if remote_batch_count > 0 || local_batch_count > 0 {
        SPIRE_REMOTE_STATUS_READY
    } else if skipped_batch_count > 0 {
        SPIRE_REMOTE_STATUS_DEGRADED_READY
    } else {
        SPIRE_REMOTE_STATUS_READY
    };

    SpireRemoteSearchMergeInputSummaryRow {
        requested_epoch: execution_summary.requested_epoch,
        remote_batch_count,
        local_batch_count,
        skipped_batch_count,
        ready_batch_count,
        blocked_batch_count,
        remote_pid_count: execution_summary.remote_pid_count,
        local_pid_count: execution_summary.local_pid_count,
        skipped_pid_count: execution_summary.skipped_pid_count,
        merge_function: SPIRE_REMOTE_SEARCH_MERGE_FUNCTION,
        dedupe_key: SPIRE_REMOTE_VEC_ID_DEDUPE_KEY,
        tie_breaker: "score_then_assignment_role_then_epoch_desc_then_node_pid_version_row_locator",
        top_k: execution_summary.top_k,
        status,
    }
}
