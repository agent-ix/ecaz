pub(super) fn build_scheduled_routing_replacement_children(
    decision: &SpireLeafReplacementScheduleDecision,
    pid_plan: &SpireLeafReplacementPidPlan,
    centroids: Vec<Vec<f32>>,
) -> Result<Vec<SpireRoutingReplacementChild>, String> {
    validate_leaf_replacement_schedule_decision_shape(decision)?;
    if pid_plan.reuses_existing_pid {
        return Err(
            "ec_spire scheduled routing replacement requires fresh replacement pids".to_owned(),
        );
    }
    if pid_plan.replacement_pids.len() != decision.replacement_leaf_count {
        return Err(format!(
            "ec_spire scheduled routing replacement pid count {} does not match decision replacement count {}",
            pid_plan.replacement_pids.len(),
            decision.replacement_leaf_count
        ));
    }
    if centroids.len() != pid_plan.replacement_pids.len() {
        return Err(format!(
            "ec_spire scheduled routing replacement centroid count {} does not match replacement pid count {}",
            centroids.len(),
            pid_plan.replacement_pids.len()
        ));
    }

    let mut seen_pids = HashSet::new();
    let mut children = Vec::with_capacity(pid_plan.replacement_pids.len());
    for (pid, centroid) in pid_plan.replacement_pids.iter().zip(centroids) {
        if *pid == 0 {
            return Err("ec_spire scheduled routing replacement pid 0 is invalid".to_owned());
        }
        if *pid >= pid_plan.next_pid {
            return Err(format!(
                "ec_spire scheduled routing replacement pid plan next_pid {} does not advance past replacement pid {pid}",
                pid_plan.next_pid
            ));
        }
        if !seen_pids.insert(*pid) {
            return Err("ec_spire scheduled routing replacement pids must be unique".to_owned());
        }
        if centroid.is_empty() {
            return Err(format!(
                "ec_spire scheduled routing replacement child pid {pid} centroid is empty"
            ));
        }
        if centroid.iter().any(|component| !component.is_finite()) {
            return Err(format!(
                "ec_spire scheduled routing replacement child pid {pid} centroid must be finite"
            ));
        }
        children.push(SpireRoutingReplacementChild {
            child_pid: *pid,
            centroid,
        });
    }

    Ok(children)
}

pub(super) fn build_scheduled_merge_replacement_centroids(
    decision: &SpireLeafReplacementScheduleDecision,
    parent: &SpireRoutingPartitionObject,
    rows: &[SpireIndexLeafSnapshotRow],
) -> Result<Vec<Vec<f32>>, String> {
    validate_leaf_replacement_schedule_decision_shape(decision)?;
    validate_leaf_replacement_schedule_rows(rows)?;
    if decision.mode != SpireLeafReplacementScheduleMode::Merge {
        return Err("ec_spire scheduled merge centroid requires a merge decision".to_owned());
    }
    if parent.header.pid != decision.replaced_parent_pid {
        return Err(format!(
            "ec_spire scheduled merge centroid parent pid {} does not match decision parent pid {}",
            parent.header.pid, decision.replaced_parent_pid
        ));
    }

    let dimensions = usize::from(parent.dimensions);
    if dimensions == 0 {
        return Err("ec_spire scheduled merge centroid parent dimensions 0 is invalid".to_owned());
    }
    let children_by_pid = parent
        .children()
        .map(|child| (child.child_pid, child))
        .collect::<HashMap<_, _>>();
    let rows_by_leaf_pid = rows
        .iter()
        .map(|row| (row.leaf_pid, row))
        .collect::<HashMap<_, _>>();
    let mut selected = Vec::with_capacity(decision.affected_leaf_pids.len());
    let mut total_weight = 0_u64;
    for leaf_pid in &decision.affected_leaf_pids {
        let row = rows_by_leaf_pid.get(leaf_pid).ok_or_else(|| {
            format!(
                "ec_spire scheduled merge centroid missing snapshot row for leaf pid {leaf_pid}"
            )
        })?;
        if row.active_epoch != decision.active_epoch {
            return Err(format!(
                "ec_spire scheduled merge centroid row epoch {} does not match decision epoch {}",
                row.active_epoch, decision.active_epoch
            ));
        }
        if row.parent_pid != decision.replaced_parent_pid {
            return Err(format!(
                "ec_spire scheduled merge centroid row parent pid {} does not match decision parent pid {}",
                row.parent_pid, decision.replaced_parent_pid
            ));
        }
        if !row.merge_recommended {
            return Err(format!(
                "ec_spire scheduled merge centroid affected leaf pid {leaf_pid} is no longer merge recommended"
            ));
        }
        let child = children_by_pid.get(leaf_pid).ok_or_else(|| {
            format!(
                "ec_spire scheduled merge centroid parent is missing affected leaf pid {leaf_pid}"
            )
        })?;
        if child.centroid.len() != dimensions {
            return Err(format!(
                "ec_spire scheduled merge centroid child pid {leaf_pid} dimensions {} do not match parent dimensions {dimensions}",
                child.centroid.len()
            ));
        }
        if child
            .centroid
            .iter()
            .any(|component| !component.is_finite())
        {
            return Err(format!(
                "ec_spire scheduled merge centroid child pid {leaf_pid} centroid must be finite"
            ));
        }
        total_weight = total_weight
            .checked_add(row.effective_assignment_count)
            .ok_or_else(|| "ec_spire scheduled merge centroid row weight overflow".to_owned())?;
        selected.push((*child, row.effective_assignment_count));
    }

    let mut centroid = vec![0.0_f64; dimensions];
    if total_weight == 0 {
        let weight = 1.0 / selected.len() as f64;
        for (child, _) in &selected {
            for (sum, component) in centroid.iter_mut().zip(child.centroid.iter()) {
                *sum += f64::from(*component) * weight;
            }
        }
    } else {
        let total_weight = total_weight as f64;
        for (child, row_weight) in &selected {
            let weight = *row_weight as f64 / total_weight;
            for (sum, component) in centroid.iter_mut().zip(child.centroid.iter()) {
                *sum += f64::from(*component) * weight;
            }
        }
    }

    let centroid = centroid
        .into_iter()
        .map(|component| component as f32)
        .collect::<Vec<_>>();
    let centroid = common_training::normalize_vector(
        "ec_spire scheduled merge centroid",
        &centroid,
        dimensions,
    )?;
    Ok(vec![centroid])
}

