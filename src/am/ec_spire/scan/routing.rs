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

#[derive(Debug, Clone, PartialEq)]
struct SpireTopGraphRoute {
    node_ordinal: u32,
    centroid_ordinal: u32,
    child_pid: u64,
    distance: f32,
}

fn route_top_graph_to_child_pids(
    root_object: &SpireRoutingPartitionObject,
    top_graph: &SpireTopGraphBuildDraft,
    query_vector: &[f32],
    search_list_size: u32,
    route_count: u32,
) -> Result<Vec<u64>, String> {
    Ok(route_top_graph_to_routes(
        root_object,
        top_graph,
        query_vector,
        search_list_size,
        route_count,
    )?
    .into_iter()
    .map(|route| route.child_pid)
    .collect())
}

fn route_top_graph_to_routes(
    root_object: &SpireRoutingPartitionObject,
    top_graph: &SpireTopGraphBuildDraft,
    query_vector: &[f32],
    search_list_size: u32,
    route_count: u32,
) -> Result<Vec<SpireTopGraphRoute>, String> {
    validate_top_graph_route_inputs(
        root_object,
        top_graph,
        query_vector,
        search_list_size,
        route_count,
    )?;
    let search_list_size = usize::try_from(search_list_size)
        .map_err(|_| "ec_spire top graph search list size exceeds usize".to_owned())?;
    let route_count = usize::try_from(route_count)
        .map_err(|_| "ec_spire top graph route count exceeds usize".to_owned())?;
    let graph = crate::am::VamanaGraph {
        neighbors: top_graph
            .nodes
            .iter()
            .map(|node| node.neighbors.clone())
            .collect(),
        max_degree: usize::try_from(top_graph.graph_degree)
            .map_err(|_| "ec_spire top graph degree exceeds usize".to_owned())?,
    };
    let query_distance_offset = max_query_centroid_inner_product(root_object, query_vector)?;
    let search = crate::am::greedy_search(&graph, top_graph.entry_node, search_list_size, |node| {
        let centroid = root_object
            .child_centroid(node as usize)
            .expect("top graph route validation checked node centroid");
        (query_distance_offset - inner_product(query_vector, centroid)).max(0.0)
    });
    let mut routes = search
        .frontier
        .into_iter()
        .map(|candidate| {
            let node = &top_graph.nodes[candidate.node as usize];
            SpireTopGraphRoute {
                node_ordinal: candidate.node,
                centroid_ordinal: node.centroid_ordinal,
                child_pid: node.child_pid,
                distance: candidate.distance,
            }
        })
        .collect::<Vec<_>>();
    routes.sort_by(top_graph_route_cmp);
    routes.truncate(route_count);
    Ok(routes)
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

    let scored_children = rank_centroid_routes_by_ip(
        "ec_spire scan routing",
        query_vector,
        usize::from(routing_object.dimensions),
        routing_object.children().map(|child| SpireCentroidRouteInput {
            centroid_index: child.centroid_index,
            pid: child.child_pid,
            centroid: child.centroid,
        }),
    )?;

    let mut selected_pids = Vec::with_capacity(requested.min(scored_children.len()));
    for child in scored_children {
        if selected_pids.contains(&child.pid) {
            continue;
        }
        selected_pids.push(child.pid);
        if selected_pids.len() == requested {
            break;
        }
    }
    Ok(selected_pids)
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

fn validate_top_graph_route_inputs(
    root_object: &SpireRoutingPartitionObject,
    top_graph: &SpireTopGraphBuildDraft,
    query_vector: &[f32],
    search_list_size: u32,
    route_count: u32,
) -> Result<(), String> {
    if root_object.header.kind != SpirePartitionObjectKind::Root {
        return Err("ec_spire top graph routing requires a root routing object".to_owned());
    }
    if root_object.header.pid != top_graph.root_pid {
        return Err(format!(
            "ec_spire top graph root pid {} does not match routing root pid {}",
            top_graph.root_pid, root_object.header.pid
        ));
    }
    if root_object.dimensions != top_graph.dimensions {
        return Err(format!(
            "ec_spire top graph dimensions {} do not match routing dimensions {}",
            top_graph.dimensions, root_object.dimensions
        ));
    }
    if usize::try_from(top_graph.node_count)
        .ok()
        .filter(|node_count| *node_count == top_graph.nodes.len())
        .is_none()
    {
        return Err(format!(
            "ec_spire top graph node count {} does not match node rows {}",
            top_graph.node_count,
            top_graph.nodes.len()
        ));
    }
    if top_graph.nodes.len() != root_object.child_count() {
        return Err(format!(
            "ec_spire top graph node count {} does not match routing child count {}",
            top_graph.nodes.len(),
            root_object.child_count()
        ));
    }
    if top_graph.entry_node >= top_graph.node_count {
        return Err(format!(
            "ec_spire top graph entry node {} is outside node count {}",
            top_graph.entry_node, top_graph.node_count
        ));
    }
    if search_list_size == 0 {
        return Err("ec_spire top graph search list size must be greater than 0".to_owned());
    }
    if route_count == 0 {
        return Err("ec_spire top graph route count must be greater than 0".to_owned());
    }
    if search_list_size < route_count {
        return Err(
            "ec_spire top graph search list size must be at least route count".to_owned(),
        );
    }
    validate_routing_query_vector(query_vector, usize::from(root_object.dimensions))?;

    let node_count = top_graph.nodes.len();
    for (node_index, (graph_node, routing_child)) in top_graph
        .nodes
        .iter()
        .zip(root_object.children())
        .enumerate()
    {
        if graph_node.child_pid != routing_child.child_pid {
            return Err(format!(
                "ec_spire top graph node {node_index} child pid {} does not match routing child pid {}",
                graph_node.child_pid, routing_child.child_pid
            ));
        }
        if graph_node.centroid_ordinal != routing_child.centroid_index {
            return Err(format!(
                "ec_spire top graph node {node_index} centroid ordinal {} does not match routing centroid ordinal {}",
                graph_node.centroid_ordinal, routing_child.centroid_index
            ));
        }
        for &neighbor in &graph_node.neighbors {
            if usize::try_from(neighbor)
                .ok()
                .filter(|neighbor| *neighbor < node_count)
                .is_none()
            {
                return Err(format!(
                    "ec_spire top graph node {node_index} neighbor {neighbor} is outside node count {node_count}"
                ));
            }
        }
    }
    Ok(())
}

fn max_query_centroid_inner_product(
    root_object: &SpireRoutingPartitionObject,
    query_vector: &[f32],
) -> Result<f32, String> {
    let mut max_ip = f32::NEG_INFINITY;
    for child in root_object.children() {
        max_ip = max_ip.max(inner_product(query_vector, child.centroid));
    }
    if !max_ip.is_finite() {
        return Err("ec_spire top graph query-to-centroid score must be finite".to_owned());
    }
    Ok(max_ip)
}

fn top_graph_route_cmp(left: &SpireTopGraphRoute, right: &SpireTopGraphRoute) -> Ordering {
    left.distance
        .total_cmp(&right.distance)
        .then_with(|| left.centroid_ordinal.cmp(&right.centroid_ordinal))
        .then_with(|| left.child_pid.cmp(&right.child_pid))
        .then_with(|| left.node_ordinal.cmp(&right.node_ordinal))
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
