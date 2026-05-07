#[derive(Debug, Clone)]
struct SpirePendingRecursiveRoutingNode {
    pid: u64,
    level: u16,
    centroid: Vec<f32>,
    source_count: u64,
    children: Vec<SpireRecursiveRoutingChildInput>,
}

pub(super) fn build_recursive_routing_hierarchy_draft(
    input: SpireRecursiveRoutingBuildInput,
    pid_allocator: &mut SpirePidAllocator,
) -> Result<SpireRecursiveRoutingBuildDraft, String> {
    input.validate()?;
    let target_fanout = usize::try_from(input.target_fanout)
        .map_err(|_| "ec_spire recursive routing fanout exceeds usize".to_owned())?;
    let mut pid_cursor = *pid_allocator;
    let mut current_children = input.children;
    let mut pending_nodes = Vec::new();

    while current_children.len() > target_fanout {
        let child_level = current_children[0].child_level;
        let parent_level = child_level
            .checked_add(1)
            .ok_or_else(|| "ec_spire recursive routing level overflow".to_owned())?;
        let source_vectors = current_children
            .iter()
            .map(|child| child.centroid.as_slice())
            .collect::<Vec<_>>();
        let model = common_training::train_spherical_kmeans(
            "ec_spire recursive routing",
            &source_vectors,
            usize::from(input.dimensions),
            target_fanout,
            input.seed.wrapping_add(u64::from(parent_level)),
            SPIRE_DEFAULT_KMEANS_ITERATIONS,
        )?;
        let mut grouped_children = vec![Vec::new(); model.centroid_count()];
        for child in current_children {
            let centroid_index = common_training::assign_vector_to_centroid(
                "ec_spire recursive routing",
                &child.centroid,
                &model,
            )?;
            grouped_children[centroid_index].push(child);
        }

        let mut next_children = Vec::new();
        for (centroid_index, children) in grouped_children.into_iter().enumerate() {
            if children.is_empty() {
                continue;
            }
            let pid = pid_cursor.allocate()?;
            let source_count = sum_recursive_source_counts(&children)?;
            let centroid = model.centroids[centroid_index].clone();
            pending_nodes.push(SpirePendingRecursiveRoutingNode {
                pid,
                level: parent_level,
                centroid: centroid.clone(),
                source_count,
                children,
            });
            next_children.push(SpireRecursiveRoutingChildInput {
                child_pid: pid,
                child_level: parent_level,
                centroid,
                source_count,
            });
        }
        current_children = next_children;
    }

    let root_level = current_children[0]
        .child_level
        .checked_add(1)
        .ok_or_else(|| "ec_spire recursive routing root level overflow".to_owned())?;
    let root_pid = pid_cursor.allocate()?;
    let pending_by_pid = pending_nodes
        .iter()
        .map(|node| (node.pid, node))
        .collect::<HashMap<_, _>>();
    let mut routing_objects = Vec::with_capacity(pending_nodes.len() + 1);
    let mut centroid_records = Vec::new();
    routing_objects.push(SpireRoutingPartitionObject::root_at_level(
        root_pid,
        input.object_version,
        root_level,
        input.dimensions,
        routing_child_entries(&current_children)?,
    )?);
    extend_recursive_centroid_records(
        &mut centroid_records,
        root_pid,
        input.dimensions,
        &current_children,
    )?;
    let mut visited_internal_pids = HashSet::with_capacity(pending_nodes.len());
    for child in &current_children {
        materialize_pending_recursive_child(
            child,
            root_pid,
            input.object_version,
            input.dimensions,
            &pending_by_pid,
            &mut visited_internal_pids,
            &mut routing_objects,
            &mut centroid_records,
        )?;
    }
    if visited_internal_pids.len() != pending_nodes.len() {
        return Err("ec_spire recursive routing contains unreachable internal nodes".to_owned());
    }

    let draft = SpireRecursiveRoutingBuildDraft {
        root_pid,
        root_level,
        routing_objects,
        centroid_records,
        next_pid: pid_cursor.next_pid(),
    };
    assert_recursive_draft_invariants(&draft)?;
    *pid_allocator = pid_cursor;
    Ok(draft)
}

