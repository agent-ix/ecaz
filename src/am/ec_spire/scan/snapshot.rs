pub(super) fn collect_snapshot_leaf_rows(
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    object_store: &impl SpireObjectReader,
) -> Result<Vec<SpireLeafScanRow>, String> {
    let snapshot = SpireValidatedEpochSnapshot::from_snapshot(*snapshot)?;
    collect_validated_snapshot_leaf_rows(&snapshot, object_store)
}

fn collect_validated_snapshot_leaf_rows(
    snapshot: &SpireValidatedEpochSnapshot<'_>,
    object_store: &impl SpireObjectReader,
) -> Result<Vec<SpireLeafScanRow>, String> {
    let mut rows = Vec::new();
    for manifest_entry in &snapshot.object_manifest().entries {
        let lookup = snapshot.require_lookup(manifest_entry.pid, "scan leaf row collection")?;
        let placement = lookup.placement;

        if should_skip_placement(snapshot.epoch_manifest().consistency_mode, placement.state)? {
            continue;
        }

        let header = object_store.read_object_header(placement)?;
        if header.kind != SpirePartitionObjectKind::Leaf {
            continue;
        }

        rows.extend(read_leaf_scan_rows(
            object_store,
            placement,
            manifest_entry.pid,
            manifest_entry.object_version,
        )?);
    }
    Ok(rows)
}

pub(super) fn collect_snapshot_routed_leaf_rows(
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    object_store: &impl SpireObjectReader,
    query_vector: &[f32],
) -> Result<SpireRoutedLeafScanRows, String> {
    let mut routed =
        collect_snapshot_routed_probe_leaf_rows(snapshot, object_store, query_vector, 1)?;
    routed
        .pop()
        .ok_or_else(|| "ec_spire routed scan found no leaf route".to_owned())
}

pub(super) fn collect_snapshot_routed_probe_leaf_rows(
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    object_store: &impl SpireObjectReader,
    query_vector: &[f32],
    nprobe: u32,
) -> Result<Vec<SpireRoutedLeafScanRows>, String> {
    let nprobe_policy = SpireRecursiveNprobePolicy::conservative(nprobe)?;
    collect_snapshot_routed_probe_leaf_rows_with_policy(
        snapshot,
        object_store,
        query_vector,
        &nprobe_policy,
    )
}

fn collect_snapshot_routed_probe_leaf_rows_with_policy(
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    object_store: &impl SpireObjectReader,
    query_vector: &[f32],
    nprobe_policy: &SpireRecursiveNprobePolicy,
) -> Result<Vec<SpireRoutedLeafScanRows>, String> {
    let snapshot = SpireValidatedEpochSnapshot::from_snapshot(*snapshot)?;
    let hierarchy = load_snapshot_routing_hierarchy(&snapshot, object_store)?;
    let leaf_routes = route_recursive_routing_objects_to_leaf_routes_with_policy(
        &hierarchy.root_object,
        &hierarchy.internal_objects_by_pid,
        query_vector,
        nprobe_policy,
    )?;
    let epoch = snapshot.epoch_manifest().epoch;

    let mut routed = Vec::with_capacity(leaf_routes.len());
    for route in leaf_routes {
        let rows = collect_snapshot_leaf_rows_for_pid(
            &snapshot,
            object_store,
            route.leaf_pid,
            route.parent_pid,
        )?;
        routed.push(SpireRoutedLeafScanRows {
            epoch,
            root_pid: hierarchy.root_pid,
            leaf_pid: route.leaf_pid,
            rows,
        });
    }
    Ok(routed)
}

pub(super) fn collect_snapshot_top_graph_routed_probe_leaf_rows(
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    object_store: &impl SpireObjectReader,
    query_vector: &[f32],
    search_list_size: u32,
    top_route_count: u32,
    nprobe_policy: &SpireRecursiveNprobePolicy,
    route_budget: SpireRecursiveRouteBudget,
    max_routed_candidate_rows: Option<usize>,
) -> Result<Vec<SpireRoutedLeafScanRows>, String> {
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
        search_list_size,
        top_route_count,
        nprobe_policy,
        route_budget,
        SpireLeafRouteRanking::AccumulatedPathScore,
        max_routed_candidate_rows,
        &mut leaf_row_count,
    )?
    .routes;
    let epoch = snapshot.epoch_manifest().epoch;

    let mut routed = Vec::with_capacity(leaf_routes.len());
    for route in leaf_routes {
        let rows = collect_snapshot_leaf_rows_for_pid(
            &snapshot,
            object_store,
            route.leaf_pid,
            route.parent_pid,
        )?;
        routed.push(SpireRoutedLeafScanRows {
            epoch,
            root_pid: hierarchy.root_pid,
            leaf_pid: route.leaf_pid,
            rows,
        });
    }
    Ok(routed)
}