pub(super) fn load_scheduled_replacement_parent_routing(
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    object_store: &impl SpireObjectReader,
    decision: &SpireLeafReplacementScheduleDecision,
) -> Result<SpireRoutingPartitionObject, String> {
    validate_leaf_replacement_schedule_decision_shape(decision)?;
    let snapshot = SpireValidatedEpochSnapshot::from_snapshot(*snapshot)?;
    if snapshot.epoch_manifest().epoch != decision.active_epoch {
        return Err(format!(
            "ec_spire scheduled replacement parent snapshot epoch {} does not match decision active epoch {}",
            snapshot.epoch_manifest().epoch, decision.active_epoch
        ));
    }
    let lookup = snapshot.require_lookup(
        decision.replaced_parent_pid,
        "scheduled replacement parent routing",
    )?;
    if lookup.placement.state != SpirePlacementState::Available {
        return Err(format!(
            "ec_spire scheduled replacement parent pid {} placement is not available",
            decision.replaced_parent_pid
        ));
    }
    let parent = object_store.read_routing_object(lookup.placement)?;
    if parent.header.pid != decision.replaced_parent_pid {
        return Err(format!(
            "ec_spire scheduled replacement loaded parent pid {} does not match decision parent pid {}",
            parent.header.pid, decision.replaced_parent_pid
        ));
    }
    let parent_child_pids = parent
        .children()
        .map(|child| child.child_pid)
        .collect::<HashSet<_>>();
    for affected_pid in &decision.affected_leaf_pids {
        if !parent_child_pids.contains(affected_pid) {
            return Err(format!(
                "ec_spire scheduled replacement parent pid {} is missing affected leaf pid {affected_pid}",
                decision.replaced_parent_pid
            ));
        }
    }
    Ok(parent)
}

pub(super) fn load_selected_scheduled_replacement_parent_routing(
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    object_store: &impl SpireObjectReader,
    selected: &SpireSelectedScheduledReplacementPublishLockPlan,
) -> Result<SpireRoutingPartitionObject, String> {
    validate_selected_scheduled_replacement_execution_snapshot(snapshot, selected)?;
    load_scheduled_replacement_parent_routing(snapshot, object_store, &selected.decision)
}

