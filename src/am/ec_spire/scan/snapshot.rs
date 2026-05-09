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
) -> Result<Vec<SpireRoutedLeafScanRows>, String> {
    let snapshot = SpireValidatedEpochSnapshot::from_snapshot(*snapshot)?;
    let hierarchy = load_snapshot_routing_hierarchy(&snapshot, object_store)?;
    let (_top_graph_pid, top_graph) = load_snapshot_top_graph_object(&snapshot, object_store)?
        .ok_or_else(|| "ec_spire scan snapshot has no available top graph object".to_owned())?;
    let leaf_routes = route_top_graph_object_to_leaf_routes(
        &hierarchy.root_object,
        &hierarchy.internal_objects_by_pid,
        &top_graph,
        query_vector,
        search_list_size,
        top_route_count,
        nprobe_policy,
        route_budget,
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
        collect_top_graph_routing_level_diagnostics(
            &hierarchy.root_object,
            &hierarchy.internal_objects_by_pid,
            &top_graph,
            query.values(),
            top_graph_plan.search_list_size.unwrap_or(scan_plan.nprobe),
            scan_plan.nprobe,
            &scan_plan.recursive_nprobe_policy,
            scan_plan.recursive_route_budget,
        )?
    } else {
        collect_recursive_routing_level_diagnostics_with_budget(
            &hierarchy.root_object,
            &hierarchy.internal_objects_by_pid,
            query.values(),
            &scan_plan.recursive_nprobe_policy,
            scan_plan.recursive_route_budget,
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
    payload_format: SpireAssignmentPayloadFormat,
    dedupe_mode: SpireCandidateDedupeMode,
    limit: Option<usize>,
    observer: &mut impl SpireRoutedScanObserver,
) -> Result<Vec<SpireScoredScanCandidate>, String> {
    let scorer =
        SpirePreparedAssignmentScorer::prepare(payload_format, query_vector.len(), query_vector)?;
    let leaf_routes = route_recursive_routing_objects_to_leaf_routes_with_budget(
        &hierarchy.root_object,
        &hierarchy.internal_objects_by_pid,
        query_vector,
        nprobe_policy,
        route_budget,
    )?;
    collect_validated_quantized_leaf_route_candidates(
        snapshot,
        object_store,
        leaf_routes,
        &scorer,
        dedupe_mode,
        limit,
        observer,
    )
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
        })
        .collect();
    collect_validated_quantized_leaf_route_candidates(
        snapshot,
        object_store,
        leaf_routes,
        &scorer,
        dedupe_mode,
        limit,
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
    observer: &mut impl SpireRoutedScanObserver,
) -> Result<Vec<SpireScoredScanCandidate>, String> {
    if limit == Some(0) {
        return Ok(Vec::new());
    }

    let mut candidates = Vec::new();
    let mut candidates_by_vec_id = match dedupe_mode {
        SpireCandidateDedupeMode::NoReplicaDedupeDisabled => None,
        SpireCandidateDedupeMode::VecIdDedupeEnabled => Some(HashMap::new()),
    };
    let delta_routes = collect_snapshot_delta_object_routes(snapshot, object_store)?;
    let mut delta_routes_by_parent = HashMap::<u64, Vec<SpireDeltaObjectRoute>>::new();
    let route_groups =
        group_leaf_and_delta_reads_by_local_store(snapshot, leaf_routes, delta_routes, observer)?;
    prefetch_store_object_read_groups(object_store, &route_groups)?;
    for route_group in &route_groups {
        for route in &route_group.delta_routes {
            delta_routes_by_parent
                .entry(route.parent_leaf_pid)
                .or_default()
                .push(*route);
        }
    }

    for route_group in route_groups {
        for route in route_group.leaf_routes {
            let leaf_pid = route.leaf_pid;
            let leaf_delta_routes = delta_routes_by_parent
                .get(&leaf_pid)
                .map(Vec::as_slice)
                .unwrap_or(&[]);
            let deleted_vec_ids = collect_delta_delete_vec_ids_for_routes(
                snapshot,
                object_store,
                leaf_delta_routes,
                observer,
            )?;
            append_quantized_leaf_candidates_for_pid(
                snapshot,
                object_store,
                route,
                scorer,
                &deleted_vec_ids,
                &mut candidates,
                &mut candidates_by_vec_id,
                observer,
            )?;
            append_quantized_delta_candidates_for_routes(
                snapshot,
                object_store,
                leaf_delta_routes,
                scorer,
                &deleted_vec_ids,
                &mut candidates,
                &mut candidates_by_vec_id,
                observer,
            )?;
        }
    }

    if let Some(candidates_by_vec_id) = candidates_by_vec_id {
        candidates.extend(candidates_by_vec_id.into_values());
    }

    let ranked = rank_bounded_scored_candidates(candidates, limit);
    observe_candidate_winners(snapshot, observer, &ranked)?;
    Ok(ranked)
}

fn prefetch_store_object_read_groups(
    object_store: &impl SpireObjectReader,
    route_groups: &[SpireStoreObjectReadGroup],
) -> Result<(), String> {
    let mut placements = Vec::new();
    for route_group in route_groups {
        collect_store_object_read_group_prefetch_placements(route_group, &mut placements);
    }
    object_store.prefetch_objects(&placements)
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
            placement: *placement,
            object_version: lookup.manifest_entry.object_version,
        };
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
