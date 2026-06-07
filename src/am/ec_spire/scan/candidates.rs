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
    collect_single_level_scan_plan_reranked_candidates_with_prefetch(
        snapshot,
        object_store,
        query_vector,
        scan_plan,
        |_| Ok(()),
        exact_score_ip,
    )
}

fn collect_single_level_scan_plan_reranked_candidates_with_prefetch<F, P>(
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    object_store: &impl SpireObjectReader,
    query_vector: &[f32],
    scan_plan: SpireSingleLevelScanPlan,
    prefetch_candidates: P,
    exact_score_ip: F,
) -> Result<Vec<SpireScoredScanCandidate>, String>
where
    F: FnMut(&SpireScoredScanCandidate) -> Result<Option<f32>, String>,
    P: FnMut(&[SpireScoredScanCandidate]) -> Result<(), String>,
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
        scan_plan.max_routed_candidate_rows,
        scan_plan.payload_format,
        scan_plan.dedupe_mode,
        scan_plan.candidate_limit,
    )?;
    rerank_scored_candidates_by_ip_with_prefetch(
        &mut candidates,
        scan_plan.rerank_width,
        prefetch_candidates,
        exact_score_ip,
    )?;
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
            route_score: 0.0,
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
    collect_top_graph_scan_plan_reranked_candidates_with_prefetch(
        snapshot,
        object_store,
        query_vector,
        scan_plan,
        top_graph_plan,
        |_| Ok(()),
        exact_score_ip,
    )
}

fn collect_top_graph_scan_plan_reranked_candidates_with_prefetch<F, P>(
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    object_store: &impl SpireObjectReader,
    query_vector: &[f32],
    scan_plan: SpireSingleLevelScanPlan,
    top_graph_plan: SpireTopGraphOptionPlan,
    prefetch_candidates: P,
    exact_score_ip: F,
) -> Result<Vec<SpireScoredScanCandidate>, String>
where
    F: FnMut(&SpireScoredScanCandidate) -> Result<Option<f32>, String>,
    P: FnMut(&[SpireScoredScanCandidate]) -> Result<(), String>,
{
    if scan_plan.nprobe == 0 {
        return Ok(Vec::new());
    }

    let scorer = SpirePreparedAssignmentScorer::prepare(
        scan_plan.payload_format,
        query_vector.len(),
        query_vector,
    )?;
    let snapshot = SpireValidatedEpochSnapshot::from_snapshot(*snapshot)?;
    let hierarchy = load_snapshot_routing_hierarchy(&snapshot, object_store)?;
    let (_top_graph_pid, top_graph) = load_snapshot_top_graph_object(&snapshot, object_store)?
        .ok_or_else(|| "ec_spire scan snapshot has no available top graph object".to_owned())?;
    let leaf_assignment_counts = &hierarchy.leaf_assignment_counts_by_pid;
    let mut leaf_row_count =
        |route| leaf_route_assignment_count_from_loaded_hierarchy(leaf_assignment_counts, route);
    let leaf_routes = route_top_graph_object_to_leaf_routes_with_row_budget(
        &hierarchy.root_object,
        &hierarchy.internal_objects_by_pid,
        &top_graph,
        query_vector,
        top_graph_plan.search_list_size.unwrap_or(scan_plan.nprobe),
        scan_plan.nprobe,
        &scan_plan.recursive_nprobe_policy,
        scan_plan.recursive_route_budget,
        scan_plan.max_routed_candidate_rows,
        &mut leaf_row_count,
    )?
    .routes;
    let mut observer = SpireNoopRoutedScanObserver;
    let mut candidates = collect_validated_quantized_leaf_route_candidates(
        &snapshot,
        object_store,
        leaf_routes,
        &scorer,
        scan_plan.dedupe_mode,
        scan_plan.candidate_limit,
        &mut observer,
    )?;
    rerank_scored_candidates_by_ip_with_prefetch(
        &mut candidates,
        scan_plan.rerank_width,
        prefetch_candidates,
        exact_score_ip,
    )?;
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
    prepare_single_level_snapshot_scan_candidates_with_prefetch(
        snapshot,
        object_store,
        query,
        options,
        |_| Ok(()),
        exact_score_ip,
    )
}