pub(super) fn build_scheduled_merge_replacement_routing_parts(
    decision: &SpireLeafReplacementScheduleDecision,
    pid_plan: &SpireLeafReplacementPidPlan,
    parent: &SpireRoutingPartitionObject,
    rows: &[SpireIndexLeafSnapshotRow],
    parent_object_version: u64,
) -> Result<SpireScheduledReplacementRoutingParts, String> {
    let centroids = build_scheduled_merge_replacement_centroids(decision, parent, rows)?;
    let replacement_children =
        build_scheduled_routing_replacement_children(decision, pid_plan, centroids)?;
    let replacement_parent = rewrite_scheduled_replacement_parent_routing(
        parent,
        decision,
        replacement_children.clone(),
        parent_object_version,
    )?;
    Ok(SpireScheduledReplacementRoutingParts {
        replacement_parent,
        replacement_children,
    })
}

pub(super) fn build_scheduled_split_replacement_routing_parts(
    decision: &SpireLeafReplacementScheduleDecision,
    pid_plan: &SpireLeafReplacementPidPlan,
    parent: &SpireRoutingPartitionObject,
    centroids: Vec<Vec<f32>>,
    parent_object_version: u64,
) -> Result<SpireScheduledReplacementRoutingParts, String> {
    validate_leaf_replacement_schedule_decision_shape(decision)?;
    if decision.mode != SpireLeafReplacementScheduleMode::Split {
        return Err("ec_spire scheduled split routing parts require a split decision".to_owned());
    }
    if parent.header.pid != decision.replaced_parent_pid {
        return Err(format!(
            "ec_spire scheduled split routing parent pid {} does not match decision parent pid {}",
            parent.header.pid, decision.replaced_parent_pid
        ));
    }
    let replacement_children =
        build_scheduled_routing_replacement_children(decision, pid_plan, centroids)?;
    let replacement_parent = rewrite_scheduled_replacement_parent_routing(
        parent,
        decision,
        replacement_children.clone(),
        parent_object_version,
    )?;
    Ok(SpireScheduledReplacementRoutingParts {
        replacement_parent,
        replacement_children,
    })
}

pub(super) fn build_relation_scheduled_merge_replacement_execution_parts(
    decision: &SpireLeafReplacementScheduleDecision,
    pid_plan: &SpireLeafReplacementPidPlan,
    parent: &SpireRoutingPartitionObject,
    rows: &[SpireIndexLeafSnapshotRow],
    replacement_leaf_rows: Vec<SpireReplacementLeafRows>,
    parent_object_version: u64,
    leaf_object_version: u64,
    published_at_micros: i64,
    retain_until_micros: i64,
) -> Result<SpireRelationScheduledReplacementExecutionParts, String> {
    let routing_parts = build_scheduled_merge_replacement_routing_parts(
        decision,
        pid_plan,
        parent,
        rows,
        parent_object_version,
    )?;
    let leaf_input =
        build_merge_replacement_leaf_object_input(decision, pid_plan, replacement_leaf_rows)?;
    let leaf_inputs = vec![leaf_input];
    validate_replacement_leaf_object_inputs(&routing_parts.replacement_children, &leaf_inputs)?;
    Ok(SpireRelationScheduledReplacementExecutionParts {
        published_at_micros,
        retain_until_micros,
        replacement_parent: routing_parts.replacement_parent,
        replacement_children: routing_parts.replacement_children,
        leaf_object_version,
        leaf_inputs,
    })
}

pub(super) fn build_relation_scheduled_split_replacement_execution_parts(
    decision: &SpireLeafReplacementScheduleDecision,
    pid_plan: &SpireLeafReplacementPidPlan,
    parent: &SpireRoutingPartitionObject,
    centroids: Vec<Vec<f32>>,
    routed_leaf_inputs: Vec<SpireReplacementLeafObjectInput>,
    parent_object_version: u64,
    leaf_object_version: u64,
    published_at_micros: i64,
    retain_until_micros: i64,
) -> Result<SpireRelationScheduledReplacementExecutionParts, String> {
    let routing_parts = build_scheduled_split_replacement_routing_parts(
        decision,
        pid_plan,
        parent,
        centroids,
        parent_object_version,
    )?;
    let leaf_inputs =
        build_split_replacement_leaf_object_inputs(decision, pid_plan, routed_leaf_inputs)?;
    validate_replacement_leaf_object_inputs(&routing_parts.replacement_children, &leaf_inputs)?;
    Ok(SpireRelationScheduledReplacementExecutionParts {
        published_at_micros,
        retain_until_micros,
        replacement_parent: routing_parts.replacement_parent,
        replacement_children: routing_parts.replacement_children,
        leaf_object_version,
        leaf_inputs,
    })
}

