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

fn load_snapshot_top_graph_object(
    snapshot: &SpireValidatedEpochSnapshot<'_>,
    object_store: &impl SpireObjectReader,
) -> Result<Option<(u64, SpireTopGraphPartitionObject)>, String> {
    let mut top_graph = None;
    for manifest_entry in &snapshot.object_manifest().entries {
        let lookup = snapshot.require_lookup(manifest_entry.pid, "scan top graph load")?;
        let placement = lookup.placement;
        if should_skip_placement(snapshot.epoch_manifest().consistency_mode, placement.state)? {
            continue;
        }

        let header = object_store.read_object_header(placement)?;
        if header.kind != SpirePartitionObjectKind::TopGraph {
            continue;
        }
        if top_graph.is_some() {
            return Err("ec_spire scan snapshot contains multiple top graph objects".to_owned());
        }
        top_graph = Some((
            manifest_entry.pid,
            object_store.read_top_graph_object(placement)?,
        ));
    }
    Ok(top_graph)
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SpireAdaptiveNprobeChoice {
    effective_nprobe: u32,
    effective_nprobe_source: &'static str,
    decision: &'static str,
}

impl SpireAdaptiveNprobeChoice {
    fn disabled(requested_nprobe: u32) -> Self {
        Self {
            effective_nprobe: requested_nprobe,
            effective_nprobe_source: "configured",
            decision: "disabled",
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct SpireTopGraphRouteNode<'a> {
    child_pid: u64,
    centroid_ordinal: u32,
    neighbors: &'a [u32],
}

trait SpireTopGraphRouteView {
    fn root_pid(&self) -> u64;
    fn dimensions(&self) -> u16;
    fn node_count(&self) -> u32;
    fn graph_degree(&self) -> u32;
    fn entry_node(&self) -> u32;
    fn nodes_len(&self) -> usize;
    fn route_node(&self, index: usize) -> SpireTopGraphRouteNode<'_>;
}

struct SpireTopGraphGreedyView<'a, T: SpireTopGraphRouteView + ?Sized> {
    top_graph: &'a T,
}

impl<T: SpireTopGraphRouteView + ?Sized> crate::am::VamanaGraphView
    for SpireTopGraphGreedyView<'_, T>
{
    fn node_count(&self) -> usize {
        self.top_graph.nodes_len()
    }

    fn neighbors(&self, node: u32) -> &[u32] {
        self.top_graph.route_node(node as usize).neighbors
    }
}

impl SpireTopGraphRouteView for SpireTopGraphBuildDraft {
    fn root_pid(&self) -> u64 {
        self.root_pid
    }

    fn dimensions(&self) -> u16 {
        self.dimensions
    }

    fn node_count(&self) -> u32 {
        self.node_count
    }

    fn graph_degree(&self) -> u32 {
        self.graph_degree
    }

    fn entry_node(&self) -> u32 {
        self.entry_node
    }

    fn nodes_len(&self) -> usize {
        self.nodes.len()
    }

    fn route_node(&self, index: usize) -> SpireTopGraphRouteNode<'_> {
        let node = &self.nodes[index];
        SpireTopGraphRouteNode {
            child_pid: node.child_pid,
            centroid_ordinal: node.centroid_ordinal,
            neighbors: &node.neighbors,
        }
    }
}

impl SpireTopGraphRouteView for SpireTopGraphPartitionObject {
    fn root_pid(&self) -> u64 {
        self.root_pid
    }

    fn dimensions(&self) -> u16 {
        self.dimensions
    }

    fn node_count(&self) -> u32 {
        self.header.child_count
    }

    fn graph_degree(&self) -> u32 {
        self.graph_degree
    }

    fn entry_node(&self) -> u32 {
        self.entry_node
    }

    fn nodes_len(&self) -> usize {
        self.nodes.len()
    }

    fn route_node(&self, index: usize) -> SpireTopGraphRouteNode<'_> {
        let node = &self.nodes[index];
        SpireTopGraphRouteNode {
            child_pid: node.child_pid,
            centroid_ordinal: node.centroid_ordinal,
            neighbors: &node.neighbors,
        }
    }
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

fn route_top_graph_object_to_child_pids(
    root_object: &SpireRoutingPartitionObject,
    top_graph: &SpireTopGraphPartitionObject,
    query_vector: &[f32],
    search_list_size: u32,
    route_count: u32,
) -> Result<Vec<u64>, String> {
    Ok(route_top_graph_view_to_routes(
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

fn route_top_graph_object_to_leaf_routes(
    root_object: &SpireRoutingPartitionObject,
    routing_objects_by_pid: &HashMap<u64, SpireRoutingPartitionObject>,
    top_graph: &SpireTopGraphPartitionObject,
    query_vector: &[f32],
    search_list_size: u32,
    top_route_count: u32,
    nprobe_policy: &SpireRecursiveNprobePolicy,
    route_budget: SpireRecursiveRouteBudget,
) -> Result<Vec<SpireRecursiveLeafRoute>, String> {
    let (selected_child_routes, _choice) = route_top_graph_view_to_routes_with_policy(
        root_object,
        top_graph,
        query_vector,
        search_list_size,
        top_route_count,
        nprobe_policy,
    )?;
    if root_object.header.level == 1 {
        let mut seen_leaf_pids = HashSet::new();
        let mut leaf_routes = Vec::new();
        for route in selected_child_routes {
            if leaf_routes.len() >= route_budget.max_leaf_routes {
                break;
            }
            if !seen_leaf_pids.insert(route.child_pid) {
                continue;
            }
            leaf_routes.push(SpireRecursiveLeafRoute {
                leaf_pid: route.child_pid,
                parent_pid: root_object.header.pid,
            });
        }
        return Ok(leaf_routes);
    }

    let mut current_parents = Vec::with_capacity(
        selected_child_routes
            .len()
            .min(route_budget.beam_width),
    );
    let mut seen_child_pids = HashSet::new();
    for route in selected_child_routes {
        if current_parents.len() >= route_budget.beam_width {
            break;
        }
        if !seen_child_pids.insert(route.child_pid) {
            continue;
        }
        let child = require_recursive_internal_child(
            routing_objects_by_pid,
            route.child_pid,
            root_object,
        )?;
        current_parents.push(SpireRecursiveParentRoute {
            parent: (*child).clone(),
            path_score: -route.distance,
        });
    }
    route_recursive_parent_objects_to_leaf_routes(
        current_parents,
        routing_objects_by_pid,
        query_vector,
        nprobe_policy,
        route_budget,
    )
}

fn route_top_graph_to_routes(
    root_object: &SpireRoutingPartitionObject,
    top_graph: &SpireTopGraphBuildDraft,
    query_vector: &[f32],
    search_list_size: u32,
    route_count: u32,
) -> Result<Vec<SpireTopGraphRoute>, String> {
    route_top_graph_view_to_routes(
        root_object,
        top_graph,
        query_vector,
        search_list_size,
        route_count,
    )
}

fn route_top_graph_view_to_routes(
    root_object: &SpireRoutingPartitionObject,
    top_graph: &impl SpireTopGraphRouteView,
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
    // The view borrows the scan-owned top-graph snapshot only for this greedy
    // traversal; future scan caching should preserve the same lifetime shape.
    let graph = SpireTopGraphGreedyView { top_graph };
    let search =
        crate::am::greedy_search_view(&graph, top_graph.entry_node(), search_list_size, |node| {
            let centroid = root_object
                .child_centroid(node as usize)
                .expect("top graph route validation checked node centroid");
            -inner_product(query_vector, centroid)
        });
    let mut routes = search
        .frontier
        .into_iter()
        .map(|candidate| {
            let node = top_graph.route_node(candidate.node as usize);
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

fn route_top_graph_view_to_routes_with_policy(
    root_object: &SpireRoutingPartitionObject,
    top_graph: &impl SpireTopGraphRouteView,
    query_vector: &[f32],
    search_list_size: u32,
    requested_route_count: u32,
    nprobe_policy: &SpireRecursiveNprobePolicy,
) -> Result<(Vec<SpireTopGraphRoute>, SpireAdaptiveNprobeChoice), String> {
    validate_top_graph_route_inputs(
        root_object,
        top_graph,
        query_vector,
        search_list_size,
        requested_route_count,
    )?;
    let search_list_size = usize::try_from(search_list_size)
        .map_err(|_| "ec_spire top graph search list size exceeds usize".to_owned())?;
    let requested_route_count = usize::try_from(requested_route_count)
        .map_err(|_| "ec_spire top graph route count exceeds usize".to_owned())?;
    let graph = SpireTopGraphGreedyView { top_graph };
    let search =
        crate::am::greedy_search_view(&graph, top_graph.entry_node(), search_list_size, |node| {
            let centroid = root_object
                .child_centroid(node as usize)
                .expect("top graph route validation checked node centroid");
            -inner_product(query_vector, centroid)
        });
    let mut routes = search
        .frontier
        .into_iter()
        .map(|candidate| {
            let node = top_graph.route_node(candidate.node as usize);
            SpireTopGraphRoute {
                node_ordinal: candidate.node,
                centroid_ordinal: node.centroid_ordinal,
                child_pid: node.child_pid,
                distance: candidate.distance,
            }
        })
        .collect::<Vec<_>>();
    routes.sort_by(top_graph_route_cmp);
    let choice = choose_adaptive_nprobe_from_top_graph_routes(
        u32::try_from(requested_route_count)
            .map_err(|_| "ec_spire top graph route count exceeds u32".to_owned())?,
        nprobe_policy,
        &routes,
    );
    routes.truncate(usize::try_from(choice.effective_nprobe).map_err(|_| {
        "ec_spire adaptive top graph route count exceeds usize".to_owned()
    })?);
    Ok((routes, choice))
}

fn route_routing_object_to_child_pids(
    routing_object: &SpireRoutingPartitionObject,
    query_vector: &[f32],
    nprobe: u32,
) -> Result<Vec<u64>, String> {
    Ok(route_routing_object_to_child_routes(routing_object, query_vector, nprobe)?
        .into_iter()
        .map(|route| route.child_pid)
        .collect())
}

fn route_routing_object_to_child_routes(
    routing_object: &SpireRoutingPartitionObject,
    query_vector: &[f32],
    nprobe: u32,
) -> Result<Vec<SpireRoutingChildRoute>, String> {
    Ok(route_routing_object_to_child_routes_with_choice(
        routing_object,
        query_vector,
        nprobe,
        None,
    )?
    .0)
}

fn route_routing_object_to_child_routes_with_policy(
    routing_object: &SpireRoutingPartitionObject,
    query_vector: &[f32],
    nprobe_policy: &SpireRecursiveNprobePolicy,
) -> Result<(Vec<SpireRoutingChildRoute>, SpireAdaptiveNprobeChoice), String> {
    route_routing_object_to_child_routes_with_choice(
        routing_object,
        query_vector,
        nprobe_policy.nprobe_for_parent_level(routing_object.header.level),
        Some(nprobe_policy),
    )
}

fn route_routing_object_to_child_routes_with_choice(
    routing_object: &SpireRoutingPartitionObject,
    query_vector: &[f32],
    nprobe: u32,
    nprobe_policy: Option<&SpireRecursiveNprobePolicy>,
) -> Result<(Vec<SpireRoutingChildRoute>, SpireAdaptiveNprobeChoice), String> {
    if routing_object.header.kind != SpirePartitionObjectKind::Root
        && routing_object.header.kind != SpirePartitionObjectKind::Internal
    {
        return Err("ec_spire scan routing requires a routing object".to_owned());
    }
    if nprobe == 0 {
        return Err("ec_spire routed scan requires nprobe > 0".to_owned());
    }
    validate_routing_query_vector(query_vector, usize::from(routing_object.dimensions))?;

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
    let choice = match nprobe_policy {
        Some(policy) => choose_adaptive_nprobe_from_ranked_routes(nprobe, policy, &scored_children),
        None => SpireAdaptiveNprobeChoice::disabled(nprobe),
    };

    let effective = usize::try_from(choice.effective_nprobe)
        .map_err(|_| "ec_spire adaptive nprobe exceeds usize".to_owned())?;
    let mut selected_routes = Vec::with_capacity(effective.min(scored_children.len()));
    let mut selected_pids = HashSet::new();
    for child in scored_children {
        if !selected_pids.insert(child.pid) {
            continue;
        }
        selected_routes.push(SpireRoutingChildRoute {
            centroid_index: child.centroid_index,
            child_pid: child.pid,
            score: child.score,
        });
        if selected_routes.len() == effective {
            break;
        }
    }
    Ok((selected_routes, choice))
}

fn choose_adaptive_nprobe_from_ranked_routes(
    requested_nprobe: u32,
    nprobe_policy: &SpireRecursiveNprobePolicy,
    ranked_routes: &[SpireRankedCentroidRoute],
) -> SpireAdaptiveNprobeChoice {
    choose_adaptive_nprobe_by_gap(
        requested_nprobe,
        nprobe_policy,
        ranked_routes.len(),
        |index| ranked_routes[index - 1].score - ranked_routes[index].score,
    )
}

fn choose_adaptive_nprobe_from_top_graph_routes(
    requested_nprobe: u32,
    nprobe_policy: &SpireRecursiveNprobePolicy,
    routes: &[SpireTopGraphRoute],
) -> SpireAdaptiveNprobeChoice {
    choose_adaptive_nprobe_by_gap(
        requested_nprobe,
        nprobe_policy,
        routes.len(),
        |index| routes[index].distance - routes[index - 1].distance,
    )
}

fn choose_adaptive_nprobe_by_gap<F>(
    requested_nprobe: u32,
    nprobe_policy: &SpireRecursiveNprobePolicy,
    ranked_len: usize,
    score_gap_at: F,
) -> SpireAdaptiveNprobeChoice
where
    F: Fn(usize) -> f32,
{
    if !nprobe_policy.adaptive_nprobe() {
        return SpireAdaptiveNprobeChoice::disabled(requested_nprobe);
    }
    if requested_nprobe <= 1 {
        return SpireAdaptiveNprobeChoice {
            effective_nprobe: requested_nprobe,
            effective_nprobe_source: "adaptive",
            decision: "kept_minimum",
        };
    }

    let adaptive_nprobe = (requested_nprobe / 2).max(1);
    let adaptive_nprobe_usize = adaptive_nprobe as usize;
    if ranked_len <= adaptive_nprobe_usize {
        return SpireAdaptiveNprobeChoice {
            effective_nprobe: requested_nprobe,
            effective_nprobe_source: "adaptive",
            decision: "kept_exhausted_frontier",
        };
    }

    let raw_gap_micros = score_gap_at(adaptive_nprobe_usize) * 1_000_000.0;
    let gap_micros = if raw_gap_micros.is_finite() && raw_gap_micros > 0.0 {
        raw_gap_micros.round() as i32
    } else {
        0
    };
    if gap_micros >= nprobe_policy.adaptive_score_gap_micros() {
        SpireAdaptiveNprobeChoice {
            effective_nprobe: adaptive_nprobe,
            effective_nprobe_source: "adaptive",
            decision: "reduced_score_gap",
        }
    } else {
        SpireAdaptiveNprobeChoice {
            effective_nprobe: requested_nprobe,
            effective_nprobe_source: "adaptive",
            decision: "kept_gap_below_threshold",
        }
    }
}

fn summarize_adaptive_choices(
    choices: &[SpireAdaptiveNprobeChoice],
) -> (u32, &'static str, &'static str) {
    if choices.is_empty() {
        return (0, "configured", "not_expanded");
    }
    let effective_nprobe = choices
        .iter()
        .map(|choice| choice.effective_nprobe)
        .max()
        .unwrap_or(0);
    let effective_nprobe_source = if choices
        .iter()
        .any(|choice| choice.effective_nprobe_source == "adaptive")
    {
        "adaptive"
    } else {
        "configured"
    };
    let first_decision = choices[0].decision;
    let adaptive_nprobe_decision = if choices
        .iter()
        .all(|choice| choice.decision == first_decision)
    {
        first_decision
    } else {
        "mixed"
    };
    (
        effective_nprobe,
        effective_nprobe_source,
        adaptive_nprobe_decision,
    )
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
    let nprobe_policy = SpireRecursiveNprobePolicy::conservative(nprobe)?;
    route_recursive_routing_objects_to_leaf_routes_with_policy(
        root_object,
        routing_objects_by_pid,
        query_vector,
        &nprobe_policy,
    )
}

fn route_recursive_routing_objects_to_leaf_routes_with_policy(
    root_object: &SpireRoutingPartitionObject,
    routing_objects_by_pid: &HashMap<u64, SpireRoutingPartitionObject>,
    query_vector: &[f32],
    nprobe_policy: &SpireRecursiveNprobePolicy,
) -> Result<Vec<SpireRecursiveLeafRoute>, String> {
    route_recursive_routing_objects_to_leaf_routes_with_budget(
        root_object,
        routing_objects_by_pid,
        query_vector,
        nprobe_policy,
        SpireRecursiveRouteBudget::unbounded(),
    )
}

fn route_recursive_routing_objects_to_leaf_routes_with_budget(
    root_object: &SpireRoutingPartitionObject,
    routing_objects_by_pid: &HashMap<u64, SpireRoutingPartitionObject>,
    query_vector: &[f32],
    nprobe_policy: &SpireRecursiveNprobePolicy,
    route_budget: SpireRecursiveRouteBudget,
) -> Result<Vec<SpireRecursiveLeafRoute>, String> {
    if root_object.header.kind != SpirePartitionObjectKind::Root {
        return Err("ec_spire recursive scan routing requires a root routing object".to_owned());
    }
    if root_object.header.level == 1 {
        let (routes, _choice) = route_routing_object_to_child_routes_with_policy(
            root_object,
            query_vector,
            nprobe_policy,
        )?;
        return Ok(routes
        .into_iter()
        .map(|route| route.child_pid)
        .take(route_budget.max_leaf_routes)
        .map(|leaf_pid| SpireRecursiveLeafRoute {
            leaf_pid,
            parent_pid: root_object.header.pid,
        })
        .collect());
    }

    route_recursive_parent_objects_to_leaf_routes(
        vec![SpireRecursiveParentRoute {
            parent: root_object.clone(),
            path_score: 0.0,
        }],
        routing_objects_by_pid,
        query_vector,
        nprobe_policy,
        route_budget,
    )
}

fn collect_recursive_routing_level_diagnostics_with_budget(
    root_object: &SpireRoutingPartitionObject,
    routing_objects_by_pid: &HashMap<u64, SpireRoutingPartitionObject>,
    query_vector: &[f32],
    nprobe_policy: &SpireRecursiveNprobePolicy,
    route_budget: SpireRecursiveRouteBudget,
) -> Result<Vec<SpireRoutingLevelDiagnostics>, String> {
    if root_object.header.kind != SpirePartitionObjectKind::Root {
        return Err("ec_spire recursive routing diagnostics require a root routing object".to_owned());
    }
    collect_recursive_parent_routing_level_diagnostics(
        vec![SpireRecursiveParentRoute {
            parent: root_object.clone(),
            path_score: 0.0,
        }],
        routing_objects_by_pid,
        query_vector,
        nprobe_policy,
        route_budget,
        0,
    )
}

fn collect_top_graph_routing_level_diagnostics(
    root_object: &SpireRoutingPartitionObject,
    routing_objects_by_pid: &HashMap<u64, SpireRoutingPartitionObject>,
    top_graph: &SpireTopGraphPartitionObject,
    query_vector: &[f32],
    search_list_size: u32,
    top_route_count: u32,
    nprobe_policy: &SpireRecursiveNprobePolicy,
    route_budget: SpireRecursiveRouteBudget,
) -> Result<Vec<SpireRoutingLevelDiagnostics>, String> {
    let (selected_child_routes, top_choice) = route_top_graph_view_to_routes_with_policy(
        root_object,
        top_graph,
        query_vector,
        search_list_size,
        top_route_count,
        nprobe_policy,
    )?;
    let top_choices = [top_choice];
    let (effective_nprobe, effective_nprobe_source, adaptive_nprobe_decision) =
        summarize_adaptive_choices(&top_choices);
    let selected_child_count = selected_child_routes.len();
    if root_object.header.level == 1 {
        let unique_leaf_count = unique_child_pid_count(&selected_child_routes);
        return Ok(vec![SpireRoutingLevelDiagnostics {
            level: root_object.header.level,
            effective_nprobe,
            effective_nprobe_source,
            adaptive_nprobe_decision,
            input_frontier_width: 1,
            expanded_parent_count: 1,
            selected_child_count,
            deduped_route_count: unique_leaf_count.min(route_budget.max_leaf_routes),
            truncation_reason: if unique_leaf_count > route_budget.max_leaf_routes {
                "max_leaf_routes"
            } else {
                "none"
            },
        }]);
    }

    let mut unique_child_count = 0usize;
    let mut current_parents = Vec::with_capacity(selected_child_count.min(route_budget.beam_width));
    let mut seen_child_pids = HashSet::new();
    for route in selected_child_routes {
        if !seen_child_pids.insert(route.child_pid) {
            continue;
        }
        unique_child_count += 1;
        if current_parents.len() < route_budget.beam_width {
            let child = require_recursive_internal_child(
                routing_objects_by_pid,
                route.child_pid,
                root_object,
            )?;
            current_parents.push(SpireRecursiveParentRoute {
                parent: (*child).clone(),
                path_score: -route.distance,
            });
        }
    }
    let mut levels = vec![SpireRoutingLevelDiagnostics {
        level: root_object.header.level,
        effective_nprobe,
        effective_nprobe_source,
        adaptive_nprobe_decision,
        input_frontier_width: 1,
        expanded_parent_count: 1,
        selected_child_count,
        deduped_route_count: unique_child_count.min(route_budget.beam_width),
        truncation_reason: if unique_child_count > route_budget.beam_width {
            "beam_width"
        } else {
            "none"
        },
    }];
    if current_parents.is_empty() {
        return Err("ec_spire recursive scan routing produced no internal children".to_owned());
    }
    levels.extend(collect_recursive_parent_routing_level_diagnostics(
        current_parents,
        routing_objects_by_pid,
        query_vector,
        nprobe_policy,
        route_budget,
        0,
    )?);
    Ok(levels)
}

fn collect_recursive_parent_routing_level_diagnostics(
    mut current_parents: Vec<SpireRecursiveParentRoute>,
    routing_objects_by_pid: &HashMap<u64, SpireRoutingPartitionObject>,
    query_vector: &[f32],
    nprobe_policy: &SpireRecursiveNprobePolicy,
    route_budget: SpireRecursiveRouteBudget,
    mut expanded_parent_count: usize,
) -> Result<Vec<SpireRoutingLevelDiagnostics>, String> {
    if current_parents.is_empty() {
        return Err("ec_spire recursive routing diagnostics produced no parent routes".to_owned());
    }

    let mut levels = Vec::new();
    loop {
        let parent_level = current_parents[0].parent.header.level;
        if parent_level == 1 {
            let input_frontier_width = current_parents.len();
            let mut level_expanded_parent_count = 0usize;
            let mut leaf_candidates = Vec::new();
            let mut nprobe_choices = Vec::new();
            for parent_route in &current_parents {
                let parent = &parent_route.parent;
                if parent.header.level != 1 {
                    return Err("ec_spire recursive routing diagnostics parent levels drifted"
                        .to_owned());
                }
                if expanded_parent_count >= route_budget.max_routing_expansions {
                    break;
                }
                expanded_parent_count += 1;
                level_expanded_parent_count += 1;
                let (routes, choice) = route_routing_object_to_child_routes_with_policy(
                    parent,
                    query_vector,
                    nprobe_policy,
                )?;
                nprobe_choices.push(choice);
                for route in routes {
                    leaf_candidates.push(SpireRecursiveScoredChildRoute {
                        parent_pid: parent.header.pid,
                        parent_level: parent.header.level,
                        child_pid: route.child_pid,
                        centroid_index: route.centroid_index,
                        path_score: parent_route.path_score,
                        score: route.score,
                    });
                }
            }
            leaf_candidates.sort_by(recursive_scored_child_route_cmp);
            let selected_child_count = leaf_candidates.len();
            let mut seen_leaf_pids = HashSet::new();
            for route in leaf_candidates {
                seen_leaf_pids.insert(route.child_pid);
            }
            let unique_leaf_count = seen_leaf_pids.len();
            let expansion_truncated = level_expanded_parent_count < input_frontier_width;
            let deduped_route_count = unique_leaf_count.min(route_budget.max_leaf_routes);
            let (effective_nprobe, effective_nprobe_source, adaptive_nprobe_decision) =
                summarize_adaptive_choices(&nprobe_choices);
            levels.push(SpireRoutingLevelDiagnostics {
                level: parent_level,
                effective_nprobe,
                effective_nprobe_source,
                adaptive_nprobe_decision,
                input_frontier_width,
                expanded_parent_count: level_expanded_parent_count,
                selected_child_count,
                deduped_route_count,
                truncation_reason: route_truncation_reason(
                    expansion_truncated,
                    unique_leaf_count > route_budget.max_leaf_routes,
                    "max_leaf_routes",
                ),
            });
            if deduped_route_count == 0 {
                return Err("ec_spire recursive scan routing produced no leaf routes".to_owned());
            }
            return Ok(levels);
        }

        let input_frontier_width = current_parents.len();
        let mut level_expanded_parent_count = 0usize;
        let mut child_candidates = Vec::new();
        let mut nprobe_choices = Vec::new();
        for parent_route in &current_parents {
            let parent = &parent_route.parent;
            if parent.header.kind != SpirePartitionObjectKind::Root
                && parent.header.kind != SpirePartitionObjectKind::Internal
            {
                return Err(
                    "ec_spire recursive routing diagnostics parent must be a routing object"
                        .to_owned(),
                );
            }
            if parent.header.level != parent_level {
                return Err(
                    "ec_spire recursive routing diagnostics parent levels drifted".to_owned(),
                );
            }
            if expanded_parent_count >= route_budget.max_routing_expansions {
                break;
            }
            expanded_parent_count += 1;
            level_expanded_parent_count += 1;
            let (routes, choice) = route_routing_object_to_child_routes_with_policy(
                parent,
                query_vector,
                nprobe_policy,
            )?;
            nprobe_choices.push(choice);
            for route in routes {
                child_candidates.push(SpireRecursiveScoredChildRoute {
                    parent_pid: parent.header.pid,
                    parent_level: parent.header.level,
                    child_pid: route.child_pid,
                    centroid_index: route.centroid_index,
                    path_score: parent_route.path_score,
                    score: route.score,
                });
            }
        }
        child_candidates.sort_by(recursive_scored_child_route_cmp);
        let selected_child_count = child_candidates.len();
        let mut unique_child_count = 0usize;
        let mut next_parents = Vec::new();
        let mut seen_child_pids = HashSet::new();
        for route in child_candidates {
            if !seen_child_pids.insert(route.child_pid) {
                continue;
            }
            unique_child_count += 1;
            if next_parents.len() < route_budget.beam_width {
                let child = require_recursive_internal_child_for_parent(
                    routing_objects_by_pid,
                    route.child_pid,
                    route.parent_pid,
                    route.parent_level,
                )?;
                next_parents.push(SpireRecursiveParentRoute {
                    parent: (*child).clone(),
                    path_score: route.total_score(),
                });
            }
        }
        let expansion_truncated = level_expanded_parent_count < input_frontier_width;
        let deduped_route_count = unique_child_count.min(route_budget.beam_width);
        let (effective_nprobe, effective_nprobe_source, adaptive_nprobe_decision) =
            summarize_adaptive_choices(&nprobe_choices);
        levels.push(SpireRoutingLevelDiagnostics {
            level: parent_level,
            effective_nprobe,
            effective_nprobe_source,
            adaptive_nprobe_decision,
            input_frontier_width,
            expanded_parent_count: level_expanded_parent_count,
            selected_child_count,
            deduped_route_count,
            truncation_reason: route_truncation_reason(
                expansion_truncated,
                unique_child_count > route_budget.beam_width,
                "beam_width",
            ),
        });
        if next_parents.is_empty() {
            return Err("ec_spire recursive scan routing produced no internal children".to_owned());
        }
        current_parents = next_parents;
    }
}

fn route_recursive_parent_objects_to_leaf_routes(
    mut current_parents: Vec<SpireRecursiveParentRoute>,
    routing_objects_by_pid: &HashMap<u64, SpireRoutingPartitionObject>,
    query_vector: &[f32],
    nprobe_policy: &SpireRecursiveNprobePolicy,
    route_budget: SpireRecursiveRouteBudget,
) -> Result<Vec<SpireRecursiveLeafRoute>, String> {
    if current_parents.is_empty() {
        return Err("ec_spire recursive scan routing produced no parent routes".to_owned());
    }
    let mut expanded_parent_count = 0usize;
    loop {
        let parent_level = current_parents[0].parent.header.level;
        if parent_level == 1 {
            let mut leaf_candidates = Vec::new();
            for parent_route in &current_parents {
                let parent = &parent_route.parent;
                if parent.header.level != 1 {
                    return Err("ec_spire recursive scan routing parent levels drifted".to_owned());
                }
                if expanded_parent_count >= route_budget.max_routing_expansions {
                    break;
                }
                expanded_parent_count += 1;
                let (routes, _choice) = route_routing_object_to_child_routes_with_policy(
                    parent,
                    query_vector,
                    nprobe_policy,
                )?;
                for route in routes {
                    leaf_candidates.push(SpireRecursiveScoredChildRoute {
                        parent_pid: parent.header.pid,
                        parent_level: parent.header.level,
                        child_pid: route.child_pid,
                        centroid_index: route.centroid_index,
                        path_score: parent_route.path_score,
                        score: route.score,
                    });
                }
            }
            leaf_candidates.sort_by(recursive_scored_child_route_cmp);
            let mut seen_leaf_pids = HashSet::new();
            let mut leaf_routes = Vec::new();
            for route in leaf_candidates {
                if leaf_routes.len() >= route_budget.max_leaf_routes {
                    break;
                }
                if !seen_leaf_pids.insert(route.child_pid) {
                    continue;
                }
                leaf_routes.push(SpireRecursiveLeafRoute {
                    leaf_pid: route.child_pid,
                    parent_pid: route.parent_pid,
                });
            }
            if leaf_routes.is_empty() {
                return Err("ec_spire recursive scan routing produced no leaf routes".to_owned());
            }
            return Ok(leaf_routes);
        }

        let mut child_candidates = Vec::new();
        for parent_route in &current_parents {
            let parent = &parent_route.parent;
            if parent.header.kind != SpirePartitionObjectKind::Root
                && parent.header.kind != SpirePartitionObjectKind::Internal
            {
                return Err("ec_spire recursive scan parent must be a routing object".to_owned());
            }
            if parent.header.level != parent_level {
                return Err("ec_spire recursive scan routing parent levels drifted".to_owned());
            }
            if expanded_parent_count >= route_budget.max_routing_expansions {
                break;
            }
            expanded_parent_count += 1;
            let (routes, _choice) = route_routing_object_to_child_routes_with_policy(
                parent,
                query_vector,
                nprobe_policy,
            )?;
            for route in routes {
                child_candidates.push(SpireRecursiveScoredChildRoute {
                    parent_pid: parent.header.pid,
                    parent_level: parent.header.level,
                    child_pid: route.child_pid,
                    centroid_index: route.centroid_index,
                    path_score: parent_route.path_score,
                    score: route.score,
                });
            }
        }
        child_candidates.sort_by(recursive_scored_child_route_cmp);
        let mut next_parents = Vec::new();
        let mut seen_child_pids = HashSet::new();
        for route in child_candidates {
            if next_parents.len() >= route_budget.beam_width {
                break;
            }
            if !seen_child_pids.insert(route.child_pid) {
                continue;
            }
            let child = require_recursive_internal_child_for_parent(
                routing_objects_by_pid,
                route.child_pid,
                route.parent_pid,
                route.parent_level,
            )?;
            next_parents.push(SpireRecursiveParentRoute {
                parent: (*child).clone(),
                path_score: route.total_score(),
            });
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
    require_recursive_internal_child_for_parent(
        routing_objects_by_pid,
        child_pid,
        parent.header.pid,
        parent.header.level,
    )
}

fn require_recursive_internal_child_for_parent<'a>(
    routing_objects_by_pid: &'a HashMap<u64, SpireRoutingPartitionObject>,
    child_pid: u64,
    parent_pid: u64,
    parent_level: u16,
) -> Result<&'a SpireRoutingPartitionObject, String> {
    let child = routing_objects_by_pid.get(&child_pid).ok_or_else(|| {
        format!("ec_spire recursive scan missing internal routing child pid {child_pid}")
    })?;
    if child.header.kind != SpirePartitionObjectKind::Internal {
        return Err(format!(
            "ec_spire recursive scan child pid {child_pid} is not an internal routing object"
        ));
    }
    if child.header.parent_pid != parent_pid {
        return Err(format!(
            "ec_spire recursive scan child pid {child_pid} parent_pid {} does not match parent pid {}",
            child.header.parent_pid, parent_pid
        ));
    }
    if child.header.level.checked_add(1) != Some(parent_level) {
        return Err(format!(
            "ec_spire recursive scan child pid {child_pid} level {} is not one below parent level {}",
            child.header.level, parent_level
        ));
    }
    Ok(child)
}

fn validate_top_graph_route_inputs(
    root_object: &SpireRoutingPartitionObject,
    top_graph: &impl SpireTopGraphRouteView,
    query_vector: &[f32],
    search_list_size: u32,
    route_count: u32,
) -> Result<(), String> {
    if root_object.header.kind != SpirePartitionObjectKind::Root {
        return Err("ec_spire top graph routing requires a root routing object".to_owned());
    }
    if root_object.header.pid != top_graph.root_pid() {
        return Err(format!(
            "ec_spire top graph root pid {} does not match routing root pid {}",
            top_graph.root_pid(), root_object.header.pid
        ));
    }
    if root_object.dimensions != top_graph.dimensions() {
        return Err(format!(
            "ec_spire top graph dimensions {} do not match routing dimensions {}",
            top_graph.dimensions(), root_object.dimensions
        ));
    }
    if usize::try_from(top_graph.node_count())
        .ok()
        .filter(|node_count| *node_count == top_graph.nodes_len())
        .is_none()
    {
        return Err(format!(
            "ec_spire top graph node count {} does not match node rows {}",
            top_graph.node_count(),
            top_graph.nodes_len()
        ));
    }
    if top_graph.nodes_len() != root_object.child_count() {
        return Err(format!(
            "ec_spire top graph node count {} does not match routing child count {}",
            top_graph.nodes_len(),
            root_object.child_count()
        ));
    }
    if top_graph.entry_node() >= top_graph.node_count() {
        return Err(format!(
            "ec_spire top graph entry node {} is outside node count {}",
            top_graph.entry_node(),
            top_graph.node_count()
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

    let node_count = top_graph.nodes_len();
    for (node_index, routing_child) in root_object.children().enumerate() {
        let graph_node = top_graph.route_node(node_index);
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
        for &neighbor in graph_node.neighbors {
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

fn top_graph_route_cmp(left: &SpireTopGraphRoute, right: &SpireTopGraphRoute) -> Ordering {
    left.distance
        .total_cmp(&right.distance)
        .then_with(|| left.centroid_ordinal.cmp(&right.centroid_ordinal))
        .then_with(|| left.child_pid.cmp(&right.child_pid))
        .then_with(|| left.node_ordinal.cmp(&right.node_ordinal))
}

fn recursive_scored_child_route_cmp(
    left: &SpireRecursiveScoredChildRoute,
    right: &SpireRecursiveScoredChildRoute,
) -> Ordering {
    right
        .total_score()
        .total_cmp(&left.total_score())
        .then_with(|| left.parent_pid.cmp(&right.parent_pid))
        .then_with(|| left.centroid_index.cmp(&right.centroid_index))
        .then_with(|| left.child_pid.cmp(&right.child_pid))
}

fn unique_child_pid_count(routes: &[SpireTopGraphRoute]) -> usize {
    routes
        .iter()
        .map(|route| route.child_pid)
        .collect::<HashSet<_>>()
        .len()
}

fn route_truncation_reason(
    expansion_truncated: bool,
    route_truncated: bool,
    route_cap_label: &'static str,
) -> &'static str {
    if expansion_truncated {
        "max_routing_expansions"
    } else if route_truncated {
        route_cap_label
    } else {
        "none"
    }
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
