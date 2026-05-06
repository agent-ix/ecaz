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
    let snapshot = SpireValidatedEpochSnapshot::from_snapshot(*snapshot)?;
    let hierarchy = load_snapshot_routing_hierarchy(&snapshot, object_store)?;
    let leaf_routes = route_recursive_routing_objects_to_leaf_routes(
        &hierarchy.root_object,
        &hierarchy.internal_objects_by_pid,
        query_vector,
        nprobe,
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
    let snapshot = SpireValidatedEpochSnapshot::from_snapshot(*snapshot)?;
    let hierarchy = load_snapshot_routing_hierarchy(&snapshot, object_store)?;
    let mut observer = SpireNoopRoutedScanObserver;
    collect_validated_recursive_quantized_routed_probe_candidates(
        &snapshot,
        object_store,
        query_vector,
        &hierarchy,
        nprobe,
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
    nprobe: u32,
    payload_format: SpireAssignmentPayloadFormat,
    dedupe_mode: SpireCandidateDedupeMode,
    limit: Option<usize>,
    observer: &mut impl SpireRoutedScanObserver,
) -> Result<Vec<SpireScoredScanCandidate>, String> {
    let scorer =
        SpirePreparedAssignmentScorer::prepare(payload_format, query_vector.len(), query_vector)?;
    let leaf_routes = route_recursive_routing_objects_to_leaf_routes(
        &hierarchy.root_object,
        &hierarchy.internal_objects_by_pid,
        query_vector,
        nprobe,
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
    for route_group in group_leaf_routes_by_local_store(snapshot, leaf_routes)? {
        for route in route_group.routes {
            let leaf_pid = route.leaf_pid;
            let deleted_vec_ids = collect_delta_delete_vec_ids_for_base_pid(
                snapshot,
                object_store,
                leaf_pid,
                observer,
            )?;
            append_quantized_leaf_candidates_for_pid(
                snapshot,
                object_store,
                leaf_pid,
                route.parent_pid,
                scorer,
                &deleted_vec_ids,
                &mut candidates,
                &mut candidates_by_vec_id,
                observer,
            )?;
            append_quantized_delta_candidates_for_base_pid(
                snapshot,
                object_store,
                leaf_pid,
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

    Ok(rank_bounded_scored_candidates(candidates, limit))
}

fn group_leaf_routes_by_local_store(
    snapshot: &SpireValidatedEpochSnapshot<'_>,
    leaf_routes: Vec<SpireRecursiveLeafRoute>,
) -> Result<Vec<SpireStoreLeafRouteGroup>, String> {
    Ok(
        group_leaf_and_delta_reads_by_local_store(snapshot, leaf_routes, Vec::new())?
            .into_iter()
            .map(|group| SpireStoreLeafRouteGroup {
                node_id: group.node_id,
                local_store_id: group.local_store_id,
                routes: group.leaf_routes,
            })
            .collect(),
    )
}

fn group_leaf_and_delta_reads_by_local_store(
    snapshot: &SpireValidatedEpochSnapshot<'_>,
    leaf_routes: Vec<SpireRecursiveLeafRoute>,
    delta_routes: Vec<SpireDeltaObjectRoute>,
) -> Result<Vec<SpireStoreObjectReadGroup>, String> {
    let selected_leaf_pids = leaf_routes
        .iter()
        .map(|route| route.leaf_pid)
        .collect::<HashSet<_>>();
    let mut reads_by_store = BTreeMap::<(u32, u32), SpireStoreObjectReadGroup>::new();

    for route in leaf_routes {
        let lookup = snapshot.require_lookup(route.leaf_pid, "scan leaf route grouping")?;
        let placement = lookup.placement;
        reads_by_store
            .entry((placement.node_id, placement.local_store_id))
            .or_insert_with(|| SpireStoreObjectReadGroup {
                node_id: placement.node_id,
                local_store_id: placement.local_store_id,
                leaf_routes: Vec::new(),
                delta_routes: Vec::new(),
            })
            .leaf_routes
            .push(route);
    }

    for route in delta_routes {
        if !selected_leaf_pids.contains(&route.parent_leaf_pid) {
            continue;
        }
        let lookup = snapshot.require_lookup(route.delta_pid, "scan delta route grouping")?;
        let placement = lookup.placement;
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