fn local_scheduled_replacement_execution_parts_from_relation_parts(
    parts: SpireRelationScheduledReplacementExecutionParts,
    placement_write_evidence: Vec<SpirePublishPlacementWriteEvidence>,
) -> SpireLocalScheduledReplacementExecutionParts {
    SpireLocalScheduledReplacementExecutionParts {
        published_at_micros: parts.published_at_micros,
        retain_until_micros: parts.retain_until_micros,
        replacement_parent: parts.replacement_parent,
        replacement_children: parts.replacement_children,
        leaf_object_version: parts.leaf_object_version,
        leaf_inputs: parts.leaf_inputs,
        placement_write_evidence,
    }
}

pub(super) fn build_local_scheduled_merge_replacement_execution_parts(
    decision: &SpireLeafReplacementScheduleDecision,
    pid_plan: &SpireLeafReplacementPidPlan,
    parent: &SpireRoutingPartitionObject,
    rows: &[SpireIndexLeafSnapshotRow],
    replacement_leaf_rows: Vec<SpireReplacementLeafRows>,
    parent_object_version: u64,
    leaf_object_version: u64,
    published_at_micros: i64,
    retain_until_micros: i64,
    placement_write_evidence: Vec<SpirePublishPlacementWriteEvidence>,
) -> Result<SpireLocalScheduledReplacementExecutionParts, String> {
    let parts = build_relation_scheduled_merge_replacement_execution_parts(
        decision,
        pid_plan,
        parent,
        rows,
        replacement_leaf_rows,
        parent_object_version,
        leaf_object_version,
        published_at_micros,
        retain_until_micros,
    )?;
    Ok(
        local_scheduled_replacement_execution_parts_from_relation_parts(
            parts,
            placement_write_evidence,
        ),
    )
}

pub(super) fn build_local_scheduled_split_replacement_execution_parts(
    decision: &SpireLeafReplacementScheduleDecision,
    pid_plan: &SpireLeafReplacementPidPlan,
    parent: &SpireRoutingPartitionObject,
    centroids: Vec<Vec<f32>>,
    routed_leaf_inputs: Vec<SpireReplacementLeafObjectInput>,
    parent_object_version: u64,
    leaf_object_version: u64,
    published_at_micros: i64,
    retain_until_micros: i64,
    placement_write_evidence: Vec<SpirePublishPlacementWriteEvidence>,
) -> Result<SpireLocalScheduledReplacementExecutionParts, String> {
    let parts = build_relation_scheduled_split_replacement_execution_parts(
        decision,
        pid_plan,
        parent,
        centroids,
        routed_leaf_inputs,
        parent_object_version,
        leaf_object_version,
        published_at_micros,
        retain_until_micros,
    )?;
    Ok(
        local_scheduled_replacement_execution_parts_from_relation_parts(
            parts,
            placement_write_evidence,
        ),
    )
}

pub(super) fn build_relation_scheduled_split_replacement_execution_input(
    publish_plan: &SpireScheduledReplacementPublishPlan,
    pid_plan: &SpireLeafReplacementPidPlan,
    decision: &SpireLeafReplacementScheduleDecision,
    parent: &SpireRoutingPartitionObject,
    centroids: Vec<Vec<f32>>,
    routed_leaf_inputs: Vec<SpireReplacementLeafObjectInput>,
    parent_object_version: u64,
    leaf_object_version: u64,
    published_at_micros: i64,
    retain_until_micros: i64,
) -> Result<SpireRelationScheduledReplacementExecutionInput, String> {
    let parts = build_relation_scheduled_split_replacement_execution_parts(
        decision,
        pid_plan,
        parent,
        centroids,
        routed_leaf_inputs,
        parent_object_version,
        leaf_object_version,
        published_at_micros,
        retain_until_micros,
    )?;
    build_relation_scheduled_replacement_execution_input_from_publish_plan(
        publish_plan,
        pid_plan,
        decision,
        parts,
    )
}

pub(super) fn build_relation_selected_scheduled_split_replacement_execution_input(
    selected: &SpireSelectedScheduledReplacementPublishLockPlan,
    parent: &SpireRoutingPartitionObject,
    centroids: Vec<Vec<f32>>,
    routed_leaf_inputs: Vec<SpireReplacementLeafObjectInput>,
    parent_object_version: u64,
    leaf_object_version: u64,
    published_at_micros: i64,
    retain_until_micros: i64,
) -> Result<SpireRelationScheduledReplacementExecutionInput, String> {
    build_relation_scheduled_split_replacement_execution_input(
        &selected.lock_plan.publish_plan,
        &selected.lock_plan.pid_plan,
        &selected.decision,
        parent,
        centroids,
        routed_leaf_inputs,
        parent_object_version,
        leaf_object_version,
        published_at_micros,
        retain_until_micros,
    )
}

