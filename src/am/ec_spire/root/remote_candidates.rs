fn remote_candidate_assignment_role_rank(candidate: &SpireRemoteSearchCandidateRow) -> u8 {
    u8::from(candidate.assignment_flags & storage::SPIRE_ASSIGNMENT_FLAG_BOUNDARY_REPLICA != 0)
}

fn remote_search_candidate_cmp(
    left: &SpireRemoteSearchCandidateRow,
    right: &SpireRemoteSearchCandidateRow,
) -> std::cmp::Ordering {
    left.score
        .total_cmp(&right.score)
        .then_with(|| {
            remote_candidate_assignment_role_rank(left)
                .cmp(&remote_candidate_assignment_role_rank(right))
        })
        .then_with(|| right.served_epoch.cmp(&left.served_epoch))
        .then_with(|| left.node_id.cmp(&right.node_id))
        .then_with(|| left.pid.cmp(&right.pid))
        .then_with(|| right.object_version.cmp(&left.object_version))
        .then_with(|| left.row_index.cmp(&right.row_index))
        .then_with(|| left.row_locator.cmp(&right.row_locator))
}

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

        let (epoch_manifest, object_manifest, placement_directory) =
            unsafe { scan::load_relation_epoch_manifests(index_relation, root_control)? };
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
                target_kind: "local",
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
                    target_kind: "remote",
                    node_id: target.node_id,
                    pid,
                    placement_state,
                });
            }
        }
        rows.extend(plan.skipped_placements.into_iter().map(|skipped| {
            SpireRemoteSearchFanoutPlanRow {
                requested_epoch: plan.requested_epoch,
                target_kind: "skipped",
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
                target_kind: "local",
                node_id: meta::SPIRE_LOCAL_NODE_ID,
                selected_pids: plan.local_selected_pids,
                pid_count,
                placement_state: "available",
                status: "ready",
            });
        }
        for target in plan.remote_targets {
            let pid_count = u64::try_from(target.selected_pids.len())
                .map_err(|_| "ec_spire remote search target plan remote PID count exceeds u64")?;
            rows.push(SpireRemoteSearchTargetPlanRow {
                requested_epoch: plan.requested_epoch,
                target_kind: "remote",
                node_id: target.node_id,
                selected_pids: target.selected_pids,
                pid_count,
                placement_state: "available",
                status: "requires_libpq_transport",
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
                target_kind: "skipped",
                node_id,
                selected_pids,
                pid_count,
                placement_state,
                status: "degraded_skipped",
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

        target_rows
            .into_iter()
            .map(|target| {
                let node = node_rows.get(&target.node_id).ok_or_else(|| {
                    format!(
                        "ec_spire remote search target readiness missing node snapshot for node_id {}",
                        target.node_id
                    )
                })?;
                let status = if target.target_kind == "skipped" {
                    target.status
                } else if node.status != "ready" {
                    node.status
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
                endpoint_function: if row.target_kind == "skipped" {
                    "none"
                } else {
                    "ec_spire_remote_search"
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
                endpoint_function: if row.target_kind == "skipped" {
                    "none"
                } else {
                    "ec_spire_remote_search"
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
        let mut local_request_count = 0_u64;
        let mut remote_request_count = 0_u64;
        let mut skipped_request_count = 0_u64;
        let mut local_pid_count = 0_u64;
        let mut remote_pid_count = 0_u64;
        let mut skipped_pid_count = 0_u64;
        let mut query_dimension = 0_u64;
        let mut top_k = 0_u64;
        let mut parsed_consistency_mode = "";

        for row in &rows {
            query_dimension = row.query_dimension;
            top_k = row.top_k;
            parsed_consistency_mode = row.consistency_mode;
            match row.target_kind {
                "local" => {
                    local_request_count += 1;
                    local_pid_count += row.pid_count;
                }
                "remote" => {
                    remote_request_count += 1;
                    remote_pid_count += row.pid_count;
                }
                "skipped" => {
                    skipped_request_count += 1;
                    skipped_pid_count += row.pid_count;
                }
                target_kind => {
                    return Err(format!(
                        "ec_spire remote search request summary found unknown target_kind '{target_kind}'"
                    ));
                }
            }
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
        let executable_pid_count = local_pid_count
            .checked_add(remote_pid_count)
            .ok_or("ec_spire remote search request summary executable PID count overflowed")?;
        let status = if top_k == 0 {
            "empty_top_k"
        } else if remote_request_count > 0 {
            "requires_libpq_transport"
        } else if skipped_request_count > 0 {
            "degraded_ready"
        } else {
            "ready"
        };

        Ok(SpireRemoteSearchRequestSummaryRow {
            requested_epoch,
            request_count,
            local_request_count,
            remote_request_count,
            skipped_request_count,
            executable_pid_count,
            local_pid_count,
            remote_pid_count,
            skipped_pid_count,
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
        let mut ready_request_count = 0_u64;
        let mut blocked_request_count = 0_u64;
        let mut local_request_count = 0_u64;
        let mut remote_request_count = 0_u64;
        let mut skipped_request_count = 0_u64;
        let mut executable_pid_count = 0_u64;
        let mut ready_pid_count = 0_u64;
        let mut blocked_pid_count = 0_u64;
        let mut skipped_pid_count = 0_u64;
        let mut missing_descriptor_request_count = 0_u64;
        let mut missing_descriptor_pid_count = 0_u64;
        let mut transport_request_count = 0_u64;
        let mut transport_pid_count = 0_u64;
        let mut query_dimension = 0_u64;
        let mut top_k = 0_u64;
        let mut parsed_consistency_mode = "";

        for row in &rows {
            query_dimension = row.query_dimension;
            top_k = row.top_k;
            parsed_consistency_mode = row.consistency_mode;
            match row.target_kind {
                "local" => {
                    local_request_count += 1;
                    executable_pid_count = executable_pid_count.checked_add(row.pid_count).ok_or(
                        "ec_spire remote search readiness summary executable PID count overflowed",
                    )?;
                }
                "remote" => {
                    remote_request_count += 1;
                    executable_pid_count = executable_pid_count.checked_add(row.pid_count).ok_or(
                        "ec_spire remote search readiness summary executable PID count overflowed",
                    )?;
                }
                "skipped" => {
                    skipped_request_count += 1;
                    skipped_pid_count = skipped_pid_count.checked_add(row.pid_count).ok_or(
                        "ec_spire remote search readiness summary skipped PID count overflowed",
                    )?;
                }
                target_kind => {
                    return Err(format!(
                        "ec_spire remote search readiness summary found unknown target_kind '{target_kind}'"
                    ));
                }
            }

            match row.status {
                "ready" => {
                    ready_request_count += 1;
                    ready_pid_count = ready_pid_count.checked_add(row.pid_count).ok_or(
                        "ec_spire remote search readiness summary ready PID count overflowed",
                    )?;
                }
                "degraded_skipped" => {}
                "requires_remote_node_descriptor" => {
                    blocked_request_count += 1;
                    missing_descriptor_request_count += 1;
                    blocked_pid_count = blocked_pid_count.checked_add(row.pid_count).ok_or(
                        "ec_spire remote search readiness summary blocked PID count overflowed",
                    )?;
                    missing_descriptor_pid_count = missing_descriptor_pid_count
                        .checked_add(row.pid_count)
                        .ok_or(
                            "ec_spire remote search readiness summary missing descriptor PID count overflowed",
                        )?;
                }
                "requires_libpq_transport" => {
                    blocked_request_count += 1;
                    transport_request_count += 1;
                    blocked_pid_count = blocked_pid_count.checked_add(row.pid_count).ok_or(
                        "ec_spire remote search readiness summary blocked PID count overflowed",
                    )?;
                    transport_pid_count = transport_pid_count.checked_add(row.pid_count).ok_or(
                        "ec_spire remote search readiness summary transport PID count overflowed",
                    )?;
                }
                status => {
                    return Err(format!(
                        "ec_spire remote search readiness summary found unknown status '{status}'"
                    ));
                }
            }
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
        let status = if top_k == 0 {
            "empty_top_k"
        } else if missing_descriptor_request_count > 0 {
            "requires_remote_node_descriptor"
        } else if transport_request_count > 0 {
            "requires_libpq_transport"
        } else if skipped_request_count > 0 {
            "degraded_ready"
        } else {
            "ready"
        };

        Ok(SpireRemoteSearchReadinessSummaryRow {
            requested_epoch,
            request_count,
            ready_request_count,
            blocked_request_count,
            local_request_count,
            remote_request_count,
            skipped_request_count,
            executable_pid_count,
            ready_pid_count,
            blocked_pid_count,
            skipped_pid_count,
            missing_descriptor_request_count,
            missing_descriptor_pid_count,
            transport_request_count,
            transport_pid_count,
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
            let (execution_transport, remote_index_source, conninfo_source, candidate_format) =
                match row.target_kind {
                    "local" => ("local_direct", "local_index_oid", "local", "local"),
                    "remote" => (
                        "libpq_pipeline",
                        "remote_node_descriptor",
                        "remote_node_descriptor",
                        "ec_spire_remote_search_v1",
                    ),
                    "skipped" => ("none", "none", "none", "none"),
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
        })
        .collect()
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
        let mut local_plan_count = 0_u64;
        let mut remote_plan_count = 0_u64;
        let mut skipped_plan_count = 0_u64;
        let mut ready_plan_count = 0_u64;
        let mut blocked_plan_count = 0_u64;
        let mut degraded_skipped_plan_count = 0_u64;
        let mut local_pid_count = 0_u64;
        let mut remote_pid_count = 0_u64;
        let mut skipped_pid_count = 0_u64;
        let mut blocked_pid_count = 0_u64;
        let mut missing_descriptor_plan_count = 0_u64;
        let mut transport_plan_count = 0_u64;
        let mut query_dimension = 0_u64;
        let mut top_k = 0_u64;
        let mut parsed_consistency_mode = "";

        for row in &rows {
            query_dimension = row.query_dimension;
            top_k = row.top_k;
            parsed_consistency_mode = row.consistency_mode;
            match row.target_kind {
                "local" => {
                    local_plan_count += 1;
                    local_pid_count = local_pid_count.checked_add(row.pid_count).ok_or(
                        "ec_spire remote search execution summary local PID count overflowed",
                    )?;
                }
                "remote" => {
                    remote_plan_count += 1;
                    remote_pid_count = remote_pid_count.checked_add(row.pid_count).ok_or(
                        "ec_spire remote search execution summary remote PID count overflowed",
                    )?;
                }
                "skipped" => {
                    skipped_plan_count += 1;
                    skipped_pid_count = skipped_pid_count.checked_add(row.pid_count).ok_or(
                        "ec_spire remote search execution summary skipped PID count overflowed",
                    )?;
                }
                target_kind => {
                    return Err(format!(
                        "ec_spire remote search execution summary found unknown target_kind '{target_kind}'"
                    ));
                }
            }

            match row.status {
                "ready" => {
                    ready_plan_count += 1;
                }
                "degraded_skipped" => {
                    degraded_skipped_plan_count += 1;
                }
                "requires_remote_node_descriptor" => {
                    blocked_plan_count += 1;
                    missing_descriptor_plan_count += 1;
                    blocked_pid_count = blocked_pid_count.checked_add(row.pid_count).ok_or(
                        "ec_spire remote search execution summary blocked PID count overflowed",
                    )?;
                }
                "requires_libpq_transport" => {
                    blocked_plan_count += 1;
                    transport_plan_count += 1;
                    blocked_pid_count = blocked_pid_count.checked_add(row.pid_count).ok_or(
                        "ec_spire remote search execution summary blocked PID count overflowed",
                    )?;
                }
                status => {
                    return Err(format!(
                        "ec_spire remote search execution summary found unknown status '{status}'"
                    ));
                }
            }
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
        let status = if top_k == 0 {
            "empty_top_k"
        } else if missing_descriptor_plan_count > 0 {
            "requires_remote_node_descriptor"
        } else if transport_plan_count > 0 {
            "requires_libpq_transport"
        } else if degraded_skipped_plan_count > 0 {
            "degraded_ready"
        } else {
            "ready"
        };

        Ok(SpireRemoteSearchExecutionSummaryRow {
            requested_epoch,
            plan_count,
            local_plan_count,
            remote_plan_count,
            skipped_plan_count,
            ready_plan_count,
            blocked_plan_count,
            degraded_skipped_plan_count,
            local_pid_count,
            remote_pid_count,
            skipped_pid_count,
            blocked_pid_count,
            query_dimension,
            top_k,
            consistency_mode: parsed_consistency_mode,
            status,
        })
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

const SPIRE_REMOTE_SEARCH_LIBPQ_SQL_TEMPLATE: &str =
    "SELECT * FROM ec_spire_remote_search($1::oid, $2::bigint, $3::real[], $4::bigint[], $5::integer, $6::text)";
const SPIRE_REMOTE_SEARCH_LIBPQ_PARAMETER_COUNT: u64 = 6;
const SPIRE_REMOTE_SEARCH_RECEIVE_VALIDATOR: &str = "validate_remote_search_candidate_batch";
const SPIRE_REMOTE_SEARCH_MERGE_FUNCTION: &str =
    "merge_validated_remote_search_candidate_batches";

fn remote_search_result_column_count() -> u64 {
    u64::try_from(remote_search_libpq_result_contract_rows().len())
        .expect("remote search result contract row count should fit in u64")
}

pub(crate) unsafe fn remote_search_libpq_request_plan_rows(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    query: Vec<f32>,
    selected_pids: Vec<u64>,
    top_k: usize,
    consistency_mode: &str,
) -> Vec<SpireRemoteSearchLibpqRequestPlanRow> {
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
    rows.into_iter()
        .filter(|row| row.target_kind == "remote")
        .map(|row| SpireRemoteSearchLibpqRequestPlanRow {
            requested_epoch: row.requested_epoch,
            node_id: row.node_id,
            selected_pids: row.selected_pids,
            pid_count: row.pid_count,
            query_dimension: row.query_dimension,
            top_k: row.top_k,
            consistency_mode: row.consistency_mode,
            execution_transport: row.execution_transport,
            sql_template: SPIRE_REMOTE_SEARCH_LIBPQ_SQL_TEMPLATE,
            parameter_count: SPIRE_REMOTE_SEARCH_LIBPQ_PARAMETER_COUNT,
            result_column_count: remote_search_result_column_count(),
            remote_index_source: row.remote_index_source,
            conninfo_source: row.conninfo_source,
            candidate_format: row.candidate_format,
            status: row.status,
        })
        .collect()
}

pub(crate) unsafe fn remote_search_libpq_request_summary_row(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    query: Vec<f32>,
    selected_pids: Vec<u64>,
    top_k: usize,
    consistency_mode: &str,
) -> SpireRemoteSearchLibpqRequestSummaryRow {
    let result = (|| -> Result<SpireRemoteSearchLibpqRequestSummaryRow, String> {
        let query_for_empty_plan = query.clone();
        let top_k_for_empty_plan = u64::try_from(top_k)
            .map_err(|_| "ec_spire remote search libpq request summary top_k exceeds u64")?;
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
        let mut ready_request_count = 0_u64;
        let mut blocked_request_count = 0_u64;
        let mut remote_pid_count = 0_u64;
        let mut blocked_pid_count = 0_u64;
        let mut missing_descriptor_request_count = 0_u64;
        let mut transport_request_count = 0_u64;
        let mut query_dimension = 0_u64;
        let mut top_k = 0_u64;
        let mut parsed_consistency_mode = "";

        for row in &rows {
            query_dimension = row.query_dimension;
            top_k = row.top_k;
            parsed_consistency_mode = row.consistency_mode;
            remote_pid_count = remote_pid_count.checked_add(row.pid_count).ok_or(
                "ec_spire remote search libpq request summary remote PID count overflowed",
            )?;
            match row.status {
                "ready" => {
                    ready_request_count += 1;
                }
                "requires_remote_node_descriptor" => {
                    blocked_request_count += 1;
                    missing_descriptor_request_count += 1;
                    blocked_pid_count = blocked_pid_count.checked_add(row.pid_count).ok_or(
                        "ec_spire remote search libpq request summary blocked PID count overflowed",
                    )?;
                }
                "requires_libpq_transport" => {
                    blocked_request_count += 1;
                    transport_request_count += 1;
                    blocked_pid_count = blocked_pid_count.checked_add(row.pid_count).ok_or(
                        "ec_spire remote search libpq request summary blocked PID count overflowed",
                    )?;
                }
                status => {
                    return Err(format!(
                        "ec_spire remote search libpq request summary found unknown status '{status}'"
                    ));
                }
            }
        }

        if rows.is_empty() {
            let query = scan::SpireScanQuery::new(query_for_empty_plan)?;
            query_dimension = u64::try_from(query.values().len()).map_err(|_| {
                "ec_spire remote search libpq request summary query dimension exceeds u64"
            })?;
            top_k = top_k_for_empty_plan;
            parsed_consistency_mode =
                consistency_mode_name(parse_remote_search_consistency_mode(consistency_mode)?);
        }

        let request_count = u64::try_from(rows.len())
            .map_err(|_| "ec_spire remote search libpq request summary request count exceeds u64")?;
        let status = if top_k == 0 {
            "empty_top_k"
        } else if missing_descriptor_request_count > 0 {
            "requires_remote_node_descriptor"
        } else if transport_request_count > 0 {
            "requires_libpq_transport"
        } else {
            "ready"
        };

        Ok(SpireRemoteSearchLibpqRequestSummaryRow {
            requested_epoch,
            request_count,
            ready_request_count,
            blocked_request_count,
            remote_pid_count,
            blocked_pid_count,
            parameter_count_per_request: SPIRE_REMOTE_SEARCH_LIBPQ_PARAMETER_COUNT,
            result_column_count: remote_search_result_column_count(),
            query_dimension,
            top_k,
            consistency_mode: parsed_consistency_mode,
            status,
        })
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

pub(crate) fn remote_search_libpq_result_contract_rows(
) -> Vec<SpireRemoteSearchLibpqResultContractRow> {
    vec![
        SpireRemoteSearchLibpqResultContractRow {
            column_ordinal: 1,
            column_name: "served_epoch",
            pg_type: "bigint",
            semantic_role: "candidate_epoch",
            nullable: false,
            validator: "must_equal_requested_epoch",
        },
        SpireRemoteSearchLibpqResultContractRow {
            column_ordinal: 2,
            column_name: "node_id",
            pg_type: "bigint",
            semantic_role: "candidate_node",
            nullable: false,
            validator: "must_equal_expected_node_id",
        },
        SpireRemoteSearchLibpqResultContractRow {
            column_ordinal: 3,
            column_name: "pid",
            pg_type: "bigint",
            semantic_role: "partition_object",
            nullable: false,
            validator: "must_be_selected_pid",
        },
        SpireRemoteSearchLibpqResultContractRow {
            column_ordinal: 4,
            column_name: "object_version",
            pg_type: "bigint",
            semantic_role: "partition_object_version",
            nullable: false,
            validator: "must_be_positive",
        },
        SpireRemoteSearchLibpqResultContractRow {
            column_ordinal: 5,
            column_name: "row_index",
            pg_type: "bigint",
            semantic_role: "candidate_row_index",
            nullable: false,
            validator: "must_fit_u32",
        },
        SpireRemoteSearchLibpqResultContractRow {
            column_ordinal: 6,
            column_name: "assignment_flags",
            pg_type: "smallint",
            semantic_role: "candidate_assignment_flags",
            nullable: false,
            validator: "must_include_primary_or_boundary_replica",
        },
        SpireRemoteSearchLibpqResultContractRow {
            column_ordinal: 7,
            column_name: "vec_id",
            pg_type: "bytea",
            semantic_role: "dedupe_key",
            nullable: false,
            validator: "must_be_nonempty",
        },
        SpireRemoteSearchLibpqResultContractRow {
            column_ordinal: 8,
            column_name: "row_locator",
            pg_type: "bytea",
            semantic_role: "origin_node_locator",
            nullable: false,
            validator: "must_be_nonempty_and_opaque",
        },
        SpireRemoteSearchLibpqResultContractRow {
            column_ordinal: 9,
            column_name: "score",
            pg_type: "real",
            semantic_role: "candidate_score",
            nullable: false,
            validator: "must_be_finite",
        },
    ]
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
    rows.into_iter()
        .map(|row| SpireRemoteSearchReceivePlanRow {
            requested_epoch: row.requested_epoch,
            node_id: row.node_id,
            selected_pids: row.selected_pids,
            pid_count: row.pid_count,
            expected_candidate_format: row.candidate_format,
            expected_result_column_count: row.result_column_count,
            validator_function: SPIRE_REMOTE_SEARCH_RECEIVE_VALIDATOR,
            row_locator_policy: "opaque_origin_node_bytes",
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
        let remote_batch_count = execution_summary.remote_plan_count;
        let local_batch_count = execution_summary.local_plan_count;
        let skipped_batch_count = execution_summary.skipped_plan_count;
        let ready_batch_count = execution_summary.ready_plan_count;
        let blocked_batch_count = execution_summary.blocked_plan_count;
        let status = if execution_summary.top_k == 0 {
            "empty_top_k"
        } else if blocked_batch_count > 0 {
            execution_summary.status
        } else if remote_batch_count > 0 || local_batch_count > 0 {
            "ready"
        } else if skipped_batch_count > 0 {
            "degraded_ready"
        } else {
            "ready"
        };

        Ok(SpireRemoteSearchMergeInputSummaryRow {
            requested_epoch,
            remote_batch_count,
            local_batch_count,
            skipped_batch_count,
            ready_batch_count,
            blocked_batch_count,
            remote_pid_count: execution_summary.remote_pid_count,
            local_pid_count: execution_summary.local_pid_count,
            skipped_pid_count: execution_summary.skipped_pid_count,
            merge_function: SPIRE_REMOTE_SEARCH_MERGE_FUNCTION,
            dedupe_key: "vec_id",
            tie_breaker: "score_then_assignment_role_then_epoch_desc_then_node_pid_version_row_locator",
            top_k: execution_summary.top_k,
            status,
        })
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

pub(crate) fn remote_search_row_locator_contract_rows(
) -> Vec<SpireRemoteSearchRowLocatorContractRow> {
    vec![
        SpireRemoteSearchRowLocatorContractRow {
            contract_item: "locator_scope",
            contract_value: "origin_node",
            status: "active_contract",
        },
        SpireRemoteSearchRowLocatorContractRow {
            contract_item: "coordinator_interpretation",
            contract_value: "opaque_bytes",
            status: "active_contract",
        },
        SpireRemoteSearchRowLocatorContractRow {
            contract_item: "receive_validation",
            contract_value: "nonempty_only",
            status: "active_contract",
        },
        SpireRemoteSearchRowLocatorContractRow {
            contract_item: "remote_heap_resolution",
            contract_value: "requires_origin_node_resolution",
            status: "deferred_until_remote_heap_fetch",
        },
    ]
}

pub(crate) unsafe fn remote_search_finalization_summary_row(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    query: Vec<f32>,
    selected_pids: Vec<u64>,
    top_k: usize,
    consistency_mode: &str,
) -> SpireRemoteSearchFinalizationSummaryRow {
    let merge_summary = unsafe {
        remote_search_merge_input_summary_row(
            index_relation,
            requested_epoch,
            query,
            selected_pids,
            top_k,
            consistency_mode,
        )
    };
    let (final_heap_fetch_status, status, recommendation) = if merge_summary.status
        == "requires_remote_node_descriptor"
    {
        (
            "blocked",
            "requires_remote_node_descriptor",
            "register remote node descriptors before remote candidate finalization",
        )
    } else if merge_summary.remote_batch_count > 0 {
        (
            "requires_remote_heap_resolution",
            "requires_remote_heap_resolution",
            "add origin-node row locator resolution before returning remote heap rows",
        )
    } else if merge_summary.local_batch_count > 0 {
        ("local_ready", "ready", "none")
    } else {
        ("no_candidate_batches", merge_summary.status, "none")
    };

    SpireRemoteSearchFinalizationSummaryRow {
        requested_epoch: merge_summary.requested_epoch,
        remote_batch_count: merge_summary.remote_batch_count,
        local_batch_count: merge_summary.local_batch_count,
        skipped_batch_count: merge_summary.skipped_batch_count,
        merge_status: merge_summary.status,
        row_locator_policy: "opaque_origin_node_bytes",
        local_heap_resolution: "coordinator_local_heap",
        remote_heap_resolution: "origin_node_row_locator",
        final_heap_fetch_status,
        status,
        recommendation,
    }
}

/// Validates one target-scoped remote candidate receive batch.
///
/// The batch must match the requested epoch, expected node, selected PID set,
/// visible assignment flags, nonempty vec_id, nonempty opaque row_locator, and
/// finite score contract before candidates can enter the merge path.
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
        if !selected.contains(&candidate.pid) {
            return Err(format!(
                "ec_spire remote candidate batch candidate PID {} was not selected for node_id {expected_node_id}",
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
        if candidate.vec_id.is_empty() {
            return Err("ec_spire remote candidate batch received empty vec_id".to_owned());
        }
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

/// Merges candidates that share one coordinator-scoped `vec_id` namespace.
///
/// Current local SPIRE writers allocate node-local vec-id bytes. Until the
/// global vec-id format lands, multi-node callers must only use this helper
/// when they can prove the input vec-id bytes are globally unique by
/// construction.
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
        if candidate.vec_id.is_empty() {
            return Err("ec_spire remote candidate merge received empty vec_id".to_owned());
        }

        match best_by_vec_id.entry(candidate.vec_id.clone()) {
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
/// The same global-vec-id precondition as `merge_remote_search_candidates`
/// applies when batches span more than one node.
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
