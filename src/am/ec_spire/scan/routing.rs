fn load_snapshot_root_routing_object(
    snapshot: &SpireValidatedEpochSnapshot<'_>,
    object_store: &impl SpireObjectReader,
) -> Result<(u64, SpireRoutingPartitionObject), String> {
    let hierarchy = load_snapshot_routing_hierarchy(snapshot, object_store)?;
    Ok((hierarchy.root_pid, hierarchy.root_object))
}

fn load_snapshot_routing_hierarchy(
    snapshot: &SpireValidatedEpochSnapshot<'_>,
    object_store: &impl SpireObjectReader,
) -> Result<SpireLoadedRoutingHierarchy, String> {
    // This loader only applies snapshot visibility and kind filtering. Recursive
    // level and parent/child coherence is validated by require_recursive_internal_child
    // during descent, where the expected parent context is available.
    let mut root = None;
    let mut internal_objects_by_pid = HashMap::new();
    for manifest_entry in &snapshot.object_manifest().entries {
        let lookup = snapshot.require_lookup(manifest_entry.pid, "scan root routing load")?;
        let placement = lookup.placement;
        if should_skip_placement(snapshot.epoch_manifest().consistency_mode, placement.state)? {
            continue;
        }

        let header = object_store.read_object_header(placement)?;
        if header.kind != SpirePartitionObjectKind::Root {
            if header.kind == SpirePartitionObjectKind::Internal {
                let object = object_store.read_routing_object(placement)?;
                if internal_objects_by_pid
                    .insert(manifest_entry.pid, object)
                    .is_some()
                {
                    return Err(format!(
                        "ec_spire scan snapshot contains duplicate internal routing pid {}",
                        manifest_entry.pid
                    ));
                }
            }
            continue;
        }
        if root.is_some() {
            return Err("ec_spire scan snapshot contains multiple root routing objects".to_owned());
        }
        root = Some((
            manifest_entry.pid,
            object_store.read_routing_object(placement)?,
        ));
    }

    let (root_pid, root_object) = root
        .ok_or_else(|| "ec_spire scan snapshot has no available root routing object".to_owned())?;
    Ok(SpireLoadedRoutingHierarchy {
        root_pid,
        root_object,
        internal_objects_by_pid,
    })
}

fn route_root_object_to_leaf_pids(
    root_object: &SpireRoutingPartitionObject,
    query_vector: &[f32],
    nprobe: u32,
) -> Result<Vec<u64>, String> {
    if root_object.header.kind != SpirePartitionObjectKind::Root {
        return Err("ec_spire scan routing requires a root routing object".to_owned());
    }
    route_routing_object_to_child_pids(root_object, query_vector, nprobe)
}

fn route_routing_object_to_child_pids(
    routing_object: &SpireRoutingPartitionObject,
    query_vector: &[f32],
    nprobe: u32,
) -> Result<Vec<u64>, String> {
    if routing_object.header.kind != SpirePartitionObjectKind::Root
        && routing_object.header.kind != SpirePartitionObjectKind::Internal
    {
        return Err("ec_spire scan routing requires a routing object".to_owned());
    }
    if nprobe == 0 {
        return Err("ec_spire routed scan requires nprobe > 0".to_owned());
    }
    validate_routing_query_vector(query_vector, usize::from(routing_object.dimensions))?;

    let requested = usize::try_from(nprobe)
        .map_err(|_| "ec_spire routed scan nprobe exceeds usize".to_owned())?;

    let mut heap = BinaryHeap::with_capacity(requested.min(routing_object.child_count()));
    for child in routing_object.children() {
        let entry = SpireRouteCandidateHeapEntry {
            candidate: SpireRouteCandidate {
                centroid_index: child.centroid_index,
                child_pid: child.child_pid,
                ip_score: inner_product(query_vector, child.centroid),
            },
        };
        if heap.len() < requested {
            heap.push(entry);
            continue;
        }

        if heap
            .peek()
            .is_some_and(|worst| route_candidate_cmp(&entry.candidate, &worst.candidate).is_lt())
        {
            heap.pop();
            heap.push(entry);
        }
    }

    let mut scored_children = heap
        .into_iter()
        .map(|entry| entry.candidate)
        .collect::<Vec<_>>();
    scored_children.sort_by(route_candidate_cmp);

    Ok(scored_children
        .into_iter()
        .map(|candidate| candidate.child_pid)
        .collect())
}