pub(super) fn count_snapshot_single_level_leaf_pids(
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    object_store: &impl SpireObjectReader,
) -> Result<u32, String> {
    let snapshot = SpireValidatedEpochSnapshot::from_snapshot(*snapshot)?;
    let (_, root_object) = load_snapshot_root_routing_object(&snapshot, object_store)?;
    u32::try_from(root_object.child_count())
        .map_err(|_| "ec_spire scan root child count exceeds u32".to_owned())
}

pub(super) fn count_snapshot_recursive_leaf_pids(
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    object_store: &impl SpireObjectReader,
) -> Result<u32, String> {
    let snapshot = SpireValidatedEpochSnapshot::from_snapshot(*snapshot)?;
    let hierarchy = load_snapshot_routing_hierarchy(&snapshot, object_store)?;
    count_recursive_routing_leaf_pids(&hierarchy.root_object, &hierarchy.internal_objects_by_pid)
}

pub(super) fn collect_scan_routing_diagnostics(
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    object_store: &impl SpireObjectReader,
    query: &SpireScanQuery,
    options: EcSpireOptions,
) -> Result<SpireScanRoutingDiagnostics, String> {
    let top_graph_plan = options.top_graph_plan()?;
    let snapshot = SpireValidatedEpochSnapshot::from_snapshot(*snapshot)?;
    let hierarchy = load_snapshot_routing_hierarchy(&snapshot, object_store)?;
    let leaf_count =
        count_recursive_routing_leaf_pids(&hierarchy.root_object, &hierarchy.internal_objects_by_pid)?;
    let scan_plan = resolve_single_level_scan_plan(leaf_count, options)?;
    if scan_plan.nprobe == 0 {
        return Ok(SpireScanRoutingDiagnostics {
            scan_plan,
            levels: Vec::new(),
        });
    }

    let levels = if top_graph_plan.enabled {
        let (_top_graph_pid, top_graph) = load_snapshot_top_graph_object(&snapshot, object_store)?
            .ok_or_else(|| "ec_spire scan snapshot has no available top graph object".to_owned())?;
        let leaf_assignment_counts = &hierarchy.leaf_assignment_counts_by_pid;
        let mut leaf_row_count = |route| {
            leaf_route_assignment_count_from_loaded_hierarchy(leaf_assignment_counts, route)
        };
        collect_top_graph_routing_level_diagnostics(
            &hierarchy.root_object,
            &hierarchy.internal_objects_by_pid,
            &top_graph,
            query.values(),
            top_graph_plan.search_list_size.unwrap_or(scan_plan.nprobe),
            scan_plan.nprobe,
            &scan_plan.recursive_nprobe_policy,
            scan_plan.recursive_route_budget,
            scan_plan.leaf_route_ranking,
            scan_plan.max_routed_candidate_rows,
            &mut leaf_row_count,
        )?
    } else {
        let leaf_assignment_counts = &hierarchy.leaf_assignment_counts_by_pid;
        let mut leaf_row_count = |route| {
            leaf_route_assignment_count_from_loaded_hierarchy(leaf_assignment_counts, route)
        };
        collect_recursive_routing_level_diagnostics_with_row_budget(
            &hierarchy.root_object,
            &hierarchy.internal_objects_by_pid,
            query.values(),
            &scan_plan.recursive_nprobe_policy,
            scan_plan.recursive_route_budget,
            scan_plan.leaf_route_ranking,
            scan_plan.max_routed_candidate_rows,
            &mut leaf_row_count,
        )?
    };

    Ok(SpireScanRoutingDiagnostics { scan_plan, levels })
}