fn prepare_single_level_snapshot_scan_candidates_with_prefetch<F, P>(
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    object_store: &impl SpireObjectReader,
    query: &SpireScanQuery,
    options: EcSpireOptions,
    prefetch_candidates: P,
    exact_score_ip: F,
) -> Result<SpirePreparedScanCandidates, String>
where
    F: FnMut(&SpireScoredScanCandidate) -> Result<Option<f32>, String>,
    P: FnMut(&[SpireScoredScanCandidate]) -> Result<(), String>,
{
    let top_graph_plan = options.top_graph_plan()?;
    let leaf_count = count_snapshot_recursive_leaf_pids(snapshot, object_store)?;
    let scan_plan = resolve_single_level_scan_plan(leaf_count, options)?;
    let candidates = if top_graph_plan.enabled {
        collect_top_graph_scan_plan_reranked_candidates_with_prefetch(
            snapshot,
            object_store,
            query.values(),
            scan_plan,
            top_graph_plan,
            prefetch_candidates,
            exact_score_ip,
        )?
    } else {
        collect_single_level_scan_plan_reranked_candidates_with_prefetch(
            snapshot,
            object_store,
            query.values(),
            scan_plan,
            prefetch_candidates,
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
    let top_graph_plan = options.top_graph_plan()?;
    let snapshot = SpireValidatedEpochSnapshot::from_snapshot(*snapshot)?;
    let hierarchy = load_snapshot_routing_hierarchy(&snapshot, object_store)?;
    let leaf_count =
        count_recursive_routing_leaf_pids(&hierarchy.root_object, &hierarchy.internal_objects_by_pid)?;
    let scan_plan = resolve_single_level_scan_plan(leaf_count, options)?;
    if top_graph_plan.enabled {
        return collect_validated_top_graph_scan_placement_diagnostics(
            &snapshot,
            object_store,
            query,
            &hierarchy,
            scan_plan,
            top_graph_plan,
        );
    }
    collect_validated_single_level_scan_placement_diagnostics(
        &snapshot,
        object_store,
        query,
        &hierarchy,
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
    let hierarchy = load_snapshot_routing_hierarchy(&snapshot, object_store)?;
    let leaf_count =
        count_recursive_routing_leaf_pids(&hierarchy.root_object, &hierarchy.internal_objects_by_pid)?;
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
        &hierarchy,
        scan_plan,
    )
}

fn collect_validated_single_level_scan_placement_diagnostics(
    snapshot: &SpireValidatedEpochSnapshot<'_>,
    object_store: &impl SpireObjectReader,
    query: &SpireScanQuery,
    hierarchy: &SpireLoadedRoutingHierarchy,
    scan_plan: SpireSingleLevelScanPlan,
) -> Result<SpireScanPlacementDiagnostics, String> {
    if scan_plan.nprobe == 0 {
        return Ok(SpireScanPlacementDiagnostics {
            scan_plan,
            stores: Vec::new(),
            leaves: Vec::new(),
        });
    }

    let mut observer = SpireScanPlacementDiagnosticsObserver::new();
    let _candidates = collect_validated_recursive_quantized_routed_probe_candidates(
        snapshot,
        object_store,
        query.values(),
        hierarchy,
        &scan_plan.recursive_nprobe_policy,
        scan_plan.recursive_route_budget,
        scan_plan.max_routed_candidate_rows,
        scan_plan.payload_format,
        scan_plan.dedupe_mode,
        scan_plan.candidate_limit,
        &mut observer,
    )?;
    let (stores, leaves) = observer.into_diagnostics();

    Ok(SpireScanPlacementDiagnostics {
        scan_plan,
        stores,
        leaves,
    })
}

fn collect_validated_top_graph_scan_placement_diagnostics(
    snapshot: &SpireValidatedEpochSnapshot<'_>,
    object_store: &impl SpireObjectReader,
    query: &SpireScanQuery,
    hierarchy: &SpireLoadedRoutingHierarchy,
    scan_plan: SpireSingleLevelScanPlan,
    top_graph_plan: SpireTopGraphOptionPlan,
) -> Result<SpireScanPlacementDiagnostics, String> {
    if scan_plan.nprobe == 0 {
        return Ok(SpireScanPlacementDiagnostics {
            scan_plan,
            stores: Vec::new(),
            leaves: Vec::new(),
        });
    }

    let (_top_graph_pid, top_graph) = load_snapshot_top_graph_object(snapshot, object_store)?
        .ok_or_else(|| "ec_spire scan placement diagnostics has no available top graph object".to_owned())?;
    let scorer = SpirePreparedAssignmentScorer::prepare(
        scan_plan.payload_format,
        query.values().len(),
        query.values(),
    )?;
    let leaf_assignment_counts = &hierarchy.leaf_assignment_counts_by_pid;
    let mut leaf_row_count =
        |route| leaf_route_assignment_count_from_loaded_hierarchy(leaf_assignment_counts, route);
    let leaf_routes = route_top_graph_object_to_leaf_routes_with_row_budget(
        &hierarchy.root_object,
        &hierarchy.internal_objects_by_pid,
        &top_graph,
        query.values(),
        top_graph_plan.search_list_size.unwrap_or(scan_plan.nprobe),
        scan_plan.nprobe,
        &scan_plan.recursive_nprobe_policy,
        scan_plan.recursive_route_budget,
        scan_plan.max_routed_candidate_rows,
        &mut leaf_row_count,
    )?
    .routes;

    let mut observer = SpireScanPlacementDiagnosticsObserver::new();
    let _candidates = collect_validated_quantized_leaf_route_candidates(
        snapshot,
        object_store,
        leaf_routes,
        &scorer,
        scan_plan.dedupe_mode,
        scan_plan.candidate_limit,
        &mut observer,
    )?;
    let (stores, leaves) = observer.into_diagnostics();

    Ok(SpireScanPlacementDiagnostics {
        scan_plan,
        stores,
        leaves,
    })
}

pub(super) fn collect_scan_leaf_block_rank_snapshot(
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    object_store: &impl SpireObjectReader,
    query: &SpireScanQuery,
    options: EcSpireOptions,
    target_local_sequences: &[u64],
) -> Result<Vec<SpireLeafBlockRankSnapshotRow>, String> {
    let top_graph_plan = options.top_graph_plan()?;
    let snapshot = SpireValidatedEpochSnapshot::from_snapshot(*snapshot)?;
    let hierarchy = load_snapshot_routing_hierarchy(&snapshot, object_store)?;
    let leaf_count =
        count_recursive_routing_leaf_pids(&hierarchy.root_object, &hierarchy.internal_objects_by_pid)?;
    let scan_plan = resolve_single_level_scan_plan(leaf_count, options)?;
    collect_validated_leaf_block_rank_snapshot(
        &snapshot,
        object_store,
        query,
        &hierarchy,
        scan_plan,
        top_graph_plan,
        target_local_sequences,
    )
}

pub(super) fn collect_scan_leaf_target_block_rank_snapshot(
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    object_store: &impl SpireObjectReader,
    query: &SpireScanQuery,
    options: EcSpireOptions,
    target_local_sequences: &[u64],
) -> Result<Vec<SpireLeafBlockRankSnapshotRow>, String> {
    let top_graph_plan = options.top_graph_plan()?;
    let snapshot = SpireValidatedEpochSnapshot::from_snapshot(*snapshot)?;
    let hierarchy = load_snapshot_routing_hierarchy(&snapshot, object_store)?;
    let leaf_count =
        count_recursive_routing_leaf_pids(&hierarchy.root_object, &hierarchy.internal_objects_by_pid)?;
    let scan_plan = resolve_single_level_scan_plan(leaf_count, options)?;
    collect_validated_leaf_target_block_rank_snapshot(
        &snapshot,
        object_store,
        query,
        &hierarchy,
        scan_plan,
        top_graph_plan,
        target_local_sequences,
    )
}

fn collect_validated_leaf_block_rank_snapshot(
    snapshot: &SpireValidatedEpochSnapshot<'_>,
    object_store: &impl SpireObjectReader,
    query: &SpireScanQuery,
    hierarchy: &SpireLoadedRoutingHierarchy,
    scan_plan: SpireSingleLevelScanPlan,
    top_graph_plan: SpireTopGraphOptionPlan,
    target_local_sequences: &[u64],
) -> Result<Vec<SpireLeafBlockRankSnapshotRow>, String> {
    let max_global_blocks = current_session_leaf_block_pruning_max_global_blocks();
    let max_global_blocks = if max_global_blocks <= 0 {
        0
    } else {
        u64::try_from(max_global_blocks)
            .map_err(|_| "ec_spire.leaf_block_pruning_max_global_blocks exceeds u64".to_owned())?
    };
    let radius_weight = current_session_leaf_block_pruning_summary_radius_weight();
    if target_local_sequences.is_empty() {
        return Ok(Vec::new());
    }
    if scan_plan.nprobe == 0 {
        return Ok(missing_leaf_block_rank_rows(
            snapshot,
            scan_plan,
            target_local_sequences,
            "nprobe_zero",
            max_global_blocks,
            radius_weight,
            0,
            None,
            &HashSet::new(),
        ));
    }

    let scorer = SpirePreparedAssignmentScorer::prepare(
        scan_plan.payload_format,
        query.values().len(),
        query.values(),
    )?;
    let leaf_assignment_counts = &hierarchy.leaf_assignment_counts_by_pid;
    let mut leaf_row_count =
        |route| leaf_route_assignment_count_from_loaded_hierarchy(leaf_assignment_counts, route);
    let leaf_routes = if top_graph_plan.enabled {
        let (_top_graph_pid, top_graph) = load_snapshot_top_graph_object(snapshot, object_store)?
            .ok_or_else(|| {
                "ec_spire scan leaf block rank snapshot has no available top graph object"
                    .to_owned()
            })?;
        route_top_graph_object_to_leaf_routes_with_row_budget(
            &hierarchy.root_object,
            &hierarchy.internal_objects_by_pid,
            &top_graph,
            query.values(),
            top_graph_plan.search_list_size.unwrap_or(scan_plan.nprobe),
            scan_plan.nprobe,
            &scan_plan.recursive_nprobe_policy,
            scan_plan.recursive_route_budget,
            scan_plan.max_routed_candidate_rows,
            &mut leaf_row_count,
        )?
        .routes
    } else {
        route_recursive_routing_objects_to_leaf_routes_with_row_budget(
            &hierarchy.root_object,
            &hierarchy.internal_objects_by_pid,
            query.values(),
            &scan_plan.recursive_nprobe_policy,
            scan_plan.recursive_route_budget,
            scan_plan.max_routed_candidate_rows,
            &mut leaf_row_count,
        )?
        .routes
    };
    let route_context_by_pid = leaf_routes
        .iter()
        .enumerate()
        .map(|(index, route)| {
            let rank = u64::try_from(index)
                .map_err(|_| "ec_spire route rank exceeds u64".to_owned())?
                .checked_add(1)
                .ok_or_else(|| "ec_spire route rank overflow".to_owned())?;
            Ok((route.leaf_pid, (rank, route.route_score)))
        })
        .collect::<Result<HashMap<_, _>, String>>()?;

    let mut observer = SpireNoopRoutedScanObserver;
    let delta_routes = collect_snapshot_delta_object_routes(snapshot, object_store)?;
    let mut delta_routes_by_parent = HashMap::<u64, Vec<SpireDeltaObjectRoute>>::new();
    let route_groups =
        group_leaf_and_delta_reads_by_local_store(snapshot, leaf_routes, delta_routes, &mut observer)?;
    prefetch_store_object_read_groups(
        object_store,
        snapshot.epoch_manifest().epoch,
        &route_groups,
        &mut observer,
    )?;
    for route_group in &route_groups {
        for route in &route_group.delta_routes {
            delta_routes_by_parent
                .entry(route.parent_leaf_pid)
                .or_default()
                .push(*route);
        }
    }

    let mut loaded_leaf_routes = Vec::new();
    for route_group in route_groups {
        for route in route_group.leaf_routes {
            let leaf_delta_routes = delta_routes_by_parent
                .get(&route.leaf_pid)
                .map(Vec::as_slice)
                .unwrap_or(&[]);
            let loaded_delta_routes =
                load_delta_rows_for_routes(snapshot, object_store, leaf_delta_routes, &mut observer)?;
            let deleted_vec_ids =
                collect_delta_delete_vec_ids_for_loaded_routes(&loaded_delta_routes);
            let Some(leaf_object) =
                read_quantized_v2_leaf_object_for_route(snapshot, object_store, route, &mut observer)?
            else {
                continue;
            };
            loaded_leaf_routes.push(SpireLoadedQuantizedLeafRoute {
                route,
                leaf_object,
                selected_row_ranges: None,
                loaded_delta_routes,
                deleted_vec_ids,
            });
        }
    }

    let (_leaves_with_summaries, mut scored_ranges) = score_global_leaf_block_ranges(
        loaded_leaf_routes.iter().map(|loaded_route| {
            (
                loaded_route.route.leaf_pid,
                loaded_route.route.route_score,
                &loaded_route.route.placement,
                loaded_route.leaf_object.summaries.as_slice(),
            )
        }),
        &scorer,
        snapshot.epoch_manifest().epoch,
        &mut observer,
    )?;
    sort_scored_leaf_block_ranges(&mut scored_ranges);
    let scored_block_count = u64::try_from(scored_ranges.len())
        .map_err(|_| "ec_spire scored leaf block count exceeds u64".to_owned())?;
    let cap_block_ip = cap_block_ip(&scored_ranges, max_global_blocks)?;
    let loaded_route_indexes = loaded_leaf_routes
        .iter()
        .enumerate()
        .map(|(index, loaded_route)| (loaded_route.route.leaf_pid, index))
        .collect::<HashMap<_, _>>();
    let mut target_ordinals_by_sequence = HashMap::<u64, Vec<u64>>::new();
    for (target_ordinal, target_local_sequence) in target_local_sequences.iter().enumerate() {
        let target_ordinal = u64::try_from(target_ordinal)
            .map_err(|_| "ec_spire target ordinal exceeds u64".to_owned())?;
        target_ordinals_by_sequence
            .entry(*target_local_sequence)
            .or_default()
            .push(target_ordinal);
    }

    let mut found_target_ordinals = HashSet::<u64>::new();
    let mut rows = Vec::new();
    for (rank_index, block) in scored_ranges.iter().enumerate() {
        let block_rank = u64::try_from(rank_index)
            .map_err(|_| "ec_spire leaf block rank exceeds u64".to_owned())?
            .checked_add(1)
            .ok_or_else(|| "ec_spire leaf block rank overflow".to_owned())?;
        let Some(loaded_route_index) = loaded_route_indexes.get(&block.leaf_pid) else {
            continue;
        };
        append_leaf_block_rank_target_hits(
            &mut rows,
            &mut found_target_ordinals,
            &target_ordinals_by_sequence,
            &loaded_leaf_routes[*loaded_route_index],
            block,
            block_rank,
            max_global_blocks,
            radius_weight,
            scored_block_count,
            cap_block_ip,
            &route_context_by_pid,
            scan_plan,
            snapshot.epoch_manifest().epoch,
        )?;
    }

    rows.extend(missing_leaf_block_rank_rows(
        snapshot,
        scan_plan,
        target_local_sequences,
        "not_found_in_routed_leaves",
        max_global_blocks,
        radius_weight,
        scored_block_count,
        cap_block_ip,
        &found_target_ordinals,
    ));
    rows.sort_by(|left, right| {
        left.target_ordinal
            .cmp(&right.target_ordinal)
            .then_with(|| {
                left.block_rank
                    .unwrap_or(u64::MAX)
                    .cmp(&right.block_rank.unwrap_or(u64::MAX))
            })
            .then_with(|| left.pid.unwrap_or(u64::MAX).cmp(&right.pid.unwrap_or(u64::MAX)))
            .then_with(|| {
                left.row_index
                    .unwrap_or(u32::MAX)
                    .cmp(&right.row_index.unwrap_or(u32::MAX))
            })
    });
    Ok(rows)
}

fn collect_validated_leaf_target_block_rank_snapshot(
    snapshot: &SpireValidatedEpochSnapshot<'_>,
    object_store: &impl SpireObjectReader,
    query: &SpireScanQuery,
    hierarchy: &SpireLoadedRoutingHierarchy,
    scan_plan: SpireSingleLevelScanPlan,
    top_graph_plan: SpireTopGraphOptionPlan,
    target_local_sequences: &[u64],
) -> Result<Vec<SpireLeafBlockRankSnapshotRow>, String> {
    let max_global_blocks = current_session_leaf_block_pruning_max_global_blocks();
    let max_global_blocks = if max_global_blocks <= 0 {
        0
    } else {
        u64::try_from(max_global_blocks)
            .map_err(|_| "ec_spire.leaf_block_pruning_max_global_blocks exceeds u64".to_owned())?
    };
    let radius_weight = current_session_leaf_block_pruning_summary_radius_weight();
    if target_local_sequences.is_empty() {
        return Ok(Vec::new());
    }
    if scan_plan.nprobe == 0 {
        return Ok(missing_leaf_block_rank_rows(
            snapshot,
            scan_plan,
            target_local_sequences,
            "nprobe_zero",
            max_global_blocks,
            radius_weight,
            0,
            None,
            &HashSet::new(),
        ));
    }

    let scorer = SpirePreparedAssignmentScorer::prepare(
        scan_plan.payload_format,
        query.values().len(),
        query.values(),
    )?;
    let leaf_assignment_counts = &hierarchy.leaf_assignment_counts_by_pid;
    let mut leaf_row_count =
        |route| leaf_route_assignment_count_from_loaded_hierarchy(leaf_assignment_counts, route);
    let leaf_routes = if top_graph_plan.enabled {
        let (_top_graph_pid, top_graph) = load_snapshot_top_graph_object(snapshot, object_store)?
            .ok_or_else(|| {
                "ec_spire scan leaf target block rank snapshot has no available top graph object"
                    .to_owned()
            })?;
        route_top_graph_object_to_leaf_routes_with_row_budget(
            &hierarchy.root_object,
            &hierarchy.internal_objects_by_pid,
            &top_graph,
            query.values(),
            top_graph_plan.search_list_size.unwrap_or(scan_plan.nprobe),
            scan_plan.nprobe,
            &scan_plan.recursive_nprobe_policy,
            scan_plan.recursive_route_budget,
            scan_plan.max_routed_candidate_rows,
            &mut leaf_row_count,
        )?
        .routes
    } else {
        route_recursive_routing_objects_to_leaf_routes_with_row_budget(
            &hierarchy.root_object,
            &hierarchy.internal_objects_by_pid,
            query.values(),
            &scan_plan.recursive_nprobe_policy,
            scan_plan.recursive_route_budget,
            scan_plan.max_routed_candidate_rows,
            &mut leaf_row_count,
        )?
        .routes
    };
    let route_context_by_pid = leaf_routes
        .iter()
        .enumerate()
        .map(|(index, route)| {
            let rank = u64::try_from(index)
                .map_err(|_| "ec_spire route rank exceeds u64".to_owned())?
                .checked_add(1)
                .ok_or_else(|| "ec_spire route rank overflow".to_owned())?;
            Ok((route.leaf_pid, (rank, route.route_score)))
        })
        .collect::<Result<HashMap<_, _>, String>>()?;

    let mut observer = SpireNoopRoutedScanObserver;
    let delta_routes = collect_snapshot_delta_object_routes(snapshot, object_store)?;
    let mut delta_routes_by_parent = HashMap::<u64, Vec<SpireDeltaObjectRoute>>::new();
    let route_groups =
        group_leaf_and_delta_reads_by_local_store(snapshot, leaf_routes, delta_routes, &mut observer)?;
    prefetch_store_object_read_groups(
        object_store,
        snapshot.epoch_manifest().epoch,
        &route_groups,
        &mut observer,
    )?;
    for route_group in &route_groups {
        for route in &route_group.delta_routes {
            delta_routes_by_parent
                .entry(route.parent_leaf_pid)
                .or_default()
                .push(*route);
        }
    }

    let mut loaded_leaf_routes = Vec::new();
    for route_group in route_groups {
        for route in route_group.leaf_routes {
            let leaf_delta_routes = delta_routes_by_parent
                .get(&route.leaf_pid)
                .map(Vec::as_slice)
                .unwrap_or(&[]);
            let loaded_delta_routes =
                load_delta_rows_for_routes(snapshot, object_store, leaf_delta_routes, &mut observer)?;
            let deleted_vec_ids =
                collect_delta_delete_vec_ids_for_loaded_routes(&loaded_delta_routes);
            let Some(leaf_object) =
                read_quantized_v2_leaf_object_for_route(snapshot, object_store, route, &mut observer)?
            else {
                continue;
            };
            loaded_leaf_routes.push(SpireLoadedQuantizedLeafRoute {
                route,
                leaf_object,
                selected_row_ranges: None,
                loaded_delta_routes,
                deleted_vec_ids,
            });
        }
    }

    let (_leaves_with_summaries, mut scored_ranges) = score_global_leaf_block_ranges(
        loaded_leaf_routes.iter().map(|loaded_route| {
            (
                loaded_route.route.leaf_pid,
                loaded_route.route.route_score,
                &loaded_route.route.placement,
                loaded_route.leaf_object.summaries.as_slice(),
            )
        }),
        &scorer,
        snapshot.epoch_manifest().epoch,
        &mut observer,
    )?;
    sort_scored_leaf_block_ranges(&mut scored_ranges);
    let scored_block_count = u64::try_from(scored_ranges.len())
        .map_err(|_| "ec_spire scored leaf block count exceeds u64".to_owned())?;
    let cap_block_ip = cap_block_ip(&scored_ranges, max_global_blocks)?;
    let loaded_route_indexes = loaded_leaf_routes
        .iter()
        .enumerate()
        .map(|(index, loaded_route)| (loaded_route.route.leaf_pid, index))
        .collect::<HashMap<_, _>>();
    let mut target_ordinals_by_sequence = HashMap::<u64, Vec<u64>>::new();
    for (target_ordinal, target_local_sequence) in target_local_sequences.iter().enumerate() {
        let target_ordinal = u64::try_from(target_ordinal)
            .map_err(|_| "ec_spire target ordinal exceeds u64".to_owned())?;
        target_ordinals_by_sequence
            .entry(*target_local_sequence)
            .or_default()
            .push(target_ordinal);
    }

    let target_hits = collect_loaded_leaf_target_block_hits(
        &loaded_leaf_routes,
        &target_ordinals_by_sequence,
    )?;
    let mut found_target_ordinals = HashSet::<u64>::new();
    let mut rows = Vec::new();
    for (rank_index, block) in scored_ranges.iter().enumerate() {
        let Some(loaded_route_index) = loaded_route_indexes.get(&block.leaf_pid) else {
            continue;
        };
        let block_rank = u64::try_from(rank_index)
            .map_err(|_| "ec_spire leaf block rank exceeds u64".to_owned())?
            .checked_add(1)
            .ok_or_else(|| "ec_spire leaf block rank overflow".to_owned())?;
        let loaded_route = &loaded_leaf_routes[*loaded_route_index];
        let route_context = route_context_by_pid.get(&loaded_route.route.leaf_pid).copied();
        let row_count = block
            .range
            .row_end
            .checked_sub(block.range.row_base)
            .ok_or_else(|| "ec_spire leaf block rank range is invalid".to_owned())?;
        let selected_by_global_cap = max_global_blocks == 0 || block_rank <= max_global_blocks;
        for hit in target_hits.iter().filter(|hit| {
            hit.leaf_pid == block.leaf_pid
                && hit.row_index >= block.range.row_base
                && hit.row_index < block.range.row_end
        }) {
            found_target_ordinals.insert(hit.target_ordinal);
            rows.push(SpireLeafBlockRankSnapshotRow {
                active_epoch: snapshot.epoch_manifest().epoch,
                effective_nprobe: scan_plan.nprobe,
                effective_nprobe_source: scan_plan.nprobe_source,
                effective_rerank_width: u64::try_from(scan_plan.rerank_width)
                    .map_err(|_| "ec_spire rerank width exceeds u64".to_owned())?,
                effective_rerank_width_source: scan_plan.rerank_width_source,
                target_ordinal: hit.target_ordinal,
                target_local_sequence: hit.target_local_sequence,
                status: "target_block_ranked",
                max_global_blocks,
                radius_weight,
                scored_block_count,
                block_rank: Some(block_rank),
                selected_by_global_cap: Some(selected_by_global_cap),
                pid: Some(loaded_route.route.leaf_pid),
                node_id: Some(loaded_route.route.placement.node_id),
                local_store_id: Some(loaded_route.route.placement.local_store_id),
                object_version: Some(loaded_route.route.object_version),
                row_index: Some(hit.row_index),
                row_base: Some(block.range.row_base),
                row_end: Some(block.range.row_end),
                row_count: Some(row_count),
                block_ip: Some(block.ip),
                cap_block_ip,
                block_ip_margin_to_cap: cap_block_ip.map(|cap_ip| block.ip - cap_ip),
                route_rank: route_context.map(|(rank, _score)| rank),
                route_score: route_context.map(|(_rank, score)| score),
                assignment_flags: Some(hit.assignment_flags),
            });
        }
    }

    rows.extend(missing_leaf_block_rank_rows(
        snapshot,
        scan_plan,
        target_local_sequences,
        "not_found_in_routed_leaves",
        max_global_blocks,
        radius_weight,
        scored_block_count,
        cap_block_ip,
        &found_target_ordinals,
    ));
    rows.sort_by(|left, right| {
        left.target_ordinal
            .cmp(&right.target_ordinal)
            .then_with(|| {
                left.block_rank
                    .unwrap_or(u64::MAX)
                    .cmp(&right.block_rank.unwrap_or(u64::MAX))
            })
            .then_with(|| left.pid.unwrap_or(u64::MAX).cmp(&right.pid.unwrap_or(u64::MAX)))
            .then_with(|| {
                left.row_index
                    .unwrap_or(u32::MAX)
                    .cmp(&right.row_index.unwrap_or(u32::MAX))
            })
    });
    Ok(rows)
}

#[derive(Debug, Clone, PartialEq)]
struct SpireTargetBlockHit {
    target_ordinal: u64,
    target_local_sequence: u64,
    leaf_pid: u64,
    row_index: u32,
    assignment_flags: u16,
}

fn collect_loaded_leaf_target_block_hits(
    loaded_leaf_routes: &[SpireLoadedQuantizedLeafRoute],
    target_ordinals_by_sequence: &HashMap<u64, Vec<u64>>,
) -> Result<Vec<SpireTargetBlockHit>, String> {
    let mut hits = Vec::new();
    for loaded_route in loaded_leaf_routes {
        for columns in loaded_route.leaf_object.column_segments()? {
            let columns = columns?;
            for row_offset in 0..columns.row_count() {
                if !is_visible_scored_assignment_flags(columns.flags[row_offset]) {
                    continue;
                }
                let row = columns.row(row_offset)?;
                let vec_id = row.vec_id()?;
                if loaded_route.deleted_vec_ids.contains(&vec_id) {
                    continue;
                }
                let Some(local_sequence) = vec_id.local_sequence() else {
                    continue;
                };
                let Some(target_ordinals) = target_ordinals_by_sequence.get(&local_sequence) else {
                    continue;
                };
                for target_ordinal in target_ordinals {
                    hits.push(SpireTargetBlockHit {
                        target_ordinal: *target_ordinal,
                        target_local_sequence: local_sequence,
                        leaf_pid: loaded_route.route.leaf_pid,
                        row_index: row.row_index,
                        assignment_flags: row.flags,
                    });
                }
            }
        }
    }
    Ok(hits)
}

fn append_leaf_block_rank_target_hits(
    rows: &mut Vec<SpireLeafBlockRankSnapshotRow>,
    found_target_ordinals: &mut HashSet<u64>,
    target_ordinals_by_sequence: &HashMap<u64, Vec<u64>>,
    loaded_route: &SpireLoadedQuantizedLeafRoute,
    block: &SpireScoredLeafBlockRange,
    block_rank: u64,
    max_global_blocks: u64,
    radius_weight: f64,
    scored_block_count: u64,
    cap_block_ip: Option<f32>,
    route_context_by_pid: &HashMap<u64, (u64, f32)>,
    scan_plan: SpireSingleLevelScanPlan,
    epoch: u64,
) -> Result<(), String> {
    let row_count = block
        .range
        .row_end
        .checked_sub(block.range.row_base)
        .ok_or_else(|| "ec_spire leaf block rank range is invalid".to_owned())?;
    let selected_by_global_cap = max_global_blocks == 0 || block_rank <= max_global_blocks;
    let route_context = route_context_by_pid.get(&loaded_route.route.leaf_pid).copied();
    for columns in loaded_route.leaf_object.column_segments()? {
        let columns = columns?;
        let column_row_count = u32::try_from(columns.row_count())
            .map_err(|_| "ec_spire leaf V2 column row count exceeds u32".to_owned())?;
        let column_row_end = columns
            .row_base
            .checked_add(column_row_count)
            .ok_or_else(|| "ec_spire leaf V2 column row range overflow".to_owned())?;
        let row_base = block.range.row_base.max(columns.row_base);
        let row_end = block.range.row_end.min(column_row_end);
        if row_base >= row_end {
            continue;
        }
        for row_index in row_base..row_end {
            let row_offset = usize::try_from(row_index - columns.row_base)
                .map_err(|_| "ec_spire leaf V2 target row offset exceeds usize".to_owned())?;
            if !is_visible_scored_assignment_flags(columns.flags[row_offset]) {
                continue;
            }
            let row = columns.row(row_offset)?;
            let vec_id = row.vec_id()?;
            if loaded_route.deleted_vec_ids.contains(&vec_id) {
                continue;
            }
            let Some(local_sequence) = vec_id.local_sequence() else {
                continue;
            };
            let Some(target_ordinals) = target_ordinals_by_sequence.get(&local_sequence) else {
                continue;
            };
            for target_ordinal in target_ordinals {
                found_target_ordinals.insert(*target_ordinal);
                rows.push(SpireLeafBlockRankSnapshotRow {
                    active_epoch: epoch,
                    effective_nprobe: scan_plan.nprobe,
                    effective_nprobe_source: scan_plan.nprobe_source,
                    effective_rerank_width: u64::try_from(scan_plan.rerank_width)
                        .map_err(|_| "ec_spire rerank width exceeds u64".to_owned())?,
                    effective_rerank_width_source: scan_plan.rerank_width_source,
                    target_ordinal: *target_ordinal,
                    target_local_sequence: local_sequence,
                    status: "block_ranked",
                    max_global_blocks,
                    radius_weight,
                    scored_block_count,
                    block_rank: Some(block_rank),
                    selected_by_global_cap: Some(selected_by_global_cap),
                    pid: Some(loaded_route.route.leaf_pid),
                    node_id: Some(loaded_route.route.placement.node_id),
                    local_store_id: Some(loaded_route.route.placement.local_store_id),
                    object_version: Some(loaded_route.route.object_version),
                    row_index: Some(row.row_index),
                    row_base: Some(block.range.row_base),
                    row_end: Some(block.range.row_end),
                    row_count: Some(row_count),
                    block_ip: Some(block.ip),
                    cap_block_ip,
                    block_ip_margin_to_cap: cap_block_ip.map(|cap_ip| block.ip - cap_ip),
                    route_rank: route_context.map(|(rank, _score)| rank),
                    route_score: route_context.map(|(_rank, score)| score),
                    assignment_flags: Some(row.flags),
                });
            }
        }
    }
    Ok(())
}

fn missing_leaf_block_rank_rows(
    snapshot: &SpireValidatedEpochSnapshot<'_>,
    scan_plan: SpireSingleLevelScanPlan,
    target_local_sequences: &[u64],
    status: &'static str,
    max_global_blocks: u64,
    radius_weight: f64,
    scored_block_count: u64,
    cap_block_ip: Option<f32>,
    found_target_ordinals: &HashSet<u64>,
) -> Vec<SpireLeafBlockRankSnapshotRow> {
    target_local_sequences
        .iter()
        .enumerate()
        .filter_map(|(target_ordinal, target_local_sequence)| {
            let target_ordinal = u64::try_from(target_ordinal).ok()?;
            if found_target_ordinals.contains(&target_ordinal) {
                return None;
            }
            Some(SpireLeafBlockRankSnapshotRow {
                active_epoch: snapshot.epoch_manifest().epoch,
                effective_nprobe: scan_plan.nprobe,
                effective_nprobe_source: scan_plan.nprobe_source,
                effective_rerank_width: u64::try_from(scan_plan.rerank_width).ok()?,
                effective_rerank_width_source: scan_plan.rerank_width_source,
                target_ordinal,
                target_local_sequence: *target_local_sequence,
                status,
                max_global_blocks,
                radius_weight,
                scored_block_count,
                block_rank: None,
                selected_by_global_cap: None,
                pid: None,
                node_id: None,
                local_store_id: None,
                object_version: None,
                row_index: None,
                row_base: None,
                row_end: None,
                row_count: None,
                block_ip: None,
                cap_block_ip,
                block_ip_margin_to_cap: None,
                route_rank: None,
                route_score: None,
                assignment_flags: None,
            })
        })
        .collect()
}

fn cap_block_ip(
    scored_ranges: &[SpireScoredLeafBlockRange],
    max_global_blocks: u64,
) -> Result<Option<f32>, String> {
    if max_global_blocks == 0 {
        return Ok(None);
    }
    let cap_index = usize::try_from(max_global_blocks - 1)
        .map_err(|_| "ec_spire max global blocks exceeds usize".to_owned())?;
    Ok(scored_ranges.get(cap_index).map(|block| block.ip))
}

pub(super) fn rerank_scored_candidates_by_ip<F>(
    candidates: &mut Vec<SpireScoredScanCandidate>,
    rerank_width: usize,
    exact_score_ip: F,
) -> Result<(), String>
where
    F: FnMut(&SpireScoredScanCandidate) -> Result<Option<f32>, String>,
{
    rerank_scored_candidates_by_ip_with_prefetch(candidates, rerank_width, |_| Ok(()), exact_score_ip)
}

fn rerank_scored_candidates_by_ip_with_prefetch<F, P>(
    candidates: &mut Vec<SpireScoredScanCandidate>,
    rerank_width: usize,
    mut prefetch_candidates: P,
    mut exact_score_ip: F,
) -> Result<(), String>
where
    F: FnMut(&SpireScoredScanCandidate) -> Result<Option<f32>, String>,
    P: FnMut(&[SpireScoredScanCandidate]) -> Result<(), String>,
{
    let rerank_len = if rerank_width == 0 {
        candidates.len()
    } else {
        rerank_width.min(candidates.len())
    };

    if rerank_len > 0 {
        prefetch_candidates(&candidates[..rerank_len])?;
    }

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

fn select_leaf_block_row_ranges(
    summaries: &[SpireLeafBlockSummary],
    max_blocks_per_leaf: i32,
    scorer: &SpirePreparedAssignmentScorer,
    epoch: u64,
    placement: &SpirePlacementEntry,
    observer: &mut impl SpireRoutedScanObserver,
) -> Result<Option<Vec<SpireLeafBlockRowRange>>, String> {
    if max_blocks_per_leaf <= 0 || summaries.is_empty() {
        if !summaries.is_empty() {
            observer.leaf_block_selection(epoch, placement, summaries.len(), summaries.len());
        }
        return Ok(None);
    }
    if scorer.payload_format() != SpireAssignmentPayloadFormat::RaBitQ {
        observer.leaf_block_selection(epoch, placement, summaries.len(), summaries.len());
        return Ok(None);
    }
    let max_blocks = usize::try_from(max_blocks_per_leaf)
        .map_err(|_| "ec_spire.leaf_block_pruning_max_blocks_per_leaf exceeds usize".to_owned())?;
    if max_blocks >= summaries.len() {
        observer.leaf_block_selection(epoch, placement, summaries.len(), summaries.len());
        return Ok(None);
    }

    let score_context = SpireLeafBlockSummaryScoreContext::new(
        scorer,
        current_session_leaf_block_pruning_summary_radius_weight(),
    )?;
    let score_started = observer.wants_candidate_timing().then(Instant::now);
    let mut scored_ranges = Vec::with_capacity(summaries.len());
    for summary in summaries {
        let row_end = summary
            .row_base
            .checked_add(summary.row_count)
            .ok_or_else(|| "ec_spire leaf V3 summary row range overflow".to_owned())?;
        let ip = score_leaf_block_summary_ip_with_context(summary, scorer, score_context)?;
        scored_ranges.push((
            ip,
            SpireLeafBlockRowRange {
                row_base: summary.row_base,
                row_end,
            },
        ));
    }
    if let Some(started) = score_started {
        observer.leaf_summary_score_time(epoch, placement, elapsed_nanos_since(started));
    }

    scored_ranges.sort_by(|left, right| {
        right
            .0
            .partial_cmp(&left.0)
            .unwrap_or(Ordering::Equal)
            .then_with(|| left.1.row_base.cmp(&right.1.row_base))
    });
    scored_ranges.truncate(max_blocks);
    let mut ranges = scored_ranges
        .into_iter()
        .map(|(_score, range)| range)
        .collect::<Vec<_>>();
    ranges.sort_by_key(|range| range.row_base);
    observer.leaf_block_selection(epoch, placement, summaries.len(), ranges.len());
    Ok(Some(ranges))
}

fn select_global_leaf_block_row_ranges<'a, I>(
    leaves: I,
    max_global_blocks: i32,
    scorer: &SpirePreparedAssignmentScorer,
    epoch: u64,
    observer: &mut impl SpireRoutedScanObserver,
) -> Result<Option<HashMap<u64, Vec<SpireLeafBlockRowRange>>>, String>
where
    I: IntoIterator<Item = (u64, f32, &'a SpirePlacementEntry, &'a [SpireLeafBlockSummary])>,
{
    if max_global_blocks <= 0 || scorer.payload_format() != SpireAssignmentPayloadFormat::RaBitQ {
        return Ok(None);
    }

    let (leaves_with_summaries, scored_ranges) =
        score_global_leaf_block_ranges(leaves, scorer, epoch, observer)?;
    if scored_ranges.is_empty() {
        return Ok(None);
    }

    let max_blocks = usize::try_from(max_global_blocks)
        .map_err(|_| "ec_spire.leaf_block_pruning_max_global_blocks exceeds usize".to_owned())?;
    if max_blocks >= scored_ranges.len() {
        return Ok(None);
    }

    Ok(Some(select_scored_global_leaf_block_row_ranges(
        leaves_with_summaries,
        scored_ranges,
        max_blocks,
    )))
}

fn select_sampled_global_leaf_block_row_ranges(
    snapshot: &SpireValidatedEpochSnapshot<'_>,
    loaded_leaf_routes: &[SpireLoadedQuantizedLeafRoute],
    max_global_blocks: i32,
    global_probe_blocks: i32,
    sample_rows_per_block: i32,
    sample_summary_prior_weight: f64,
    scorer: &SpirePreparedAssignmentScorer,
    accumulator: &mut SpireScoredCandidateAccumulator,
    observer: &mut impl SpireRoutedScanObserver,
) -> Result<Option<HashMap<u64, Vec<SpireLeafBlockRowRange>>>, String> {
    if max_global_blocks <= 0 || scorer.payload_format() != SpireAssignmentPayloadFormat::RaBitQ {
        return Ok(None);
    }

    let epoch = snapshot.epoch_manifest().epoch;
    let (leaves_with_summaries, mut scored_ranges) = score_global_leaf_block_ranges(
        loaded_leaf_routes.iter().map(|loaded_route| {
            (
                loaded_route.route.leaf_pid,
                loaded_route.route.route_score,
                &loaded_route.route.placement,
                loaded_route.leaf_object.summaries.as_slice(),
            )
        }),
        scorer,
        epoch,
        observer,
    )?;
    if scored_ranges.is_empty() {
        return Ok(None);
    }

    let max_blocks = usize::try_from(max_global_blocks)
        .map_err(|_| "ec_spire.leaf_block_pruning_max_global_blocks exceeds usize".to_owned())?;
    if max_blocks >= scored_ranges.len() {
        return Ok(None);
    }

    let probe_blocks = usize::try_from(global_probe_blocks).map_err(|_| {
        "ec_spire.leaf_block_pruning_global_probe_blocks exceeds usize".to_owned()
    })?;
    let sample_rows = usize::try_from(sample_rows_per_block).map_err(|_| {
        "ec_spire.leaf_block_pruning_sample_rows_per_block exceeds usize".to_owned()
    })?;
    if probe_blocks <= max_blocks || sample_rows == 0 {
        return Ok(Some(select_scored_global_leaf_block_row_ranges(
            leaves_with_summaries,
            scored_ranges,
            max_blocks,
        )));
    }
    if !sample_summary_prior_weight.is_finite()
        || !(0.0..=1.0).contains(&sample_summary_prior_weight)
    {
        return Err(
            "ec_spire.leaf_block_pruning_sample_summary_prior_weight must be finite in [0, 1]"
                .to_owned(),
        );
    }

    sort_scored_leaf_block_ranges(&mut scored_ranges);
    scored_ranges.truncate(probe_blocks.min(scored_ranges.len()));
    let sampled_ranges = sample_global_leaf_block_probe_candidates(
        snapshot,
        loaded_leaf_routes,
        &scored_ranges,
        sample_rows,
        sample_summary_prior_weight,
        scorer,
        accumulator,
        observer,
    )?;

    Ok(Some(select_scored_global_leaf_block_row_ranges(
        leaves_with_summaries,
        sampled_ranges,
        max_blocks,
    )))
}

#[derive(Debug, Clone, PartialEq)]
struct SpireScoredLeafBlockRange {
    ip: f32,
    leaf_pid: u64,
    range: SpireLeafBlockRowRange,
}

#[derive(Debug, Clone, Copy)]
struct SpireLeafBlockSummaryScoreContext {
    payload_format: SpireAssignmentPayloadFormat,
    payload_format_tag: u8,
    payload_stride: usize,
    radius_bonus_scale: f32,
}

impl SpireLeafBlockSummaryScoreContext {
    fn new(
        scorer: &SpirePreparedAssignmentScorer,
        radius_weight: f64,
    ) -> Result<Self, String> {
        if !radius_weight.is_finite() || !(0.0..=1.0).contains(&radius_weight) {
            return Err(
                "ec_spire.leaf_block_pruning_summary_radius_weight must be finite in [0, 1]"
                    .to_owned(),
            );
        }
        let payload_format = scorer.payload_format();
        let radius_bonus_scale = if payload_format == SpireAssignmentPayloadFormat::RaBitQ {
            scorer.query_l2_norm() * radius_weight as f32
        } else {
            0.0
        };
        Ok(Self {
            payload_format,
            payload_format_tag: payload_format.tag(),
            payload_stride: scorer.payload_stride()?,
            radius_bonus_scale,
        })
    }
}

fn score_global_leaf_block_ranges<'a, I>(
    leaves: I,
    scorer: &SpirePreparedAssignmentScorer,
    epoch: u64,
    observer: &mut impl SpireRoutedScanObserver,
) -> Result<(Vec<u64>, Vec<SpireScoredLeafBlockRange>), String>
where
    I: IntoIterator<Item = (u64, f32, &'a SpirePlacementEntry, &'a [SpireLeafBlockSummary])>,
{
    score_global_leaf_block_ranges_with_route_prior_weight(
        leaves,
        scorer,
        epoch,
        observer,
        current_session_leaf_block_pruning_route_prior_weight(),
    )
}

fn score_global_leaf_block_ranges_with_route_prior_weight<'a, I>(
    leaves: I,
    scorer: &SpirePreparedAssignmentScorer,
    epoch: u64,
    observer: &mut impl SpireRoutedScanObserver,
    route_prior_weight: f64,
) -> Result<(Vec<u64>, Vec<SpireScoredLeafBlockRange>), String>
where
    I: IntoIterator<Item = (u64, f32, &'a SpirePlacementEntry, &'a [SpireLeafBlockSummary])>,
{
    if !route_prior_weight.is_finite() || !(0.0..=1.0).contains(&route_prior_weight) {
        return Err(
            "ec_spire.leaf_block_pruning_route_prior_weight must be finite in [0, 1]".to_owned(),
        );
    }
    let route_prior_weight = route_prior_weight as f32;
    let leaves = leaves.into_iter().collect::<Vec<_>>();
    let mut leaves_with_summaries = Vec::new();
    let mut scored_ranges = Vec::with_capacity(
        leaves
            .iter()
            .map(|(_leaf_pid, _route_score, _placement, summaries)| summaries.len())
            .sum(),
    );
    let score_context = SpireLeafBlockSummaryScoreContext::new(
        scorer,
        current_session_leaf_block_pruning_summary_radius_weight(),
    )?;
    for (leaf_pid, route_score, placement, summaries) in leaves {
        if !summaries.is_empty() {
            leaves_with_summaries.push(leaf_pid);
        }
        let score_started = observer.wants_candidate_timing().then(Instant::now);
        for summary in summaries {
            let row_end = summary
                .row_base
                .checked_add(summary.row_count)
                .ok_or_else(|| "ec_spire leaf V3 summary row range overflow".to_owned())?;
            let ip = score_leaf_block_summary_ip_with_context(summary, scorer, score_context)?;
            let ip = if route_prior_weight > 0.0 {
                if !route_score.is_finite() {
                    return Err(
                        "ec_spire routed leaf score prior must be finite when route prior is enabled"
                            .to_owned(),
                    );
                }
                ip + route_prior_weight * route_score
            } else {
                ip
            };
            if !ip.is_finite() {
                return Err("ec_spire leaf V3 global block score is non-finite".to_owned());
            }
            scored_ranges.push(SpireScoredLeafBlockRange {
                ip,
                leaf_pid,
                range: SpireLeafBlockRowRange {
                    row_base: summary.row_base,
                    row_end,
                },
            });
        }
        if let Some(started) = score_started {
            observer.leaf_summary_score_time(epoch, placement, elapsed_nanos_since(started));
        }
    }

    Ok((leaves_with_summaries, scored_ranges))
}

