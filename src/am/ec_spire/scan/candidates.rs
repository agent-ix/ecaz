pub(super) fn collect_reranked_quantized_routed_probe_candidates<F>(
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    object_store: &impl SpireObjectReader,
    query_vector: &[f32],
    nprobe: u32,
    payload_format: SpireAssignmentPayloadFormat,
    dedupe_mode: SpireCandidateDedupeMode,
    limit: Option<usize>,
    rerank_width: usize,
    exact_score_ip: F,
) -> Result<Vec<SpireScoredScanCandidate>, String>
where
    F: FnMut(&SpireScoredScanCandidate) -> Result<Option<f32>, String>,
{
    let mut candidates = collect_quantized_routed_probe_candidates(
        snapshot,
        object_store,
        query_vector,
        nprobe,
        payload_format,
        dedupe_mode,
        limit,
    )?;
    rerank_scored_candidates_by_ip(&mut candidates, rerank_width, exact_score_ip)?;
    Ok(candidates)
}

pub(super) fn collect_single_level_scan_plan_reranked_candidates<F>(
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    object_store: &impl SpireObjectReader,
    query_vector: &[f32],
    scan_plan: SpireSingleLevelScanPlan,
    exact_score_ip: F,
) -> Result<Vec<SpireScoredScanCandidate>, String>
where
    F: FnMut(&SpireScoredScanCandidate) -> Result<Option<f32>, String>,
{
    if scan_plan.nprobe == 0 {
        return Ok(Vec::new());
    }

    let mut candidates = collect_quantized_routed_probe_candidates_with_policy(
        snapshot,
        object_store,
        query_vector,
        &scan_plan.recursive_nprobe_policy,
        scan_plan.recursive_route_budget,
        scan_plan.payload_format,
        scan_plan.dedupe_mode,
        scan_plan.candidate_limit,
    )?;
    rerank_scored_candidates_by_ip(&mut candidates, scan_plan.rerank_width, exact_score_ip)?;
    Ok(candidates)
}

pub(super) fn collect_quantized_selected_leaf_candidates(
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    object_store: &impl SpireObjectReader,
    query_vector: &[f32],
    selected_leaf_pids: &[u64],
    payload_format: SpireAssignmentPayloadFormat,
    dedupe_mode: SpireCandidateDedupeMode,
    limit: Option<usize>,
) -> Result<Vec<SpireScoredScanCandidate>, String> {
    if selected_leaf_pids.is_empty() || limit == Some(0) {
        return Ok(Vec::new());
    }

    // The storage-node endpoint scores leaves selected by the coordinator; it
    // does not run top-graph or recursive routing itself.
    let snapshot = SpireValidatedEpochSnapshot::from_snapshot(*snapshot)?;
    let scorer =
        SpirePreparedAssignmentScorer::prepare(payload_format, query_vector.len(), query_vector)?;
    let leaf_routes =
        selected_leaf_routes_from_snapshot(&snapshot, object_store, selected_leaf_pids)?;
    let mut observer = SpireNoopRoutedScanObserver;
    collect_validated_quantized_leaf_route_candidates(
        &snapshot,
        object_store,
        leaf_routes,
        &scorer,
        dedupe_mode,
        limit,
        &mut observer,
    )
}

fn selected_leaf_routes_from_snapshot(
    snapshot: &SpireValidatedEpochSnapshot<'_>,
    object_store: &impl SpireObjectReader,
    selected_leaf_pids: &[u64],
) -> Result<Vec<SpireRecursiveLeafRoute>, String> {
    let mut seen = HashSet::new();
    let mut routes = Vec::with_capacity(selected_leaf_pids.len());

    for &leaf_pid in selected_leaf_pids {
        if leaf_pid == 0 {
            return Err("ec_spire remote search selected PID 0 is invalid".to_owned());
        }
        if !seen.insert(leaf_pid) {
            return Err(format!(
                "ec_spire remote search selected PID {leaf_pid} appears more than once"
            ));
        }

        let lookup = snapshot.require_lookup(leaf_pid, "remote search selected leaf")?;
        if should_skip_placement(
            snapshot.epoch_manifest().consistency_mode,
            lookup.placement.state,
        )? {
            continue;
        }
        let header = object_store.read_object_header(lookup.placement)?;
        if header.kind != SpirePartitionObjectKind::Leaf {
            return Err(format!(
                "ec_spire remote search selected PID {leaf_pid} is not a leaf object"
            ));
        }
        routes.push(SpireRecursiveLeafRoute {
            leaf_pid,
            parent_pid: header.parent_pid,
        });
    }

    Ok(routes)
}