pub(super) fn collect_ranked_routed_probe_candidates<F>(
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    object_store: &impl SpireObjectReader,
    query_vector: &[f32],
    nprobe: u32,
    score_ip: F,
    dedupe_mode: SpireCandidateDedupeMode,
    limit: Option<usize>,
) -> Result<Vec<SpireScoredScanCandidate>, String>
where
    F: FnMut(&SpireLeafAssignmentRow) -> Result<f32, String>,
{
    let routed_rows =
        collect_snapshot_routed_probe_leaf_rows(snapshot, object_store, query_vector, nprobe)?;
    rank_routed_leaf_rows_by_ip(routed_rows, score_ip, dedupe_mode, limit)
}

pub(super) fn collect_quantized_routed_probe_candidates(
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    object_store: &impl SpireObjectReader,
    query_vector: &[f32],
    nprobe: u32,
    payload_format: SpireAssignmentPayloadFormat,
    dedupe_mode: SpireCandidateDedupeMode,
    limit: Option<usize>,
) -> Result<Vec<SpireScoredScanCandidate>, String> {
    let nprobe_policy = SpireRecursiveNprobePolicy::conservative(nprobe)?;
    collect_quantized_routed_probe_candidates_with_policy(
        snapshot,
        object_store,
        query_vector,
        &nprobe_policy,
        SpireRecursiveRouteBudget::unbounded(),
        None,
        payload_format,
        dedupe_mode,
        limit,
    )
}

fn collect_quantized_routed_probe_candidates_with_policy(
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    object_store: &impl SpireObjectReader,
    query_vector: &[f32],
    nprobe_policy: &SpireRecursiveNprobePolicy,
    route_budget: SpireRecursiveRouteBudget,
    max_routed_candidate_rows: Option<usize>,
    payload_format: SpireAssignmentPayloadFormat,
    dedupe_mode: SpireCandidateDedupeMode,
    limit: Option<usize>,
) -> Result<Vec<SpireScoredScanCandidate>, String> {
    let snapshot = SpireValidatedEpochSnapshot::from_snapshot(*snapshot)?;
    let hierarchy = load_snapshot_routing_hierarchy(&snapshot, object_store)?;
    let mut observer = SpireNoopRoutedScanObserver;
    collect_validated_recursive_quantized_routed_probe_candidates(
        &snapshot,
        object_store,
        query_vector,
        &hierarchy,
        nprobe_policy,
        route_budget,
        max_routed_candidate_rows,
        payload_format,
        dedupe_mode,
        limit,
        &mut observer,
    )
}

fn collect_validated_recursive_quantized_routed_probe_candidates(
    snapshot: &SpireValidatedEpochSnapshot<'_>,
    object_store: &impl SpireObjectReader,
    query_vector: &[f32],
    hierarchy: &SpireLoadedRoutingHierarchy,
    nprobe_policy: &SpireRecursiveNprobePolicy,
    route_budget: SpireRecursiveRouteBudget,
    max_routed_candidate_rows: Option<usize>,
    payload_format: SpireAssignmentPayloadFormat,
    dedupe_mode: SpireCandidateDedupeMode,
    limit: Option<usize>,
    observer: &mut impl SpireRoutedScanObserver,
) -> Result<Vec<SpireScoredScanCandidate>, String> {
    let scorer =
        SpirePreparedAssignmentScorer::prepare(payload_format, query_vector.len(), query_vector)?;
    let leaf_assignment_counts = &hierarchy.leaf_assignment_counts_by_pid;
    let mut leaf_row_count =
        |route| leaf_route_assignment_count_from_loaded_hierarchy(leaf_assignment_counts, route);
    let leaf_routes = route_recursive_routing_objects_to_leaf_routes_with_row_budget(
        &hierarchy.root_object,
        &hierarchy.internal_objects_by_pid,
        query_vector,
        nprobe_policy,
        route_budget,
        SpireLeafRouteRanking::AccumulatedPathScore,
        max_routed_candidate_rows,
        &mut leaf_row_count,
    )?
    .routes;
    collect_validated_quantized_leaf_route_candidates(
        snapshot,
        object_store,
        leaf_routes,
        &scorer,
        dedupe_mode,
        limit,
        None,
        observer,
    )
}