fn score_leaf_block_summary_ip(
    summary: &SpireLeafBlockSummary,
    scorer: &SpirePreparedAssignmentScorer,
) -> Result<f32, String> {
    score_leaf_block_summary_ip_with_radius_weight(
        summary,
        scorer,
        current_session_leaf_block_pruning_summary_radius_weight(),
    )
}

fn score_leaf_block_summary_ip_with_radius_weight(
    summary: &SpireLeafBlockSummary,
    scorer: &SpirePreparedAssignmentScorer,
    radius_weight: f64,
) -> Result<f32, String> {
    let score_context = SpireLeafBlockSummaryScoreContext::new(scorer, radius_weight)?;
    score_leaf_block_summary_ip_with_context(summary, scorer, score_context)
}

fn score_leaf_block_summary_ip_with_context(
    summary: &SpireLeafBlockSummary,
    scorer: &SpirePreparedAssignmentScorer,
    score_context: SpireLeafBlockSummaryScoreContext,
) -> Result<f32, String> {
    if summary.payload_format != score_context.payload_format_tag {
        let summary_format = SpireAssignmentPayloadFormat::from_tag(summary.payload_format)?;
        return Err(format!(
            "ec_spire leaf V3 summary payload format {:?} does not match prepared scorer {:?}",
            summary_format, score_context.payload_format
        ));
    }
    if score_context.payload_format == SpireAssignmentPayloadFormat::RaBitQ && summary.gamma < 0.0
    {
        return Err("ec_spire leaf V3 RaBitQ summary radius must be non-negative".to_owned());
    }
    if summary.encoded_payload.is_empty()
        || summary.encoded_payload.len() % score_context.payload_stride != 0
    {
        return Err(format!(
            "ec_spire leaf V3 summary payload length {} is not a positive multiple of scorer stride {payload_stride}",
            summary.encoded_payload.len(),
            payload_stride = score_context.payload_stride
        ));
    }
    let representative_ip = scorer.score_zero_gamma_payload_chunks_max_prevalidated(
        score_context.payload_stride,
        &summary.encoded_payload,
    );
    let ip = if score_context.payload_format == SpireAssignmentPayloadFormat::RaBitQ {
        representative_ip + score_context.radius_bonus_scale * summary.gamma
    } else {
        representative_ip
    };
    if !ip.is_finite() {
        return Err("ec_spire leaf V3 summary scorer returned a non-finite score".to_owned());
    }
    Ok(ip)
}