pub(super) fn collect_top_graph_scan_plan_reranked_candidates<F>(
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    object_store: &impl SpireObjectReader,
    query_vector: &[f32],
    scan_plan: SpireSingleLevelScanPlan,
    top_graph_plan: SpireTopGraphOptionPlan,
    exact_score_ip: F,
) -> Result<Vec<SpireScoredScanCandidate>, String>
where
    F: FnMut(&SpireScoredScanCandidate) -> Result<Option<f32>, String>,
{
    if scan_plan.nprobe == 0 {
        return Ok(Vec::new());
    }

    let scorer = SpirePreparedAssignmentScorer::prepare(
        scan_plan.payload_format,
        query_vector.len(),
        query_vector,
    )?;
    let routed_rows = collect_snapshot_top_graph_routed_probe_leaf_rows(
        snapshot,
        object_store,
        query_vector,
        top_graph_plan.search_list_size.unwrap_or(scan_plan.nprobe),
        scan_plan.nprobe,
        &scan_plan.recursive_nprobe_policy,
        scan_plan.recursive_route_budget,
    )?;
    let mut candidates = rank_routed_leaf_rows_by_ip(
        routed_rows,
        |row| scorer.score_assignment_ip(row),
        scan_plan.dedupe_mode,
        scan_plan.candidate_limit,
    )?;
    rerank_scored_candidates_by_ip(&mut candidates, scan_plan.rerank_width, exact_score_ip)?;
    Ok(candidates)
}

pub(super) fn prepare_single_level_snapshot_scan_candidates<F>(
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    object_store: &impl SpireObjectReader,
    query: &SpireScanQuery,
    options: EcSpireOptions,
    exact_score_ip: F,
) -> Result<SpirePreparedScanCandidates, String>
where
    F: FnMut(&SpireScoredScanCandidate) -> Result<Option<f32>, String>,
{
    let top_graph_plan = options.top_graph_plan()?;
    let leaf_count = count_snapshot_single_level_leaf_pids(snapshot, object_store)?;
    let scan_plan = resolve_single_level_scan_plan(leaf_count, options)?;
    let candidates = if top_graph_plan.enabled {
        collect_top_graph_scan_plan_reranked_candidates(
            snapshot,
            object_store,
            query.values(),
            scan_plan,
            top_graph_plan,
            exact_score_ip,
        )?
    } else {
        collect_single_level_scan_plan_reranked_candidates(
            snapshot,
            object_store,
            query.values(),
            scan_plan,
            exact_score_ip,
        )?
    };

    Ok(SpirePreparedScanCandidates {
        scan_plan,
        candidates,
    })
}

pub(super) fn collect_single_level_scan_placement_diagnostics(
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    object_store: &impl SpireObjectReader,
    query: &SpireScanQuery,
    options: EcSpireOptions,
) -> Result<SpireScanPlacementDiagnostics, String> {
    let snapshot = SpireValidatedEpochSnapshot::from_snapshot(*snapshot)?;
    let (root_pid, root_object) = load_snapshot_root_routing_object(&snapshot, object_store)?;
    let leaf_count = u32::try_from(root_object.child_count())
        .map_err(|_| "ec_spire scan root child count exceeds u32".to_owned())?;
    let scan_plan = resolve_single_level_scan_plan(leaf_count, options)?;
    collect_validated_single_level_scan_placement_diagnostics(
        &snapshot,
        object_store,
        query,
        root_pid,
        root_object,
        scan_plan,
    )
}