fn leaf_route_assignment_count_from_loaded_hierarchy(
    leaf_assignment_counts_by_pid: &HashMap<u64, usize>,
    route: SpireRecursiveLeafRoute,
) -> Result<usize, String> {
    leaf_assignment_counts_by_pid
        .get(&route.leaf_pid)
        .copied()
        .ok_or_else(|| {
            format!(
                "ec_spire routed row budget selected PID {} is not a visible leaf object",
                route.leaf_pid
            )
        })
}

fn collect_validated_quantized_routed_probe_candidates(
    snapshot: &SpireValidatedEpochSnapshot<'_>,
    object_store: &impl SpireObjectReader,
    query_vector: &[f32],
    root_pid: u64,
    root_object: &SpireRoutingPartitionObject,
    nprobe: u32,
    payload_format: SpireAssignmentPayloadFormat,
    dedupe_mode: SpireCandidateDedupeMode,
    limit: Option<usize>,
    observer: &mut impl SpireRoutedScanObserver,
) -> Result<Vec<SpireScoredScanCandidate>, String> {
    let scorer =
        SpirePreparedAssignmentScorer::prepare(payload_format, query_vector.len(), query_vector)?;
    let leaf_routes = route_root_object_to_leaf_pids(root_object, query_vector, nprobe)?
        .into_iter()
        .map(|leaf_pid| SpireRecursiveLeafRoute {
            leaf_pid,
            parent_pid: root_pid,
            route_score: 0.0,
        })
        .collect();
    collect_validated_quantized_leaf_route_candidates(
        snapshot,
        object_store,
        leaf_routes,
        &scorer,
        dedupe_mode,
        limit,
        None,
        observer,
    )
}

fn collect_validated_quantized_leaf_route_candidates(
    snapshot: &SpireValidatedEpochSnapshot<'_>,
    object_store: &impl SpireObjectReader,
    leaf_routes: Vec<SpireRecursiveLeafRoute>,
    scorer: &SpirePreparedAssignmentScorer,
    dedupe_mode: SpireCandidateDedupeMode,
    limit: Option<usize>,
    initial_threshold_score: Option<f32>,
    observer: &mut impl SpireRoutedScanObserver,
) -> Result<Vec<SpireScoredScanCandidate>, String> {
    if limit == Some(0) {
        return Ok(Vec::new());
    }

    let mut accumulator = SpireScoredCandidateAccumulator::new(dedupe_mode, limit);
    let delta_routes = collect_snapshot_delta_object_routes(snapshot, object_store)?;
    let mut delta_routes_by_parent = HashMap::<u64, Vec<SpireDeltaObjectRoute>>::new();
    let route_groups =
        group_leaf_and_delta_reads_by_local_store(snapshot, leaf_routes, delta_routes, observer)?;
    prefetch_store_object_read_groups(
        object_store,
        snapshot.epoch_manifest().epoch,
        &route_groups,
        observer,
    )?;
    for route_group in &route_groups {
        for route in &route_group.delta_routes {
            delta_routes_by_parent
                .entry(route.parent_leaf_pid)
                .or_default()
                .push(*route);
        }
    }
    if current_session_leaf_block_pruning_max_global_blocks() > 0
        && scorer.payload_format() == SpireAssignmentPayloadFormat::RaBitQ
    {
        return collect_validated_quantized_leaf_route_candidates_with_global_block_pruning(
            snapshot,
            object_store,
            route_groups,
            delta_routes_by_parent,
            scorer,
            dedupe_mode,
            limit,
            observer,
        );
    }
    if let Some(threshold_score) = initial_threshold_score {
        if scorer.payload_format() == SpireAssignmentPayloadFormat::RaBitQ {
            return collect_validated_quantized_leaf_route_candidates_with_threshold_pruning(
                snapshot,
                object_store,
                route_groups,
                delta_routes_by_parent,
                scorer,
                dedupe_mode,
                limit,
                threshold_score,
                observer,
            );
        }
    }

    for route_group in route_groups {
        for route in route_group.leaf_routes {
            let leaf_pid = route.leaf_pid;
            let leaf_delta_routes = delta_routes_by_parent
                .get(&leaf_pid)
                .map(Vec::as_slice)
                .unwrap_or(&[]);
            let loaded_delta_routes = load_delta_rows_for_routes(
                snapshot,
                object_store,
                leaf_delta_routes,
                observer,
            )?;
            let deleted_vec_ids =
                collect_delta_delete_vec_ids_for_loaded_routes(&loaded_delta_routes);
            append_quantized_leaf_candidates_for_pid(
                snapshot,
                object_store,
                route,
                scorer,
                &deleted_vec_ids,
                &mut accumulator,
                observer,
            )?;
            append_quantized_delta_candidates_for_loaded_routes(
                snapshot,
                &loaded_delta_routes,
                scorer,
                &deleted_vec_ids,
                &mut accumulator,
                observer,
            )?;
        }
    }

    let ranked = accumulator.into_ranked();
    observe_candidate_winners(snapshot, observer, &ranked)?;
    Ok(ranked)
}