fn select_scored_global_leaf_block_row_ranges(
    leaves_with_summaries: Vec<u64>,
    mut scored_ranges: Vec<SpireScoredLeafBlockRange>,
    max_blocks: usize,
) -> HashMap<u64, Vec<SpireLeafBlockRowRange>> {
    let mut selected_by_leaf = leaves_with_summaries
        .into_iter()
        .map(|leaf_pid| (leaf_pid, Vec::new()))
        .collect::<HashMap<_, _>>();

    sort_scored_leaf_block_ranges(&mut scored_ranges);
    scored_ranges.truncate(max_blocks);
    for block in scored_ranges {
        selected_by_leaf
            .entry(block.leaf_pid)
            .or_default()
            .push(block.range);
    }
    for ranges in selected_by_leaf.values_mut() {
        ranges.sort_by_key(|range| range.row_base);
    }

    selected_by_leaf
}

fn sort_scored_leaf_block_ranges(scored_ranges: &mut [SpireScoredLeafBlockRange]) {
    scored_ranges.sort_by(|left, right| {
        right
            .ip
            .partial_cmp(&left.ip)
            .unwrap_or(Ordering::Equal)
            .then_with(|| left.leaf_pid.cmp(&right.leaf_pid))
            .then_with(|| left.range.row_base.cmp(&right.range.row_base))
    });
}