pub(super) fn collect_single_level_scan_plan_placement_diagnostics(
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    object_store: &impl SpireObjectReader,
    query: &SpireScanQuery,
    scan_plan: SpireSingleLevelScanPlan,
) -> Result<SpireScanPlacementDiagnostics, String> {
    let snapshot = SpireValidatedEpochSnapshot::from_snapshot(*snapshot)?;
    let (root_pid, root_object) = load_snapshot_root_routing_object(&snapshot, object_store)?;
    let leaf_count = u32::try_from(root_object.child_count())
        .map_err(|_| "ec_spire scan root child count exceeds u32".to_owned())?;
    if scan_plan.leaf_count != leaf_count {
        return Err(format!(
            "ec_spire scan placement diagnostics plan leaf_count {} does not match snapshot leaf_count {leaf_count}",
            scan_plan.leaf_count
        ));
    }
    collect_validated_single_level_scan_placement_diagnostics(
        &snapshot,
        object_store,
        query,
        root_pid,
        root_object,
        scan_plan,
    )
}

fn collect_validated_single_level_scan_placement_diagnostics(
    snapshot: &SpireValidatedEpochSnapshot<'_>,
    object_store: &impl SpireObjectReader,
    query: &SpireScanQuery,
    root_pid: u64,
    root_object: SpireRoutingPartitionObject,
    scan_plan: SpireSingleLevelScanPlan,
) -> Result<SpireScanPlacementDiagnostics, String> {
    if scan_plan.nprobe == 0 {
        return Ok(SpireScanPlacementDiagnostics {
            scan_plan,
            stores: Vec::new(),
        });
    }

    let mut observer = SpireScanPlacementDiagnosticsObserver::new();
    let _candidates = collect_validated_quantized_routed_probe_candidates(
        snapshot,
        object_store,
        query.values(),
        root_pid,
        &root_object,
        scan_plan.nprobe,
        scan_plan.payload_format,
        scan_plan.dedupe_mode,
        scan_plan.candidate_limit,
        &mut observer,
    )?;

    Ok(SpireScanPlacementDiagnostics {
        scan_plan,
        stores: observer.into_stores(),
    })
}

pub(super) fn rerank_scored_candidates_by_ip<F>(
    candidates: &mut Vec<SpireScoredScanCandidate>,
    rerank_width: usize,
    mut exact_score_ip: F,
) -> Result<(), String>
where
    F: FnMut(&SpireScoredScanCandidate) -> Result<Option<f32>, String>,
{
    let rerank_len = if rerank_width == 0 {
        candidates.len()
    } else {
        rerank_width.min(candidates.len())
    };

    let mut reranked = Vec::with_capacity(rerank_len);
    let mut tail = candidates.split_off(rerank_len);
    for mut candidate in candidates.drain(..) {
        let Some(ip) = exact_score_ip(&candidate)? else {
            continue;
        };
        if !ip.is_finite() {
            return Err(
                "ec_spire routed candidate reranker returned a non-finite score".to_owned(),
            );
        }
        candidate.score = -ip;
        reranked.push(candidate);
    }

    reranked.sort_by(scored_candidate_cmp);
    if rerank_width == 0 {
        reranked.append(&mut tail);
    }
    *candidates = reranked;
    Ok(())
}

pub(super) fn collect_snapshot_delta_rows(
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    object_store: &impl SpireObjectReader,
) -> Result<Vec<SpireDeltaScanRow>, String> {
    let snapshot = SpireValidatedEpochSnapshot::from_snapshot(*snapshot)?;
    collect_validated_snapshot_delta_rows(&snapshot, object_store)
}

fn collect_validated_snapshot_delta_rows(
    snapshot: &SpireValidatedEpochSnapshot<'_>,
    object_store: &impl SpireObjectReader,
) -> Result<Vec<SpireDeltaScanRow>, String> {
    let mut rows = Vec::new();
    for manifest_entry in &snapshot.object_manifest().entries {
        let lookup = snapshot.require_lookup(manifest_entry.pid, "scan delta row collection")?;
        let placement = lookup.placement;

        if should_skip_placement(snapshot.epoch_manifest().consistency_mode, placement.state)? {
            continue;
        }

        let header = object_store.read_object_header(placement)?;
        if header.kind != SpirePartitionObjectKind::Delta {
            continue;
        }

        let delta_object = object_store.read_delta_object(placement)?;
        for (row_index, assignment) in delta_object.assignments.into_iter().enumerate() {
            let row_index = u32::try_from(row_index)
                .map_err(|_| "ec_spire scan row index exceeds u32".to_owned())?;
            rows.push(SpireDeltaScanRow {
                pid: manifest_entry.pid,
                object_version: manifest_entry.object_version,
                row_index,
                assignment,
            });
        }
    }
    Ok(rows)
}

