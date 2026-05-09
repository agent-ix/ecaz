pub(crate) unsafe fn index_insert_debt_snapshot(
    index_relation: pg_sys::Relation,
) -> SpireIndexInsertDebtSnapshot {
    let root_control = unsafe { page::read_root_control_page(index_relation) };
    let leaf_rows = unsafe { index_leaf_snapshot(index_relation) };
    let active_leaf_count = u64::try_from(leaf_rows.len())
        .unwrap_or_else(|_| pgrx::error!("ec_spire leaf row count exceeds u64"));
    let leaf_count_with_deltas = leaf_rows
        .iter()
        .filter(|row| row.delta_object_count > 0)
        .count()
        .try_into()
        .unwrap_or_else(|_| pgrx::error!("ec_spire leaf delta row count exceeds u64"));
    let delta_object_count = leaf_rows
        .iter()
        .map(|row| row.delta_object_count)
        .sum::<u64>();
    let delta_insert_assignment_count = leaf_rows
        .iter()
        .map(|row| row.delta_insert_assignment_count)
        .sum::<u64>();
    let max_delta_objects_per_leaf = leaf_rows
        .iter()
        .map(|row| row.delta_object_count)
        .max()
        .unwrap_or(0);
    let batching_recommended =
        max_delta_objects_per_leaf > 1 || delta_object_count > active_leaf_count;
    let recommendation = if batching_recommended {
        "batch post-build inserts by routed base leaf before publishing replacement epochs"
    } else {
        "none"
    };

    SpireIndexInsertDebtSnapshot {
        active_epoch: root_control.active_epoch,
        active_leaf_count,
        leaf_count_with_deltas,
        delta_object_count,
        delta_insert_assignment_count,
        max_delta_objects_per_leaf,
        insert_batching_supported: false,
        batching_recommended,
        recommendation,
    }
}

fn empty_top_graph_snapshot(
    active_epoch: u64,
    top_graph_plan: options::SpireTopGraphOptionPlan,
    status: &'static str,
    recommendation: &'static str,
) -> SpireIndexTopGraphSnapshot {
    SpireIndexTopGraphSnapshot {
        active_epoch,
        top_graph_enabled: top_graph_plan.enabled,
        top_graph_count: 0,
        top_graph_pid: 0,
        root_pid: 0,
        frontier_kind: "root_top_routing_children",
        frontier_parent_level: 0,
        frontier_child_level: 0,
        frontier_node_count: 0,
        root_child_count: 0,
        active_leaf_count: 0,
        object_version: 0,
        published_epoch_backref: 0,
        level: 0,
        node_count: 0,
        dimensions: 0,
        graph_degree: top_graph_plan.graph_degree,
        build_list_size: top_graph_plan.build_list_size,
        alpha: top_graph_plan.alpha,
        entry_node: 0,
        edge_count: 0,
        max_node_degree: 0,
        effective_route_count: 0,
        effective_search_list_size: top_graph_plan.search_list_size.unwrap_or(0),
        configured_search_list_size: top_graph_plan.search_list_size,
        object_bytes: 0,
        object_tuple_count: 0,
        object_meta_tuple_count: 0,
        object_segment_count: 0,
        object_segment_tuple_count: 0,
        status,
        recommendation,
    }
}

fn active_root_top_frontier_summary(
    snapshot: &meta::SpireValidatedEpochSnapshot<'_>,
    object_store: &impl storage::SpireObjectReader,
) -> Result<Option<(u64, u16, u64)>, String> {
    let mut root = None;
    for manifest_entry in &snapshot.object_manifest().entries {
        let lookup = snapshot.require_lookup(manifest_entry.pid, "top graph root frontier summary")?;
        if lookup.placement.state != meta::SpirePlacementState::Available {
            continue;
        }
        let header = object_store.read_object_header(lookup.placement)?;
        if header.kind != storage::SpirePartitionObjectKind::Root {
            continue;
        }
        if root.is_some() {
            return Err("ec_spire top graph snapshot found multiple available root objects".to_owned());
        }
        let root_object = object_store.read_routing_object(lookup.placement)?;
        let child_count = u64::try_from(root_object.child_count())
            .map_err(|_| "ec_spire top graph root child count exceeds u64".to_owned())?;
        root = Some((manifest_entry.pid, root_object.header.level, child_count));
    }
    Ok(root)
}

fn parse_remote_search_consistency_mode(input: &str) -> Result<meta::SpireConsistencyMode, String> {
    match input {
        "strict" => Ok(meta::SpireConsistencyMode::Strict),
        "degraded" => Ok(meta::SpireConsistencyMode::Degraded),
        other => Err(format!(
            "ec_spire remote search consistency_mode must be 'strict' or 'degraded', got '{other}'"
        )),
    }
}

fn remote_search_row_locator(heap_tid: crate::storage::page::ItemPointer) -> Vec<u8> {
    // Opaque to coordinators: only the node that emitted this locator may
    // interpret it for future row fetch or rerank work.
    let mut row_locator = Vec::with_capacity(crate::storage::page::ITEM_POINTER_BYTES);
    heap_tid.encode_into(&mut row_locator);
    row_locator
}

fn decode_remote_search_local_heap_locator(
    candidate: &SpireRemoteSearchCandidateRow,
    context: &str,
) -> Result<crate::storage::page::ItemPointer, String> {
    crate::storage::page::ItemPointer::decode(&candidate.row_locator).map_err(|e| {
        format!(
            "ec_spire {context} failed to decode local row locator for pid {} row_index {} vec_id {}: {e}",
            candidate.pid,
            candidate.row_index,
            hex::encode(&candidate.vec_id)
        )
    })
}

fn remote_search_coordinator_ready_status(skipped_placement_count: u64) -> &'static str {
    if skipped_placement_count > 0 {
        "degraded_ready"
    } else {
        "ready"
    }
}

fn remote_search_status_allows_local_heap_rows(status: &str) -> bool {
    matches!(
        status,
        SPIRE_REMOTE_STATUS_READY | SPIRE_REMOTE_STATUS_DEGRADED_READY
    )
}

unsafe fn load_relation_epoch_manifests_for_coordinator_fanout(
    index_relation: pg_sys::Relation,
    root_control: meta::SpireRootControlState,
) -> Result<
    (
        meta::SpireEpochManifest,
        meta::SpireObjectManifest,
        meta::SpirePlacementDirectory,
    ),
    String,
> {
    if root_control.active_epoch == 0 {
        return Err("ec_spire cannot load manifests for empty active epoch".to_owned());
    }
    let epoch_bytes =
        unsafe { page::read_object_tuple(index_relation, root_control.epoch_manifest_tid)? };
    let object_bytes =
        unsafe { page::read_object_tuple(index_relation, root_control.object_manifest_tid)? };
    let placement_bytes =
        unsafe { page::read_object_tuple(index_relation, root_control.placement_directory_tid)? };
    let epoch_manifest = meta::SpireEpochManifest::decode(&epoch_bytes)?;
    let object_manifest = meta::SpireObjectManifest::decode(&object_bytes)?;
    let placement_directory = meta::SpirePlacementDirectory::decode(&placement_bytes)?;
    if epoch_manifest.epoch != root_control.active_epoch {
        return Err(format!(
            "ec_spire root/control active epoch {} does not match epoch manifest {}",
            root_control.active_epoch, epoch_manifest.epoch
        ));
    }
    meta::SpirePublishedEpochSnapshot::new(
        &epoch_manifest,
        &object_manifest,
        &placement_directory,
    )?;
    Ok((epoch_manifest, object_manifest, placement_directory))
}