fn route_recursive_routing_objects_to_leaf_pids(
    root_object: &SpireRoutingPartitionObject,
    routing_objects_by_pid: &HashMap<u64, SpireRoutingPartitionObject>,
    query_vector: &[f32],
    nprobe: u32,
) -> Result<Vec<u64>, String> {
    Ok(route_recursive_routing_objects_to_leaf_routes(
        root_object,
        routing_objects_by_pid,
        query_vector,
        nprobe,
    )?
    .into_iter()
    .map(|route| route.leaf_pid)
    .collect())
}

fn route_recursive_routing_objects_to_leaf_routes(
    root_object: &SpireRoutingPartitionObject,
    routing_objects_by_pid: &HashMap<u64, SpireRoutingPartitionObject>,
    query_vector: &[f32],
    nprobe: u32,
) -> Result<Vec<SpireRecursiveLeafRoute>, String> {
    if root_object.header.kind != SpirePartitionObjectKind::Root {
        return Err("ec_spire recursive scan routing requires a root routing object".to_owned());
    }
    let nprobe_policy = SpireConservativeRecursiveNprobePolicy::new(nprobe)?;
    if root_object.header.level == 1 {
        return Ok(route_root_object_to_leaf_pids(
            root_object,
            query_vector,
            nprobe_policy.nprobe_for_parent_level(root_object.header.level),
        )?
        .into_iter()
        .map(|leaf_pid| SpireRecursiveLeafRoute {
            leaf_pid,
            parent_pid: root_object.header.pid,
        })
        .collect());
    }

    let mut current_parents = vec![root_object.clone()];
    loop {
        let parent_level = current_parents[0].header.level;
        if parent_level == 1 {
            let mut leaf_routes = Vec::new();
            for parent in &current_parents {
                if parent.header.level != 1 {
                    return Err("ec_spire recursive scan routing parent levels drifted".to_owned());
                }
                leaf_routes.extend(
                    route_routing_object_to_child_pids(
                        parent,
                        query_vector,
                        nprobe_policy.nprobe_for_parent_level(parent.header.level),
                    )?
                    .into_iter()
                    .map(|leaf_pid| SpireRecursiveLeafRoute {
                        leaf_pid,
                        parent_pid: parent.header.pid,
                    }),
                );
            }
            return Ok(leaf_routes);
        }

        let mut next_parents = Vec::new();
        for parent in &current_parents {
            if parent.header.kind != SpirePartitionObjectKind::Root
                && parent.header.kind != SpirePartitionObjectKind::Internal
            {
                return Err("ec_spire recursive scan parent must be a routing object".to_owned());
            }
            if parent.header.level != parent_level {
                return Err("ec_spire recursive scan routing parent levels drifted".to_owned());
            }
            let child_pids = route_routing_object_to_child_pids(
                parent,
                query_vector,
                nprobe_policy.nprobe_for_parent_level(parent.header.level),
            )?;
            for child_pid in child_pids {
                let child =
                    require_recursive_internal_child(routing_objects_by_pid, child_pid, parent)?;
                next_parents.push((*child).clone());
            }
        }
        if next_parents.is_empty() {
            return Err("ec_spire recursive scan routing produced no internal children".to_owned());
        }
        current_parents = next_parents;
    }
}