pub(super) fn collect_snapshot_visible_primary_rows(
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    object_store: &impl SpireObjectReader,
) -> Result<Vec<SpireLeafScanRow>, String> {
    let snapshot = SpireValidatedEpochSnapshot::from_snapshot(*snapshot)?;
    collect_validated_snapshot_visible_primary_rows(&snapshot, object_store)
}

pub(super) fn collect_validated_snapshot_visible_primary_rows(
    snapshot: &SpireValidatedEpochSnapshot<'_>,
    object_store: &impl SpireObjectReader,
) -> Result<Vec<SpireLeafScanRow>, String> {
    let delta_rows = collect_validated_snapshot_delta_rows(snapshot, object_store)?;
    let deleted_vec_ids: HashSet<_> = delta_rows
        .iter()
        .filter(|row| is_delete_delta_assignment(&row.assignment))
        .map(|row| row.assignment.vec_id.clone())
        .collect();

    let mut visible_rows = Vec::new();
    visible_rows.extend(
        collect_validated_snapshot_leaf_rows(snapshot, object_store)?
            .into_iter()
            .filter(|row| {
                is_visible_primary_assignment(&row.assignment)
                    && !deleted_vec_ids.contains(&row.assignment.vec_id)
            }),
    );
    visible_rows.extend(delta_rows.into_iter().filter_map(|row| {
        if is_visible_primary_assignment(&row.assignment)
            && !deleted_vec_ids.contains(&row.assignment.vec_id)
        {
            Some(SpireLeafScanRow {
                pid: row.pid,
                object_version: row.object_version,
                row_index: row.row_index,
                assignment: row.assignment,
            })
        } else {
            None
        }
    }));

    let mut visible_vec_ids = HashSet::new();
    for row in &visible_rows {
        if !visible_vec_ids.insert(row.assignment.vec_id.clone()) {
            return Err(
                "ec_spire visible snapshot contains duplicate primary vec_id assignments"
                    .to_owned(),
            );
        }
    }

    Ok(visible_rows)
}

fn append_quantized_leaf_candidates_for_pid(
    snapshot: &SpireValidatedEpochSnapshot<'_>,
    object_store: &impl SpireObjectReader,
    route: SpireLeafObjectReadRoute,
    scorer: &SpirePreparedAssignmentScorer,
    deleted_vec_ids: &HashSet<SpireVecId>,
    candidates: &mut Vec<SpireScoredScanCandidate>,
    candidates_by_vec_id: &mut Option<HashMap<SpireVecId, SpireScoredScanCandidate>>,
    observer: &mut impl SpireRoutedScanObserver,
) -> Result<(), String> {
    let leaf_pid = route.leaf_pid;
    let placement = &route.placement;
    if should_skip_placement(snapshot.epoch_manifest().consistency_mode, placement.state)? {
        return Ok(());
    }

    let header = object_store.read_object_header(placement)?;
    if header.kind != SpirePartitionObjectKind::Leaf {
        return Err(format!(
            "ec_spire quantized routed scan pid {leaf_pid} is not a leaf object"
        ));
    }
    if header.parent_pid != route.parent_pid {
        return Err(format!(
            "ec_spire quantized routed scan leaf pid {leaf_pid} parent {} does not match expected parent pid {}",
            header.parent_pid,
            route.parent_pid,
        ));
    }
    observer.scanned_leaf(snapshot.epoch_manifest().epoch, &route.placement);

    match object_store.read_leaf_object_v2(placement) {
        Ok(leaf_object) => {
            for columns in leaf_object.column_segments()? {
                let columns = columns?;
                append_quantized_v2_column_candidates(
                    columns,
                    snapshot.epoch_manifest().epoch,
                    leaf_pid,
                    route.object_version,
                    scorer,
                    deleted_vec_ids,
                    candidates,
                    candidates_by_vec_id,
                    &route.placement,
                    observer,
                )?;
            }
            Ok(())
        }
        Err(v2_error) => {
            let leaf_object = object_store.read_leaf_object(placement).map_err(|v1_error| {
                format!(
                    "ec_spire quantized scan could not read leaf pid {leaf_pid} as V2 or V1: V2 error: {v2_error}; V1 error: {v1_error}"
                )
            })?;
            append_quantized_v1_leaf_candidates(
                leaf_object,
                snapshot.epoch_manifest().epoch,
                leaf_pid,
                route.object_version,
                scorer,
                deleted_vec_ids,
                candidates,
                candidates_by_vec_id,
                &route.placement,
                observer,
            )
        }
    }
}