fn collect_validated_quantized_leaf_route_candidates_with_threshold_pruning(
    snapshot: &SpireValidatedEpochSnapshot<'_>,
    object_store: &impl SpireObjectReader,
    route_groups: Vec<SpireStoreObjectReadGroup>,
    delta_routes_by_parent: HashMap<u64, Vec<SpireDeltaObjectRoute>>,
    scorer: &SpirePreparedAssignmentScorer,
    dedupe_mode: SpireCandidateDedupeMode,
    limit: Option<usize>,
    threshold_score: f32,
    observer: &mut impl SpireRoutedScanObserver,
) -> Result<Vec<SpireScoredScanCandidate>, String> {
    let mut accumulator = SpireScoredCandidateAccumulator::new(dedupe_mode, limit);
    let mut loaded_leaf_routes = Vec::new();

    for route_group in route_groups {
        for route in route_group.leaf_routes {
            let leaf_pid = route.leaf_pid;
            let leaf_delta_routes = delta_routes_by_parent
                .get(&leaf_pid)
                .map(Vec::as_slice)
                .unwrap_or(&[]);
            let loaded_delta_routes =
                load_delta_rows_for_routes(snapshot, object_store, leaf_delta_routes, observer)?;
            let deleted_vec_ids =
                collect_delta_delete_vec_ids_for_loaded_routes(&loaded_delta_routes);
            let Some(leaf_summaries) =
                read_quantized_v2_leaf_summaries_for_route(snapshot, object_store, route, observer)?
            else {
                append_quantized_delta_candidates_for_loaded_routes(
                    snapshot,
                    &loaded_delta_routes,
                    scorer,
                    &deleted_vec_ids,
                    &mut accumulator,
                    observer,
                )?;
                continue;
            };
            loaded_leaf_routes.push(SpireLoadedQuantizedLeafSummaryRoute {
                route,
                leaf_summaries,
                selected_row_ranges: None,
                loaded_delta_routes,
                deleted_vec_ids,
            });
        }
    }

    for loaded_route in &mut loaded_leaf_routes {
        loaded_route.selected_row_ranges = select_threshold_leaf_block_row_ranges(
            loaded_route.leaf_summaries.summaries.as_slice(),
            threshold_score,
            scorer,
            snapshot.epoch_manifest().epoch,
            &loaded_route.route.placement,
            observer,
        )?;
    }

    for loaded_route in loaded_leaf_routes {
        let should_read_leaf_segments = match loaded_route.selected_row_ranges.as_ref() {
            Some(ranges) => !ranges.is_empty(),
            None => true,
        };
        if should_read_leaf_segments {
            append_quantized_v2_leaf_summary_route_candidates(
                snapshot,
                object_store,
                &loaded_route.leaf_summaries,
                loaded_route.route,
                scorer,
                &loaded_route.deleted_vec_ids,
                &mut accumulator,
                loaded_route.selected_row_ranges.as_deref(),
                observer,
            )?;
        }
        append_quantized_delta_candidates_for_loaded_routes(
            snapshot,
            &loaded_route.loaded_delta_routes,
            scorer,
            &loaded_route.deleted_vec_ids,
            &mut accumulator,
            observer,
        )?;
    }

    let ranked = accumulator.into_ranked();
    observe_candidate_winners(snapshot, observer, &ranked)?;
    Ok(ranked)
}