pub(super) fn build_relation_selected_scheduled_split_replacement_execution_input_from_snapshot(
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    object_store: &impl SpireObjectReader,
    selected: &SpireSelectedScheduledReplacementPublishLockPlan,
    centroids: Vec<Vec<f32>>,
    routed_leaf_inputs: Vec<SpireReplacementLeafObjectInput>,
    parent_object_version: u64,
    leaf_object_version: u64,
    published_at_micros: i64,
    retain_until_micros: i64,
) -> Result<SpireRelationScheduledReplacementExecutionInput, String> {
    let parent =
        load_selected_scheduled_replacement_parent_routing(snapshot, object_store, selected)?;
    build_relation_selected_scheduled_split_replacement_execution_input(
        selected,
        &parent,
        centroids,
        routed_leaf_inputs,
        parent_object_version,
        leaf_object_version,
        published_at_micros,
        retain_until_micros,
    )
}

pub(super) fn build_relation_selected_scheduled_split_replacement_execution_input_from_snapshot_sources(
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    object_store: &impl SpireObjectReader,
    selected: &SpireSelectedScheduledReplacementPublishLockPlan,
    fetched_sources: Vec<SpireSplitReplacementFetchedSourceVector>,
    dimensions: usize,
    seed: u64,
    max_iterations: usize,
    parent_object_version: u64,
    leaf_object_version: u64,
    published_at_micros: i64,
    retain_until_micros: i64,
) -> Result<SpireRelationScheduledReplacementExecutionInput, String> {
    let parent =
        load_selected_scheduled_replacement_parent_routing(snapshot, object_store, selected)?;
    let replacement_rows =
        collect_selected_scheduled_replacement_leaf_rows(snapshot, object_store, selected)?;
    let materialized = build_split_replacement_leaf_materialization_from_rows(
        &selected.decision,
        &selected.lock_plan.pid_plan,
        replacement_rows,
        fetched_sources,
        dimensions,
        seed,
        max_iterations,
    )?;
    build_relation_selected_scheduled_split_replacement_execution_input(
        selected,
        &parent,
        materialized.centroids,
        materialized.leaf_inputs,
        parent_object_version,
        leaf_object_version,
        published_at_micros,
        retain_until_micros,
    )
}

pub(super) unsafe fn build_relation_selected_scheduled_split_replacement_execution_input_from_heap_sources(
    heap_relation: pgrx::pg_sys::Relation,
    heap_snapshot: pgrx::pg_sys::Snapshot,
    slot: *mut pgrx::pg_sys::TupleTableSlot,
    indexed_attribute: source::IndexedVectorAttribute,
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    object_store: &impl SpireObjectReader,
    selected: &SpireSelectedScheduledReplacementPublishLockPlan,
    dimensions: usize,
    seed: u64,
    max_iterations: usize,
    parent_object_version: u64,
    leaf_object_version: u64,
    published_at_micros: i64,
    retain_until_micros: i64,
) -> Result<SpireRelationScheduledReplacementExecutionInput, String> {
    let replacement_rows =
        collect_selected_scheduled_replacement_leaf_rows(snapshot, object_store, selected)?;
    let fetched_sources = unsafe {
        fetch_split_replacement_source_vectors(
            heap_relation,
            heap_snapshot,
            slot,
            indexed_attribute,
            &replacement_rows,
        )?
    };
    let replacement_rows =
        filter_split_replacement_rows_to_fetched_sources(replacement_rows, &fetched_sources)?;
    let parent =
        load_selected_scheduled_replacement_parent_routing(snapshot, object_store, selected)?;
    let materialized = build_split_replacement_leaf_materialization_from_rows(
        &selected.decision,
        &selected.lock_plan.pid_plan,
        replacement_rows,
        fetched_sources,
        dimensions,
        seed,
        max_iterations,
    )?;
    build_relation_selected_scheduled_split_replacement_execution_input(
        selected,
        &parent,
        materialized.centroids,
        materialized.leaf_inputs,
        parent_object_version,
        leaf_object_version,
        published_at_micros,
        retain_until_micros,
    )
}