fn append_quantized_v2_column_candidates(
    columns: SpireLeafObjectColumns<'_>,
    epoch: u64,
    pid: u64,
    object_version: u64,
    scorer: &SpirePreparedAssignmentScorer,
    deleted_vec_ids: &HashSet<SpireVecId>,
    candidates: &mut Vec<SpireScoredScanCandidate>,
    candidates_by_vec_id: &mut Option<HashMap<SpireVecId, SpireScoredScanCandidate>>,
    placement: &SpirePlacementEntry,
    observer: &mut impl SpireRoutedScanObserver,
) -> Result<(), String> {
    let column_format = SpireAssignmentPayloadFormat::from_tag(columns.payload_format)?;
    if column_format != scorer.payload_format() {
        return Err(format!(
            "ec_spire leaf V2 payload format {:?} does not match prepared scorer {:?}",
            column_format,
            scorer.payload_format()
        ));
    }

    let mut scores = vec![0.0; columns.row_count()];
    scorer.score_batch_ip(
        columns.payload_stride,
        columns.payloads,
        columns.gammas,
        &mut scores,
    )?;

    for (row_offset, ip) in scores.into_iter().enumerate() {
        if !is_visible_scored_assignment_flags(columns.flags[row_offset]) {
            continue;
        }
        if !ip.is_finite() {
            return Err(
                "ec_spire routed candidate batch scorer returned a non-finite score".to_owned(),
            );
        }

        let row = columns.row(row_offset)?;
        let vec_id = SpireVecId::local(row.local_vec_seq()?);
        if deleted_vec_ids.contains(&vec_id) {
            continue;
        }
        observer.visible_leaf_candidate(epoch, placement);
        let candidate = SpireScoredScanCandidate {
            epoch,
            pid,
            object_version,
            row_index: row.row_index,
            assignment_flags: row.flags,
            vec_id,
            heap_tid: row.heap_tid,
            score: -ip,
        };
        append_scored_candidate(candidate, candidates, candidates_by_vec_id);
    }
    Ok(())
}

fn append_quantized_delta_candidates_for_routes(
    snapshot: &SpireValidatedEpochSnapshot<'_>,
    object_store: &impl SpireObjectReader,
    delta_routes: &[SpireDeltaObjectRoute],
    scorer: &SpirePreparedAssignmentScorer,
    deleted_vec_ids: &HashSet<SpireVecId>,
    candidates: &mut Vec<SpireScoredScanCandidate>,
    candidates_by_vec_id: &mut Option<HashMap<SpireVecId, SpireScoredScanCandidate>>,
    observer: &mut impl SpireRoutedScanObserver,
) -> Result<(), String> {
    for route in delta_routes {
        let placement = &route.placement;
        if should_skip_placement(snapshot.epoch_manifest().consistency_mode, placement.state)? {
            continue;
        }

        let delta_object = object_store.read_delta_object(placement)?;
        if delta_object.header.parent_pid != route.parent_leaf_pid {
            return Err(format!(
                "ec_spire delta route parent {} does not match object parent {}",
                route.parent_leaf_pid, delta_object.header.parent_pid
            ));
        }
        for (row_index, assignment) in delta_object.assignments.into_iter().enumerate() {
            if is_delete_delta_assignment(&assignment) {
                continue;
            }
            if !is_visible_scored_assignment(&assignment) {
                continue;
            }
            if deleted_vec_ids.contains(&assignment.vec_id) {
                continue;
            }
            let ip = scorer.score_assignment_ip(&assignment)?;
            if !ip.is_finite() {
                return Err(
                    "ec_spire routed delta candidate scorer returned a non-finite score".to_owned(),
                );
            }
            let row_index = u32::try_from(row_index)
                .map_err(|_| "ec_spire scan row index exceeds u32".to_owned())?;
            observer.visible_delta_candidate(snapshot.epoch_manifest().epoch, placement);
            let candidate = SpireScoredScanCandidate {
                epoch: snapshot.epoch_manifest().epoch,
                pid: route.delta_pid,
                object_version: route.object_version,
                row_index,
                assignment_flags: assignment.flags,
                vec_id: assignment.vec_id,
                heap_tid: assignment.heap_tid,
                score: -ip,
            };
            append_scored_candidate(candidate, candidates, candidates_by_vec_id);
        }
    }
    Ok(())
}