pub(super) fn build_recursive_epoch_input_from_centroid_plan(
    input: SpireRecursiveBuildCoordinatorInput,
    pid_allocator: &mut SpirePidAllocator,
    local_vec_id_allocator: &mut SpireLocalVecIdAllocator,
) -> Result<SpireRecursiveBuildCoordinatorDraft, String> {
    input.centroid_plan.validate()?;
    let centroid_count = input.centroid_plan.centroid_count();
    if input.assignments.len() != input.centroid_plan.assignment_indexes.len() {
        return Err(format!(
            "ec_spire recursive build assignment count {} does not match centroid assignment count {}",
            input.assignments.len(),
            input.centroid_plan.assignment_indexes.len()
        ));
    }
    if input.source_vectors.len() != input.assignments.len() {
        return Err(format!(
            "ec_spire recursive build source vector count {} does not match assignment count {}",
            input.source_vectors.len(),
            input.assignments.len()
        ));
    }

    let assignments_by_centroid = group_assignments_by_centroid(
        input.assignments.clone(),
        &input.centroid_plan.assignment_indexes,
        centroid_count,
    )?;
    let mut pid_cursor = *pid_allocator;
    let mut local_vec_id_cursor = *local_vec_id_allocator;
    let mut leaf_pids = Vec::with_capacity(centroid_count);
    for _ in 0..centroid_count {
        leaf_pids.push(pid_cursor.allocate()?);
    }
    let routing_draft = build_recursive_routing_hierarchy_draft(
        SpireRecursiveRoutingBuildInput {
            object_version: input.object_version,
            dimensions: input.centroid_plan.dimensions,
            target_fanout: input.target_fanout,
            seed: input.seed,
            children: leaf_pids
                .iter()
                .copied()
                .zip(input.centroid_plan.centroids.iter())
                .map(|(child_pid, centroid)| SpireRecursiveRoutingChildInput {
                    child_pid,
                    child_level: 0,
                    centroid: centroid.clone(),
                    // First-level recursive children are trained leaf centroids, so this counts
                    // one centroid source rather than rows assigned to the eventual leaf object.
                    source_count: 1,
                })
                .collect(),
        },
        &mut pid_cursor,
    )?;
    let leaf_parent_pids = assert_recursive_draft_invariants(&routing_draft)?.leaf_parent_pids;
    let route_map = SpireSingleLevelRouteMap::from_centroid_plan(&input.centroid_plan, &leaf_pids)?;
    let rows_by_leaf_pid = build_recursive_leaf_rows_by_pid(
        input.assignments,
        input.source_vectors,
        assignments_by_centroid,
        &route_map,
        input.boundary_replica_count,
        &mut local_vec_id_cursor,
    )?;
    let mut leaf_inputs = Vec::with_capacity(centroid_count);
    for pid in leaf_pids.iter().copied() {
        let parent_pid = *leaf_parent_pids.get(&pid).ok_or_else(|| {
            format!("ec_spire recursive build missing routing parent for leaf pid {pid}")
        })?;
        let rows = rows_by_leaf_pid
            .get(&pid)
            .cloned()
            .ok_or_else(|| format!("ec_spire recursive build missing leaf rows for pid {pid}"))?;
        leaf_inputs.push(SpireRecursiveLeafObjectInput {
            pid,
            object_version: input.object_version,
            parent_pid,
            rows,
        });
    }

    let draft = SpireRecursiveBuildCoordinatorDraft {
        epoch_input: SpireRecursiveRoutingEpochObjectInput {
            epoch: input.epoch,
            published_at_micros: input.published_at_micros,
            retain_until_micros: input.retain_until_micros,
            consistency_mode: input.consistency_mode,
            routing_draft,
            leaf_inputs,
        },
        leaf_pids,
        next_pid: pid_cursor.next_pid(),
        next_local_vec_seq: local_vec_id_cursor.next_local_vec_seq(),
    };
    *pid_allocator = pid_cursor;
    *local_vec_id_allocator = local_vec_id_cursor;
    Ok(draft)
}