#[derive(Debug, Clone)]
struct SpireLeafBlockSamplePlan {
    block: SpireScoredLeafBlockRange,
    sample_row_indices: Vec<u32>,
    best_sample_ip: Option<f32>,
}

fn sample_global_leaf_block_probe_candidates(
    snapshot: &SpireValidatedEpochSnapshot<'_>,
    loaded_leaf_routes: &[SpireLoadedQuantizedLeafRoute],
    probe_blocks: &[SpireScoredLeafBlockRange],
    sample_rows_per_block: usize,
    sample_summary_prior_weight: f64,
    scorer: &SpirePreparedAssignmentScorer,
    accumulator: &mut SpireScoredCandidateAccumulator,
    observer: &mut impl SpireRoutedScanObserver,
) -> Result<Vec<SpireScoredLeafBlockRange>, String> {
    let mut probe_blocks_by_leaf = HashMap::<u64, Vec<SpireScoredLeafBlockRange>>::new();
    for block in probe_blocks {
        probe_blocks_by_leaf
            .entry(block.leaf_pid)
            .or_default()
            .push(block.clone());
    }

    let mut sampled_blocks = Vec::with_capacity(probe_blocks.len());
    for loaded_route in loaded_leaf_routes {
        let Some(blocks) = probe_blocks_by_leaf.remove(&loaded_route.route.leaf_pid) else {
            continue;
        };
        sampled_blocks.extend(sample_leaf_block_probe_candidates(
            snapshot,
            loaded_route,
            blocks,
            sample_rows_per_block,
            sample_summary_prior_weight,
            scorer,
            accumulator,
            observer,
        )?);
    }
    for blocks in probe_blocks_by_leaf.into_values() {
        sampled_blocks.extend(blocks);
    }
    Ok(sampled_blocks)
}