fn collect_delta_delete_vec_ids_for_routes(
    snapshot: &SpireValidatedEpochSnapshot<'_>,
    object_store: &impl SpireObjectReader,
    delta_routes: &[SpireDeltaObjectRoute],
    observer: &mut impl SpireRoutedScanObserver,
) -> Result<HashSet<SpireVecId>, String> {
    let mut deleted_vec_ids = HashSet::new();
    for route in delta_routes {
        let placement = &route.placement;
        if should_skip_placement(snapshot.epoch_manifest().consistency_mode, placement.state)? {
            continue;
        }

        observer.scanned_delta(snapshot.epoch_manifest().epoch, placement);

        let delta_object = object_store.read_delta_object(placement)?;
        if delta_object.header.parent_pid != route.parent_leaf_pid {
            return Err(format!(
                "ec_spire delete delta route parent {} does not match object parent {}",
                route.parent_leaf_pid, delta_object.header.parent_pid
            ));
        }
        for assignment in delta_object.assignments {
            if is_delete_delta_assignment(&assignment) {
                observer.delete_delta_row(snapshot.epoch_manifest().epoch, placement);
                deleted_vec_ids.insert(assignment.vec_id);
            }
        }
    }
    Ok(deleted_vec_ids)
}

fn append_quantized_v1_leaf_candidates(
    leaf_object: SpireLeafPartitionObject,
    epoch: u64,
    pid: u64,
    object_version: u64,
    scorer: &SpirePreparedAssignmentScorer,
    deleted_vec_ids: &HashSet<SpireVecId>,
    candidates: &mut Vec<SpireScoredScanCandidate>,
    candidates_by_vec_id: &mut Option<HashMap<SpireVecId, SpireScoredScanCandidate>>,
    placement: &SpirePlacementEntry,
    observer: &mut impl SpireRoutedScanObserver,
) -> Result<(), String> {
    for (row_index, assignment) in leaf_object.assignments.into_iter().enumerate() {
        if !is_visible_scored_assignment(&assignment) {
            continue;
        }
        if deleted_vec_ids.contains(&assignment.vec_id) {
            continue;
        }
        let ip = scorer.score_assignment_ip(&assignment)?;
        if !ip.is_finite() {
            return Err("ec_spire routed candidate scorer returned a non-finite score".to_owned());
        }
        let row_index = u32::try_from(row_index)
            .map_err(|_| "ec_spire scan row index exceeds u32".to_owned())?;
        observer.visible_leaf_candidate(epoch, placement);
        let candidate = SpireScoredScanCandidate {
            epoch,
            pid,
            object_version,
            row_index,
            assignment_flags: assignment.flags,
            vec_id: assignment.vec_id,
            heap_tid: assignment.heap_tid,
            score: -ip,
        };
        append_scored_candidate(candidate, candidates, candidates_by_vec_id);
    }
    Ok(())
}

