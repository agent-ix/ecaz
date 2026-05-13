#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SpireRemoteSearchMergeResult {
    pub(crate) candidates: Vec<SpireRemoteSearchCandidateRow>,
    pub(crate) input_count: u64,
    pub(crate) duplicate_vec_id_count: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SpireRemoteSearchCandidateBatch {
    pub(crate) node_id: u32,
    pub(crate) selected_pids: Vec<u64>,
    pub(crate) candidates: Vec<SpireRemoteSearchCandidateRow>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SpireRemoteSearchFanoutTarget {
    node_id: u32,
    selected_pids: Vec<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SpireRemoteSearchSkippedPlacement {
    node_id: u32,
    pid: u64,
    state: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SpireRemoteSearchFanoutPlan {
    requested_epoch: u64,
    local_selected_pids: Vec<u64>,
    remote_targets: Vec<SpireRemoteSearchFanoutTarget>,
    skipped_placements: Vec<SpireRemoteSearchSkippedPlacement>,
}

fn plan_remote_search_fanout(
    snapshot: &meta::SpirePublishedEpochSnapshot<'_>,
    selected_leaf_pids: &[u64],
) -> Result<SpireRemoteSearchFanoutPlan, String> {
    let snapshot = meta::SpireValidatedEpochSnapshot::from_snapshot(*snapshot)?;
    if selected_leaf_pids.is_empty() {
        return Ok(SpireRemoteSearchFanoutPlan {
            requested_epoch: snapshot.epoch_manifest().epoch,
            local_selected_pids: Vec::new(),
            remote_targets: Vec::new(),
            skipped_placements: Vec::new(),
        });
    }

    let mut seen = HashSet::new();
    let mut local_selected_pids = Vec::new();
    let mut remote_by_node = BTreeMap::<u32, Vec<u64>>::new();
    let mut skipped_placements = Vec::new();

    for &pid in selected_leaf_pids {
        if pid == 0 {
            return Err("ec_spire remote search fanout selected PID 0 is invalid".to_owned());
        }
        if !seen.insert(pid) {
            return Err(format!(
                "ec_spire remote search fanout selected PID {pid} appears more than once"
            ));
        }

        let lookup = snapshot.require_lookup(pid, "remote search fanout selected leaf")?;
        if fanout_should_skip_placement(
            snapshot.epoch_manifest().consistency_mode,
            lookup.placement.state,
        )? {
            skipped_placements.push(SpireRemoteSearchSkippedPlacement {
                node_id: lookup.placement.node_id,
                pid,
                state: fanout_placement_state_name(lookup.placement.state),
            });
            continue;
        }

        if lookup.placement.node_id == meta::SPIRE_LOCAL_NODE_ID {
            local_selected_pids.push(pid);
        } else {
            remote_by_node
                .entry(lookup.placement.node_id)
                .or_default()
                .push(pid);
        }
    }

    let remote_targets = remote_by_node
        .into_iter()
        .map(|(node_id, selected_pids)| SpireRemoteSearchFanoutTarget {
            node_id,
            selected_pids,
        })
        .collect();

    Ok(SpireRemoteSearchFanoutPlan {
        requested_epoch: snapshot.epoch_manifest().epoch,
        local_selected_pids,
        remote_targets,
        skipped_placements,
    })
}

fn fanout_should_skip_placement(
    consistency_mode: meta::SpireConsistencyMode,
    state: meta::SpirePlacementState,
) -> Result<bool, String> {
    match (consistency_mode, state) {
        (_, meta::SpirePlacementState::Available) => Ok(false),
        (meta::SpireConsistencyMode::Degraded, meta::SpirePlacementState::Unavailable)
        | (meta::SpireConsistencyMode::Degraded, meta::SpirePlacementState::Skipped) => Ok(true),
        (meta::SpireConsistencyMode::Strict, state) => Err(format!(
            "ec_spire strict remote search fanout cannot skip {:?} placement",
            state
        )),
        (meta::SpireConsistencyMode::Degraded, meta::SpirePlacementState::Stale) => {
            Err("ec_spire degraded remote search fanout cannot use stale placement".to_owned())
        }
    }
}

fn fanout_placement_state_name(state: meta::SpirePlacementState) -> &'static str {
    match state {
        meta::SpirePlacementState::Available => "available",
        meta::SpirePlacementState::Stale => "stale",
        meta::SpirePlacementState::Unavailable => "unavailable",
        meta::SpirePlacementState::Skipped => "skipped",
    }
}

pub(crate) unsafe fn remote_search_fanout_plan_rows(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    selected_pids: Vec<u64>,
    consistency_mode: &str,
) -> Vec<SpireRemoteSearchFanoutPlanRow> {
    let result = (|| -> Result<Vec<SpireRemoteSearchFanoutPlanRow>, String> {
        if requested_epoch == 0 {
            return Err(
                "ec_spire remote search fanout requested_epoch must be greater than 0".to_owned(),
            );
        }
        let requested_consistency_mode = parse_remote_search_consistency_mode(consistency_mode)?;
        let root_control = unsafe { page::read_root_control_page(index_relation) };
        if root_control.active_epoch != requested_epoch {
            return Err(format!(
                "ec_spire remote search fanout requested epoch {requested_epoch} does not match active epoch {}",
                root_control.active_epoch
            ));
        }

        let (epoch_manifest, object_manifest, placement_directory) = unsafe {
            load_relation_epoch_manifests_for_coordinator_fanout(index_relation, root_control)?
        };
        if epoch_manifest.consistency_mode != requested_consistency_mode {
            return Err(format!(
                "ec_spire remote search fanout requested consistency_mode '{consistency_mode}' does not match active epoch consistency mode '{}'",
                consistency_mode_name(epoch_manifest.consistency_mode)
            ));
        }
        let snapshot = meta::SpirePublishedEpochSnapshot::new(
            &epoch_manifest,
            &object_manifest,
            &placement_directory,
        )?;
        let plan = plan_remote_search_fanout(&snapshot, &selected_pids)?;
        let validated_snapshot = meta::SpireValidatedEpochSnapshot::from_snapshot(snapshot)?;
        let mut rows = Vec::with_capacity(
            plan.local_selected_pids.len()
                + plan
                    .remote_targets
                    .iter()
                    .map(|target| target.selected_pids.len())
                    .sum::<usize>()
                + plan.skipped_placements.len(),
        );
        for pid in plan.local_selected_pids {
            let placement_state = fanout_placement_state_name(
                validated_snapshot
                    .require_lookup(pid, "remote search fanout local row")?
                    .placement
                    .state,
            );
            rows.push(SpireRemoteSearchFanoutPlanRow {
                requested_epoch: plan.requested_epoch,
                target_kind: SPIRE_REMOTE_TARGET_LOCAL,
                node_id: meta::SPIRE_LOCAL_NODE_ID,
                pid,
                placement_state,
            });
        }
        for target in plan.remote_targets {
            for pid in target.selected_pids {
                let placement_state = fanout_placement_state_name(
                    validated_snapshot
                        .require_lookup(pid, "remote search fanout remote row")?
                        .placement
                        .state,
                );
                rows.push(SpireRemoteSearchFanoutPlanRow {
                    requested_epoch: plan.requested_epoch,
                    target_kind: SPIRE_REMOTE_TARGET_REMOTE,
                    node_id: target.node_id,
                    pid,
                    placement_state,
                });
            }
        }
        rows.extend(plan.skipped_placements.into_iter().map(|skipped| {
            SpireRemoteSearchFanoutPlanRow {
                requested_epoch: plan.requested_epoch,
                target_kind: SPIRE_REMOTE_TARGET_SKIPPED,
                node_id: skipped.node_id,
                pid: skipped.pid,
                placement_state: skipped.state,
            }
        }));
        Ok(rows)
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

pub(crate) unsafe fn remote_search_target_plan_rows(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    selected_pids: Vec<u64>,
    consistency_mode: &str,
) -> Vec<SpireRemoteSearchTargetPlanRow> {
    let result = (|| -> Result<Vec<SpireRemoteSearchTargetPlanRow>, String> {
        if requested_epoch == 0 {
            return Err(
                "ec_spire remote search target plan requested_epoch must be greater than 0"
                    .to_owned(),
            );
        }
        let requested_consistency_mode = parse_remote_search_consistency_mode(consistency_mode)?;
        let root_control = unsafe { page::read_root_control_page(index_relation) };
        if root_control.active_epoch != requested_epoch {
            return Err(format!(
                "ec_spire remote search target plan requested epoch {requested_epoch} does not match active epoch {}",
                root_control.active_epoch
            ));
        }

        let (epoch_manifest, object_manifest, placement_directory) = unsafe {
            load_relation_epoch_manifests_for_coordinator_fanout(index_relation, root_control)?
        };
        if epoch_manifest.consistency_mode != requested_consistency_mode {
            return Err(format!(
                "ec_spire remote search target plan requested consistency_mode '{consistency_mode}' does not match active epoch consistency mode '{}'",
                consistency_mode_name(epoch_manifest.consistency_mode)
            ));
        }
        let snapshot = meta::SpirePublishedEpochSnapshot::new(
            &epoch_manifest,
            &object_manifest,
            &placement_directory,
        )?;
        let plan = plan_remote_search_fanout(&snapshot, &selected_pids)?;
        let mut rows = Vec::new();
        if !plan.local_selected_pids.is_empty() {
            let pid_count = u64::try_from(plan.local_selected_pids.len())
                .map_err(|_| "ec_spire remote search target plan local PID count exceeds u64")?;
            rows.push(SpireRemoteSearchTargetPlanRow {
                requested_epoch: plan.requested_epoch,
                target_kind: SPIRE_REMOTE_TARGET_LOCAL,
                node_id: meta::SPIRE_LOCAL_NODE_ID,
                selected_pids: plan.local_selected_pids,
                pid_count,
                placement_state: "available",
                status: SPIRE_REMOTE_STATUS_READY,
            });
        }
        for target in plan.remote_targets {
            let pid_count = u64::try_from(target.selected_pids.len())
                .map_err(|_| "ec_spire remote search target plan remote PID count exceeds u64")?;
            rows.push(SpireRemoteSearchTargetPlanRow {
                requested_epoch: plan.requested_epoch,
                target_kind: SPIRE_REMOTE_TARGET_REMOTE,
                node_id: target.node_id,
                selected_pids: target.selected_pids,
                pid_count,
                placement_state: "available",
                status: SPIRE_REMOTE_STATUS_REQUIRES_LIBPQ,
            });
        }

        let mut skipped_by_node_state = BTreeMap::<(u32, &'static str), Vec<u64>>::new();
        for skipped in plan.skipped_placements {
            skipped_by_node_state
                .entry((skipped.node_id, skipped.state))
                .or_default()
                .push(skipped.pid);
        }
        for ((node_id, placement_state), selected_pids) in skipped_by_node_state {
            let pid_count = u64::try_from(selected_pids.len())
                .map_err(|_| "ec_spire remote search target plan skipped PID count exceeds u64")?;
            rows.push(SpireRemoteSearchTargetPlanRow {
                requested_epoch: plan.requested_epoch,
                target_kind: SPIRE_REMOTE_TARGET_SKIPPED,
                node_id,
                selected_pids,
                pid_count,
                placement_state,
                status: SPIRE_REMOTE_STATUS_DEGRADED_SKIPPED,
            });
        }

        Ok(rows)
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

pub(crate) unsafe fn remote_search_target_readiness_rows(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    selected_pids: Vec<u64>,
    consistency_mode: &str,
) -> Vec<SpireRemoteSearchTargetReadinessRow> {
    let result = (|| -> Result<Vec<SpireRemoteSearchTargetReadinessRow>, String> {
        let target_rows = unsafe {
            remote_search_target_plan_rows(
                index_relation,
                requested_epoch,
                selected_pids,
                consistency_mode,
            )
        };
        let node_rows = unsafe { remote_node_snapshot(index_relation) }
            .into_iter()
            .map(|row| (row.node_id, row))
            .collect::<BTreeMap<_, _>>();
        let capability_rows = node_rows
            .values()
            .cloned()
            .map(remote_node_capability_plan_row)
            .map(|row| (row.node_id, row))
            .collect::<BTreeMap<_, _>>();

        target_rows
            .into_iter()
            .map(|target| {
                let node = node_rows.get(&target.node_id).ok_or_else(|| {
                    format!(
                        "ec_spire remote search target readiness missing node snapshot for node_id {}",
                        target.node_id
                    )
                })?;
                let capability = capability_rows.get(&target.node_id).ok_or_else(|| {
                    format!(
                        "ec_spire remote search target readiness missing capability plan for node_id {}",
                        target.node_id
                    )
                })?;
                let status = if target.target_kind == SPIRE_REMOTE_TARGET_SKIPPED {
                    target.status
                } else if matches!(
                    node.descriptor_state,
                    SPIRE_REMOTE_DESCRIPTOR_STATE_DISABLED | SPIRE_REMOTE_DESCRIPTOR_STATE_FAILED
                ) {
                    SPIRE_REMOTE_STATUS_REQUIRES_DESCRIPTOR
                } else if node.status != SPIRE_REMOTE_STATUS_READY {
                    node.status
                } else if target.target_kind == SPIRE_REMOTE_TARGET_REMOTE
                    && capability.status != SPIRE_REMOTE_STATUS_READY
                {
                    capability.status
                } else {
                    target.status
                };
                Ok(SpireRemoteSearchTargetReadinessRow {
                    requested_epoch: target.requested_epoch,
                    target_kind: target.target_kind,
                    node_id: target.node_id,
                    selected_pids: target.selected_pids,
                    pid_count: target.pid_count,
                    placement_state: target.placement_state,
                    node_kind: node.node_kind,
                    descriptor_state: node.descriptor_state,
                    node_status: node.status,
                    status,
                })
            })
            .collect()
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

pub(crate) unsafe fn remote_search_request_plan_rows(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    query: Vec<f32>,
    selected_pids: Vec<u64>,
    top_k: usize,
    consistency_mode: &str,
) -> Vec<SpireRemoteSearchRequestPlanRow> {
    let result = (|| -> Result<Vec<SpireRemoteSearchRequestPlanRow>, String> {
        let query = scan::SpireScanQuery::new(query)?;
        let query_dimension = u64::try_from(query.values().len())
            .map_err(|_| "ec_spire remote search request plan query dimension exceeds u64")?;
        let top_k = u64::try_from(top_k)
            .map_err(|_| "ec_spire remote search request plan top_k exceeds u64")?;
        let requested_consistency_mode = parse_remote_search_consistency_mode(consistency_mode)?;
        let rows = unsafe {
            remote_search_target_plan_rows(
                index_relation,
                requested_epoch,
                selected_pids,
                consistency_mode,
            )
        };
        Ok(rows
            .into_iter()
            .map(|row| SpireRemoteSearchRequestPlanRow {
                requested_epoch: row.requested_epoch,
                target_kind: row.target_kind,
                node_id: row.node_id,
                selected_pids: row.selected_pids,
                pid_count: row.pid_count,
                query_dimension,
                top_k,
                consistency_mode: consistency_mode_name(requested_consistency_mode),
                endpoint_function: if row.target_kind == SPIRE_REMOTE_TARGET_SKIPPED {
                    SPIRE_REMOTE_NONE
                } else {
                    SPIRE_REMOTE_ENDPOINT_SEARCH
                },
                status: row.status,
            })
            .collect())
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

pub(crate) unsafe fn remote_search_request_readiness_rows(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    query: Vec<f32>,
    selected_pids: Vec<u64>,
    top_k: usize,
    consistency_mode: &str,
) -> Vec<SpireRemoteSearchRequestReadinessRow> {
    let result = (|| -> Result<Vec<SpireRemoteSearchRequestReadinessRow>, String> {
        let query = scan::SpireScanQuery::new(query)?;
        let query_dimension = u64::try_from(query.values().len())
            .map_err(|_| "ec_spire remote search request readiness query dimension exceeds u64")?;
        let top_k = u64::try_from(top_k)
            .map_err(|_| "ec_spire remote search request readiness top_k exceeds u64")?;
        let requested_consistency_mode = parse_remote_search_consistency_mode(consistency_mode)?;
        let rows = unsafe {
            remote_search_target_readiness_rows(
                index_relation,
                requested_epoch,
                selected_pids,
                consistency_mode,
            )
        };
        Ok(rows
            .into_iter()
            .map(|row| SpireRemoteSearchRequestReadinessRow {
                requested_epoch: row.requested_epoch,
                target_kind: row.target_kind,
                node_id: row.node_id,
                selected_pids: row.selected_pids,
                pid_count: row.pid_count,
                query_dimension,
                top_k,
                consistency_mode: consistency_mode_name(requested_consistency_mode),
                endpoint_function: if row.target_kind == SPIRE_REMOTE_TARGET_SKIPPED {
                    SPIRE_REMOTE_NONE
                } else {
                    SPIRE_REMOTE_ENDPOINT_SEARCH
                },
                node_kind: row.node_kind,
                descriptor_state: row.descriptor_state,
                node_status: row.node_status,
                status: row.status,
            })
            .collect())
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

pub(crate) unsafe fn remote_search_request_summary_row(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    query: Vec<f32>,
    selected_pids: Vec<u64>,
    top_k: usize,
    consistency_mode: &str,
) -> SpireRemoteSearchRequestSummaryRow {
    let result = (|| -> Result<SpireRemoteSearchRequestSummaryRow, String> {
        let query_for_empty_plan = query.clone();
        let top_k_for_empty_plan = u64::try_from(top_k)
            .map_err(|_| "ec_spire remote search request summary top_k exceeds u64")?;
        let rows = unsafe {
            remote_search_request_plan_rows(
                index_relation,
                requested_epoch,
                query,
                selected_pids,
                top_k,
                consistency_mode,
            )
        };
        let mut rollup = SpireRemoteCountRollup::default();
        let mut query_dimension = 0_u64;
        let mut top_k = 0_u64;
        let mut parsed_consistency_mode = "";

        for row in &rows {
            query_dimension = row.query_dimension;
            top_k = row.top_k;
            parsed_consistency_mode = row.consistency_mode;
            rollup.record_target(row.target_kind, row.pid_count, "remote search request summary")?;
        }

        if rows.is_empty() {
            let query = scan::SpireScanQuery::new(query_for_empty_plan)?;
            query_dimension = u64::try_from(query.values().len()).map_err(|_| {
                "ec_spire remote search request summary query dimension exceeds u64"
            })?;
            top_k = top_k_for_empty_plan;
            parsed_consistency_mode =
                consistency_mode_name(parse_remote_search_consistency_mode(consistency_mode)?);
        }

        let request_count = u64::try_from(rows.len())
            .map_err(|_| "ec_spire remote search request summary request count exceeds u64")?;
        let executable_pid_count = rollup.executable_pid_count("remote search request summary")?;
        let status = rollup.summary_status(top_k, SpireRemoteSummaryStatusMode::RequestPlan);

        Ok(SpireRemoteSearchRequestSummaryRow {
            requested_epoch,
            request_count,
            local_request_count: rollup.local_count,
            remote_request_count: rollup.remote_count,
            skipped_request_count: rollup.skipped_count,
            executable_pid_count,
            local_pid_count: rollup.local_pid_count,
            remote_pid_count: rollup.remote_pid_count,
            skipped_pid_count: rollup.skipped_pid_count,
            query_dimension,
            top_k,
            consistency_mode: parsed_consistency_mode,
            status,
        })
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

pub(crate) unsafe fn remote_search_readiness_summary_row(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    query: Vec<f32>,
    selected_pids: Vec<u64>,
    top_k: usize,
    consistency_mode: &str,
) -> SpireRemoteSearchReadinessSummaryRow {
    let result = (|| -> Result<SpireRemoteSearchReadinessSummaryRow, String> {
        let query_for_empty_plan = query.clone();
        let top_k_for_empty_plan = u64::try_from(top_k)
            .map_err(|_| "ec_spire remote search readiness summary top_k exceeds u64")?;
        let rows = unsafe {
            remote_search_request_readiness_rows(
                index_relation,
                requested_epoch,
                query,
                selected_pids,
                top_k,
                consistency_mode,
            )
        };
        let mut rollup = SpireRemoteCountRollup::default();
        let mut query_dimension = 0_u64;
        let mut top_k = 0_u64;
        let mut parsed_consistency_mode = "";

        for row in &rows {
            query_dimension = row.query_dimension;
            top_k = row.top_k;
            parsed_consistency_mode = row.consistency_mode;
            rollup.record_target(row.target_kind, row.pid_count, "remote search readiness summary")?;
            rollup.record_status(row.status, row.pid_count, "remote search readiness summary")?;
        }

        if rows.is_empty() {
            let query = scan::SpireScanQuery::new(query_for_empty_plan)?;
            query_dimension = u64::try_from(query.values().len()).map_err(|_| {
                "ec_spire remote search readiness summary query dimension exceeds u64"
            })?;
            top_k = top_k_for_empty_plan;
            parsed_consistency_mode =
                consistency_mode_name(parse_remote_search_consistency_mode(consistency_mode)?);
        }

        let request_count = u64::try_from(rows.len())
            .map_err(|_| "ec_spire remote search readiness summary request count exceeds u64")?;
        let status = rollup.summary_status(top_k, SpireRemoteSummaryStatusMode::Readiness);

        Ok(SpireRemoteSearchReadinessSummaryRow {
            requested_epoch,
            request_count,
            ready_request_count: rollup.ready_count,
            blocked_request_count: rollup.blocked_count,
            local_request_count: rollup.local_count,
            remote_request_count: rollup.remote_count,
            skipped_request_count: rollup.skipped_count,
            executable_pid_count: rollup.executable_pid_count("remote search readiness summary")?,
            ready_pid_count: rollup.ready_pid_count,
            blocked_pid_count: rollup.blocked_pid_count,
            skipped_pid_count: rollup.skipped_pid_count,
            missing_descriptor_request_count: rollup.missing_descriptor_count,
            missing_descriptor_pid_count: rollup.missing_descriptor_pid_count,
            transport_request_count: rollup.transport_count,
            transport_pid_count: rollup.transport_pid_count,
            query_dimension,
            top_k,
            consistency_mode: parsed_consistency_mode,
            status,
        })
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

pub(crate) unsafe fn remote_search_execution_plan_rows(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    query: Vec<f32>,
    selected_pids: Vec<u64>,
    top_k: usize,
    consistency_mode: &str,
) -> Vec<SpireRemoteSearchExecutionPlanRow> {
    let rows = unsafe {
        remote_search_request_readiness_rows(
            index_relation,
            requested_epoch,
            query,
            selected_pids,
            top_k,
            consistency_mode,
        )
    };
    rows.into_iter()
        .map(|row| {
            remote_search_execution_plan_row_from_readiness(row)
        })
        .collect()
}

fn remote_search_execution_plan_row_from_readiness(
    row: SpireRemoteSearchRequestReadinessRow,
) -> SpireRemoteSearchExecutionPlanRow {
    let (execution_transport, remote_index_source, conninfo_source, candidate_format) =
        match row.target_kind {
            SPIRE_REMOTE_TARGET_LOCAL => (
                SPIRE_REMOTE_TRANSPORT_LOCAL_DIRECT,
                SPIRE_REMOTE_INDEX_SOURCE_LOCAL_OID,
                SPIRE_REMOTE_CANDIDATE_FORMAT_LOCAL,
                SPIRE_REMOTE_CANDIDATE_FORMAT_LOCAL,
            ),
            SPIRE_REMOTE_TARGET_REMOTE => (
                SPIRE_REMOTE_TRANSPORT_LIBPQ_PIPELINE,
                SPIRE_REMOTE_DESCRIPTOR_SOURCE,
                SPIRE_REMOTE_DESCRIPTOR_SOURCE,
                SPIRE_REMOTE_CANDIDATE_FORMAT_V1,
            ),
            SPIRE_REMOTE_TARGET_SKIPPED => (
                SPIRE_REMOTE_NONE,
                SPIRE_REMOTE_NONE,
                SPIRE_REMOTE_NONE,
                SPIRE_REMOTE_NONE,
            ),
            _ => ("unknown", "unknown", "unknown", "unknown"),
        };
    SpireRemoteSearchExecutionPlanRow {
        requested_epoch: row.requested_epoch,
        target_kind: row.target_kind,
        node_id: row.node_id,
        selected_pids: row.selected_pids,
        pid_count: row.pid_count,
        query_dimension: row.query_dimension,
        top_k: row.top_k,
        consistency_mode: row.consistency_mode,
        execution_transport,
        endpoint_function: row.endpoint_function,
        remote_index_source,
        conninfo_source,
        candidate_format,
        status: row.status,
    }
}

pub(crate) unsafe fn remote_search_execution_summary_row(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    query: Vec<f32>,
    selected_pids: Vec<u64>,
    top_k: usize,
    consistency_mode: &str,
) -> SpireRemoteSearchExecutionSummaryRow {
    let result = (|| -> Result<SpireRemoteSearchExecutionSummaryRow, String> {
        let query_for_empty_plan = query.clone();
        let top_k_for_empty_plan = u64::try_from(top_k)
            .map_err(|_| "ec_spire remote search execution summary top_k exceeds u64")?;
        let rows = unsafe {
            remote_search_execution_plan_rows(
                index_relation,
                requested_epoch,
                query,
                selected_pids,
                top_k,
                consistency_mode,
            )
        };
        remote_search_execution_summary_from_plan_rows(
            requested_epoch,
            &rows,
            query_for_empty_plan,
            top_k_for_empty_plan,
            consistency_mode,
        )
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

fn remote_search_execution_summary_from_plan_rows(
    requested_epoch: u64,
    rows: &[SpireRemoteSearchExecutionPlanRow],
    query_for_empty_plan: Vec<f32>,
    top_k_for_empty_plan: u64,
    consistency_mode: &str,
) -> Result<SpireRemoteSearchExecutionSummaryRow, String> {
        let mut rollup = SpireRemoteCountRollup::default();
        let mut query_dimension = 0_u64;
        let mut top_k = 0_u64;
        let mut parsed_consistency_mode = "";

        for row in rows {
            query_dimension = row.query_dimension;
            top_k = row.top_k;
            parsed_consistency_mode = row.consistency_mode;
            rollup.record_target(row.target_kind, row.pid_count, "remote search execution summary")?;
            rollup.record_status(row.status, row.pid_count, "remote search execution summary")?;
        }

        if rows.is_empty() {
            let query = scan::SpireScanQuery::new(query_for_empty_plan)?;
            query_dimension = u64::try_from(query.values().len()).map_err(|_| {
                "ec_spire remote search execution summary query dimension exceeds u64"
            })?;
            top_k = top_k_for_empty_plan;
            parsed_consistency_mode =
                consistency_mode_name(parse_remote_search_consistency_mode(consistency_mode)?);
        }

        let plan_count = u64::try_from(rows.len())
            .map_err(|_| "ec_spire remote search execution summary plan count exceeds u64")?;
        let status = rollup.summary_status(top_k, SpireRemoteSummaryStatusMode::Execution);

        Ok(SpireRemoteSearchExecutionSummaryRow {
            requested_epoch,
            plan_count,
            local_plan_count: rollup.local_count,
            remote_plan_count: rollup.remote_count,
            skipped_plan_count: rollup.skipped_count,
            ready_plan_count: rollup.ready_count,
            blocked_plan_count: rollup.blocked_count,
            degraded_skipped_plan_count: rollup.degraded_skipped_count,
            local_pid_count: rollup.local_pid_count,
            remote_pid_count: rollup.remote_pid_count,
            skipped_pid_count: rollup.skipped_pid_count,
            blocked_pid_count: rollup.blocked_pid_count,
            query_dimension,
            top_k,
            consistency_mode: parsed_consistency_mode,
            status,
        })
}