fn collect_validated_quantized_leaf_route_candidates_with_global_block_pruning(
    snapshot: &SpireValidatedEpochSnapshot<'_>,
    object_store: &impl SpireObjectReader,
    route_groups: Vec<SpireStoreObjectReadGroup>,
    delta_routes_by_parent: HashMap<u64, Vec<SpireDeltaObjectRoute>>,
    scorer: &SpirePreparedAssignmentScorer,
    dedupe_mode: SpireCandidateDedupeMode,
    limit: Option<usize>,
    observer: &mut impl SpireRoutedScanObserver,
) -> Result<Vec<SpireScoredScanCandidate>, String> {
    let max_global_blocks = current_session_leaf_block_pruning_max_global_blocks();
    let sampled_global_probe_blocks = current_session_leaf_block_pruning_global_probe_blocks();
    let sampled_rows_per_block = current_session_leaf_block_pruning_sample_rows_per_block();
    if sampled_global_probe_blocks > max_global_blocks && sampled_rows_per_block > 0 {
        return collect_validated_quantized_leaf_route_candidates_with_sampled_global_block_pruning(
            snapshot,
            object_store,
            route_groups,
            delta_routes_by_parent,
            scorer,
            dedupe_mode,
            limit,
            observer,
        );
    }

    let mut accumulator = SpireScoredCandidateAccumulator::new(dedupe_mode, limit);
    let mut loaded_leaf_routes = Vec::new();

    for route_group in route_groups {
        for route in route_group.leaf_routes {
            let leaf_pid = route.leaf_pid;
            let leaf_delta_routes = delta_routes_by_parent
                .get(&leaf_pid)
                .map(Vec::as_slice)
                .unwrap_or(&[]);
            let loaded_delta_routes =
                load_delta_rows_for_routes(snapshot, object_store, leaf_delta_routes, observer)?;
            let deleted_vec_ids =
                collect_delta_delete_vec_ids_for_loaded_routes(&loaded_delta_routes);
            let Some(leaf_summaries) =
                read_quantized_v2_leaf_summaries_for_route(snapshot, object_store, route, observer)?
            else {
                append_quantized_delta_candidates_for_loaded_routes(
                    snapshot,
                    &loaded_delta_routes,
                    scorer,
                    &deleted_vec_ids,
                    &mut accumulator,
                    observer,
                )?;
                continue;
            };
            loaded_leaf_routes.push(SpireLoadedQuantizedLeafSummaryRoute {
                route,
                leaf_summaries,
                selected_row_ranges: None,
                loaded_delta_routes,
                deleted_vec_ids,
            });
        }
    }

    let selected_by_leaf = select_global_leaf_block_row_ranges(
        loaded_leaf_routes.iter().map(|loaded_route| {
            (
                loaded_route.route.leaf_pid,
                loaded_route.route.route_score,
                &loaded_route.route.placement,
                loaded_route.leaf_summaries.summaries.as_slice(),
            )
        }),
        max_global_blocks,
        scorer,
        snapshot.epoch_manifest().epoch,
        observer,
    )?;
    if let Some(selected_by_leaf) = selected_by_leaf {
        for loaded_route in &mut loaded_leaf_routes {
            loaded_route.selected_row_ranges = selected_by_leaf
                .get(&loaded_route.route.leaf_pid)
                .cloned();
        }
    }
    for loaded_route in &loaded_leaf_routes {
        if loaded_route.leaf_summaries.summaries.is_empty() {
            continue;
        }
        let selected_blocks = loaded_route
            .selected_row_ranges
            .as_ref()
            .map_or(loaded_route.leaf_summaries.summaries.len(), Vec::len);
        observer.leaf_block_selection(
            snapshot.epoch_manifest().epoch,
            &loaded_route.route.placement,
            loaded_route.leaf_summaries.summaries.len(),
            selected_blocks,
        );
    }

    for loaded_route in loaded_leaf_routes {
        let should_read_leaf_segments = match loaded_route.selected_row_ranges.as_ref() {
            Some(ranges) => !ranges.is_empty(),
            None => true,
        };
        if should_read_leaf_segments {
            append_quantized_v2_leaf_summary_route_candidates(
                snapshot,
                object_store,
                &loaded_route.leaf_summaries,
                loaded_route.route,
                scorer,
                &loaded_route.deleted_vec_ids,
                &mut accumulator,
                loaded_route.selected_row_ranges.as_deref(),
                observer,
            )?;
        }
        append_quantized_delta_candidates_for_loaded_routes(
            snapshot,
            &loaded_route.loaded_delta_routes,
            scorer,
            &loaded_route.deleted_vec_ids,
            &mut accumulator,
            observer,
        )?;
    }

    let ranked = accumulator.into_ranked();
    observe_candidate_winners(snapshot, observer, &ranked)?;
    Ok(ranked)
}