pub(super) fn build_local_selected_scheduled_split_replacement_execution_input(
    selected: &SpireSelectedScheduledReplacementPublishLockPlan,
    parent: &SpireRoutingPartitionObject,
    centroids: Vec<Vec<f32>>,
    routed_leaf_inputs: Vec<SpireReplacementLeafObjectInput>,
    parent_object_version: u64,
    leaf_object_version: u64,
    published_at_micros: i64,
    retain_until_micros: i64,
    placement_write_evidence: Vec<SpirePublishPlacementWriteEvidence>,
) -> Result<SpireLocalScheduledReplacementExecutionInput, String> {
    build_local_scheduled_split_replacement_execution_input(
        &selected.lock_plan.publish_plan,
        &selected.lock_plan.pid_plan,
        &selected.decision,
        parent,
        centroids,
        routed_leaf_inputs,
        parent_object_version,
        leaf_object_version,
        published_at_micros,
        retain_until_micros,
        placement_write_evidence,
    )
}

pub(super) fn build_local_selected_scheduled_split_replacement_execution_input_from_snapshot(
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    object_store: &impl SpireObjectReader,
    selected: &SpireSelectedScheduledReplacementPublishLockPlan,
    centroids: Vec<Vec<f32>>,
    routed_leaf_inputs: Vec<SpireReplacementLeafObjectInput>,
    parent_object_version: u64,
    leaf_object_version: u64,
    published_at_micros: i64,
    retain_until_micros: i64,
    placement_write_evidence: Vec<SpirePublishPlacementWriteEvidence>,
) -> Result<SpireLocalScheduledReplacementExecutionInput, String> {
    let parent =
        load_selected_scheduled_replacement_parent_routing(snapshot, object_store, selected)?;
    build_local_selected_scheduled_split_replacement_execution_input(
        selected,
        &parent,
        centroids,
        routed_leaf_inputs,
        parent_object_version,
        leaf_object_version,
        published_at_micros,
        retain_until_micros,
        placement_write_evidence,
    )
}

pub(super) fn build_local_scheduled_split_replacement_execution_input(
    publish_plan: &SpireScheduledReplacementPublishPlan,
    pid_plan: &SpireLeafReplacementPidPlan,
    decision: &SpireLeafReplacementScheduleDecision,
    parent: &SpireRoutingPartitionObject,
    centroids: Vec<Vec<f32>>,
    routed_leaf_inputs: Vec<SpireReplacementLeafObjectInput>,
    parent_object_version: u64,
    leaf_object_version: u64,
    published_at_micros: i64,
    retain_until_micros: i64,
    placement_write_evidence: Vec<SpirePublishPlacementWriteEvidence>,
) -> Result<SpireLocalScheduledReplacementExecutionInput, String> {
    let parts = build_local_scheduled_split_replacement_execution_parts(
        decision,
        pid_plan,
        parent,
        centroids,
        routed_leaf_inputs,
        parent_object_version,
        leaf_object_version,
        published_at_micros,
        retain_until_micros,
        placement_write_evidence,
    )?;
    build_local_scheduled_replacement_execution_input_from_publish_plan(
        publish_plan,
        pid_plan,
        decision,
        parts,
    )
}

pub(super) fn build_local_scheduled_merge_replacement_execution_input(
    publish_plan: &SpireScheduledReplacementPublishPlan,
    pid_plan: &SpireLeafReplacementPidPlan,
    decision: &SpireLeafReplacementScheduleDecision,
    parent: &SpireRoutingPartitionObject,
    rows: &[SpireIndexLeafSnapshotRow],
    replacement_leaf_rows: Vec<SpireReplacementLeafRows>,
    parent_object_version: u64,
    leaf_object_version: u64,
    published_at_micros: i64,
    retain_until_micros: i64,
    placement_write_evidence: Vec<SpirePublishPlacementWriteEvidence>,
) -> Result<SpireLocalScheduledReplacementExecutionInput, String> {
    let parts = build_local_scheduled_merge_replacement_execution_parts(
        decision,
        pid_plan,
        parent,
        rows,
        replacement_leaf_rows,
        parent_object_version,
        leaf_object_version,
        published_at_micros,
        retain_until_micros,
        placement_write_evidence,
    )?;
    build_local_scheduled_replacement_execution_input_from_publish_plan(
        publish_plan,
        pid_plan,
        decision,
        parts,
    )
}

