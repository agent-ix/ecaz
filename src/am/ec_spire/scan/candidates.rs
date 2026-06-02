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
        return Ok(None);
    }
    if scorer.payload_format() != SpireAssignmentPayloadFormat::RaBitQ {
        return Ok(None);
    }
    let max_blocks = usize::try_from(max_blocks_per_leaf)
        .map_err(|_| "ec_spire.leaf_block_pruning_max_blocks_per_leaf exceeds usize".to_owned())?;
    if max_blocks >= summaries.len() {
        return Ok(None);
    }

    let score_started = observer.wants_candidate_timing().then(Instant::now);
    let mut scored_ranges = Vec::with_capacity(summaries.len());
    for summary in summaries {
        let summary_format = SpireAssignmentPayloadFormat::from_tag(summary.payload_format)?;
        if summary_format != scorer.payload_format() {
            return Err(format!(
                "ec_spire leaf V3 summary payload format {:?} does not match prepared scorer {:?}",
                summary_format,
                scorer.payload_format()
            ));
        }
        let row_end = summary
            .row_base
            .checked_add(summary.row_count)
            .ok_or_else(|| "ec_spire leaf V3 summary row range overflow".to_owned())?;
        if summary.gamma < 0.0 {
            return Err("ec_spire leaf V3 RaBitQ summary radius must be non-negative".to_owned());
        }
        let mean_ip = scorer.score_payload_ip(summary_format, 0.0, &summary.encoded_payload)?;
        let ip = mean_ip + scorer.query_l2_norm() * summary.gamma;
        if !ip.is_finite() {
            return Err(
                "ec_spire leaf V3 summary scorer returned a non-finite score".to_owned(),
            );
        }
        scored_ranges.push((
            ip,
            SpireLeafBlockRowRange {
                row_base: summary.row_base,
                row_end,
            },
        ));
    }
    if let Some(started) = score_started {
        observer.candidate_score_time(epoch, placement, elapsed_nanos_since(started));
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
    I: IntoIterator<Item = (u64, &'a SpirePlacementEntry, &'a [SpireLeafBlockSummary])>,
{
    if max_global_blocks <= 0 || scorer.payload_format() != SpireAssignmentPayloadFormat::RaBitQ {
        return Ok(None);
    }

    let leaves = leaves.into_iter().collect::<Vec<_>>();
    let summary_count = leaves
        .iter()
        .map(|(_leaf_pid, _placement, summaries)| summaries.len())
        .sum::<usize>();
    if summary_count == 0 {
        return Ok(None);
    }

    let max_blocks = usize::try_from(max_global_blocks)
        .map_err(|_| "ec_spire.leaf_block_pruning_max_global_blocks exceeds usize".to_owned())?;
    if max_blocks >= summary_count {
        return Ok(None);
    }

    let mut selected_by_leaf = leaves
        .iter()
        .filter(|(_leaf_pid, _placement, summaries)| !summaries.is_empty())
        .map(|(leaf_pid, _placement, _summaries)| (*leaf_pid, Vec::new()))
        .collect::<HashMap<_, _>>();
    let mut scored_ranges = Vec::with_capacity(summary_count);

    for (leaf_pid, placement, summaries) in leaves {
        let score_started = observer.wants_candidate_timing().then(Instant::now);
        for summary in summaries {
            let summary_format = SpireAssignmentPayloadFormat::from_tag(summary.payload_format)?;
            if summary_format != scorer.payload_format() {
                return Err(format!(
                    "ec_spire leaf V3 summary payload format {:?} does not match prepared scorer {:?}",
                    summary_format,
                    scorer.payload_format()
                ));
            }
            let row_end = summary
                .row_base
                .checked_add(summary.row_count)
                .ok_or_else(|| "ec_spire leaf V3 summary row range overflow".to_owned())?;
            let ip = scorer.score_payload_ip(summary_format, 0.0, &summary.encoded_payload)?;
            if !ip.is_finite() {
                return Err(
                    "ec_spire leaf V3 summary scorer returned a non-finite score".to_owned(),
                );
            }
            scored_ranges.push((
                ip,
                leaf_pid,
                SpireLeafBlockRowRange {
                    row_base: summary.row_base,
                    row_end,
                },
            ));
        }
        if let Some(started) = score_started {
            observer.candidate_score_time(epoch, placement, elapsed_nanos_since(started));
        }
    }

    scored_ranges.sort_by(|left, right| {
        right
            .0
            .partial_cmp(&left.0)
            .unwrap_or(Ordering::Equal)
            .then_with(|| left.1.cmp(&right.1))
            .then_with(|| left.2.row_base.cmp(&right.2.row_base))
    });
    scored_ranges.truncate(max_blocks);
    for (_score, leaf_pid, range) in scored_ranges {
        selected_by_leaf.entry(leaf_pid).or_default().push(range);
    }
    for ranges in selected_by_leaf.values_mut() {
        ranges.sort_by_key(|range| range.row_base);
    }

    Ok(Some(selected_by_leaf))
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

    Ok(Some(leaf_object))
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
        observer.candidate_score_time(epoch, placement, elapsed_nanos_since(started));
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
                            observer.candidate_score_time(
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
            observer.candidate_score_time(epoch, placement, elapsed_nanos_since(started));
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
            observer.candidate_score_time(epoch, placement, elapsed_nanos_since(started));
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
