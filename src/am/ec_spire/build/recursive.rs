#[derive(Debug, Clone)]
struct SpirePendingRecursiveRoutingNode {
    pid: u64,
    level: u16,
    centroid: Vec<f32>,
    source_count: u64,
    children: Vec<SpireRecursiveRoutingChildInput>,
}

#[derive(Debug, Clone, Default, PartialEq)]
struct SpireRecursiveLeafMaterializationRows {
    rows: Vec<SpireLeafAssignmentRow>,
    source_vectors: Vec<Vec<f32>>,
}

pub(super) fn build_recursive_routing_hierarchy_draft(
    input: SpireRecursiveRoutingBuildInput,
    pid_allocator: &mut SpirePidAllocator,
) -> Result<SpireRecursiveRoutingBuildDraft, String> {
    let mut timing = SpireBuildTiming::default();
    build_recursive_routing_hierarchy_draft_with_timing(input, pid_allocator, &mut timing)
}

fn build_recursive_routing_hierarchy_draft_with_timing(
    input: SpireRecursiveRoutingBuildInput,
    pid_allocator: &mut SpirePidAllocator,
    timing: &mut SpireBuildTiming,
) -> Result<SpireRecursiveRoutingBuildDraft, String> {
    input.validate()?;
    let target_fanout = usize::try_from(input.target_fanout)
        .map_err(|_| "ec_spire recursive routing fanout exceeds usize".to_owned())?;
    let mut pid_cursor = *pid_allocator;
    let initial_child_count = input.children.len();
    let mut current_children = input.children;
    let mut pending_nodes = Vec::new();
    let mut routing_iterations = 0_usize;

    while current_children.len() > target_fanout {
        routing_iterations += 1;
        let child_level = current_children[0].child_level;
        let parent_level = child_level
            .checked_add(1)
            .ok_or_else(|| "ec_spire recursive routing level overflow".to_owned())?;
        let source_vectors = current_children
            .iter()
            .map(|child| child.centroid.as_slice())
            .collect::<Vec<_>>();
        let kmeans_started = Instant::now();
        let model = common_training::train_spherical_kmeans(
            "ec_spire recursive routing",
            &source_vectors,
            usize::from(input.dimensions),
            target_fanout,
            input.seed.wrapping_add(u64::from(parent_level)),
            SPIRE_DEFAULT_KMEANS_ITERATIONS,
        )?;
        timing.add_recursive_kmeans(parent_level, kmeans_started.elapsed());
        let child_centroids = current_children
            .iter()
            .map(|child| child.centroid.as_slice())
            .collect::<Vec<_>>();
        let assignment_started = Instant::now();
        let centroid_indexes = common_training::assign_vectors_to_centroids(
            "ec_spire recursive routing",
            &child_centroids,
            &model,
        )?;
        timing.add_recursive_assignment(assignment_started.elapsed());
        let mut grouped_children = vec![Vec::new(); model.centroid_count()];
        for (child, centroid_index) in current_children.into_iter().zip(centroid_indexes) {
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
    timing.record_recursive_routing_children(
        initial_child_count,
        current_children.len(),
        routing_iterations,
    );

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
    let mut timing = SpireBuildTiming::default();
    build_recursive_epoch_input_from_centroid_plan_with_timing(
        input,
        pid_allocator,
        local_vec_id_allocator,
        &mut timing,
    )
}

fn build_recursive_epoch_input_from_centroid_plan_with_timing(
    input: SpireRecursiveBuildCoordinatorInput,
    pid_allocator: &mut SpirePidAllocator,
    local_vec_id_allocator: &mut SpireLocalVecIdAllocator,
    timing: &mut SpireBuildTiming,
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

    if input.centroid_plan.assignment_indexes.len() != input.assignments.len() {
        return Err(format!(
            "ec_spire recursive build centroid assignment count {} does not match assignment count {}",
            input.centroid_plan.assignment_indexes.len(),
            input.assignments.len()
        ));
    }
    let mut pid_cursor = *pid_allocator;
    let mut local_vec_id_cursor = *local_vec_id_allocator;
    let pid_alloc_started = Instant::now();
    let mut leaf_pids = Vec::with_capacity(centroid_count);
    for _ in 0..centroid_count {
        leaf_pids.push(pid_cursor.allocate()?);
    }
    timing.add_draft_pid_alloc(pid_alloc_started.elapsed());
    let recursive_routing_started = Instant::now();
    let routing_draft = build_recursive_routing_hierarchy_draft_with_timing(
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
        timing,
    )?;
    timing.add_draft_recursive_routing(recursive_routing_started.elapsed());
    let validation_started = Instant::now();
    let leaf_parent_pids = assert_recursive_draft_invariants(&routing_draft)?.leaf_parent_pids;
    timing.add_draft_validation(validation_started.elapsed());
    let route_map_started = Instant::now();
    let route_map = SpireSingleLevelRouteMap::from_centroid_plan(&input.centroid_plan, &leaf_pids)?;
    timing.add_draft_route_map(route_map_started.elapsed());
    let leaf_rows_started = Instant::now();
    let rows_by_leaf_pid = build_recursive_leaf_rows_by_pid(
        input.assignments,
        input.source_vectors,
        &input.centroid_plan.assignment_indexes,
        &route_map,
        input.boundary_replica_count,
        &mut local_vec_id_cursor,
    )?;
    timing.add_draft_leaf_rows(leaf_rows_started.elapsed());
    let leaf_inputs_started = Instant::now();
    let summary_block_rows = current_leaf_summary_block_rows()?;
    let mut leaf_inputs = Vec::with_capacity(centroid_count);
    for pid in leaf_pids.iter().copied() {
        let parent_pid = *leaf_parent_pids.get(&pid).ok_or_else(|| {
            format!("ec_spire recursive build missing routing parent for leaf pid {pid}")
        })?;
        let materialization_rows = rows_by_leaf_pid
            .get(&pid)
            .cloned()
            .ok_or_else(|| format!("ec_spire recursive build missing leaf rows for pid {pid}"))?;
        let summaries = match summary_block_rows {
            Some(block_rows) => build_leaf_block_summaries(
                &materialization_rows.rows,
                &materialization_rows.source_vectors,
                block_rows,
            )?,
            None => Vec::new(),
        };
        leaf_inputs.push(SpireRecursiveLeafObjectInput {
            pid,
            object_version: input.object_version,
            parent_pid,
            rows: materialization_rows.rows,
            summaries,
            summary_block_rows: summary_block_rows.unwrap_or(0),
        });
    }
    timing.add_draft_leaf_inputs(leaf_inputs_started.elapsed());

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
    assignments: Vec<SpireLeafAssignmentIdentityInput>,
    source_vectors: Vec<Vec<f32>>,
    assignment_indexes: &[u32],
    route_map: &SpireSingleLevelRouteMap,
    boundary_replica_count: u32,
    local_vec_id_cursor: &mut SpireLocalVecIdAllocator,
) -> Result<HashMap<u64, SpireRecursiveLeafMaterializationRows>, String> {
    let mut rows_by_leaf_pid = route_map
        .entries
        .iter()
        .map(|entry| (entry.pid, SpireRecursiveLeafMaterializationRows::default()))
        .collect::<HashMap<_, _>>();
    if boundary_replica_count == 0 {
        let mut source_vectors_by_placement = Vec::with_capacity(source_vectors.len());
        let boundary_inputs = assignments
            .into_iter()
            .zip(source_vectors.into_iter())
            .zip(assignment_indexes.iter())
            .map(|((assignment, source_vector), assignment_index)| {
                let primary_pid = route_map
                    .get(*assignment_index)
                    .ok_or_else(|| {
                        format!(
                            "ec_spire recursive build assignment index {} has no leaf route",
                            assignment_index
                        )
                    })?
                    .pid;
                source_vectors_by_placement.push(source_vector);
                Ok(SpireBoundaryLeafAssignmentIdentityInput {
                    primary_pid,
                    replica_pids: Vec::new(),
                    assignment,
                })
            })
            .collect::<Result<Vec<_>, String>>()?;
        let placements = build_boundary_leaf_assignment_placements_with_identity(
            local_vec_id_cursor,
            boundary_inputs,
        )?;
        if placements.len() != source_vectors_by_placement.len() {
            return Err(format!(
                "ec_spire recursive primary assignment placement count {} does not match source vector count {}",
                placements.len(),
                source_vectors_by_placement.len()
            ));
        }
        for (placement, source_vector) in placements.into_iter().zip(source_vectors_by_placement) {
            let rows = rows_by_leaf_pid.get_mut(&placement.pid).ok_or_else(|| {
                format!(
                    "ec_spire recursive primary assignment resolved unknown leaf pid {}",
                    placement.pid
                )
            })?;
            rows.rows.push(placement.row);
            rows.source_vectors.push(source_vector);
        }
        return Ok(rows_by_leaf_pid);
    }

    let mut source_vectors_by_placement = Vec::new();
    let boundary_inputs = assignments
        .into_iter()
        .zip(source_vectors.into_iter())
        .zip(assignment_indexes.iter())
        .map(|((assignment, source_vector), assignment_index)| {
            let primary_pid = route_map
                .get(*assignment_index)
                .ok_or_else(|| {
                    format!(
                        "ec_spire recursive build assignment index {} has no leaf route",
                        assignment_index
                    )
                })?
                .pid;
            let boundary_plan = route_map.route_boundary_assignment_for_vector(
                &source_vector,
                boundary_replica_count.saturating_add(1),
            )?;
            let replica_pids: Vec<u64> = std::iter::once(boundary_plan.primary_pid)
                .chain(boundary_plan.replica_pids.into_iter())
                .filter(|pid| *pid != primary_pid)
                .take(usize::try_from(boundary_replica_count).unwrap_or(usize::MAX))
                .collect();
            let placement_count = 1_usize
                .checked_add(replica_pids.len())
                .ok_or_else(|| "ec_spire recursive boundary placement count overflow".to_owned())?;
            for _ in 0..placement_count {
                source_vectors_by_placement.push(source_vector.clone());
            }
            Ok(SpireBoundaryLeafAssignmentIdentityInput {
                primary_pid,
                replica_pids,
                assignment,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let placements = build_boundary_leaf_assignment_placements_with_identity(
        local_vec_id_cursor,
        boundary_inputs,
    )?;
    if placements.len() != source_vectors_by_placement.len() {
        return Err(format!(
            "ec_spire recursive boundary assignment placement count {} does not match source vector count {}",
            placements.len(),
            source_vectors_by_placement.len()
        ));
    }
    for (placement, source_vector) in placements.into_iter().zip(source_vectors_by_placement) {
        let rows = rows_by_leaf_pid.get_mut(&placement.pid).ok_or_else(|| {
            format!(
                "ec_spire recursive boundary assignment resolved unknown leaf pid {}",
                placement.pid
            )
        })?;
        rows.rows.push(placement.row);
        rows.source_vectors.push(source_vector);
    }
    Ok(rows_by_leaf_pid)
}

fn current_leaf_summary_block_rows() -> Result<Option<u32>, String> {
    let block_rows = options::current_session_leaf_block_rows();
    if block_rows <= 0 {
        return Ok(None);
    }
    u32::try_from(block_rows)
        .map(Some)
        .map_err(|_| "ec_spire.leaf_block_rows exceeds u32".to_owned())
}

fn build_leaf_block_summaries(
    rows: &[SpireLeafAssignmentRow],
    source_vectors: &[Vec<f32>],
    block_rows: u32,
) -> Result<Vec<SpireLeafBlockSummary>, String> {
    if block_rows == 0 {
        return Err("ec_spire leaf summary block_rows 0 is invalid".to_owned());
    }
    if rows.len() != source_vectors.len() {
        return Err(format!(
            "ec_spire leaf summary source vector count {} does not match row count {}",
            source_vectors.len(),
            rows.len()
        ));
    }
    if rows.is_empty() {
        return Ok(Vec::new());
    }

    let payload_format = SpireAssignmentPayloadFormat::from_tag(rows[0].payload_format)?;
    let dimensions = source_vectors[0].len();
    if dimensions == 0 {
        return Err("ec_spire leaf summary source vector dimensions must be > 0".to_owned());
    }
    for (row_index, (row, source_vector)) in rows.iter().zip(source_vectors.iter()).enumerate() {
        let row_payload_format = SpireAssignmentPayloadFormat::from_tag(row.payload_format)?;
        if row_payload_format != payload_format {
            return Err(format!(
                "ec_spire leaf summary row {row_index} payload format {:?} does not match first row {:?}",
                row_payload_format, payload_format
            ));
        }
        if source_vector.len() != dimensions {
            return Err(format!(
                "ec_spire leaf summary source vector {row_index} dimensions {} do not match expected {dimensions}",
                source_vector.len()
            ));
        }
        if source_vector.iter().any(|value| !value.is_finite()) {
            return Err(format!(
                "ec_spire leaf summary source vector {row_index} contains a non-finite value"
            ));
        }
    }

    let block_rows_usize = usize::try_from(block_rows)
        .map_err(|_| "ec_spire leaf summary block_rows exceeds usize".to_owned())?;
    let mut summaries = Vec::with_capacity(rows.len().div_ceil(block_rows_usize));
    for (block_index, block_vectors) in source_vectors.chunks(block_rows_usize).enumerate() {
        let row_base_usize = block_index
            .checked_mul(block_rows_usize)
            .ok_or_else(|| "ec_spire leaf summary row_base overflow".to_owned())?;
        let row_base = u32::try_from(row_base_usize)
            .map_err(|_| "ec_spire leaf summary row_base exceeds u32".to_owned())?;
        let row_count = u32::try_from(block_vectors.len())
            .map_err(|_| "ec_spire leaf summary row_count exceeds u32".to_owned())?;
        let mut mean = vec![0.0_f32; dimensions];
        for source_vector in block_vectors {
            for (accum, value) in mean.iter_mut().zip(source_vector.iter()) {
                *accum += *value;
            }
        }
        let inv = 1.0_f32 / block_vectors.len() as f32;
        for value in &mut mean {
            *value *= inv;
        }
        let mut summary_gamma = 0.0_f32;
        if payload_format == SpireAssignmentPayloadFormat::RaBitQ {
            for source_vector in block_vectors {
                let residual_l2 = source_vector
                    .iter()
                    .zip(mean.iter())
                    .map(|(value, mean_value)| {
                        let residual = value - mean_value;
                        residual * residual
                    })
                    .sum::<f32>()
                    .sqrt();
                summary_gamma = summary_gamma.max(residual_l2);
            }
        }
        let (gamma, encoded_payload) = quantizer::encode_assignment_payload(payload_format, &mean)?;
        if payload_format != SpireAssignmentPayloadFormat::RaBitQ {
            summary_gamma = gamma;
        }
        summaries.push(SpireLeafBlockSummary {
            row_base,
            row_count,
            payload_format: payload_format.tag(),
            gamma: summary_gamma,
            encoded_payload,
        });
    }
    Ok(summaries)
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

pub(super) fn build_local_recursive_top_graph_epoch_draft(
    input: SpireRecursiveTopGraphEpochInput,
    object_store: &mut SpireLocalObjectStore,
) -> Result<SpireRecursiveRoutingEpochDraft, String> {
    build_recursive_top_graph_epoch_draft_with_store(input, object_store)
}

pub(super) fn build_local_recursive_routing_epoch_from_leaf_inputs(
    input: SpireRecursiveRoutingEpochObjectInput,
    object_store: &mut SpireLocalObjectStore,
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
        leaf_placements.push(write_recursive_leaf_object_from_input(
            object_store,
            input.epoch,
            &leaf_input,
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

fn build_recursive_top_graph_epoch_from_leaf_inputs_with_store(
    input: SpireRecursiveRoutingEpochObjectInput,
    top_graph_params: SpireTopGraphBuildParams,
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
        leaf_placements.push(write_recursive_leaf_object_from_input(
            object_store,
            input.epoch,
            &leaf_input,
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

    build_recursive_top_graph_epoch_draft_with_store(
        SpireRecursiveTopGraphEpochInput {
            epoch_input: SpireRecursiveRoutingEpochInput {
                epoch: input.epoch,
                published_at_micros: input.published_at_micros,
                retain_until_micros: input.retain_until_micros,
                consistency_mode: input.consistency_mode,
                routing_draft: input.routing_draft,
                leaf_placements,
            },
            top_graph_params,
        },
        object_store,
    )
}

fn write_recursive_leaf_object_from_input(
    object_store: &mut impl SpireBuildObjectStore,
    epoch: u64,
    leaf_input: &SpireRecursiveLeafObjectInput,
) -> Result<SpirePlacementEntry, String> {
    if leaf_input.summaries.is_empty() {
        return object_store.write_leaf_object_v2_from_rows(
            epoch,
            leaf_input.pid,
            leaf_input.object_version,
            leaf_input.parent_pid,
            &leaf_input.rows,
        );
    }
    object_store.write_leaf_object_v3_from_rows_and_summaries(
        epoch,
        leaf_input.pid,
        leaf_input.object_version,
        leaf_input.parent_pid,
        &leaf_input.rows,
        &leaf_input.summaries,
        leaf_input.summary_block_rows,
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

    let mut placements = Vec::with_capacity(
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

/// Publish a finished SPIRE recursive-routing epoch draft to the index
/// relation, writing placement entries and the manifest bundle in WAL.
///
/// # Safety
///
/// `index_relation` must be the SPIRE index relation that PostgreSQL has
/// opened for the calling `ambuild` / publish phase with at least
/// `AccessExclusiveLock`; it must remain open for the duration of this
/// call. The `draft` placement directory must have been validated by
/// recursive routing against the same relation. `next_local_vec_seq` and
/// `local_store_config` must reflect the state captured at the start of
/// the recursive build — passing values from an earlier or later epoch
/// will publish an inconsistent manifest.
pub(super) unsafe fn publish_relation_recursive_routing_epoch_draft(
    index_relation: pg_sys::Relation,
    draft: &SpireRecursiveRoutingEpochDraft,
    next_local_vec_seq: u64,
    local_store_config: SpireLocalStoreConfig,
) -> Result<(), String> {
    let placement_evidence = {
        // SAFETY: index_relation is the open PostgreSQL index relation for this
        // publish, and the draft placement directory was validated while building
        // the recursive epoch before being serialized into relation pages.
        write_placement_entries_to_relation(index_relation, &draft.placement_directory)?
    };
    let object_manifest = object_manifest_from_placement_writes(
        draft.epoch_manifest.epoch,
        &draft.placement_directory,
        &placement_evidence,
    )?;
    let input =
        draft.relation_publish_input(&object_manifest, next_local_vec_seq, local_store_config);
    let manifests = encode_manifest_bundle_for_publish(input.clone())?;
    // SAFETY: manifests were encoded from the just-assembled publish input, and
    // index_relation still names the target relation that receives those
    // manifest pages.
    let locators = write_manifest_bundle_to_relation(index_relation, &manifests)?;
    let root_control = root_control_state_for_publish(input, locators)?;
    // SAFETY: root_control references locators returned by the manifest write
    // above and is written to the root control page of the same open relation.
    page::initialize_root_control_page(index_relation, root_control);
    Ok(())
}