fn count_recursive_routing_leaf_pids(
    root_object: &SpireRoutingPartitionObject,
    routing_objects_by_pid: &HashMap<u64, SpireRoutingPartitionObject>,
) -> Result<u32, String> {
    if root_object.header.kind != SpirePartitionObjectKind::Root {
        return Err("ec_spire recursive scan leaf count requires a root routing object".to_owned());
    }

    let mut current_parents = vec![root_object];
    loop {
        let parent_level = current_parents[0].header.level;
        if parent_level == 1 {
            let mut leaf_count = 0usize;
            for parent in &current_parents {
                if parent.header.level != 1 {
                    return Err(
                        "ec_spire recursive scan leaf count parent levels drifted".to_owned()
                    );
                }
                leaf_count = leaf_count
                    .checked_add(parent.child_count())
                    .ok_or_else(|| "ec_spire recursive scan leaf count overflow".to_owned())?;
            }
            return u32::try_from(leaf_count)
                .map_err(|_| "ec_spire recursive scan leaf count exceeds u32".to_owned());
        }

        let mut next_parents = Vec::new();
        for parent in &current_parents {
            if parent.header.kind != SpirePartitionObjectKind::Root
                && parent.header.kind != SpirePartitionObjectKind::Internal
            {
                return Err(
                    "ec_spire recursive scan leaf count parent must be a routing object".to_owned(),
                );
            }
            if parent.header.level != parent_level {
                return Err("ec_spire recursive scan leaf count parent levels drifted".to_owned());
            }
            for child in parent.children() {
                next_parents.push(require_recursive_internal_child(
                    routing_objects_by_pid,
                    child.child_pid,
                    parent,
                )?);
            }
        }
        if next_parents.is_empty() {
            return Err("ec_spire recursive scan leaf count found no internal children".to_owned());
        }
        current_parents = next_parents;
    }
}

fn require_recursive_internal_child<'a>(
    routing_objects_by_pid: &'a HashMap<u64, SpireRoutingPartitionObject>,
    child_pid: u64,
    parent: &SpireRoutingPartitionObject,
) -> Result<&'a SpireRoutingPartitionObject, String> {
    let child = routing_objects_by_pid.get(&child_pid).ok_or_else(|| {
        format!("ec_spire recursive scan missing internal routing child pid {child_pid}")
    })?;
    if child.header.kind != SpirePartitionObjectKind::Internal {
        return Err(format!(
            "ec_spire recursive scan child pid {child_pid} is not an internal routing object"
        ));
    }
    if child.header.parent_pid != parent.header.pid {
        return Err(format!(
            "ec_spire recursive scan child pid {child_pid} parent_pid {} does not match parent pid {}",
            child.header.parent_pid, parent.header.pid
        ));
    }
    if child.header.level.checked_add(1) != Some(parent.header.level) {
        return Err(format!(
            "ec_spire recursive scan child pid {child_pid} level {} is not one below parent level {}",
            child.header.level, parent.header.level
        ));
    }
    Ok(child)
}

fn route_candidate_cmp(left: &SpireRouteCandidate, right: &SpireRouteCandidate) -> Ordering {
    right
        .ip_score
        .total_cmp(&left.ip_score)
        .then_with(|| left.centroid_index.cmp(&right.centroid_index))
        .then_with(|| left.child_pid.cmp(&right.child_pid))
}

fn validate_routing_query_vector(query_vector: &[f32], dimensions: usize) -> Result<(), String> {
    if query_vector.len() != dimensions {
        return Err(format!(
            "ec_spire vector dimensions mismatch: got {}, expected {dimensions}",
            query_vector.len()
        ));
    }
    if query_vector.iter().any(|value| !value.is_finite()) {
        return Err("ec_spire vector contains a non-finite value".to_owned());
    }
    let norm_sq = query_vector
        .iter()
        .map(|value| (*value as f64) * (*value as f64))
        .sum::<f64>();
    if norm_sq <= f64::EPSILON {
        return Err("ec_spire spherical routing requires non-zero vectors".to_owned());
    }
    Ok(())
}

fn inner_product(left: &[f32], right: &[f32]) -> f32 {
    left.iter()
        .zip(right.iter())
        .map(|(left, right)| left * right)
        .sum()
}