pub(super) fn build_relation_scheduled_merge_replacement_execution_input(
    publish_plan: &SpireScheduledReplacementPublishPlan,
    pid_plan: &SpireLeafReplacementPidPlan,
    decision: &SpireLeafReplacementScheduleDecision,
    parent: &SpireRoutingPartitionObject,
    rows: &[SpireIndexLeafSnapshotRow],
    replacement_leaf_rows: Vec<SpireReplacementLeafRows>,
    parent_object_version: u64,
    leaf_object_version: u64,
    published_at_micros: i64,
    retain_until_micros: i64,
) -> Result<SpireRelationScheduledReplacementExecutionInput, String> {
    let parts = build_relation_scheduled_merge_replacement_execution_parts(
        decision,
        pid_plan,
        parent,
        rows,
        replacement_leaf_rows,
        parent_object_version,
        leaf_object_version,
        published_at_micros,
        retain_until_micros,
    )?;
    build_relation_scheduled_replacement_execution_input_from_publish_plan(
        publish_plan,
        pid_plan,
        decision,
        parts,
    )
}

pub(super) fn build_relation_selected_scheduled_merge_replacement_execution_input(
    selected: &SpireSelectedScheduledReplacementPublishLockPlan,
    parent: &SpireRoutingPartitionObject,
    rows: &[SpireIndexLeafSnapshotRow],
    replacement_leaf_rows: Vec<SpireReplacementLeafRows>,
    parent_object_version: u64,
    leaf_object_version: u64,
    published_at_micros: i64,
    retain_until_micros: i64,
) -> Result<SpireRelationScheduledReplacementExecutionInput, String> {
    build_relation_scheduled_merge_replacement_execution_input(
        &selected.lock_plan.publish_plan,
        &selected.lock_plan.pid_plan,
        &selected.decision,
        parent,
        rows,
        replacement_leaf_rows,
        parent_object_version,
        leaf_object_version,
        published_at_micros,
        retain_until_micros,
    )
}

pub(super) fn build_relation_selected_scheduled_merge_replacement_execution_input_from_snapshot(
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    object_store: &impl SpireObjectReader,
    selected: &SpireSelectedScheduledReplacementPublishLockPlan,
    rows: &[SpireIndexLeafSnapshotRow],
    parent_object_version: u64,
    leaf_object_version: u64,
    published_at_micros: i64,
    retain_until_micros: i64,
) -> Result<SpireRelationScheduledReplacementExecutionInput, String> {
    let parent =
        load_selected_scheduled_replacement_parent_routing(snapshot, object_store, selected)?;
    let replacement_leaf_rows =
        collect_selected_scheduled_replacement_leaf_rows(snapshot, object_store, selected)?;
    build_relation_selected_scheduled_merge_replacement_execution_input(
        selected,
        &parent,
        rows,
        replacement_leaf_rows,
        parent_object_version,
        leaf_object_version,
        published_at_micros,
        retain_until_micros,
    )
}

pub(super) fn build_local_selected_scheduled_merge_replacement_execution_input(
    selected: &SpireSelectedScheduledReplacementPublishLockPlan,
    parent: &SpireRoutingPartitionObject,
    rows: &[SpireIndexLeafSnapshotRow],
    replacement_leaf_rows: Vec<SpireReplacementLeafRows>,
    parent_object_version: u64,
    leaf_object_version: u64,
    published_at_micros: i64,
    retain_until_micros: i64,
    placement_write_evidence: Vec<SpirePublishPlacementWriteEvidence>,
) -> Result<SpireLocalScheduledReplacementExecutionInput, String> {
    build_local_scheduled_merge_replacement_execution_input(
        &selected.lock_plan.publish_plan,
        &selected.lock_plan.pid_plan,
        &selected.decision,
        parent,
        rows,
        replacement_leaf_rows,
        parent_object_version,
        leaf_object_version,
        published_at_micros,
        retain_until_micros,
        placement_write_evidence,
    )
}

pub(super) fn build_local_selected_scheduled_merge_replacement_execution_input_from_snapshot(
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    object_store: &impl SpireObjectReader,
    selected: &SpireSelectedScheduledReplacementPublishLockPlan,
    rows: &[SpireIndexLeafSnapshotRow],
    parent_object_version: u64,
    leaf_object_version: u64,
    published_at_micros: i64,
    retain_until_micros: i64,
    placement_write_evidence: Vec<SpirePublishPlacementWriteEvidence>,
) -> Result<SpireLocalScheduledReplacementExecutionInput, String> {
    let parent =
        load_selected_scheduled_replacement_parent_routing(snapshot, object_store, selected)?;
    let replacement_leaf_rows =
        collect_selected_scheduled_replacement_leaf_rows(snapshot, object_store, selected)?;
    build_local_selected_scheduled_merge_replacement_execution_input(
        selected,
        &parent,
        rows,
        replacement_leaf_rows,
        parent_object_version,
        leaf_object_version,
        published_at_micros,
        retain_until_micros,
        placement_write_evidence,
    )
}