fn collect_validated_quantized_leaf_route_candidates_with_sampled_global_block_pruning(
    snapshot: &SpireValidatedEpochSnapshot<'_>,
    object_store: &impl SpireObjectReader,
    route_groups: Vec<SpireStoreObjectReadGroup>,
    delta_routes_by_parent: HashMap<u64, Vec<SpireDeltaObjectRoute>>,
    scorer: &SpirePreparedAssignmentScorer,
    dedupe_mode: SpireCandidateDedupeMode,
    limit: Option<usize>,
    observer: &mut impl SpireRoutedScanObserver,
) -> Result<Vec<SpireScoredScanCandidate>, String> {
    let mut accumulator = SpireScoredCandidateAccumulator::new(dedupe_mode, limit);
    let mut loaded_leaf_routes = Vec::new();

    for route_group in route_groups {
        for route in route_group.leaf_routes {
            let leaf_pid = route.leaf_pid;
            let leaf_delta_routes = delta_routes_by_parent
                .get(&leaf_pid)
                .map(Vec::as_slice)
                .unwrap_or(&[]);
            let loaded_delta_routes = load_delta_rows_for_routes(
                snapshot,
                object_store,
                leaf_delta_routes,
                observer,
            )?;
            let deleted_vec_ids =
                collect_delta_delete_vec_ids_for_loaded_routes(&loaded_delta_routes);
            let Some(leaf_object) =
                read_quantized_v2_leaf_object_for_route(snapshot, object_store, route, observer)?
            else {
                append_quantized_delta_candidates_for_loaded_routes(
                    snapshot,
                    &loaded_delta_routes,
                    scorer,
                    &deleted_vec_ids,
                    &mut accumulator,
                    observer,
                )?;
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

    let selected_by_leaf = select_sampled_global_leaf_block_row_ranges(
        snapshot,
        &loaded_leaf_routes,
        current_session_leaf_block_pruning_max_global_blocks(),
        current_session_leaf_block_pruning_global_probe_blocks(),
        current_session_leaf_block_pruning_sample_rows_per_block(),
        current_session_leaf_block_pruning_sample_summary_prior_weight(),
        scorer,
        &mut accumulator,
        observer,
    )?;
    if let Some(selected_by_leaf) = selected_by_leaf {
        for loaded_route in &mut loaded_leaf_routes {
            loaded_route.selected_row_ranges = selected_by_leaf
                .get(&loaded_route.route.leaf_pid)
                .cloned();
        }
    }
    for loaded_route in &loaded_leaf_routes {
        if loaded_route.leaf_object.summaries.is_empty() {
            continue;
        }
        let selected_blocks = loaded_route
            .selected_row_ranges
            .as_ref()
            .map_or(loaded_route.leaf_object.summaries.len(), Vec::len);
        observer.leaf_block_selection(
            snapshot.epoch_manifest().epoch,
            &loaded_route.route.placement,
            loaded_route.leaf_object.summaries.len(),
            selected_blocks,
        );
    }

    for loaded_route in loaded_leaf_routes {
        append_quantized_v2_leaf_candidates(
            snapshot,
            &loaded_route.leaf_object,
            loaded_route.route,
            scorer,
            &loaded_route.deleted_vec_ids,
            &mut accumulator,
            loaded_route.selected_row_ranges.as_deref(),
            observer,
        )?;
        append_quantized_delta_candidates_for_loaded_routes(
            snapshot,
            &loaded_route.loaded_delta_routes,
            scorer,
            &loaded_route.deleted_vec_ids,
            &mut accumulator,
            observer,
        )?;
    }

    let ranked = accumulator.into_ranked();
    observe_candidate_winners(snapshot, observer, &ranked)?;
    Ok(ranked)
}

fn prefetch_store_object_read_groups(
    object_store: &impl SpireObjectReader,
    epoch: u64,
    route_groups: &[SpireStoreObjectReadGroup],
    observer: &mut impl SpireRoutedScanObserver,
) -> Result<(), String> {
    let mut placements = Vec::new();
    for route_group in route_groups {
        observer.store_read_batch(epoch, route_group.node_id, route_group.local_store_id);
        collect_store_object_read_group_prefetch_placements(route_group, &mut placements);
    }
    object_store.prefetch_objects(&placements)?;
    for placement in placements {
        observer.prefetched_object(epoch, &placement);
    }
    Ok(())
}

fn collect_snapshot_delta_object_routes(
    snapshot: &SpireValidatedEpochSnapshot<'_>,
    object_store: &impl SpireObjectReader,
) -> Result<Vec<SpireDeltaObjectRoute>, String> {
    let mut delta_routes = Vec::new();
    for manifest_entry in &snapshot.object_manifest().entries {
        let lookup = snapshot.require_lookup(manifest_entry.pid, "scan delta route discovery")?;
        let placement = lookup.placement;
        if should_skip_placement(snapshot.epoch_manifest().consistency_mode, placement.state)? {
            continue;
        }

        let header = object_store.read_object_header(placement)?;
        if header.kind == SpirePartitionObjectKind::Delta {
            delta_routes.push(SpireDeltaObjectRoute {
                delta_pid: manifest_entry.pid,
                parent_leaf_pid: header.parent_pid,
                placement: *placement,
                object_version: manifest_entry.object_version,
            });
        }
    }
    Ok(delta_routes)
}

fn collect_store_object_read_group_prefetch_placements(
    route_group: &SpireStoreObjectReadGroup,
    placements: &mut Vec<SpirePlacementEntry>,
) {
    for route in &route_group.leaf_routes {
        placements.push(route.placement);
    }
    for route in &route_group.delta_routes {
        placements.push(route.placement);
    }
}

fn group_leaf_and_delta_reads_by_local_store(
    snapshot: &SpireValidatedEpochSnapshot<'_>,
    leaf_routes: Vec<SpireRecursiveLeafRoute>,
    delta_routes: Vec<SpireDeltaObjectRoute>,
    observer: &mut impl SpireRoutedScanObserver,
) -> Result<Vec<SpireStoreObjectReadGroup>, String> {
    let selected_leaf_pids = leaf_routes
        .iter()
        .map(|route| route.leaf_pid)
        .collect::<HashSet<_>>();
    let mut reads_by_store = BTreeMap::<(u32, u32), SpireStoreObjectReadGroup>::new();

    // Output order is store-keyed by (node_id, local_store_id), not scan phase-keyed.
    // Callers that need phase order must traverse leaf_routes and delta_routes explicitly.
    for route in leaf_routes {
        let lookup = snapshot.require_lookup(route.leaf_pid, "scan leaf route grouping")?;
        let placement = lookup.placement;
        let read_route = SpireLeafObjectReadRoute {
            leaf_pid: route.leaf_pid,
            parent_pid: route.parent_pid,
            route_score: route.route_score,
            placement: *placement,
            object_version: lookup.manifest_entry.object_version,
        };
        observer.routed_leaf(snapshot.epoch_manifest().epoch, placement);
        reads_by_store
            .entry((placement.node_id, placement.local_store_id))
            .or_insert_with(|| SpireStoreObjectReadGroup {
                node_id: placement.node_id,
                local_store_id: placement.local_store_id,
                leaf_routes: Vec::new(),
                delta_routes: Vec::new(),
            })
            .leaf_routes
            .push(read_route);
    }

    for route in delta_routes {
        let placement = &route.placement;
        if !selected_leaf_pids.contains(&route.parent_leaf_pid) {
            observer.dropped_unselected_delta_route(snapshot.epoch_manifest().epoch, placement);
            continue;
        }
        observer.routed_delta(snapshot.epoch_manifest().epoch, placement);
        reads_by_store
            .entry((placement.node_id, placement.local_store_id))
            .or_insert_with(|| SpireStoreObjectReadGroup {
                node_id: placement.node_id,
                local_store_id: placement.local_store_id,
                leaf_routes: Vec::new(),
                delta_routes: Vec::new(),
            })
            .delta_routes
            .push(route);
    }

    Ok(reads_by_store.into_values().collect())
}