fn build_recursive_leaf_rows_by_pid(
    assignments: Vec<SpireLeafAssignmentInput>,
    source_vectors: Vec<Vec<f32>>,
    _assignments_by_centroid: Vec<Vec<SpireLeafAssignmentInput>>,
    route_map: &SpireSingleLevelRouteMap,
    boundary_replica_count: u32,
    local_vec_id_cursor: &mut SpireLocalVecIdAllocator,
) -> Result<HashMap<u64, Vec<SpireLeafAssignmentRow>>, String> {
    let mut rows_by_leaf_pid = route_map
        .entries
        .iter()
        .map(|entry| (entry.pid, Vec::new()))
        .collect::<HashMap<_, _>>();
    let boundary_inputs = assignments
        .into_iter()
        .zip(source_vectors.into_iter())
        .map(|(assignment, source_vector)| {
            let plan =
                route_map.route_boundary_assignment_for_vector(&source_vector, boundary_replica_count)?;
            Ok(SpireBoundaryLeafAssignmentInput {
                primary_pid: plan.primary_pid,
                replica_pids: plan.replica_pids,
                assignment,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    for placement in build_boundary_leaf_assignment_placements(local_vec_id_cursor, boundary_inputs)? {
        let rows = rows_by_leaf_pid.get_mut(&placement.pid).ok_or_else(|| {
            format!(
                "ec_spire recursive boundary assignment resolved unknown leaf pid {}",
                placement.pid
            )
        })?;
        rows.push(placement.row);
    }
    Ok(rows_by_leaf_pid)
}

impl SpireRecursiveRoutingBuildInput {
    fn validate(&self) -> Result<(), String> {
        if self.object_version == 0 {
            return Err("ec_spire recursive routing object_version 0 is invalid".to_owned());
        }
        if self.dimensions == 0 {
            return Err("ec_spire recursive routing requires dimensions > 0".to_owned());
        }
        if self.target_fanout < 2 {
            return Err("ec_spire recursive routing fanout must be at least 2".to_owned());
        }
        if self.children.is_empty() {
            return Err("ec_spire recursive routing requires at least one child".to_owned());
        }
        let expected_level = self.children[0].child_level;
        let mut child_pids = HashSet::with_capacity(self.children.len());
        for child in &self.children {
            if child.child_pid == 0 {
                return Err("ec_spire recursive routing child pid 0 is invalid".to_owned());
            }
            if !child_pids.insert(child.child_pid) {
                return Err(format!(
                    "ec_spire recursive routing duplicate child pid {}",
                    child.child_pid
                ));
            }
            if child.child_level != expected_level {
                return Err(format!(
                    "ec_spire recursive routing child pid {} level {} does not match expected level {expected_level}",
                    child.child_pid, child.child_level
                ));
            }
            if child.source_count == 0 {
                return Err(format!(
                    "ec_spire recursive routing child pid {} source_count 0 is invalid",
                    child.child_pid
                ));
            }
            validate_recursive_centroid(self.dimensions, child.child_pid, &child.centroid)?;
        }
        Ok(())
    }
}

fn validate_recursive_centroid(
    dimensions: u16,
    child_pid: u64,
    centroid: &[f32],
) -> Result<(), String> {
    let expected = usize::from(dimensions);
    if centroid.len() != expected {
        return Err(format!(
            "ec_spire recursive routing child pid {child_pid} centroid dimensions mismatch: got {}, expected {expected}",
            centroid.len()
        ));
    }
    if centroid.iter().any(|component| !component.is_finite()) {
        return Err(format!(
            "ec_spire recursive routing child pid {child_pid} centroid must be finite"
        ));
    }
    Ok(())
}

fn sum_recursive_source_counts(
    children: &[SpireRecursiveRoutingChildInput],
) -> Result<u64, String> {
    children.iter().try_fold(0_u64, |sum, child| {
        sum.checked_add(child.source_count)
            .ok_or_else(|| "ec_spire recursive routing source_count overflow".to_owned())
    })
}

fn routing_child_entries(
    children: &[SpireRecursiveRoutingChildInput],
) -> Result<Vec<SpireRoutingChildEntry>, String> {
    children
        .iter()
        .enumerate()
        .map(|(index, child)| {
            Ok(SpireRoutingChildEntry {
                centroid_index: u32::try_from(index)
                    .map_err(|_| "ec_spire recursive routing child index exceeds u32".to_owned())?,
                child_pid: child.child_pid,
                centroid: child.centroid.clone(),
            })
        })
        .collect()
}

fn extend_recursive_centroid_records(
    centroid_records: &mut Vec<SpireRecursiveCentroidRecord>,
    parent_pid: u64,
    dimensions: u16,
    children: &[SpireRecursiveRoutingChildInput],
) -> Result<(), String> {
    for (index, child) in children.iter().enumerate() {
        centroid_records.push(SpireRecursiveCentroidRecord {
            parent_pid,
            child_pid: child.child_pid,
            child_level: child.child_level,
            centroid_ordinal: u32::try_from(index)
                .map_err(|_| "ec_spire recursive centroid ordinal exceeds u32".to_owned())?,
            dimensions,
            centroid: child.centroid.clone(),
            source_count: child.source_count,
        });
    }
    Ok(())
}

fn materialize_pending_recursive_child(
    child: &SpireRecursiveRoutingChildInput,
    parent_pid: u64,
    object_version: u64,
    dimensions: u16,
    pending_by_pid: &HashMap<u64, &SpirePendingRecursiveRoutingNode>,
    visited_internal_pids: &mut HashSet<u64>,
    routing_objects: &mut Vec<SpireRoutingPartitionObject>,
    centroid_records: &mut Vec<SpireRecursiveCentroidRecord>,
) -> Result<(), String> {
    if child.child_level == 0 {
        return Ok(());
    }
    let node = pending_by_pid.get(&child.child_pid).ok_or_else(|| {
        format!(
            "ec_spire recursive routing missing internal node pid {}",
            child.child_pid
        )
    })?;
    if node.level != child.child_level {
        return Err(format!(
            "ec_spire recursive routing internal node pid {} level {} does not match child level {}",
            node.pid, node.level, child.child_level
        ));
    }
    if node.source_count != child.source_count {
        return Err(format!(
            "ec_spire recursive routing internal node pid {} source_count {} does not match child source_count {}",
            node.pid, node.source_count, child.source_count
        ));
    }
    if node.centroid != child.centroid {
        return Err(format!(
            "ec_spire recursive routing internal node pid {} centroid drift",
            node.pid
        ));
    }
    if !visited_internal_pids.insert(node.pid) {
        return Err(format!(
            "ec_spire recursive routing internal node pid {} reached twice",
            node.pid
        ));
    }
    routing_objects.push(SpireRoutingPartitionObject::internal(
        node.pid,
        object_version,
        node.level,
        parent_pid,
        dimensions,
        routing_child_entries(&node.children)?,
    )?);
    extend_recursive_centroid_records(centroid_records, node.pid, dimensions, &node.children)?;
    for child in &node.children {
        materialize_pending_recursive_child(
            child,
            node.pid,
            object_version,
            dimensions,
            pending_by_pid,
            visited_internal_pids,
            routing_objects,
            centroid_records,
        )?;
    }
    Ok(())
}

fn validate_recursive_routing_build_draft(
    draft: &SpireRecursiveRoutingBuildDraft,
) -> Result<(), String> {
    // Recursive drafts pass three validation barriers:
    // 1. this in-memory routing-object and centroid-record shape check;
    // 2. epoch leaf-placement validation after object writes;
    // 3. snapshot-time hierarchy validation before scan descent.
    if draft.routing_objects.is_empty() {
        return Err("ec_spire recursive routing draft requires routing objects".to_owned());
    }
    if draft.routing_objects[0].header.kind != super::storage::SpirePartitionObjectKind::Root {
        return Err("ec_spire recursive routing draft first object must be root".to_owned());
    }
    if draft.routing_objects[0].header.pid != draft.root_pid {
        return Err("ec_spire recursive routing draft root pid mismatch".to_owned());
    }
    if draft.routing_objects[0].header.level != draft.root_level {
        return Err("ec_spire recursive routing draft root level mismatch".to_owned());
    }
    let mut pids = HashSet::with_capacity(draft.routing_objects.len());
    for object in &draft.routing_objects {
        if !pids.insert(object.header.pid) {
            return Err(format!(
                "ec_spire recursive routing draft duplicate routing pid {}",
                object.header.pid
            ));
        }
    }
    let mut centroid_keys = HashSet::with_capacity(draft.centroid_records.len());
    let mut centroid_ordinals_by_parent = HashMap::<u64, Vec<u32>>::new();
    for record in &draft.centroid_records {
        validate_recursive_centroid(record.dimensions, record.child_pid, &record.centroid)?;
        if !centroid_keys.insert((record.parent_pid, record.child_pid)) {
            return Err(format!(
                "ec_spire recursive routing draft duplicate centroid record parent {} child {}",
                record.parent_pid, record.child_pid
            ));
        }
        if record.source_count == 0 {
            return Err(format!(
                "ec_spire recursive routing draft centroid record child {} source_count 0",
                record.child_pid
            ));
        }
        centroid_ordinals_by_parent
            .entry(record.parent_pid)
            .or_default()
            .push(record.centroid_ordinal);
    }
    for (parent_pid, mut ordinals) in centroid_ordinals_by_parent {
        ordinals.sort_unstable();
        for (position, ordinal) in ordinals.into_iter().enumerate() {
            let expected = u32::try_from(position).map_err(|_| {
                "ec_spire recursive routing centroid ordinal exceeds u32".to_owned()
            })?;
            if ordinal != expected {
                return Err(format!(
                    "ec_spire recursive routing draft centroid ordinals for parent {parent_pid} are not dense at position {position}: got {ordinal}"
                ));
            }
        }
    }
    Ok(())
}

fn assert_recursive_draft_invariants(
    draft: &SpireRecursiveRoutingBuildDraft,
) -> Result<SpireRecursiveDraftInvariants, String> {
    validate_recursive_routing_build_draft(draft)?;
    Ok(SpireRecursiveDraftInvariants {
        leaf_parent_pids: recursive_routing_leaf_parent_pids(draft)?,
    })
}

pub(super) fn build_local_recursive_routing_epoch_draft(
    input: SpireRecursiveRoutingEpochInput,
    object_store: &mut SpireLocalObjectStore,
) -> Result<SpireRecursiveRoutingEpochDraft, String> {
    build_recursive_routing_epoch_draft_with_store(input, object_store)
}

pub(super) unsafe fn build_relation_recursive_routing_epoch_draft(
    input: SpireRecursiveRoutingEpochInput,
    object_store: &mut SpireRelationObjectStore,
) -> Result<SpireRecursiveRoutingEpochDraft, String> {
    build_recursive_routing_epoch_draft_with_store(input, object_store)
}

pub(super) fn build_local_recursive_top_graph_epoch_draft(
    input: SpireRecursiveTopGraphEpochInput,
    object_store: &mut SpireLocalObjectStore,
) -> Result<SpireRecursiveRoutingEpochDraft, String> {
    build_recursive_top_graph_epoch_draft_with_store(input, object_store)
}

pub(super) unsafe fn build_relation_recursive_top_graph_epoch_draft(
    input: SpireRecursiveTopGraphEpochInput,
    object_store: &mut SpireRelationObjectStore,
) -> Result<SpireRecursiveRoutingEpochDraft, String> {
    build_recursive_top_graph_epoch_draft_with_store(input, object_store)
}

pub(super) fn build_local_recursive_routing_epoch_from_leaf_inputs(
    input: SpireRecursiveRoutingEpochObjectInput,
    object_store: &mut SpireLocalObjectStore,
) -> Result<SpireRecursiveRoutingEpochDraft, String> {
    build_recursive_routing_epoch_from_leaf_inputs_with_store(input, object_store)
}

pub(super) unsafe fn build_relation_recursive_routing_epoch_from_leaf_inputs(
    input: SpireRecursiveRoutingEpochObjectInput,
    object_store: &mut SpireRelationObjectStore,
) -> Result<SpireRecursiveRoutingEpochDraft, String> {
    build_recursive_routing_epoch_from_leaf_inputs_with_store(input, object_store)
}

fn build_recursive_routing_epoch_from_leaf_inputs_with_store(
    input: SpireRecursiveRoutingEpochObjectInput,
    object_store: &mut impl SpireBuildObjectStore,
) -> Result<SpireRecursiveRoutingEpochDraft, String> {
    let invariants = assert_recursive_draft_invariants(&input.routing_draft)?;
    let expected_leaf_parents = invariants.leaf_parent_pids;
    let mut seen_leaf_pids = HashSet::with_capacity(input.leaf_inputs.len());
    let mut leaf_placements = Vec::with_capacity(input.leaf_inputs.len());
    for leaf_input in input.leaf_inputs {
        if !seen_leaf_pids.insert(leaf_input.pid) {
            return Err(format!(
                "ec_spire recursive routing epoch duplicate leaf object input pid {}",
                leaf_input.pid
            ));
        }
        let expected_parent_pid = expected_leaf_parents.get(&leaf_input.pid).ok_or_else(|| {
            format!(
                "ec_spire recursive routing epoch unexpected leaf object input pid {}",
                leaf_input.pid
            )
        })?;
        if leaf_input.parent_pid != *expected_parent_pid {
            return Err(format!(
                "ec_spire recursive routing epoch leaf object input pid {} parent {} does not match routing parent {}",
                leaf_input.pid, leaf_input.parent_pid, expected_parent_pid
            ));
        }
        leaf_placements.push(object_store.write_leaf_object_v2_from_rows(
            input.epoch,
            leaf_input.pid,
            leaf_input.object_version,
            leaf_input.parent_pid,
            &leaf_input.rows,
        )?);
    }
    let expected_leaf_pids = expected_leaf_parents
        .keys()
        .copied()
        .collect::<HashSet<_>>();
    if seen_leaf_pids != expected_leaf_pids {
        let missing = expected_leaf_pids
            .difference(&seen_leaf_pids)
            .copied()
            .collect::<Vec<_>>();
        let extra = seen_leaf_pids
            .difference(&expected_leaf_pids)
            .copied()
            .collect::<Vec<_>>();
        return Err(format!(
            "ec_spire recursive routing epoch leaf object input mismatch: missing {missing:?}, extra {extra:?}"
        ));
    }

    build_recursive_routing_epoch_draft_with_store(
        SpireRecursiveRoutingEpochInput {
            epoch: input.epoch,
            published_at_micros: input.published_at_micros,
            retain_until_micros: input.retain_until_micros,
            consistency_mode: input.consistency_mode,
            routing_draft: input.routing_draft,
            leaf_placements,
        },
        object_store,
    )
}

fn build_recursive_routing_epoch_draft_with_store(
    input: SpireRecursiveRoutingEpochInput,
    object_store: &mut impl SpireBuildObjectStore,
) -> Result<SpireRecursiveRoutingEpochDraft, String> {
    build_recursive_routing_epoch_draft_with_extra_placements(input, object_store, Vec::new(), None)
}

fn build_recursive_top_graph_epoch_draft_with_store(
    input: SpireRecursiveTopGraphEpochInput,
    object_store: &mut impl SpireBuildObjectStore,
) -> Result<SpireRecursiveRoutingEpochDraft, String> {
    let invariants = assert_recursive_draft_invariants(&input.epoch_input.routing_draft)?;
    validate_recursive_epoch_leaf_placements(
        &input.epoch_input,
        &invariants.leaf_parent_pids,
        object_store,
    )?;
    let root_object = input
        .epoch_input
        .routing_draft
        .routing_objects
        .iter()
        .find(|object| object.header.kind == SpirePartitionObjectKind::Root)
        .ok_or_else(|| "ec_spire top graph epoch requires a root routing object".to_owned())?;
    let top_graph_draft =
        build_spire_top_graph_draft_from_routing_object(root_object, input.top_graph_params)?;
    let top_graph_pid = next_recursive_epoch_pid(
        input.epoch_input.routing_draft.next_pid,
        &input.epoch_input.leaf_placements,
    )?;
    let top_graph_object = spire_top_graph_partition_object_from_build_draft(
        top_graph_pid,
        root_object.header.object_version,
        root_object.header.level,
        &top_graph_draft,
    )?;
    let top_graph_placement =
        object_store.write_top_graph_object(input.epoch_input.epoch, &top_graph_object)?;
    build_recursive_routing_epoch_draft_with_extra_placements(
        input.epoch_input,
        object_store,
        vec![top_graph_placement],
        Some(top_graph_object),
    )
}

fn build_recursive_routing_epoch_draft_with_extra_placements(
    input: SpireRecursiveRoutingEpochInput,
    object_store: &mut impl SpireBuildObjectStore,
    extra_placements: Vec<SpirePlacementEntry>,
    top_graph_object: Option<SpireTopGraphPartitionObject>,
) -> Result<SpireRecursiveRoutingEpochDraft, String> {
    let invariants = assert_recursive_draft_invariants(&input.routing_draft)?;

    let epoch_manifest = SpireEpochManifest {
        epoch: input.epoch,
        state: SpireEpochState::Published,
        consistency_mode: input.consistency_mode,
        published_at_micros: input.published_at_micros,
        retain_until_micros: input.retain_until_micros,
        active_query_count: 0,
    };
    epoch_manifest.validate()?;

    validate_recursive_epoch_leaf_placements(&input, &invariants.leaf_parent_pids, object_store)?;

    let mut placements =
        Vec::with_capacity(
            input.routing_draft.routing_objects.len()
                + input.leaf_placements.len()
                + extra_placements.len(),
        );
    for object in &input.routing_draft.routing_objects {
        placements.push(object_store.write_routing_object(input.epoch, object)?);
    }
    placements.extend(input.leaf_placements);
    placements.extend(extra_placements);

    let object_manifest = SpireObjectManifest::from_entries(
        input.epoch,
        placements
            .iter()
            .map(|placement| SpireManifestEntry {
                epoch: placement.epoch,
                pid: placement.pid,
                object_version: placement.object_version,
                placement_tid: placement.object_tid,
            })
            .collect(),
    )?;
    let placement_directory = SpirePlacementDirectory::from_entries(input.epoch, placements)?;

    let next_pid =
        next_recursive_epoch_pid(input.routing_draft.next_pid, &placement_directory.entries)?;
    let draft = SpireRecursiveRoutingEpochDraft {
        epoch_manifest,
        object_manifest,
        placement_directory,
        root_pid: input.routing_draft.root_pid,
        centroid_records: input.routing_draft.centroid_records.clone(),
        routing_objects: input.routing_draft.routing_objects,
        top_graph_object,
        next_pid,
    };
    SpireValidatedEpochSnapshot::new(
        &draft.epoch_manifest,
        &draft.object_manifest,
        &draft.placement_directory,
    )?;
    Ok(draft)
}

fn validate_recursive_epoch_leaf_placements(
    input: &SpireRecursiveRoutingEpochInput,
    expected_leaf_parents: &HashMap<u64, u64>,
    object_store: &impl SpireObjectReader,
) -> Result<(), String> {
    let expected_leaf_pids = expected_leaf_parents
        .keys()
        .copied()
        .collect::<HashSet<_>>();
    let mut actual_leaf_pids = HashSet::with_capacity(input.leaf_placements.len());
    for placement in &input.leaf_placements {
        placement.encode()?;
        if placement.epoch != input.epoch {
            return Err(format!(
                "ec_spire recursive routing epoch leaf placement pid {} epoch {} does not match epoch {}",
                placement.pid, placement.epoch, input.epoch
            ));
        }
        if !actual_leaf_pids.insert(placement.pid) {
            return Err(format!(
                "ec_spire recursive routing epoch duplicate leaf placement pid {}",
                placement.pid
            ));
        }
        let header = object_store.read_object_header(placement)?;
        if header.kind != SpirePartitionObjectKind::Leaf {
            return Err(format!(
                "ec_spire recursive routing epoch placement pid {} is not a leaf object",
                placement.pid
            ));
        }
        let expected_parent_pid = expected_leaf_parents.get(&placement.pid).ok_or_else(|| {
            format!(
                "ec_spire recursive routing epoch unexpected leaf placement pid {}",
                placement.pid
            )
        })?;
        if header.parent_pid != *expected_parent_pid {
            return Err(format!(
                "ec_spire recursive routing epoch leaf pid {} parent {} does not match routing parent {}",
                placement.pid, header.parent_pid, expected_parent_pid
            ));
        }
    }
    if actual_leaf_pids != expected_leaf_pids {
        let missing = expected_leaf_pids
            .difference(&actual_leaf_pids)
            .copied()
            .collect::<Vec<_>>();
        let extra = actual_leaf_pids
            .difference(&expected_leaf_pids)
            .copied()
            .collect::<Vec<_>>();
        return Err(format!(
            "ec_spire recursive routing epoch leaf placement mismatch: missing {missing:?}, extra {extra:?}"
        ));
    }
    Ok(())
}

fn recursive_routing_leaf_parent_pids(
    draft: &SpireRecursiveRoutingBuildDraft,
) -> Result<HashMap<u64, u64>, String> {
    let mut leaf_parents = HashMap::new();
    for object in &draft.routing_objects {
        if object.header.level != 1 {
            continue;
        }
        for child in object.children() {
            if leaf_parents
                .insert(child.child_pid, object.header.pid)
                .is_some()
            {
                return Err(format!(
                    "ec_spire recursive routing epoch duplicate leaf child pid {}",
                    child.child_pid
                ));
            }
        }
    }
    if leaf_parents.is_empty() {
        return Err("ec_spire recursive routing epoch requires leaf child pids".to_owned());
    }
    Ok(leaf_parents)
}

fn next_recursive_epoch_pid(
    routing_next_pid: u64,
    placements: &[SpirePlacementEntry],
) -> Result<u64, String> {
    placements
        .iter()
        .try_fold(routing_next_pid, |next_pid, placement| {
            let after_placement = placement
                .pid
                .checked_add(1)
                .ok_or_else(|| "ec_spire recursive routing epoch pid overflow".to_owned())?;
            Ok(next_pid.max(after_placement))
        })
}

impl SpireSingleLevelBuildDraft {
    fn publish_input(&self) -> SpirePublishCoordinatorInput<'_> {
        SpirePublishCoordinatorInput {
            epoch_manifest: &self.epoch_manifest,
            object_manifest: &self.object_manifest,
            placement_directory: &self.placement_directory,
            local_store_config: SpireLocalStoreConfig::from_placement_directory(
                self.epoch_manifest.epoch,
                &self.placement_directory,
            )
            .expect("single-level draft placements should form a local store config"),
            next_pid: self.next_pid,
            next_local_vec_seq: self.next_local_vec_seq,
        }
    }

    pub(super) fn encode_manifest_bundle(&self) -> Result<SpireEncodedManifestBundle, String> {
        encode_manifest_bundle_for_publish(self.publish_input())
    }

    pub(super) fn root_control_state(
        &self,
        locators: SpirePublishedManifestLocators,
    ) -> Result<SpireRootControlState, String> {
        root_control_state_for_publish(self.publish_input(), locators)
    }

    pub(super) fn encode_publish_bundle(
        &self,
        locators: SpirePublishedManifestLocators,
    ) -> Result<SpireEncodedPublishBundle, String> {
        encode_publish_bundle_for_publish(self.publish_input(), locators)
    }
}

impl SpirePartitionedSingleLevelBuildDraft {
    fn publish_input(&self) -> SpirePublishCoordinatorInput<'_> {
        SpirePublishCoordinatorInput {
            epoch_manifest: &self.epoch_manifest,
            object_manifest: &self.object_manifest,
            placement_directory: &self.placement_directory,
            local_store_config: SpireLocalStoreConfig::from_placement_directory(
                self.epoch_manifest.epoch,
                &self.placement_directory,
            )
            .expect("partitioned draft placements should form a local store config"),
            next_pid: self.next_pid,
            next_local_vec_seq: self.next_local_vec_seq,
        }
    }

    pub(super) fn encode_manifest_bundle(&self) -> Result<SpireEncodedManifestBundle, String> {
        encode_manifest_bundle_for_publish(self.publish_input())
    }

    pub(super) fn root_control_state(
        &self,
        locators: SpirePublishedManifestLocators,
    ) -> Result<SpireRootControlState, String> {
        root_control_state_for_publish(self.publish_input(), locators)
    }

    pub(super) fn encode_publish_bundle(
        &self,
        locators: SpirePublishedManifestLocators,
    ) -> Result<SpireEncodedPublishBundle, String> {
        encode_publish_bundle_for_publish(self.publish_input(), locators)
    }
}

impl SpireRecursiveRoutingEpochDraft {
    fn publish_input(&self, next_local_vec_seq: u64) -> SpirePublishCoordinatorInput<'_> {
        SpirePublishCoordinatorInput {
            epoch_manifest: &self.epoch_manifest,
            object_manifest: &self.object_manifest,
            placement_directory: &self.placement_directory,
            local_store_config: SpireLocalStoreConfig::from_placement_directory(
                self.epoch_manifest.epoch,
                &self.placement_directory,
            )
            .expect("recursive draft placements should form a local store config"),
            next_pid: self.next_pid,
            next_local_vec_seq,
        }
    }

    pub(super) fn encode_manifest_bundle(
        &self,
        next_local_vec_seq: u64,
    ) -> Result<SpireEncodedManifestBundle, String> {
        encode_manifest_bundle_for_publish(self.publish_input(next_local_vec_seq))
    }

    pub(super) fn root_control_state(
        &self,
        next_local_vec_seq: u64,
        locators: SpirePublishedManifestLocators,
    ) -> Result<SpireRootControlState, String> {
        root_control_state_for_publish(self.publish_input(next_local_vec_seq), locators)
    }

    pub(super) fn encode_publish_bundle(
        &self,
        next_local_vec_seq: u64,
        locators: SpirePublishedManifestLocators,
    ) -> Result<SpireEncodedPublishBundle, String> {
        encode_publish_bundle_for_publish(self.publish_input(next_local_vec_seq), locators)
    }

    fn relation_publish_input<'a>(
        &'a self,
        object_manifest: &'a SpireObjectManifest,
        next_local_vec_seq: u64,
        local_store_config: SpireLocalStoreConfig,
    ) -> SpirePublishCoordinatorInput<'a> {
        SpirePublishCoordinatorInput {
            epoch_manifest: &self.epoch_manifest,
            object_manifest,
            placement_directory: &self.placement_directory,
            local_store_config,
            next_pid: self.next_pid,
            next_local_vec_seq,
        }
    }
}

pub(super) unsafe fn publish_relation_recursive_routing_epoch_draft(
    index_relation: pg_sys::Relation,
    draft: &SpireRecursiveRoutingEpochDraft,
    next_local_vec_seq: u64,
    local_store_config: SpireLocalStoreConfig,
) -> Result<(), String> {
    let placement_evidence =
        unsafe { write_placement_entries_to_relation(index_relation, &draft.placement_directory)? };
    let object_manifest = object_manifest_from_placement_writes(
        draft.epoch_manifest.epoch,
        &draft.placement_directory,
        &placement_evidence,
    )?;
    let input = draft.relation_publish_input(&object_manifest, next_local_vec_seq, local_store_config);
    let manifests = encode_manifest_bundle_for_publish(input.clone())?;
    let locators = unsafe { write_manifest_bundle_to_relation(index_relation, &manifests)? };
    let root_control = root_control_state_for_publish(input, locators)?;
    unsafe { page::initialize_root_control_page(index_relation, root_control) };
    Ok(())
}