fn sample_leaf_block_probe_candidates(
    snapshot: &SpireValidatedEpochSnapshot<'_>,
    loaded_route: &SpireLoadedQuantizedLeafRoute,
    blocks: Vec<SpireScoredLeafBlockRange>,
    sample_rows_per_block: usize,
    sample_summary_prior_weight: f64,
    scorer: &SpirePreparedAssignmentScorer,
    accumulator: &mut SpireScoredCandidateAccumulator,
    observer: &mut impl SpireRoutedScanObserver,
) -> Result<Vec<SpireScoredLeafBlockRange>, String> {
    let mut plans = blocks
        .into_iter()
        .map(|block| {
            Ok(SpireLeafBlockSamplePlan {
                sample_row_indices: sample_leaf_block_row_indices(block.range, sample_rows_per_block)?,
                block,
                best_sample_ip: None,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let epoch = snapshot.epoch_manifest().epoch;
    let route = loaded_route.route;
    let placement = &route.placement;

    for columns in loaded_route.leaf_object.column_segments()? {
        let columns = columns?;
        let column_format = SpireAssignmentPayloadFormat::from_tag(columns.payload_format)?;
        if column_format != scorer.payload_format() {
            return Err(format!(
                "ec_spire leaf V2 payload format {:?} does not match prepared scorer {:?}",
                column_format,
                scorer.payload_format()
            ));
        }
        let column_row_count = u32::try_from(columns.row_count())
            .map_err(|_| "ec_spire leaf V2 column row count exceeds u32".to_owned())?;
        let column_row_end = columns
            .row_base
            .checked_add(column_row_count)
            .ok_or_else(|| "ec_spire leaf V2 column row range overflow".to_owned())?;
        for plan in &mut plans {
            for &row_index in &plan.sample_row_indices {
                if row_index < columns.row_base || row_index >= column_row_end {
                    continue;
                }
                let row_offset = usize::try_from(row_index - columns.row_base)
                    .map_err(|_| "ec_spire leaf V2 sampled row offset exceeds usize".to_owned())?;
                if !is_visible_scored_assignment_flags(columns.flags[row_offset]) {
                    continue;
                }

                let materialize_started = observer.wants_candidate_timing().then(Instant::now);
                let row = columns.row(row_offset)?;
                let vec_id = row.vec_id()?;
                if let Some(started) = materialize_started {
                    observer.candidate_materialize_time(epoch, placement, elapsed_nanos_since(started));
                }
                if loaded_route.deleted_vec_ids.contains(&vec_id) {
                    continue;
                }

                let score_started = observer.wants_candidate_timing().then(Instant::now);
                let ip = scorer.score_payload_ip(column_format, row.gamma, row.encoded_payload)?;
                if let Some(started) = score_started {
                    observer.leaf_row_score_time(epoch, placement, elapsed_nanos_since(started));
                }
                if !ip.is_finite() {
                    return Err(
                        "ec_spire sampled leaf block scorer returned a non-finite score"
                            .to_owned(),
                    );
                }
                plan.best_sample_ip = Some(plan.best_sample_ip.map_or(ip, |best| best.max(ip)));
                observer.visible_leaf_candidate(epoch, placement, row.flags);
                let candidate = SpireScoredScanCandidate {
                    epoch,
                    pid: route.leaf_pid,
                    object_version: route.object_version,
                    row_index: row.row_index,
                    assignment_flags: row.flags,
                    vec_id,
                    heap_tid: row.heap_tid,
                    score: -ip,
                };
                let append_started = observer.wants_candidate_timing().then(Instant::now);
                let outcome = accumulator.append(candidate);
                if let Some(started) = append_started {
                    observer.candidate_heap_append_time(epoch, placement, elapsed_nanos_since(started));
                }
                observe_candidate_append_outcome(snapshot, observer, outcome)?;
            }
        }
    }

    Ok(plans
        .into_iter()
        .map(|plan| SpireScoredLeafBlockRange {
            ip: blend_leaf_block_summary_sample_score(
                plan.block.ip,
                plan.best_sample_ip,
                sample_summary_prior_weight,
            ),
            leaf_pid: plan.block.leaf_pid,
            range: plan.block.range,
        })
        .collect())
}

fn blend_leaf_block_summary_sample_score(
    summary_ip: f32,
    best_sample_ip: Option<f32>,
    sample_summary_prior_weight: f64,
) -> f32 {
    let Some(sample_ip) = best_sample_ip else {
        return summary_ip;
    };
    if sample_ip <= summary_ip {
        return summary_ip;
    }
    let sample_weight = (1.0 - sample_summary_prior_weight) as f32;
    summary_ip + (sample_ip - summary_ip) * sample_weight
}

fn sample_leaf_block_row_indices(
    range: SpireLeafBlockRowRange,
    max_sample_rows: usize,
) -> Result<Vec<u32>, String> {
    if max_sample_rows == 0 {
        return Ok(Vec::new());
    }
    let row_count = range
        .row_end
        .checked_sub(range.row_base)
        .ok_or_else(|| "ec_spire leaf block sample range is invalid".to_owned())?;
    if row_count == 0 {
        return Ok(Vec::new());
    }
    let row_count = usize::try_from(row_count)
        .map_err(|_| "ec_spire leaf block sample row count exceeds usize".to_owned())?;
    let sample_count = max_sample_rows.min(row_count);
    let mut indices = Vec::with_capacity(sample_count);
    for sample_index in 0..sample_count {
        let offset = ((sample_index * 2 + 1) * row_count) / (sample_count * 2);
        let offset = offset.min(row_count - 1);
        let offset = u32::try_from(offset)
            .map_err(|_| "ec_spire leaf block sample row offset exceeds u32".to_owned())?;
        indices.push(
            range
                .row_base
                .checked_add(offset)
                .ok_or_else(|| "ec_spire leaf block sample row index overflow".to_owned())?,
        );
    }
    indices.dedup();
    Ok(indices)
}

fn leaf_column_row_is_selected(
    columns: &SpireLeafObjectColumns<'_>,
    row_offset: usize,
    selected_row_ranges: Option<&[SpireLeafBlockRowRange]>,
) -> Result<bool, String> {
    let Some(selected_row_ranges) = selected_row_ranges else {
        return Ok(true);
    };
    let row_offset = u32::try_from(row_offset)
        .map_err(|_| "ec_spire leaf V2 column row offset exceeds u32".to_owned())?;
    let row_index = columns
        .row_base
        .checked_add(row_offset)
        .ok_or_else(|| "ec_spire leaf V2 column row index overflow".to_owned())?;
    Ok(selected_row_ranges
        .iter()
        .any(|range| row_index >= range.row_base && row_index < range.row_end))
}

fn read_quantized_v2_leaf_object_for_route(
    snapshot: &SpireValidatedEpochSnapshot<'_>,
    object_store: &impl SpireObjectReader,
    route: SpireLeafObjectReadRoute,
    observer: &mut impl SpireRoutedScanObserver,
) -> Result<Option<SpireLeafPartitionObjectV2>, String> {
    let leaf_pid = route.leaf_pid;
    let placement = &route.placement;
    if should_skip_placement(snapshot.epoch_manifest().consistency_mode, placement.state)? {
        return Ok(None);
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

    let epoch = snapshot.epoch_manifest().epoch;
    observer.scanned_leaf(epoch, &route.placement);
    let read_started = observer.wants_candidate_timing().then(Instant::now);
    let leaf_object = object_store.read_leaf_object_v2(placement).map_err(|error| {
        format!(
            "ec_spire global leaf block pruning requires routed leaf pid {leaf_pid} to be readable as V2: {error}"
        )
    })?;
    if let Some(started) = read_started {
        observer.leaf_object_read_time(epoch, placement, elapsed_nanos_since(started));
    }
    observer.leaf_block_storage(
        epoch,
        placement,
        leaf_object.meta.summary_bytes_total,
        leaf_object.meta.row_segment_bytes_total()?,
    );

    Ok(Some(leaf_object))
}

fn read_quantized_v2_leaf_summaries_for_route(
    snapshot: &SpireValidatedEpochSnapshot<'_>,
    object_store: &impl SpireObjectReader,
    route: SpireLeafObjectReadRoute,
    observer: &mut impl SpireRoutedScanObserver,
) -> Result<Option<SpireLeafPartitionObjectV2Summaries>, String> {
    let leaf_pid = route.leaf_pid;
    let placement = &route.placement;
    if should_skip_placement(snapshot.epoch_manifest().consistency_mode, placement.state)? {
        return Ok(None);
    }

    let epoch = snapshot.epoch_manifest().epoch;
    observer.scanned_leaf(epoch, &route.placement);
    let read_started = observer.wants_candidate_timing().then(Instant::now);
    let leaf_summaries = object_store
        .read_leaf_object_v2_summaries(placement)
        .map_err(|error| {
            format!(
                "ec_spire global leaf block pruning requires routed leaf pid {leaf_pid} summaries to be readable as V2/V3/V4: {error}"
            )
        })?;
    if let Some(started) = read_started {
        observer.leaf_object_read_time(epoch, placement, elapsed_nanos_since(started));
    }
    observer.leaf_block_storage(
        epoch,
        placement,
        leaf_summaries.meta.summary_bytes_total,
        leaf_summaries.meta.row_segment_bytes_total()?,
    );
    if leaf_summaries.meta.header.parent_pid != route.parent_pid {
        return Err(format!(
            "ec_spire quantized routed scan leaf pid {leaf_pid} parent {} does not match expected parent pid {}",
            leaf_summaries.meta.header.parent_pid,
            route.parent_pid,
        ));
    }

    Ok(Some(leaf_summaries))
}

fn append_quantized_v2_leaf_summary_route_candidates(
    snapshot: &SpireValidatedEpochSnapshot<'_>,
    object_store: &impl SpireObjectReader,
    leaf_summaries: &SpireLeafPartitionObjectV2Summaries,
    route: SpireLeafObjectReadRoute,
    scorer: &SpirePreparedAssignmentScorer,
    deleted_vec_ids: &HashSet<SpireVecId>,
    accumulator: &mut SpireScoredCandidateAccumulator,
    selected_row_ranges: Option<&[SpireLeafBlockRowRange]>,
    observer: &mut impl SpireRoutedScanObserver,
) -> Result<(), String> {
    let storage_row_ranges = selected_row_ranges
        .map(|ranges| {
            ranges
                .iter()
                .map(|range| (range.row_base, range.row_end))
                .collect::<Vec<_>>()
        });
    let read_started = observer.wants_candidate_timing().then(Instant::now);
    let segments = object_store.read_leaf_object_v2_segments_for_row_ranges(
        &route.placement,
        &leaf_summaries.meta,
        storage_row_ranges.as_deref(),
    )?;
    let segment_bytes = segments.iter().try_fold(0_u64, |total, segment| {
        let bytes = u64::try_from(segment.encoded_len(&leaf_summaries.meta)?)
            .map_err(|_| "ec_spire leaf V2 segment encoded length exceeds u64".to_owned())?;
        total
            .checked_add(bytes)
            .ok_or_else(|| "ec_spire leaf V2 selected segment byte length overflow".to_owned())
    })?;
    observer.leaf_row_segments_read(
        snapshot.epoch_manifest().epoch,
        &route.placement,
        segments.len(),
        segment_bytes,
    );
    if let Some(started) = read_started {
        observer.leaf_object_read_time(
            snapshot.epoch_manifest().epoch,
            &route.placement,
            elapsed_nanos_since(started),
        );
    }
    append_quantized_v2_leaf_segments_candidates(
        snapshot,
        &leaf_summaries.meta,
        &segments,
        route,
        scorer,
        deleted_vec_ids,
        accumulator,
        selected_row_ranges,
        observer,
    )
}

fn append_quantized_v2_leaf_segments_candidates(
    snapshot: &SpireValidatedEpochSnapshot<'_>,
    meta: &SpireLeafPartitionObjectV2Meta,
    segments: &[SpireLeafPartitionObjectV2Segment],
    route: SpireLeafObjectReadRoute,
    scorer: &SpirePreparedAssignmentScorer,
    deleted_vec_ids: &HashSet<SpireVecId>,
    accumulator: &mut SpireScoredCandidateAccumulator,
    selected_row_ranges: Option<&[SpireLeafBlockRowRange]>,
    observer: &mut impl SpireRoutedScanObserver,
) -> Result<(), String> {
    for segment in segments {
        let columns = segment.columns(meta)?;
        append_quantized_v2_column_candidates(
            snapshot,
            columns,
            snapshot.epoch_manifest().epoch,
            route.leaf_pid,
            route.object_version,
            scorer,
            deleted_vec_ids,
            accumulator,
            &route.placement,
            selected_row_ranges,
            observer,
        )?;
    }
    Ok(())
}

fn append_quantized_v2_leaf_candidates(
    snapshot: &SpireValidatedEpochSnapshot<'_>,
    leaf_object: &SpireLeafPartitionObjectV2,
    route: SpireLeafObjectReadRoute,
    scorer: &SpirePreparedAssignmentScorer,
    deleted_vec_ids: &HashSet<SpireVecId>,
    accumulator: &mut SpireScoredCandidateAccumulator,
    selected_row_ranges: Option<&[SpireLeafBlockRowRange]>,
    observer: &mut impl SpireRoutedScanObserver,
) -> Result<(), String> {
    for columns in leaf_object.column_segments()? {
        let columns = columns?;
        append_quantized_v2_column_candidates(
            snapshot,
            columns,
            snapshot.epoch_manifest().epoch,
            route.leaf_pid,
            route.object_version,
            scorer,
            deleted_vec_ids,
            accumulator,
            &route.placement,
            selected_row_ranges,
            observer,
        )?;
    }
    Ok(())
}

fn append_quantized_leaf_candidates_for_pid(
    snapshot: &SpireValidatedEpochSnapshot<'_>,
    object_store: &impl SpireObjectReader,
    route: SpireLeafObjectReadRoute,
    scorer: &SpirePreparedAssignmentScorer,
    deleted_vec_ids: &HashSet<SpireVecId>,
    accumulator: &mut SpireScoredCandidateAccumulator,
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
    let epoch = snapshot.epoch_manifest().epoch;
    observer.scanned_leaf(epoch, &route.placement);
    let read_started = observer.wants_candidate_timing().then(Instant::now);

    match object_store.read_leaf_object_v2(placement) {
        Ok(leaf_object) => {
            if let Some(started) = read_started {
                observer.leaf_object_read_time(epoch, placement, elapsed_nanos_since(started));
            }
            observer.leaf_block_storage(
                epoch,
                placement,
                leaf_object.meta.summary_bytes_total,
                leaf_object.meta.row_segment_bytes_total()?,
            );
            let selected_row_ranges = select_leaf_block_row_ranges(
                &leaf_object.summaries,
                current_session_leaf_block_pruning_max_blocks_per_leaf(),
                scorer,
                epoch,
                placement,
                observer,
            )?;
            append_quantized_v2_leaf_candidates(
                snapshot,
                &leaf_object,
                route,
                scorer,
                deleted_vec_ids,
                accumulator,
                selected_row_ranges.as_deref(),
                observer,
            )
        }
        Err(v2_error) => {
            let leaf_object = object_store.read_leaf_object(placement).map_err(|v1_error| {
                format!(
                    "ec_spire quantized scan could not read leaf pid {leaf_pid} as V2 or V1: V2 error: {v2_error}; V1 error: {v1_error}"
                )
            })?;
            if let Some(started) = read_started {
                observer.leaf_object_read_time(epoch, placement, elapsed_nanos_since(started));
            }
            append_quantized_v1_leaf_candidates(
                snapshot,
                leaf_object,
                snapshot.epoch_manifest().epoch,
                leaf_pid,
                route.object_version,
                scorer,
                deleted_vec_ids,
                accumulator,
                &route.placement,
                observer,
            )
        }
    }
}

fn append_quantized_v2_column_candidates(
    snapshot: &SpireValidatedEpochSnapshot<'_>,
    columns: SpireLeafObjectColumns<'_>,
    epoch: u64,
    pid: u64,
    object_version: u64,
    scorer: &SpirePreparedAssignmentScorer,
    deleted_vec_ids: &HashSet<SpireVecId>,
    accumulator: &mut SpireScoredCandidateAccumulator,
    placement: &SpirePlacementEntry,
    selected_row_ranges: Option<&[SpireLeafBlockRowRange]>,
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

    if selected_row_ranges.is_some()
        || (scorer.payload_format() == SpireAssignmentPayloadFormat::RaBitQ
            && accumulator.is_bounded())
    {
        return append_quantized_v2_column_candidates_with_rabitq_cutoff(
            snapshot,
            columns,
            epoch,
            pid,
            object_version,
            scorer,
            deleted_vec_ids,
            accumulator,
            placement,
            selected_row_ranges,
            observer,
        );
    }

    let mut scores = vec![0.0; columns.row_count()];
    let score_started = observer.wants_candidate_timing().then(Instant::now);
    scorer.score_batch_ip(
        columns.payload_stride,
        columns.payloads,
        columns.gammas,
        &mut scores,
    )?;
    if let Some(started) = score_started {
        observer.leaf_row_score_time(epoch, placement, elapsed_nanos_since(started));
    }

    for (row_offset, ip) in scores.into_iter().enumerate() {
        if !is_visible_scored_assignment_flags(columns.flags[row_offset]) {
            continue;
        }
        if !ip.is_finite() {
            return Err(
                "ec_spire routed candidate batch scorer returned a non-finite score".to_owned(),
            );
        }

        let materialize_started = observer.wants_candidate_timing().then(Instant::now);
        let row = columns.row(row_offset)?;
        let vec_id = row.vec_id()?;
        if let Some(started) = materialize_started {
            observer.candidate_materialize_time(epoch, placement, elapsed_nanos_since(started));
        }
        if deleted_vec_ids.contains(&vec_id) {
            continue;
        }
        observer.visible_leaf_candidate(epoch, placement, row.flags);
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
        let append_started = observer.wants_candidate_timing().then(Instant::now);
        let outcome = accumulator.append(candidate);
        if let Some(started) = append_started {
            observer.candidate_heap_append_time(epoch, placement, elapsed_nanos_since(started));
        }
        observe_candidate_append_outcome(snapshot, observer, outcome)?;
    }
    Ok(())
}

fn append_quantized_v2_column_candidates_with_rabitq_cutoff(
    snapshot: &SpireValidatedEpochSnapshot<'_>,
    columns: SpireLeafObjectColumns<'_>,
    epoch: u64,
    pid: u64,
    object_version: u64,
    scorer: &SpirePreparedAssignmentScorer,
    deleted_vec_ids: &HashSet<SpireVecId>,
    accumulator: &mut SpireScoredCandidateAccumulator,
    placement: &SpirePlacementEntry,
    selected_row_ranges: Option<&[SpireLeafBlockRowRange]>,
    observer: &mut impl SpireRoutedScanObserver,
) -> Result<(), String> {
    let column_format = SpireAssignmentPayloadFormat::from_tag(columns.payload_format)?;
    for row_offset in 0..columns.row_count() {
        if !leaf_column_row_is_selected(&columns, row_offset, selected_row_ranges)? {
            continue;
        }
        if !is_visible_scored_assignment_flags(columns.flags[row_offset]) {
            continue;
        }

        let materialize_started = observer.wants_candidate_timing().then(Instant::now);
        let row = columns.row(row_offset)?;
        let vec_id = row.vec_id()?;
        if let Some(started) = materialize_started {
            observer.candidate_materialize_time(epoch, placement, elapsed_nanos_since(started));
        }
        if deleted_vec_ids.contains(&vec_id) {
            continue;
        }
        observer.visible_leaf_candidate(epoch, placement, row.flags);

        let score_started = observer.wants_candidate_timing().then(Instant::now);
        let ip = match accumulator.min_ip_to_keep() {
            Some(min_ip_to_keep) => {
                match scorer.try_score_payload_ip(
                    column_format,
                    row.gamma,
                    row.encoded_payload,
                    min_ip_to_keep,
                )? {
                    Some(ip) => ip,
                    None => {
                        if let Some(started) = score_started {
                            observer.leaf_row_score_time(
                                epoch,
                                placement,
                                elapsed_nanos_since(started),
                            );
                        }
                        let pruned = SpireScoredScanCandidate {
                            epoch,
                            pid,
                            object_version,
                            row_index: row.row_index,
                            assignment_flags: row.flags,
                            vec_id,
                            heap_tid: row.heap_tid,
                            score: -min_ip_to_keep,
                        };
                        observe_truncated_candidate(snapshot, observer, &pruned)?;
                        continue;
                    }
                }
            }
            None => scorer.score_payload_ip(column_format, row.gamma, row.encoded_payload)?,
        };
        if let Some(started) = score_started {
            observer.leaf_row_score_time(epoch, placement, elapsed_nanos_since(started));
        }
        if !ip.is_finite() {
            return Err(
                "ec_spire routed candidate scorer returned a non-finite score".to_owned(),
            );
        }

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
        let append_started = observer.wants_candidate_timing().then(Instant::now);
        let outcome = accumulator.append(candidate);
        if let Some(started) = append_started {
            observer.candidate_heap_append_time(epoch, placement, elapsed_nanos_since(started));
        }
        observe_candidate_append_outcome(snapshot, observer, outcome)?;
    }
    Ok(())
}

fn append_quantized_delta_candidates_for_loaded_routes(
    snapshot: &SpireValidatedEpochSnapshot<'_>,
    delta_routes: &[SpireLoadedDeltaObjectRoute],
    scorer: &SpirePreparedAssignmentScorer,
    deleted_vec_ids: &HashSet<SpireVecId>,
    accumulator: &mut SpireScoredCandidateAccumulator,
    observer: &mut impl SpireRoutedScanObserver,
) -> Result<(), String> {
    for loaded_route in delta_routes {
        let placement = &loaded_route.route.placement;
        for row in &loaded_route.rows {
            let assignment = &row.assignment;
            if is_delete_delta_assignment(assignment) {
                continue;
            }
            if !is_visible_scored_assignment(assignment) {
                continue;
            }
            if deleted_vec_ids.contains(&assignment.vec_id) {
                continue;
            }
            let score_started = observer.wants_candidate_timing().then(Instant::now);
            let ip = scorer.score_assignment_ip(assignment)?;
            if let Some(started) = score_started {
                observer.candidate_score_time(
                    snapshot.epoch_manifest().epoch,
                    placement,
                    elapsed_nanos_since(started),
                );
            }
            if !ip.is_finite() {
                return Err(
                    "ec_spire routed delta candidate scorer returned a non-finite score".to_owned(),
                );
            }
            observer.visible_delta_candidate(
                snapshot.epoch_manifest().epoch,
                placement,
                assignment.flags,
            );
            let candidate = SpireScoredScanCandidate {
                epoch: snapshot.epoch_manifest().epoch,
                pid: row.pid,
                object_version: row.object_version,
                row_index: row.row_index,
                assignment_flags: assignment.flags,
                vec_id: assignment.vec_id.clone(),
                heap_tid: assignment.heap_tid,
                score: -ip,
            };
            let append_started = observer.wants_candidate_timing().then(Instant::now);
            let outcome = accumulator.append(candidate);
            if let Some(started) = append_started {
                observer.candidate_heap_append_time(
                    snapshot.epoch_manifest().epoch,
                    placement,
                    elapsed_nanos_since(started),
                );
            }
            observe_candidate_append_outcome(snapshot, observer, outcome)?;
        }
    }
    Ok(())
}

fn load_delta_rows_for_routes(
    snapshot: &SpireValidatedEpochSnapshot<'_>,
    object_store: &impl SpireObjectReader,
    delta_routes: &[SpireDeltaObjectRoute],
    observer: &mut impl SpireRoutedScanObserver,
) -> Result<Vec<SpireLoadedDeltaObjectRoute>, String> {
    let mut loaded_routes = Vec::with_capacity(delta_routes.len());
    for route in delta_routes {
        let placement = &route.placement;
        if should_skip_placement(snapshot.epoch_manifest().consistency_mode, placement.state)? {
            continue;
        }

        observer.scanned_delta(snapshot.epoch_manifest().epoch, placement);

        let delta_object = object_store.read_delta_object(placement)?;
        if delta_object.header.parent_pid != route.parent_leaf_pid {
            return Err(format!(
                "ec_spire delta route parent {} does not match object parent {}",
                route.parent_leaf_pid, delta_object.header.parent_pid
            ));
        }
        let mut rows = Vec::with_capacity(delta_object.assignments.len());
        for (row_index, assignment) in delta_object.assignments.into_iter().enumerate() {
            let row_index = u32::try_from(row_index)
                .map_err(|_| "ec_spire scan row index exceeds u32".to_owned())?;
            if is_delete_delta_assignment(&assignment) {
                observer.delete_delta_row(snapshot.epoch_manifest().epoch, placement);
            }
            rows.push(SpireDeltaScanRow {
                pid: route.delta_pid,
                object_version: route.object_version,
                row_index,
                assignment,
            });
        }
        loaded_routes.push(SpireLoadedDeltaObjectRoute {
            route: *route,
            rows,
        });
    }
    Ok(loaded_routes)
}

fn collect_delta_delete_vec_ids_for_loaded_routes(
    delta_routes: &[SpireLoadedDeltaObjectRoute],
) -> HashSet<SpireVecId> {
    let mut deleted_vec_ids = HashSet::new();
    for loaded_route in delta_routes {
        for row in &loaded_route.rows {
            if is_delete_delta_assignment(&row.assignment) {
                deleted_vec_ids.insert(row.assignment.vec_id.clone());
            }
        }
    }
    deleted_vec_ids
}

fn append_quantized_v1_leaf_candidates(
    snapshot: &SpireValidatedEpochSnapshot<'_>,
    leaf_object: SpireLeafPartitionObject,
    epoch: u64,
    pid: u64,
    object_version: u64,
    scorer: &SpirePreparedAssignmentScorer,
    deleted_vec_ids: &HashSet<SpireVecId>,
    accumulator: &mut SpireScoredCandidateAccumulator,
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
        let score_started = observer.wants_candidate_timing().then(Instant::now);
        let ip = scorer.score_assignment_ip(&assignment)?;
        if let Some(started) = score_started {
            observer.leaf_row_score_time(epoch, placement, elapsed_nanos_since(started));
        }
        if !ip.is_finite() {
            return Err("ec_spire routed candidate scorer returned a non-finite score".to_owned());
        }
        let materialize_started = observer.wants_candidate_timing().then(Instant::now);
        let row_index = u32::try_from(row_index)
            .map_err(|_| "ec_spire scan row index exceeds u32".to_owned())?;
        if let Some(started) = materialize_started {
            observer.candidate_materialize_time(epoch, placement, elapsed_nanos_since(started));
        }
        observer.visible_leaf_candidate(epoch, placement, assignment.flags);
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
        let append_started = observer.wants_candidate_timing().then(Instant::now);
        let outcome = accumulator.append(candidate);
        if let Some(started) = append_started {
            observer.candidate_heap_append_time(epoch, placement, elapsed_nanos_since(started));
        }
        observe_candidate_append_outcome(snapshot, observer, outcome)?;
    }
    Ok(())
}

fn elapsed_nanos_since(started: Instant) -> u64 {
    u64::try_from(started.elapsed().as_nanos()).unwrap_or(u64::MAX)
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

    let mut accumulator = SpireScoredCandidateAccumulator::new(dedupe_mode, limit);
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
            let _ = accumulator.append(candidate);
        }
    }

    Ok(accumulator.into_ranked())
}

fn observe_candidate_append_outcome(
    snapshot: &SpireValidatedEpochSnapshot<'_>,
    observer: &mut impl SpireRoutedScanObserver,
    outcome: SpireCandidateAppendOutcome,
) -> Result<(), String> {
    if let Some(candidate) = outcome.deduped {
        observe_deduped_candidate(snapshot, observer, &candidate)?;
    }
    if let Some(candidate) = outcome.truncated {
        observe_truncated_candidate(snapshot, observer, &candidate)?;
    }
    Ok(())
}

fn observe_deduped_candidate(
    snapshot: &SpireValidatedEpochSnapshot<'_>,
    observer: &mut impl SpireRoutedScanObserver,
    candidate: &SpireScoredScanCandidate,
) -> Result<(), String> {
    let lookup = snapshot.require_lookup(candidate.pid, "scan deduped candidate diagnostics")?;
    observer.deduped_candidate(
        snapshot.epoch_manifest().epoch,
        lookup.placement,
        candidate.assignment_flags,
    );
    Ok(())
}

fn observe_truncated_candidate(
    snapshot: &SpireValidatedEpochSnapshot<'_>,
    observer: &mut impl SpireRoutedScanObserver,
    candidate: &SpireScoredScanCandidate,
) -> Result<(), String> {
    let lookup = snapshot.require_lookup(candidate.pid, "scan truncated candidate diagnostics")?;
    observer.truncated_candidate(
        snapshot.epoch_manifest().epoch,
        lookup.placement,
        candidate.assignment_flags,
    );
    Ok(())
}

fn observe_candidate_winners(
    snapshot: &SpireValidatedEpochSnapshot<'_>,
    observer: &mut impl SpireRoutedScanObserver,
    candidates: &[SpireScoredScanCandidate],
) -> Result<(), String> {
    for candidate in candidates {
        let lookup = snapshot.require_lookup(candidate.pid, "scan candidate winner diagnostics")?;
        observer.candidate_winner(
            snapshot.epoch_manifest().epoch,
            lookup.placement,
            candidate.assignment_flags,
        );
    }
    Ok(())
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

#[derive(Default)]
struct SpireCandidateAppendOutcome {
    deduped: Option<SpireScoredScanCandidate>,
    truncated: Option<SpireScoredScanCandidate>,
}

impl SpireCandidateAppendOutcome {
    fn deduped(candidate: SpireScoredScanCandidate) -> Self {
        Self {
            deduped: Some(candidate),
            truncated: None,
        }
    }

    fn truncated(candidate: SpireScoredScanCandidate) -> Self {
        Self {
            deduped: None,
            truncated: Some(candidate),
        }
    }
}

struct SpireScoredCandidateAccumulator {
    limit: Option<usize>,
    dedupe_mode: SpireCandidateDedupeMode,
    candidates: Vec<SpireScoredScanCandidate>,
    heap: BinaryHeap<SpireScoredScanCandidateHeapEntry>,
    candidates_by_vec_id: HashMap<SpireVecId, SpireScoredScanCandidate>,
}

impl SpireScoredCandidateAccumulator {
    fn new(dedupe_mode: SpireCandidateDedupeMode, limit: Option<usize>) -> Self {
        Self {
            limit,
            dedupe_mode,
            candidates: Vec::new(),
            heap: BinaryHeap::new(),
            candidates_by_vec_id: HashMap::new(),
        }
    }

    fn append(&mut self, candidate: SpireScoredScanCandidate) -> SpireCandidateAppendOutcome {
        match (self.dedupe_mode, self.limit) {
            (SpireCandidateDedupeMode::NoReplicaDedupeDisabled, None) => {
                self.candidates.push(candidate);
                SpireCandidateAppendOutcome::default()
            }
            (SpireCandidateDedupeMode::NoReplicaDedupeDisabled, Some(limit)) => {
                self.append_bounded(candidate, limit)
            }
            (SpireCandidateDedupeMode::VecIdDedupeEnabled, None) => {
                self.append_unbounded_deduped(candidate)
            }
            (SpireCandidateDedupeMode::VecIdDedupeEnabled, Some(limit)) => {
                self.append_bounded_deduped(candidate, limit)
            }
        }
    }

    fn is_bounded(&self) -> bool {
        self.limit.is_some()
    }

    fn min_ip_to_keep(&mut self) -> Option<f32> {
        let limit = self.limit?;
        if limit == 0 {
            return Some(f32::INFINITY);
        }
        match self.dedupe_mode {
            SpireCandidateDedupeMode::NoReplicaDedupeDisabled => {
                if self.heap.len() < limit {
                    None
                } else {
                    self.heap.peek().map(|entry| -entry.candidate.score)
                }
            }
            SpireCandidateDedupeMode::VecIdDedupeEnabled => {
                if self.candidates_by_vec_id.len() < limit {
                    None
                } else {
                    self.peek_live_worst_deduped()
                        .map(|candidate| -candidate.score)
                }
            }
        }
    }

    fn append_unbounded_deduped(
        &mut self,
        candidate: SpireScoredScanCandidate,
    ) -> SpireCandidateAppendOutcome {
        // With vec-id dedupe enabled, return the row suppressed by the
        // collision regardless of whether the incoming or incumbent candidate
        // wins.
        match self.candidates_by_vec_id.entry(candidate.vec_id.clone()) {
            std::collections::hash_map::Entry::Occupied(mut entry) => {
                if scored_candidate_cmp(&candidate, entry.get()) == Ordering::Less {
                    SpireCandidateAppendOutcome::deduped(std::mem::replace(
                        entry.get_mut(),
                        candidate,
                    ))
                } else {
                    SpireCandidateAppendOutcome::deduped(candidate)
                }
            }
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert(candidate);
                SpireCandidateAppendOutcome::default()
            }
        }
    }

    fn append_bounded(
        &mut self,
        candidate: SpireScoredScanCandidate,
        limit: usize,
    ) -> SpireCandidateAppendOutcome {
        if limit == 0 {
            return SpireCandidateAppendOutcome::truncated(candidate);
        }
        let entry = SpireScoredScanCandidateHeapEntry { candidate };
        if self.heap.len() < limit {
            self.heap.push(entry);
            return SpireCandidateAppendOutcome::default();
        }

        if self
            .heap
            .peek()
            .is_some_and(|worst| scored_candidate_cmp(&entry.candidate, &worst.candidate).is_lt())
        {
            let truncated = self.heap.pop().map(|entry| entry.candidate);
            self.heap.push(entry);
            if let Some(candidate) = truncated {
                return SpireCandidateAppendOutcome::truncated(candidate);
            }
            return SpireCandidateAppendOutcome::default();
        }
        SpireCandidateAppendOutcome::truncated(entry.candidate)
    }

    fn append_bounded_deduped(
        &mut self,
        candidate: SpireScoredScanCandidate,
        limit: usize,
    ) -> SpireCandidateAppendOutcome {
        if limit == 0 {
            return SpireCandidateAppendOutcome::truncated(candidate);
        }

        if self.candidates_by_vec_id.contains_key(&candidate.vec_id) {
            return self.replace_retained_deduped(candidate);
        }

        if self.candidates_by_vec_id.len() < limit {
            self.heap.push(SpireScoredScanCandidateHeapEntry {
                candidate: candidate.clone(),
            });
            self.candidates_by_vec_id
                .insert(candidate.vec_id.clone(), candidate);
            return SpireCandidateAppendOutcome::default();
        }

        let Some(worst) = self.peek_live_worst_deduped() else {
            self.heap.push(SpireScoredScanCandidateHeapEntry {
                candidate: candidate.clone(),
            });
            self.candidates_by_vec_id
                .insert(candidate.vec_id.clone(), candidate);
            return SpireCandidateAppendOutcome::default();
        };
        if scored_candidate_cmp(&candidate, &worst).is_lt() {
            let Some(evicted) = self.pop_live_worst_deduped() else {
                return SpireCandidateAppendOutcome::default();
            };
            self.candidates_by_vec_id.remove(&evicted.vec_id);
            self.heap.push(SpireScoredScanCandidateHeapEntry {
                candidate: candidate.clone(),
            });
            self.candidates_by_vec_id
                .insert(candidate.vec_id.clone(), candidate);
            return SpireCandidateAppendOutcome::truncated(evicted);
        }
        SpireCandidateAppendOutcome::truncated(candidate)
    }

    fn replace_retained_deduped(
        &mut self,
        candidate: SpireScoredScanCandidate,
    ) -> SpireCandidateAppendOutcome {
        let incumbent = self
            .candidates_by_vec_id
            .get_mut(&candidate.vec_id)
            .expect("checked retained vec_id");
        if scored_candidate_cmp(&candidate, incumbent) == Ordering::Less {
            let suppressed = std::mem::replace(incumbent, candidate.clone());
            self.heap.push(SpireScoredScanCandidateHeapEntry { candidate });
            SpireCandidateAppendOutcome::deduped(suppressed)
        } else {
            SpireCandidateAppendOutcome::deduped(candidate)
        }
    }

    fn peek_live_worst_deduped(&mut self) -> Option<SpireScoredScanCandidate> {
        while let Some(entry) = self.heap.peek() {
            if self
                .candidates_by_vec_id
                .get(&entry.candidate.vec_id)
                .is_some_and(|candidate| candidate == &entry.candidate)
            {
                return Some(entry.candidate.clone());
            }
            self.heap.pop();
        }
        None
    }

    fn pop_live_worst_deduped(&mut self) -> Option<SpireScoredScanCandidate> {
        while let Some(entry) = self.heap.pop() {
            if self
                .candidates_by_vec_id
                .get(&entry.candidate.vec_id)
                .is_some_and(|candidate| candidate == &entry.candidate)
            {
                return Some(entry.candidate);
            }
        }
        None
    }

    fn into_ranked(mut self) -> Vec<SpireScoredScanCandidate> {
        let mut ranked = match (self.dedupe_mode, self.limit) {
            (SpireCandidateDedupeMode::NoReplicaDedupeDisabled, None) => self.candidates,
            (SpireCandidateDedupeMode::NoReplicaDedupeDisabled, Some(_)) => self
                .heap
                .into_iter()
                .map(|entry| entry.candidate)
                .collect::<Vec<_>>(),
            (SpireCandidateDedupeMode::VecIdDedupeEnabled, None) => {
                self.candidates_by_vec_id.into_values().collect::<Vec<_>>()
            }
            (SpireCandidateDedupeMode::VecIdDedupeEnabled, Some(_)) => {
                while self.pop_live_worst_deduped().is_some() {}
                self.candidates_by_vec_id.into_values().collect::<Vec<_>>()
            }
        };
        ranked.sort_by(scored_candidate_cmp);
        ranked
    }
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

    let mut heap = BinaryHeap::new();
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