pub(super) fn rewrite_scheduled_replacement_parent_routing(
    parent: &SpireRoutingPartitionObject,
    decision: &SpireLeafReplacementScheduleDecision,
    replacement_children: Vec<SpireRoutingReplacementChild>,
    object_version: u64,
) -> Result<SpireRoutingPartitionObject, String> {
    validate_leaf_replacement_schedule_decision_shape(decision)?;
    if parent.header.pid != decision.replaced_parent_pid {
        return Err(format!(
            "ec_spire scheduled routing replacement parent pid {} does not match decision parent pid {}",
            parent.header.pid, decision.replaced_parent_pid
        ));
    }
    if replacement_children.len() != decision.replacement_leaf_count {
        return Err(format!(
            "ec_spire scheduled routing replacement child count {} does not match decision replacement count {}",
            replacement_children.len(),
            decision.replacement_leaf_count
        ));
    }
    if object_version == 0 {
        return Err(
            "ec_spire scheduled routing replacement object_version 0 is invalid".to_owned(),
        );
    }
    rewrite_routing_partition_for_leaf_replacement(
        parent,
        &decision.affected_leaf_pids,
        replacement_children,
        object_version,
    )
}

fn leaf_replacement_schedule_decisions_match(
    observed: &SpireLeafReplacementScheduleDecision,
    expected: &SpireLeafReplacementScheduleDecision,
) -> bool {
    observed.mode == expected.mode
        && observed.active_epoch == expected.active_epoch
        && observed.replaced_parent_pid == expected.replaced_parent_pid
        && observed.affected_leaf_pids == expected.affected_leaf_pids
        && observed.replacement_leaf_count == expected.replacement_leaf_count
}

fn validate_leaf_replacement_schedule_decision_shape(
    decision: &SpireLeafReplacementScheduleDecision,
) -> Result<(), String> {
    if decision.active_epoch == 0 {
        return Err("ec_spire replacement scheduler decision active_epoch 0 is invalid".to_owned());
    }
    if decision.replaced_parent_pid == 0 {
        return Err("ec_spire replacement scheduler decision parent pid 0 is invalid".to_owned());
    }
    validate_affected_leaf_pids(&decision.affected_leaf_pids)?;
    match decision.mode {
        SpireLeafReplacementScheduleMode::Split => {
            if decision.affected_leaf_pids.len() != 1 || decision.replacement_leaf_count < 2 {
                return Err(
                    "ec_spire replacement scheduler split decision requires one affected leaf and at least two replacement leaves"
                        .to_owned(),
                );
            }
        }
        SpireLeafReplacementScheduleMode::Merge => {
            if decision.affected_leaf_pids.len() < 2 || decision.replacement_leaf_count != 1 {
                return Err(
                    "ec_spire replacement scheduler merge decision requires at least two affected leaves and one replacement leaf"
                        .to_owned(),
                );
            }
        }
    }
    Ok(())
}

fn validate_leaf_replacement_schedule_rows(
    rows: &[SpireIndexLeafSnapshotRow],
) -> Result<(), String> {
    let mut active_epoch = None;
    let mut seen_leaf_pids = HashSet::new();
    for row in rows {
        if row.active_epoch == 0 {
            return Err("ec_spire replacement scheduler row active_epoch 0 is invalid".to_owned());
        }
        if row.leaf_pid == 0 {
            return Err("ec_spire replacement scheduler row leaf_pid 0 is invalid".to_owned());
        }
        if !seen_leaf_pids.insert(row.leaf_pid) {
            return Err(format!(
                "ec_spire replacement scheduler duplicate row for leaf pid {}",
                row.leaf_pid
            ));
        }
        if row.parent_pid == 0 && (row.split_recommended || row.merge_recommended) {
            return Err(format!(
                "ec_spire replacement scheduler candidate leaf pid {} has parent_pid 0",
                row.leaf_pid
            ));
        }
        if row.split_recommended && row.merge_recommended {
            return Err(format!(
                "ec_spire replacement scheduler leaf pid {} cannot be both split and merge recommended",
                row.leaf_pid
            ));
        }
        match active_epoch {
            Some(epoch) if epoch != row.active_epoch => {
                return Err(format!(
                    "ec_spire replacement scheduler rows span multiple active epochs: {epoch} and {}",
                    row.active_epoch
                ));
            }
            Some(_) => {}
            None => active_epoch = Some(row.active_epoch),
        }
    }
    Ok(())
}