pub(crate) unsafe fn remote_search_candidates(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    query: Vec<f32>,
    selected_pids: Vec<u64>,
    top_k: usize,
    consistency_mode: &str,
) -> Vec<SpireRemoteSearchCandidateRow> {
    let result = unsafe {
        remote_search_candidates_result(
            index_relation,
            requested_epoch,
            query,
            selected_pids,
            top_k,
            consistency_mode,
        )
    };
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

unsafe fn remote_search_candidates_result(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    query: Vec<f32>,
    selected_pids: Vec<u64>,
    top_k: usize,
    consistency_mode: &str,
) -> Result<Vec<SpireRemoteSearchCandidateRow>, String> {
    if requested_epoch == 0 {
        return Err("ec_spire remote search requested_epoch must be greater than 0".to_owned());
    }
    if top_k == 0 {
        // Valid empty candidate request, useful for endpoint contract probes.
        return Ok(Vec::new());
    }

    let requested_consistency_mode = parse_remote_search_consistency_mode(consistency_mode)?;
    let query = scan::SpireScanQuery::new(query)?;
    let root_control = unsafe { page::read_root_control_page(index_relation) };
    if root_control.active_epoch != requested_epoch {
        return Err(format!(
            "ec_spire remote search requested epoch {requested_epoch} does not match active epoch {}",
            root_control.active_epoch
        ));
    }

    let (epoch_manifest, object_manifest, placement_directory) =
        unsafe { scan::load_relation_epoch_manifests(index_relation, root_control)? };
    if epoch_manifest.consistency_mode != requested_consistency_mode {
        return Err(format!(
            "ec_spire remote search requested consistency_mode '{consistency_mode}' does not match active epoch consistency mode '{}'",
            consistency_mode_name(epoch_manifest.consistency_mode)
        ));
    }
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
    let candidates = scan::collect_quantized_selected_leaf_candidates(
        &snapshot,
        &object_store,
        query.values(),
        &selected_pids,
        relation_options.assignment_payload_format(),
        if relation_options.boundary_replica_count > 0 {
            options::SpireCandidateDedupeMode::VecIdDedupeEnabled
        } else {
            options::SpireCandidateDedupeMode::NoReplicaDedupeDisabled
        },
        Some(top_k),
    )?;

    Ok(candidates
        .into_iter()
        .map(|candidate| SpireRemoteSearchCandidateRow {
            served_epoch: candidate.epoch,
            node_id: meta::SPIRE_LOCAL_NODE_ID,
            pid: candidate.pid,
            object_version: candidate.object_version,
            row_index: candidate.row_index,
            assignment_flags: candidate.assignment_flags,
            vec_id: candidate.vec_id.as_bytes().to_vec(),
            row_locator: remote_search_row_locator(candidate.heap_tid),
            score: candidate.score,
        })
        .collect())
}

pub(crate) unsafe fn remote_search_coordinator_local_candidates(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    query: Vec<f32>,
    selected_pids: Vec<u64>,
    top_k: usize,
    consistency_mode: &str,
) -> Vec<SpireRemoteSearchCandidateRow> {
    let result = unsafe {
        remote_search_coordinator_local_candidates_result(
            index_relation,
            requested_epoch,
            query,
            selected_pids,
            top_k,
            consistency_mode,
        )
    };
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

unsafe fn remote_search_coordinator_local_candidates_result(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    query: Vec<f32>,
    selected_pids: Vec<u64>,
    top_k: usize,
    consistency_mode: &str,
) -> Result<Vec<SpireRemoteSearchCandidateRow>, String> {
    if requested_epoch == 0 {
        return Err(
            "ec_spire remote search coordinator requested_epoch must be greater than 0".to_owned(),
        );
    }
    if top_k == 0 {
        return Ok(Vec::new());
    }

    let requested_consistency_mode = parse_remote_search_consistency_mode(consistency_mode)?;
    let query = scan::SpireScanQuery::new(query)?;
    let root_control = unsafe { page::read_root_control_page(index_relation) };
    if root_control.active_epoch != requested_epoch {
        return Err(format!(
            "ec_spire remote search coordinator requested epoch {requested_epoch} does not match active epoch {}",
            root_control.active_epoch
        ));
    }

    let (epoch_manifest, object_manifest, placement_directory) = unsafe {
        load_relation_epoch_manifests_for_coordinator_fanout(index_relation, root_control)?
    };
    if epoch_manifest.consistency_mode != requested_consistency_mode {
        return Err(format!(
            "ec_spire remote search coordinator requested consistency_mode '{consistency_mode}' does not match active epoch consistency mode '{}'",
            consistency_mode_name(epoch_manifest.consistency_mode)
        ));
    }
    let snapshot = meta::SpirePublishedEpochSnapshot::new(
        &epoch_manifest,
        &object_manifest,
        &placement_directory,
    )?;
    let plan = plan_remote_search_fanout(&snapshot, &selected_pids)?;
    if !plan.remote_targets.is_empty() {
        return Err(format!(
            "ec_spire remote search coordinator requires libpq transport for {} remote target(s)",
            plan.remote_targets.len()
        ));
    }

    let object_store = unsafe {
        storage::SpireRelationObjectStoreSet::for_index_relation_and_placements(
            index_relation,
            &placement_directory,
            pg_sys::AccessShareLock as pg_sys::LOCKMODE,
        )?
    };
    let relation_options = unsafe { options::relation_options(index_relation) };
    // The local collector caps one batch; the merge cap remains load-bearing
    // once remote batches compete in the same coordinator result set.
    let candidates = scan::collect_quantized_selected_leaf_candidates(
        &snapshot,
        &object_store,
        query.values(),
        &plan.local_selected_pids,
        relation_options.assignment_payload_format(),
        if relation_options.boundary_replica_count > 0 {
            options::SpireCandidateDedupeMode::VecIdDedupeEnabled
        } else {
            options::SpireCandidateDedupeMode::NoReplicaDedupeDisabled
        },
        Some(top_k),
    )?
    .into_iter()
    .map(|candidate| SpireRemoteSearchCandidateRow {
        served_epoch: candidate.epoch,
        node_id: meta::SPIRE_LOCAL_NODE_ID,
        pid: candidate.pid,
        object_version: candidate.object_version,
        row_index: candidate.row_index,
        assignment_flags: candidate.assignment_flags,
        vec_id: candidate.vec_id.as_bytes().to_vec(),
        row_locator: remote_search_row_locator(candidate.heap_tid),
        score: candidate.score,
    })
    .collect();
    let merged = merge_validated_remote_search_candidate_batches(
        requested_epoch,
        vec![SpireRemoteSearchCandidateBatch {
            node_id: meta::SPIRE_LOCAL_NODE_ID,
            selected_pids: plan.local_selected_pids,
            candidates,
        }],
        Some(top_k),
    )?;

    Ok(merged.candidates)
}

unsafe fn remote_search_coordinator_local_candidates_for_result_summary(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    query: Vec<f32>,
    selected_pids: Vec<u64>,
    top_k: usize,
    consistency_mode: &str,
) -> Result<Vec<SpireRemoteSearchCandidateRow>, String> {
    if requested_epoch == 0 {
        return Err(
            "ec_spire remote search coordinator requested_epoch must be greater than 0".to_owned(),
        );
    }
    if top_k == 0 {
        return Ok(Vec::new());
    }

    let requested_consistency_mode = parse_remote_search_consistency_mode(consistency_mode)?;
    let query = scan::SpireScanQuery::new(query)?;
    let root_control = unsafe { page::read_root_control_page(index_relation) };
    if root_control.active_epoch != requested_epoch {
        return Err(format!(
            "ec_spire remote search coordinator requested epoch {requested_epoch} does not match active epoch {}",
            root_control.active_epoch
        ));
    }

    let (epoch_manifest, object_manifest, placement_directory) = unsafe {
        load_relation_epoch_manifests_for_coordinator_fanout(index_relation, root_control)?
    };
    if epoch_manifest.consistency_mode != requested_consistency_mode {
        return Err(format!(
            "ec_spire remote search coordinator requested consistency_mode '{consistency_mode}' does not match active epoch consistency mode '{}'",
            consistency_mode_name(epoch_manifest.consistency_mode)
        ));
    }
    let snapshot = meta::SpirePublishedEpochSnapshot::new(
        &epoch_manifest,
        &object_manifest,
        &placement_directory,
    )?;
    let plan = plan_remote_search_fanout(&snapshot, &selected_pids)?;
    if plan.local_selected_pids.is_empty() {
        return Ok(Vec::new());
    }

    let local_pid_set = plan
        .local_selected_pids
        .iter()
        .copied()
        .collect::<HashSet<_>>();
    let local_placement_directory = meta::SpirePlacementDirectory::from_entries(
        placement_directory.epoch,
        placement_directory
            .entries
            .iter()
            .filter(|entry| {
                entry.node_id == meta::SPIRE_LOCAL_NODE_ID && local_pid_set.contains(&entry.pid)
            })
            .cloned()
            .collect(),
    )?;
    let local_object_manifest = meta::SpireObjectManifest::from_entries(
        object_manifest.epoch,
        object_manifest
            .entries
            .iter()
            .filter(|entry| local_pid_set.contains(&entry.pid))
            .copied()
            .collect(),
    )?;
    let local_snapshot = meta::SpirePublishedEpochSnapshot::new(
        &epoch_manifest,
        &local_object_manifest,
        &local_placement_directory,
    )?;
    let object_store = unsafe {
        storage::SpireRelationObjectStoreSet::for_index_relation_and_placements(
            index_relation,
            &local_placement_directory,
            pg_sys::AccessShareLock as pg_sys::LOCKMODE,
        )?
    };
    let relation_options = unsafe { options::relation_options(index_relation) };
    let candidates = scan::collect_quantized_selected_leaf_candidates(
        &local_snapshot,
        &object_store,
        query.values(),
        &plan.local_selected_pids,
        relation_options.assignment_payload_format(),
        if relation_options.boundary_replica_count > 0 {
            options::SpireCandidateDedupeMode::VecIdDedupeEnabled
        } else {
            options::SpireCandidateDedupeMode::NoReplicaDedupeDisabled
        },
        Some(top_k),
    )?
    .into_iter()
    .map(|candidate| SpireRemoteSearchCandidateRow {
        served_epoch: candidate.epoch,
        node_id: meta::SPIRE_LOCAL_NODE_ID,
        pid: candidate.pid,
        object_version: candidate.object_version,
        row_index: candidate.row_index,
        assignment_flags: candidate.assignment_flags,
        vec_id: candidate.vec_id.as_bytes().to_vec(),
        row_locator: remote_search_row_locator(candidate.heap_tid),
        score: candidate.score,
    })
    .collect();
    let merged = merge_validated_remote_search_candidate_batches(
        requested_epoch,
        vec![SpireRemoteSearchCandidateBatch {
            node_id: meta::SPIRE_LOCAL_NODE_ID,
            selected_pids: plan.local_selected_pids,
            candidates,
        }],
        Some(top_k),
    )?;

    Ok(merged.candidates)
}

pub(crate) unsafe fn remote_search_coordinator_local_summary(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    query: Vec<f32>,
    selected_pids: Vec<u64>,
    top_k: usize,
    consistency_mode: &str,
) -> SpireRemoteSearchCoordinatorLocalSummaryRow {
    let result = unsafe {
        remote_search_coordinator_local_summary_result(
            index_relation,
            requested_epoch,
            query,
            selected_pids,
            top_k,
            consistency_mode,
        )
    };
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

pub(crate) unsafe fn remote_search_local_heap_resolution_plan_rows(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    query: Vec<f32>,
    selected_pids: Vec<u64>,
    top_k: usize,
    consistency_mode: &str,
) -> Vec<SpireRemoteSearchLocalHeapResolutionPlanRow> {
    let result = (|| -> Result<Vec<SpireRemoteSearchLocalHeapResolutionPlanRow>, String> {
        let candidates = unsafe {
            remote_search_coordinator_local_candidates_result(
                index_relation,
                requested_epoch,
                query,
                selected_pids,
                top_k,
                consistency_mode,
            )?
        };
        candidates
            .into_iter()
            .map(|candidate| {
                let heap_tid = decode_remote_search_local_heap_locator(
                    &candidate,
                    "local heap resolution plan",
                )?;
                Ok(SpireRemoteSearchLocalHeapResolutionPlanRow {
                    requested_epoch,
                    node_id: candidate.node_id,
                    pid: candidate.pid,
                    row_index: candidate.row_index,
                    vec_id: candidate.vec_id,
                    row_locator: candidate.row_locator,
                    heap_block: heap_tid.block_number,
                    heap_offset: heap_tid.offset_number,
                    heap_lookup_owner: SPIRE_REMOTE_LOCAL_HEAP_RESOLUTION,
                    status: SPIRE_REMOTE_STATUS_READY,
                })
            })
            .collect()
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

pub(crate) unsafe fn remote_search_local_heap_candidate_rows(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    query: Vec<f32>,
    selected_pids: Vec<u64>,
    top_k: usize,
    consistency_mode: &str,
) -> Vec<SpireRemoteSearchLocalHeapCandidateRow> {
    let result = (|| -> Result<Vec<SpireRemoteSearchLocalHeapCandidateRow>, String> {
        let candidates = unsafe {
            remote_search_coordinator_local_candidates_result(
                index_relation,
                requested_epoch,
                query,
                selected_pids,
                top_k,
                consistency_mode,
            )?
        };
        candidates
            .into_iter()
            .map(|candidate| {
                let heap_tid = decode_remote_search_local_heap_locator(
                    &candidate,
                    "local heap candidate rows",
                )?;
                Ok(SpireRemoteSearchLocalHeapCandidateRow {
                    requested_epoch,
                    served_epoch: candidate.served_epoch,
                    node_id: candidate.node_id,
                    pid: candidate.pid,
                    object_version: candidate.object_version,
                    row_index: candidate.row_index,
                    assignment_flags: candidate.assignment_flags,
                    vec_id: candidate.vec_id,
                    row_locator: candidate.row_locator,
                    heap_block: heap_tid.block_number,
                    heap_offset: heap_tid.offset_number,
                    score: candidate.score,
                    heap_lookup_owner: SPIRE_REMOTE_LOCAL_HEAP_RESOLUTION,
                    status: SPIRE_REMOTE_STATUS_READY,
                })
            })
            .collect()
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

unsafe fn remote_search_local_heap_candidate_rows_for_result_summary(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    query: Vec<f32>,
    selected_pids: Vec<u64>,
    top_k: usize,
    consistency_mode: &str,
) -> Vec<SpireRemoteSearchLocalHeapCandidateRow> {
    let result = (|| -> Result<Vec<SpireRemoteSearchLocalHeapCandidateRow>, String> {
        let candidates = unsafe {
            remote_search_coordinator_local_candidates_for_result_summary(
                index_relation,
                requested_epoch,
                query,
                selected_pids,
                top_k,
                consistency_mode,
            )?
        };
        candidates
            .into_iter()
            .map(|candidate| {
                let heap_tid = decode_remote_search_local_heap_locator(
                    &candidate,
                    "coordinator result summary local heap candidates",
                )?;
                Ok(SpireRemoteSearchLocalHeapCandidateRow {
                    requested_epoch,
                    served_epoch: candidate.served_epoch,
                    node_id: candidate.node_id,
                    pid: candidate.pid,
                    object_version: candidate.object_version,
                    row_index: candidate.row_index,
                    assignment_flags: candidate.assignment_flags,
                    vec_id: candidate.vec_id,
                    row_locator: candidate.row_locator,
                    heap_block: heap_tid.block_number,
                    heap_offset: heap_tid.offset_number,
                    score: candidate.score,
                    heap_lookup_owner: SPIRE_REMOTE_LOCAL_HEAP_RESOLUTION,
                    status: SPIRE_REMOTE_STATUS_READY,
                })
            })
            .collect()
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

pub(crate) unsafe fn remote_search_local_heap_candidate_summary_row(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    query: Vec<f32>,
    selected_pids: Vec<u64>,
    top_k: usize,
    consistency_mode: &str,
) -> SpireRemoteSearchLocalHeapCandidateSummaryRow {
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
    unsafe {
        remote_search_local_heap_candidate_summary_from_gate(
            index_relation,
            &gate,
            requested_epoch,
            query,
            selected_pids,
            top_k,
            consistency_mode,
        )
    }
}

unsafe fn remote_search_local_heap_candidate_summary_from_gate(
    index_relation: pg_sys::Relation,
    gate: &SpireRemoteSearchCoordinatorGateSummaryRow,
    requested_epoch: u64,
    query: Vec<f32>,
    selected_pids: Vec<u64>,
    top_k: usize,
    consistency_mode: &str,
) -> SpireRemoteSearchLocalHeapCandidateSummaryRow {
    let returned_candidate_count = if gate.remote_plan_count == 0
        && remote_search_status_allows_local_heap_rows(gate.status)
    {
        let rows = unsafe {
            remote_search_local_heap_candidate_rows(
                index_relation,
                requested_epoch,
                query,
                selected_pids,
                top_k,
                consistency_mode,
            )
        };
        u64::try_from(rows.len())
            .unwrap_or_else(|_| pgrx::error!("ec_spire local heap candidate count overflow"))
    } else {
        0
    };

    SpireRemoteSearchLocalHeapCandidateSummaryRow {
        requested_epoch: gate.requested_epoch,
        local_plan_count: gate.local_plan_count,
        remote_plan_count: gate.remote_plan_count,
        skipped_plan_count: gate.skipped_plan_count,
        local_pid_count: gate.local_pid_count,
        remote_pid_count: gate.remote_pid_count,
        decoded_local_locator_count: returned_candidate_count,
        returned_candidate_count,
        status: gate.status,
        recommendation: gate.recommendation,
    }
}

fn remote_search_heap_candidate_cmp_for_result(
    left: &SpireRemoteSearchLocalHeapCandidateRow,
    right: &SpireRemoteSearchLocalHeapCandidateRow,
) -> std::cmp::Ordering {
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
}

fn remote_search_heap_candidate_dedupe_key_for_result(
    candidate: &SpireRemoteSearchLocalHeapCandidateRow,
) -> Result<Vec<u8>, String> {
    remote_search_candidate_dedupe_key(&SpireRemoteSearchCandidateRow {
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
}

fn merge_remote_search_heap_candidates_for_result(
    candidates: Vec<SpireRemoteSearchLocalHeapCandidateRow>,
    top_k: usize,
) -> Result<Vec<SpireRemoteSearchLocalHeapCandidateRow>, String> {
    let mut best_by_vec_id = HashMap::<Vec<u8>, SpireRemoteSearchLocalHeapCandidateRow>::new();
    for candidate in candidates {
        let dedupe_key = remote_search_heap_candidate_dedupe_key_for_result(&candidate)?;
        match best_by_vec_id.entry(dedupe_key) {
            std::collections::hash_map::Entry::Occupied(mut entry) => {
                if remote_search_heap_candidate_cmp_for_result(&candidate, entry.get()).is_lt() {
                    *entry.get_mut() = candidate;
                }
            }
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert(candidate);
            }
        }
    }

    let mut candidates = best_by_vec_id.into_values().collect::<Vec<_>>();
    candidates.sort_by(remote_search_heap_candidate_cmp_for_result);
    candidates.truncate(top_k);
    Ok(candidates)
}

pub(crate) unsafe fn remote_search_coordinator_result_summary_row(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    query: Vec<f32>,
    selected_pids: Vec<u64>,
    top_k: usize,
    consistency_mode: &str,
) -> SpireRemoteSearchCoordinatorResultSummaryRow {
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
    let mut heap_candidates = Vec::new();
    if gate.local_plan_count > 0
        && (remote_search_status_allows_local_heap_rows(gate.status)
            || gate.status == SPIRE_REMOTE_EXECUTOR_REQUIRED)
    {
        heap_candidates.extend(unsafe {
            remote_search_local_heap_candidate_rows_for_result_summary(
                index_relation,
                requested_epoch,
                query.clone(),
                selected_pids.clone(),
                top_k,
                consistency_mode,
            )
        });
    }
    if gate.remote_plan_count > 0 && gate.status == SPIRE_REMOTE_EXECUTOR_REQUIRED {
        heap_candidates.extend(unsafe {
            remote_search_libpq_executor_heap_candidate_rows(
                index_relation,
                requested_epoch,
                query,
                selected_pids,
                top_k,
                consistency_mode,
            )
        });
    }
    let heap_candidates = merge_remote_search_heap_candidates_for_result(heap_candidates, top_k)
        .unwrap_or_else(|e| pgrx::error!("{e}"));

    let returned_candidate_count = u64::try_from(heap_candidates.len())
        .unwrap_or_else(|_| pgrx::error!("ec_spire coordinator result candidate count overflow"));
    let decoded_local_locator_count = heap_candidates
        .iter()
        .filter(|row| row.heap_lookup_owner == SPIRE_REMOTE_LOCAL_HEAP_RESOLUTION)
        .count()
        .try_into()
        .unwrap_or_else(|_| {
            pgrx::error!("ec_spire coordinator result local locator count overflow")
        });
    let has_remote_heap_candidates = heap_candidates
        .iter()
        .any(|row| row.heap_lookup_owner == SPIRE_REMOTE_HEAP_RESOLUTION);

    let result_source = if has_remote_heap_candidates {
        SPIRE_REMOTE_RESULT_SOURCE_REMOTE_HEAP_CANDIDATES
    } else if returned_candidate_count > 0 {
        SPIRE_REMOTE_RESULT_SOURCE_LOCAL_HEAP_CANDIDATES
    } else if gate.next_blocker != SPIRE_REMOTE_NONE {
        SPIRE_REMOTE_RESULT_SOURCE_BLOCKED
    } else {
        SPIRE_REMOTE_NONE
    };
    let final_heap_fetch_status = if has_remote_heap_candidates {
        SPIRE_REMOTE_FINAL_STATUS_REMOTE_READY
    } else {
        gate.final_heap_fetch_status
    };
    let next_blocker = if returned_candidate_count > 0 {
        SPIRE_REMOTE_NONE
    } else {
        gate.next_blocker
    };
    let status = if returned_candidate_count > 0 {
        if gate.skipped_plan_count > 0 {
            SPIRE_REMOTE_STATUS_DEGRADED_READY
        } else {
            SPIRE_REMOTE_STATUS_READY
        }
    } else {
        gate.status
    };
    let recommendation = if returned_candidate_count > 0 {
        SPIRE_REMOTE_NONE
    } else if gate.recommendation != SPIRE_REMOTE_NONE {
        gate.recommendation
    } else {
        SPIRE_REMOTE_NONE
    };

    SpireRemoteSearchCoordinatorResultSummaryRow {
        requested_epoch: gate.requested_epoch,
        local_plan_count: gate.local_plan_count,
        remote_plan_count: gate.remote_plan_count,
        skipped_plan_count: gate.skipped_plan_count,
        local_pid_count: gate.local_pid_count,
        remote_pid_count: gate.remote_pid_count,
        skipped_pid_count: gate.skipped_pid_count,
        decoded_local_locator_count,
        returned_candidate_count,
        result_source,
        libpq_receive_count: gate.libpq_receive_count,
        libpq_receive_status: gate.libpq_receive_status,
        final_heap_fetch_status,
        next_blocker,
        status,
        recommendation,
    }
}

unsafe fn remote_search_coordinator_local_summary_result(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    query: Vec<f32>,
    selected_pids: Vec<u64>,
    top_k: usize,
    consistency_mode: &str,
) -> Result<SpireRemoteSearchCoordinatorLocalSummaryRow, String> {
    if requested_epoch == 0 {
        return Err(
            "ec_spire remote search coordinator requested_epoch must be greater than 0".to_owned(),
        );
    }

    let requested_consistency_mode = parse_remote_search_consistency_mode(consistency_mode)?;
    let query = scan::SpireScanQuery::new(query)?;
    let root_control = unsafe { page::read_root_control_page(index_relation) };
    if root_control.active_epoch != requested_epoch {
        return Err(format!(
            "ec_spire remote search coordinator requested epoch {requested_epoch} does not match active epoch {}",
            root_control.active_epoch
        ));
    }

    let (epoch_manifest, object_manifest, placement_directory) = unsafe {
        load_relation_epoch_manifests_for_coordinator_fanout(index_relation, root_control)?
    };
    if epoch_manifest.consistency_mode != requested_consistency_mode {
        return Err(format!(
            "ec_spire remote search coordinator requested consistency_mode '{consistency_mode}' does not match active epoch consistency mode '{}'",
            consistency_mode_name(epoch_manifest.consistency_mode)
        ));
    }
    let snapshot = meta::SpirePublishedEpochSnapshot::new(
        &epoch_manifest,
        &object_manifest,
        &placement_directory,
    )?;
    // Unlike the candidate path, the summary still plans empty top-k probes so
    // operators can inspect fanout and transport readiness without fetching rows.
    let plan = plan_remote_search_fanout(&snapshot, &selected_pids)?;
    let local_pid_count = u64::try_from(plan.local_selected_pids.len())
        .map_err(|_| "ec_spire coordinator local PID count exceeds u64".to_owned())?;
    let remote_target_count = u64::try_from(plan.remote_targets.len())
        .map_err(|_| "ec_spire coordinator remote target count exceeds u64".to_owned())?;
    let remote_pid_count = plan.remote_targets.iter().try_fold(0_u64, |acc, target| {
        let count = u64::try_from(target.selected_pids.len())
            .map_err(|_| "ec_spire coordinator remote PID count exceeds u64".to_owned())?;
        acc.checked_add(count)
            .ok_or_else(|| "ec_spire coordinator remote PID count overflow".to_owned())
    })?;
    let skipped_placement_count = u64::try_from(plan.skipped_placements.len())
        .map_err(|_| "ec_spire coordinator skipped placement count exceeds u64".to_owned())?;
    if !plan.remote_targets.is_empty() {
        return Ok(SpireRemoteSearchCoordinatorLocalSummaryRow {
            requested_epoch,
            local_pid_count,
            remote_target_count,
            remote_pid_count,
            skipped_placement_count,
            candidate_input_count: 0,
            duplicate_vec_id_count: 0,
            returned_candidate_count: 0,
            status: "requires_libpq_transport",
        });
    }
    if top_k == 0 {
        return Ok(SpireRemoteSearchCoordinatorLocalSummaryRow {
            requested_epoch,
            local_pid_count,
            remote_target_count,
            remote_pid_count,
            skipped_placement_count,
            candidate_input_count: 0,
            duplicate_vec_id_count: 0,
            returned_candidate_count: 0,
            status: "empty_top_k",
        });
    }

    let object_store = unsafe {
        storage::SpireRelationObjectStoreSet::for_index_relation_and_placements(
            index_relation,
            &placement_directory,
            pg_sys::AccessShareLock as pg_sys::LOCKMODE,
        )?
    };
    let relation_options = unsafe { options::relation_options(index_relation) };
    let candidates = scan::collect_quantized_selected_leaf_candidates(
        &snapshot,
        &object_store,
        query.values(),
        &plan.local_selected_pids,
        relation_options.assignment_payload_format(),
        if relation_options.boundary_replica_count > 0 {
            options::SpireCandidateDedupeMode::VecIdDedupeEnabled
        } else {
            options::SpireCandidateDedupeMode::NoReplicaDedupeDisabled
        },
        Some(top_k),
    )?
    .into_iter()
    .map(|candidate| SpireRemoteSearchCandidateRow {
        served_epoch: candidate.epoch,
        node_id: meta::SPIRE_LOCAL_NODE_ID,
        pid: candidate.pid,
        object_version: candidate.object_version,
        row_index: candidate.row_index,
        assignment_flags: candidate.assignment_flags,
        vec_id: candidate.vec_id.as_bytes().to_vec(),
        row_locator: remote_search_row_locator(candidate.heap_tid),
        score: candidate.score,
    })
    .collect();
    let merged = merge_validated_remote_search_candidate_batches(
        requested_epoch,
        vec![SpireRemoteSearchCandidateBatch {
            node_id: meta::SPIRE_LOCAL_NODE_ID,
            selected_pids: plan.local_selected_pids,
            candidates,
        }],
        Some(top_k),
    )?;
    let returned_candidate_count = u64::try_from(merged.candidates.len())
        .map_err(|_| "ec_spire coordinator returned candidate count exceeds u64".to_owned())?;

    Ok(SpireRemoteSearchCoordinatorLocalSummaryRow {
        requested_epoch,
        local_pid_count,
        remote_target_count,
        remote_pid_count,
        skipped_placement_count,
        candidate_input_count: merged.input_count,
        duplicate_vec_id_count: merged.duplicate_vec_id_count,
        returned_candidate_count,
        status: remote_search_coordinator_ready_status(skipped_placement_count),
    })
}

pub(crate) unsafe fn index_top_graph_snapshot(
    index_relation: pg_sys::Relation,
) -> SpireIndexTopGraphSnapshot {
    let result = (|| -> Result<SpireIndexTopGraphSnapshot, String> {
        let relation_options = unsafe { options::relation_options(index_relation) };
        let top_graph_plan = relation_options.top_graph_plan()?;
        let root_control = unsafe { page::read_root_control_page(index_relation) };

        if root_control.active_epoch == 0 {
            return Ok(empty_top_graph_snapshot(
                root_control.active_epoch,
                top_graph_plan,
                "empty",
                "populate the index before expecting a published SPIRE top graph",
            ));
        }

        let (epoch_manifest, object_manifest, placement_directory) =
            unsafe { scan::load_relation_epoch_manifests(index_relation, root_control)? };
        let snapshot = meta::SpireValidatedEpochSnapshot::new(
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
        let active_leaf_count = count_snapshot_options_leaf_pids(
            &meta::SpirePublishedEpochSnapshot::new(
                &epoch_manifest,
                &object_manifest,
                &placement_directory,
            )?,
            &object_store,
            relation_options.recursive_fanout().is_some(),
        )?;
        let active_leaf_count_u64 = u64::from(active_leaf_count);
        let root_frontier_summary = active_root_top_frontier_summary(&snapshot, &object_store)?;
        let (active_root_pid, active_root_level, root_child_count) =
            root_frontier_summary.unwrap_or((0, 0, 0));
        let frontier_child_level = active_root_level.saturating_sub(1);
        let relation_nprobe = u32::try_from(relation_options.nprobe)
            .map_err(|_| "ec_spire nprobe reloption must be non-negative".to_owned())?;
        let nprobe = options::resolve_scan_nprobe(active_leaf_count, relation_nprobe);
        let effective_search_list_size = top_graph_plan
            .search_list_size
            .unwrap_or(nprobe.effective_nprobe);

        let mut top_graphs = Vec::new();
        for manifest_entry in &snapshot.object_manifest().entries {
            let lookup = snapshot.require_lookup(manifest_entry.pid, "top graph snapshot")?;
            if lookup.placement.state != meta::SpirePlacementState::Available {
                continue;
            }
            let header =
                storage::SpireObjectReader::read_object_header(&object_store, lookup.placement)?;
            if header.kind == storage::SpirePartitionObjectKind::TopGraph {
                top_graphs.push((
                    lookup.placement,
                    storage::SpireObjectReader::read_top_graph_object(
                        &object_store,
                        lookup.placement,
                    )?,
                ));
            }
        }

        if top_graphs.is_empty() {
            let (status, recommendation) = if top_graph_plan.enabled {
                (
                    "missing_top_graph",
                    "rebuild the recursive index or disable top_graph_enabled; enabled scans fail closed without a graph object",
                )
            } else {
                ("disabled", "none")
            };
            let mut snapshot = empty_top_graph_snapshot(
                root_control.active_epoch,
                top_graph_plan,
                status,
                recommendation,
            );
            snapshot.root_pid = active_root_pid;
            snapshot.frontier_parent_level = active_root_level;
            snapshot.frontier_child_level = frontier_child_level;
            snapshot.root_child_count = root_child_count;
            snapshot.active_leaf_count = active_leaf_count_u64;
            snapshot.effective_route_count = nprobe.effective_nprobe;
            snapshot.effective_search_list_size = effective_search_list_size;
            return Ok(snapshot);
        }

        top_graphs.sort_by_key(|(_, graph)| graph.header.pid);
        let top_graph_count = u64::try_from(top_graphs.len())
            .map_err(|_| "ec_spire top graph snapshot count exceeds u64".to_owned())?;
        let (placement, top_graph) = &top_graphs[0];
        let object_tuple_count =
            u64::try_from(unsafe { object_store.active_object_tuple_locators(placement)? }.len())
                .map_err(|_| "ec_spire top graph object tuple count exceeds u64".to_owned())?;
        let object_meta_tuple_count = u64::from(object_tuple_count > 0);
        let object_segment_count = object_tuple_count.saturating_sub(object_meta_tuple_count);
        let object_segment_tuple_count = object_segment_count;
        let mut edge_count = 0_u64;
        let mut max_node_degree = 0_u64;
        let node_count = u64::try_from(top_graph.node_count())
            .map_err(|_| "ec_spire top graph snapshot node count exceeds u64".to_owned())?;
        for node in &top_graph.nodes {
            let node_degree = u64::try_from(node.neighbors.len())
                .map_err(|_| "ec_spire top graph snapshot node degree exceeds u64".to_owned())?;
            edge_count = edge_count
                .checked_add(node_degree)
                .ok_or_else(|| "ec_spire top graph snapshot edge count overflow".to_owned())?;
            max_node_degree = max_node_degree.max(node_degree);
        }
        let (status, recommendation) = if top_graph_count > 1 {
            (
                "multiple_top_graphs",
                "repair or rebuild the index; enabled scans fail closed when multiple top graph objects are visible",
            )
        } else if active_root_pid == 0 {
            (
                "missing_root",
                "repair or rebuild the index; top graph snapshots require one available root routing object",
            )
        } else if top_graph.root_pid != active_root_pid {
            (
                "root_mismatch",
                "repair or rebuild the index; top graph root_pid does not match the active root routing object",
            )
        } else if top_graph.header.level != active_root_level {
            (
                "level_mismatch",
                "repair or rebuild the index; top graph parent level does not match the active root routing level",
            )
        } else if node_count != root_child_count {
            (
                "frontier_mismatch",
                "repair or rebuild the index; top graph node count does not match the active root/top routing child frontier",
            )
        } else if top_graph_count == 1 && top_graph_plan.enabled {
            ("ready", "none")
        } else {
            ("available_disabled", "none")
        };

        Ok(SpireIndexTopGraphSnapshot {
            active_epoch: root_control.active_epoch,
            top_graph_enabled: top_graph_plan.enabled,
            top_graph_count,
            top_graph_pid: top_graph.header.pid,
            root_pid: top_graph.root_pid,
            frontier_kind: "root_top_routing_children",
            frontier_parent_level: active_root_level,
            frontier_child_level,
            frontier_node_count: node_count,
            root_child_count,
            active_leaf_count: active_leaf_count_u64,
            object_version: top_graph.header.object_version,
            published_epoch_backref: top_graph.header.published_epoch_backref,
            level: top_graph.header.level,
            node_count,
            dimensions: top_graph.dimensions,
            graph_degree: top_graph.graph_degree,
            build_list_size: top_graph.build_list_size,
            alpha: top_graph.alpha,
            entry_node: u64::from(top_graph.entry_node),
            edge_count,
            max_node_degree,
            effective_route_count: nprobe.effective_nprobe,
            effective_search_list_size,
            configured_search_list_size: top_graph_plan.search_list_size,
            object_bytes: u64::from(placement.object_bytes),
            object_tuple_count,
            object_meta_tuple_count,
            object_segment_count,
            object_segment_tuple_count,
            status,
            recommendation,
        })
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

pub(crate) unsafe fn index_hierarchy_snapshot(
    index_relation: pg_sys::Relation,
) -> SpireIndexHierarchySnapshot {
    let result = (|| -> Result<SpireIndexHierarchySnapshot, String> {
        let root_control = unsafe { page::read_root_control_page(index_relation) };
        if root_control.active_epoch == 0 {
            let (status, recommendation) = hierarchy_snapshot_status(0, 0, 0, true, false);
            return Ok(SpireIndexHierarchySnapshot {
                active_epoch: 0,
                root_pid: 0,
                root_level: 0,
                max_observed_level: 0,
                hierarchy_depth: 0,
                routing_object_count: 0,
                root_routing_object_count: 0,
                internal_routing_object_count: 0,
                leaf_object_count: 0,
                delta_object_count: 0,
                centroid_dimensions: 0,
                root_child_count: 0,
                distinct_leaf_parent_count: 0,
                recursive_routing_supported: false,
                per_level_nprobe_supported: false,
                status,
                recommendation,
            });
        }

        let (epoch_manifest, object_manifest, placement_directory) =
            unsafe { scan::load_relation_epoch_manifests(index_relation, root_control)? };
        let snapshot = meta::SpireValidatedEpochSnapshot::new(
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

        let mut root_pid = 0_u64;
        let mut root_level = 0_u16;
        let mut max_observed_level = 0_u16;
        let mut routing_object_count = 0_u64;
        let mut root_routing_object_count = 0_u64;
        let mut internal_routing_object_count = 0_u64;
        let mut leaf_object_count = 0_u64;
        let mut delta_object_count = 0_u64;
        let mut centroid_dimensions = 0_u16;
        let mut root_child_count = 0_u64;
        let mut leaf_parent_pids = HashSet::new();
        let mut hierarchy_objects = Vec::new();

        for manifest_entry in &snapshot.object_manifest().entries {
            let lookup = snapshot.require_lookup(manifest_entry.pid, "hierarchy snapshot")?;
            let placement = lookup.placement;
            if placement.state != meta::SpirePlacementState::Available {
                continue;
            }
            let header = storage::SpireObjectReader::read_object_header(&object_store, placement)?;
            max_observed_level = max_observed_level.max(header.level);
            match header.kind {
                storage::SpirePartitionObjectKind::Root => {
                    let routing_object =
                        storage::SpireObjectReader::read_routing_object(&object_store, placement)?;
                    routing_object_count =
                        routing_object_count.checked_add(1).ok_or_else(|| {
                            "ec_spire hierarchy snapshot routing object count overflow".to_owned()
                        })?;
                    root_routing_object_count =
                        root_routing_object_count.checked_add(1).ok_or_else(|| {
                            "ec_spire hierarchy snapshot root object count overflow".to_owned()
                        })?;
                    root_pid = header.pid;
                    root_level = header.level;
                    centroid_dimensions = routing_object.dimensions;
                    hierarchy_objects.push(hierarchy_object_summary(
                        &routing_object.header,
                        routing_object.child_pids.clone(),
                    ));
                    root_child_count =
                        u64::try_from(routing_object.child_count()).map_err(|_| {
                            "ec_spire hierarchy snapshot root child count exceeds u64".to_owned()
                        })?;
                }
                storage::SpirePartitionObjectKind::Internal => {
                    routing_object_count =
                        routing_object_count.checked_add(1).ok_or_else(|| {
                            "ec_spire hierarchy snapshot routing object count overflow".to_owned()
                        })?;
                    internal_routing_object_count = internal_routing_object_count
                        .checked_add(1)
                        .ok_or_else(|| {
                            "ec_spire hierarchy snapshot internal object count overflow".to_owned()
                        })?;
                    let routing_object =
                        storage::SpireObjectReader::read_routing_object(&object_store, placement)?;
                    hierarchy_objects.push(hierarchy_object_summary(
                        &routing_object.header,
                        routing_object.child_pids.clone(),
                    ));
                }
                storage::SpirePartitionObjectKind::Leaf => {
                    leaf_object_count = leaf_object_count.checked_add(1).ok_or_else(|| {
                        "ec_spire hierarchy snapshot leaf object count overflow".to_owned()
                    })?;
                    leaf_parent_pids.insert(header.parent_pid);
                    hierarchy_objects.push(hierarchy_object_summary(&header, Vec::new()));
                }
                storage::SpirePartitionObjectKind::Delta => {
                    delta_object_count = delta_object_count.checked_add(1).ok_or_else(|| {
                        "ec_spire hierarchy snapshot delta object count overflow".to_owned()
                    })?;
                    hierarchy_objects.push(hierarchy_object_summary(&header, Vec::new()));
                }
                storage::SpirePartitionObjectKind::TopGraph => {}
            }
        }

        let hierarchy_depth = if root_routing_object_count == 0 {
            0
        } else {
            max_observed_level.max(root_level)
        };
        let hierarchy_shape_valid = validate_recursive_hierarchy_shape(&hierarchy_objects).is_ok();
        let per_level_nprobe_supported = hierarchy_shape_valid && internal_routing_object_count > 0;
        let (status, recommendation) = hierarchy_snapshot_status(
            root_routing_object_count,
            internal_routing_object_count,
            leaf_object_count,
            hierarchy_shape_valid,
            per_level_nprobe_supported,
        );

        Ok(SpireIndexHierarchySnapshot {
            active_epoch: root_control.active_epoch,
            root_pid,
            root_level,
            max_observed_level,
            hierarchy_depth,
            routing_object_count,
            root_routing_object_count,
            internal_routing_object_count,
            leaf_object_count,
            delta_object_count,
            centroid_dimensions,
            root_child_count,
            distinct_leaf_parent_count: u64::try_from(leaf_parent_pids.len()).map_err(|_| {
                "ec_spire hierarchy snapshot leaf parent count exceeds u64".to_owned()
            })?,
            recursive_routing_supported: hierarchy_shape_valid && internal_routing_object_count > 0,
            per_level_nprobe_supported,
            status,
            recommendation,
        })
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

pub(crate) unsafe fn index_object_snapshot(
    index_relation: pg_sys::Relation,
) -> Vec<SpireIndexObjectSnapshotRow> {
    let result = (|| -> Result<Vec<SpireIndexObjectSnapshotRow>, String> {
        let root_control = unsafe { page::read_root_control_page(index_relation) };
        if root_control.active_epoch == 0 {
            return Ok(Vec::new());
        }

        let (epoch_manifest, object_manifest, placement_directory) =
            unsafe { scan::load_relation_epoch_manifests(index_relation, root_control)? };
        let snapshot = meta::SpireValidatedEpochSnapshot::new(
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
        let mut rows = Vec::with_capacity(snapshot.object_manifest().entries.len());

        for manifest_entry in &snapshot.object_manifest().entries {
            let lookup = snapshot.require_lookup(manifest_entry.pid, "object snapshot")?;
            let placement = lookup.placement;
            let mut row = SpireIndexObjectSnapshotRow {
                active_epoch: root_control.active_epoch,
                pid: manifest_entry.pid,
                object_kind: "unreadable",
                object_version: manifest_entry.object_version,
                published_epoch_backref: 0,
                level: 0,
                parent_pid: 0,
                child_count: 0,
                assignment_count: 0,
                node_id: placement.node_id,
                local_store_id: placement.local_store_id,
                store_relid: placement.store_relid,
                placement_state: placement_state_name(placement.state),
                object_bytes: u64::from(placement.object_bytes),
                object_readable: false,
            };
            if placement.state == meta::SpirePlacementState::Available {
                let header =
                    storage::SpireObjectReader::read_object_header(&object_store, placement)?;
                row.object_kind = partition_object_kind_name(header.kind);
                row.object_version = header.object_version;
                row.published_epoch_backref = header.published_epoch_backref;
                row.level = header.level;
                row.parent_pid = header.parent_pid;
                row.child_count = u64::from(header.child_count);
                row.assignment_count = u64::from(header.assignment_count);
                row.object_readable = true;
            }
            rows.push(row);
        }

        rows.sort_by_key(|row| row.pid);
        Ok(rows)
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

pub(crate) unsafe fn index_delta_snapshot(
    index_relation: pg_sys::Relation,
) -> Vec<SpireIndexDeltaSnapshotRow> {
    let result = (|| -> Result<Vec<SpireIndexDeltaSnapshotRow>, String> {
        let root_control = unsafe { page::read_root_control_page(index_relation) };
        if root_control.active_epoch == 0 {
            return Ok(Vec::new());
        }

        let (epoch_manifest, object_manifest, placement_directory) =
            unsafe { scan::load_relation_epoch_manifests(index_relation, root_control)? };
        let snapshot = meta::SpireValidatedEpochSnapshot::new(
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
        let mut rows = Vec::new();

        for manifest_entry in &snapshot.object_manifest().entries {
            let lookup = snapshot.require_lookup(manifest_entry.pid, "delta snapshot")?;
            let placement = lookup.placement;
            if placement.state != meta::SpirePlacementState::Available {
                continue;
            }
            let header = storage::SpireObjectReader::read_object_header(&object_store, placement)?;
            if header.kind != storage::SpirePartitionObjectKind::Delta {
                continue;
            }
            let delta_object =
                storage::SpireObjectReader::read_delta_object(&object_store, placement)?;
            let mut insert_assignment_count = 0_u64;
            let mut delete_assignment_count = 0_u64;
            for assignment in &delta_object.assignments {
                if storage::is_delete_delta_assignment(assignment) {
                    delete_assignment_count =
                        delete_assignment_count.checked_add(1).ok_or_else(|| {
                            "ec_spire delta snapshot delete assignment count overflow".to_owned()
                        })?;
                } else if assignment.flags & storage::SPIRE_ASSIGNMENT_FLAG_DELTA_INSERT != 0 {
                    insert_assignment_count =
                        insert_assignment_count.checked_add(1).ok_or_else(|| {
                            "ec_spire delta snapshot insert assignment count overflow".to_owned()
                        })?;
                }
            }
            rows.push(SpireIndexDeltaSnapshotRow {
                active_epoch: root_control.active_epoch,
                delta_pid: header.pid,
                parent_leaf_pid: header.parent_pid,
                object_version: header.object_version,
                published_epoch_backref: header.published_epoch_backref,
                node_id: placement.node_id,
                local_store_id: placement.local_store_id,
                store_relid: placement.store_relid,
                placement_state: placement_state_name(placement.state),
                assignment_count: u64::from(header.assignment_count),
                insert_assignment_count,
                delete_assignment_count,
                object_bytes: u64::from(placement.object_bytes),
            });
        }

        rows.sort_by_key(|row| row.delta_pid);
        Ok(rows)
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

pub(crate) unsafe fn index_scan_placement_snapshot(
    index_relation: pg_sys::Relation,
    query_values: Vec<f32>,
) -> Vec<SpireIndexScanPlacementSnapshotRow> {
    let result = (|| -> Result<Vec<SpireIndexScanPlacementSnapshotRow>, String> {
        let query = scan::SpireScanQuery::new(query_values)?;
        let root_control = unsafe { page::read_root_control_page(index_relation) };
        if root_control.active_epoch == 0 {
            return Ok(Vec::new());
        }

        let (epoch_manifest, object_manifest, placement_directory) =
            unsafe { scan::load_relation_epoch_manifests(index_relation, root_control)? };
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
        let diagnostics = scan::collect_single_level_scan_placement_diagnostics(
            &snapshot,
            &object_store,
            &query,
            options::relation_options(index_relation),
        )?;
        let rows = diagnostics
            .stores
            .into_iter()
            .map(|store| SpireIndexScanPlacementSnapshotRow {
                active_epoch: store.epoch,
                effective_nprobe: diagnostics.scan_plan.nprobe,
                effective_nprobe_source: diagnostics.scan_plan.nprobe_source,
                effective_rerank_width: diagnostics.scan_plan.rerank_width as u64,
                effective_rerank_width_source: diagnostics.scan_plan.rerank_width_source,
                node_id: store.node_id,
                local_store_id: store.local_store_id,
                route_count: store.route_count as u64,
                leaf_route_count: store.leaf_route_count as u64,
                delta_route_count: store.delta_route_count as u64,
                prefetched_object_count: store.prefetched_object_count as u64,
                scanned_pid_count: store.scanned_pid_count as u64,
                leaf_pid_count: store.leaf_pid_count as u64,
                delta_pid_count: store.delta_pid_count as u64,
                candidate_row_count: store.candidate_row_count as u64,
                leaf_candidate_row_count: store.leaf_candidate_row_count as u64,
                delta_candidate_row_count: store.delta_candidate_row_count as u64,
                primary_candidate_row_count: store.primary_candidate_row_count as u64,
                boundary_replica_candidate_row_count: store.boundary_replica_candidate_row_count
                    as u64,
                deduped_candidate_row_count: store.deduped_candidate_row_count as u64,
                deduped_primary_candidate_row_count: store.deduped_primary_candidate_row_count
                    as u64,
                deduped_boundary_replica_candidate_row_count: store
                    .deduped_boundary_replica_candidate_row_count
                    as u64,
                truncated_candidate_row_count: store.truncated_candidate_row_count as u64,
                truncated_primary_candidate_row_count: store.truncated_primary_candidate_row_count
                    as u64,
                truncated_boundary_replica_candidate_row_count: store
                    .truncated_boundary_replica_candidate_row_count
                    as u64,
                candidate_winner_count: store.candidate_winner_count as u64,
                primary_candidate_winner_count: store.primary_candidate_winner_count as u64,
                boundary_replica_candidate_winner_count: store
                    .boundary_replica_candidate_winner_count
                    as u64,
                delete_delta_row_count: store.delete_delta_row_count as u64,
                dropped_unselected_delta_route_count: store.dropped_unselected_delta_route_count
                    as u64,
            })
            .collect();
        Ok(rows)
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

pub(crate) unsafe fn index_scan_routing_snapshot(
    index_relation: pg_sys::Relation,
    query_values: Vec<f32>,
) -> Vec<SpireIndexScanRoutingSnapshotRow> {
    let result = (|| -> Result<Vec<SpireIndexScanRoutingSnapshotRow>, String> {
        let query = scan::SpireScanQuery::new(query_values)?;
        let root_control = unsafe { page::read_root_control_page(index_relation) };
        if root_control.active_epoch == 0 {
            return Ok(Vec::new());
        }

        let (epoch_manifest, object_manifest, placement_directory) =
            unsafe { scan::load_relation_epoch_manifests(index_relation, root_control)? };
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
        let diagnostics = scan::collect_scan_routing_diagnostics(
            &snapshot,
            &object_store,
            &query,
            options::relation_options(index_relation),
        )?;
        let scan_plan = diagnostics.scan_plan;
        let rows = diagnostics
            .levels
            .into_iter()
            .map(|level| {
                Ok(SpireIndexScanRoutingSnapshotRow {
                    active_epoch: epoch_manifest.epoch,
                    effective_nprobe: scan_plan.nprobe,
                    effective_nprobe_source: scan_plan.nprobe_source,
                    recursive_beam_width: u64::try_from(
                        scan_plan.recursive_route_budget.beam_width,
                    )
                    .map_err(|_| {
                        "ec_spire routing diagnostics beam width exceeds u64".to_owned()
                    })?,
                    max_leaf_routes: u64::try_from(
                        scan_plan.recursive_route_budget.max_leaf_routes,
                    )
                    .map_err(|_| {
                        "ec_spire routing diagnostics max leaf routes exceeds u64".to_owned()
                    })?,
                    max_routing_expansions: u64::try_from(
                        scan_plan.recursive_route_budget.max_routing_expansions,
                    )
                    .map_err(|_| {
                        "ec_spire routing diagnostics max routing expansions exceeds u64"
                            .to_owned()
                    })?,
                    routing_level: level.level,
                    input_frontier_width: u64::try_from(level.input_frontier_width).map_err(
                        |_| {
                            "ec_spire routing diagnostics input frontier width exceeds u64"
                                .to_owned()
                        },
                    )?,
                    expanded_parent_count: u64::try_from(level.expanded_parent_count).map_err(
                        |_| {
                            "ec_spire routing diagnostics expanded parent count exceeds u64"
                                .to_owned()
                        },
                    )?,
                    selected_child_count: u64::try_from(level.selected_child_count).map_err(
                        |_| {
                            "ec_spire routing diagnostics selected child count exceeds u64"
                                .to_owned()
                        },
                    )?,
                    deduped_route_count: u64::try_from(level.deduped_route_count).map_err(
                        |_| {
                            "ec_spire routing diagnostics deduped route count exceeds u64"
                                .to_owned()
                        },
                    )?,
                    truncation_reason: level.truncation_reason,
                })
            })
            .collect::<Result<Vec<_>, String>>()?;
        Ok(rows)
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

pub(crate) unsafe fn index_root_routing_snapshot(
    index_relation: pg_sys::Relation,
) -> Vec<SpireIndexRootRoutingSnapshotRow> {
    let result = (|| -> Result<Vec<SpireIndexRootRoutingSnapshotRow>, String> {
        let root_control = unsafe { page::read_root_control_page(index_relation) };
        if root_control.active_epoch == 0 {
            return Ok(Vec::new());
        }

        let (epoch_manifest, object_manifest, placement_directory) =
            unsafe { scan::load_relation_epoch_manifests(index_relation, root_control)? };
        let snapshot = meta::SpireValidatedEpochSnapshot::new(
            &epoch_manifest,
            &object_manifest,
            &placement_directory,
        )?;
        let object_store =
            unsafe { storage::SpireRelationObjectStore::for_index_relation(index_relation)? };
        collect_root_routing_snapshot_rows(&snapshot, &object_store)
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

pub(crate) unsafe fn index_routing_centroid_snapshot(
    index_relation: pg_sys::Relation,
) -> Vec<SpireIndexRoutingCentroidSnapshotRow> {
    let result = (|| -> Result<Vec<SpireIndexRoutingCentroidSnapshotRow>, String> {
        let root_control = unsafe { page::read_root_control_page(index_relation) };
        if root_control.active_epoch == 0 {
            return Ok(Vec::new());
        }

        let (epoch_manifest, object_manifest, placement_directory) =
            unsafe { scan::load_relation_epoch_manifests(index_relation, root_control)? };
        let snapshot = meta::SpireValidatedEpochSnapshot::new(
            &epoch_manifest,
            &object_manifest,
            &placement_directory,
        )?;
        let object_store =
            unsafe { storage::SpireRelationObjectStore::for_index_relation(index_relation)? };
        collect_routing_centroid_snapshot_rows(&snapshot, &object_store)
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

fn collect_root_routing_snapshot_rows(
    snapshot: &meta::SpireValidatedEpochSnapshot<'_>,
    object_store: &impl storage::SpireObjectReader,
) -> Result<Vec<SpireIndexRootRoutingSnapshotRow>, String> {
    let mut root = None;
    // Walk the full manifest so malformed epochs with multiple roots are reported.
    for manifest_entry in &snapshot.object_manifest().entries {
        let lookup = snapshot.require_lookup(manifest_entry.pid, "root routing snapshot")?;
        let header = object_store.read_object_header(lookup.placement)?;
        if header.kind != storage::SpirePartitionObjectKind::Root {
            continue;
        }
        if root.is_some() {
            return Err("ec_spire root routing snapshot found multiple root objects".to_owned());
        }
        root = Some((
            manifest_entry.pid,
            manifest_entry.object_version,
            object_store.read_routing_object(lookup.placement)?,
        ));
    }

    let Some((root_pid, root_object_version, root_object)) = root else {
        return Err("ec_spire root routing snapshot found no active root object".to_owned());
    };
    let root_child_count = u64::try_from(root_object.child_count())
        .map_err(|_| "ec_spire root routing child count exceeds u64".to_owned())?;
    root_object
        .children()
        .map(|child| {
            let child_lookup = snapshot.require_lookup(child.child_pid, "root routing child")?;
            let child_header = object_store.read_object_header(child_lookup.placement)?;
            Ok(SpireIndexRootRoutingSnapshotRow {
                active_epoch: snapshot.epoch_manifest().epoch,
                root_pid,
                root_object_version,
                root_level: root_object.header.level,
                root_child_count,
                centroid_dimensions: root_object.dimensions,
                centroid_index: child.centroid_index,
                child_pid: child.child_pid,
                child_kind: partition_object_kind_name(child_header.kind),
                child_object_version: child_header.object_version,
                child_level: child_header.level,
                child_parent_pid: child_header.parent_pid,
                child_assignment_count: u64::from(child_header.assignment_count),
                child_node_id: child_lookup.placement.node_id,
                child_local_store_id: child_lookup.placement.local_store_id,
                child_store_relid: child_lookup.placement.store_relid,
                child_placement_state: placement_state_name(child_lookup.placement.state),
                child_object_bytes: u64::from(child_lookup.placement.object_bytes),
            })
        })
        .collect::<Result<Vec<_>, String>>()
}

fn collect_routing_centroid_snapshot_rows(
    snapshot: &meta::SpireValidatedEpochSnapshot<'_>,
    object_store: &impl storage::SpireObjectReader,
) -> Result<Vec<SpireIndexRoutingCentroidSnapshotRow>, String> {
    let mut rows = Vec::new();
    for manifest_entry in &snapshot.object_manifest().entries {
        let lookup =
            snapshot.require_lookup(manifest_entry.pid, "routing centroid snapshot parent")?;
        let parent_header = object_store.read_object_header(lookup.placement)?;
        if parent_header.kind != storage::SpirePartitionObjectKind::Root
            && parent_header.kind != storage::SpirePartitionObjectKind::Internal
        {
            continue;
        }
        let parent = object_store.read_routing_object(lookup.placement)?;
        let parent_child_count = u64::try_from(parent.child_count())
            .map_err(|_| "ec_spire routing centroid child count exceeds u64".to_owned())?;
        for child in parent.children() {
            let child_lookup =
                snapshot.require_lookup(child.child_pid, "routing centroid snapshot child")?;
            let child_header = object_store.read_object_header(child_lookup.placement)?;
            rows.push(SpireIndexRoutingCentroidSnapshotRow {
                active_epoch: snapshot.epoch_manifest().epoch,
                parent_pid: parent.header.pid,
                parent_kind: partition_object_kind_name(parent.header.kind),
                parent_object_version: parent.header.object_version,
                parent_level: parent.header.level,
                parent_child_count,
                centroid_dimensions: parent.dimensions,
                centroid_index: child.centroid_index,
                child_pid: child.child_pid,
                child_kind: partition_object_kind_name(child_header.kind),
                child_object_version: child_header.object_version,
                child_level: child_header.level,
                child_parent_pid: child_header.parent_pid,
                child_assignment_count: u64::from(child_header.assignment_count),
                child_node_id: child_lookup.placement.node_id,
                child_local_store_id: child_lookup.placement.local_store_id,
                child_store_relid: child_lookup.placement.store_relid,
                child_placement_state: placement_state_name(child_lookup.placement.state),
                child_object_bytes: u64::from(child_lookup.placement.object_bytes),
                centroid: child.centroid.to_vec(),
            });
        }
    }
    Ok(rows)
}