fn rank_routed_leaf_rows_by_ip<F>(
    routed_rows: Vec<SpireRoutedLeafScanRows>,
    mut score_ip: F,
    dedupe_mode: SpireCandidateDedupeMode,
    limit: Option<usize>,
) -> Result<Vec<SpireScoredScanCandidate>, String>
where
    F: FnMut(&SpireLeafAssignmentRow) -> Result<f32, String>,
{
    if limit == Some(0) {
        return Ok(Vec::new());
    }

    let mut candidates = Vec::new();
    let mut candidates_by_vec_id = match dedupe_mode {
        SpireCandidateDedupeMode::NoReplicaDedupeDisabled => None,
        SpireCandidateDedupeMode::VecIdDedupeEnabled => Some(HashMap::new()),
    };
    for routed in routed_rows {
        for row in routed.rows {
            if !is_visible_scored_assignment(&row.assignment) {
                continue;
            }
            let ip = score_ip(&row.assignment)?;
            if !ip.is_finite() {
                return Err(
                    "ec_spire routed candidate scorer returned a non-finite score".to_owned(),
                );
            }
            let candidate = SpireScoredScanCandidate {
                epoch: routed.epoch,
                pid: row.pid,
                object_version: row.object_version,
                row_index: row.row_index,
                assignment_flags: row.assignment.flags,
                vec_id: row.assignment.vec_id.clone(),
                heap_tid: row.assignment.heap_tid,
                score: -ip,
            };
            append_scored_candidate(candidate, &mut candidates, &mut candidates_by_vec_id);
        }
    }

    if let Some(candidates_by_vec_id) = candidates_by_vec_id {
        candidates.extend(candidates_by_vec_id.into_values());
    }

    Ok(rank_bounded_scored_candidates(candidates, limit))
}

fn append_scored_candidate(
    candidate: SpireScoredScanCandidate,
    candidates: &mut Vec<SpireScoredScanCandidate>,
    candidates_by_vec_id: &mut Option<HashMap<SpireVecId, SpireScoredScanCandidate>>,
) {
    if let Some(candidates_by_vec_id) = candidates_by_vec_id.as_mut() {
        match candidates_by_vec_id.entry(candidate.vec_id.clone()) {
            std::collections::hash_map::Entry::Occupied(mut entry) => {
                if scored_candidate_cmp(&candidate, entry.get()) == Ordering::Less {
                    *entry.get_mut() = candidate;
                }
            }
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert(candidate);
            }
        }
    } else {
        candidates.push(candidate);
    }
}

fn scored_candidate_cmp(
    left: &SpireScoredScanCandidate,
    right: &SpireScoredScanCandidate,
) -> Ordering {
    left.score
        .total_cmp(&right.score)
        .then_with(|| right.epoch.cmp(&left.epoch))
        .then_with(|| {
            candidate_assignment_role_rank(left).cmp(&candidate_assignment_role_rank(right))
        })
        .then_with(|| left.heap_tid.block_number.cmp(&right.heap_tid.block_number))
        .then_with(|| {
            left.heap_tid
                .offset_number
                .cmp(&right.heap_tid.offset_number)
        })
        .then_with(|| left.pid.cmp(&right.pid))
        .then_with(|| left.row_index.cmp(&right.row_index))
        .then_with(|| left.vec_id.as_bytes().cmp(right.vec_id.as_bytes()))
}

fn candidate_assignment_role_rank(candidate: &SpireScoredScanCandidate) -> u8 {
    u8::from(candidate.assignment_flags & SPIRE_ASSIGNMENT_FLAG_BOUNDARY_REPLICA != 0)
}

fn rank_bounded_scored_candidates<I>(
    candidates: I,
    limit: Option<usize>,
) -> Vec<SpireScoredScanCandidate>
where
    I: IntoIterator<Item = SpireScoredScanCandidate>,
{
    let Some(limit) = limit else {
        let mut ranked = candidates.into_iter().collect::<Vec<_>>();
        ranked.sort_by(scored_candidate_cmp);
        return ranked;
    };

    if limit == 0 {
        return Vec::new();
    }

    let mut heap = BinaryHeap::with_capacity(limit);
    for candidate in candidates {
        let entry = SpireScoredScanCandidateHeapEntry { candidate };
        if heap.len() < limit {
            heap.push(entry);
            continue;
        }

        if heap
            .peek()
            .is_some_and(|worst| scored_candidate_cmp(&entry.candidate, &worst.candidate).is_lt())
        {
            heap.pop();
            heap.push(entry);
        }
    }

    let mut ranked = heap
        .into_iter()
        .map(|entry| entry.candidate)
        .collect::<Vec<_>>();
    ranked.sort_by(scored_candidate_cmp);
    ranked
}
