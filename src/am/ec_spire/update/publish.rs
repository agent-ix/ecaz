pub(super) fn validate_scheduled_replacement_pid_plan_output(
    decision: &SpireLeafReplacementScheduleDecision,
    pid_plan: &SpireLeafReplacementPidPlan,
    replacement_object_placements: &SpireReplacementObjectPlacements,
    next_pid: u64,
) -> Result<(), String> {
    validate_leaf_replacement_schedule_decision_shape(decision)?;
    if pid_plan.reuses_existing_pid {
        return Err(
            "ec_spire scheduled replacement publish requires fresh replacement pids".to_owned(),
        );
    }
    if pid_plan.replacement_pids.len() != decision.replacement_leaf_count {
        return Err(format!(
            "ec_spire scheduled replacement pid plan count {} does not match decision replacement count {}",
            pid_plan.replacement_pids.len(),
            decision.replacement_leaf_count
        ));
    }
    if replacement_object_placements.parent_placement.pid != decision.replaced_parent_pid {
        return Err(format!(
            "ec_spire scheduled replacement parent placement pid {} does not match decision parent pid {}",
            replacement_object_placements.parent_placement.pid,
            decision.replaced_parent_pid
        ));
    }
    let placement_pids = replacement_object_placements
        .leaf_placements
        .iter()
        .map(|placement| placement.pid)
        .collect::<Vec<_>>();
    if placement_pids != pid_plan.replacement_pids {
        return Err(format!(
            "ec_spire scheduled replacement leaf placement pids {:?} do not match pid plan {:?}",
            placement_pids, pid_plan.replacement_pids
        ));
    }
    if next_pid != pid_plan.next_pid {
        return Err(format!(
            "ec_spire scheduled replacement next_pid {next_pid} does not match pid plan next_pid {}",
            pid_plan.next_pid
        ));
    }
    Ok(())
}

pub(super) fn plan_scheduled_replacement_publish_epoch(
    root_control: &SpireRootControlState,
    active_epoch_manifest: &SpireEpochManifest,
    decision: &SpireLeafReplacementScheduleDecision,
    pid_plan: &SpireLeafReplacementPidPlan,
) -> Result<SpireScheduledReplacementPublishPlan, String> {
    validate_leaf_replacement_schedule_decision_shape(decision)?;
    if root_control.active_epoch != decision.active_epoch {
        return Err(format!(
            "ec_spire scheduled replacement root/control active epoch {} does not match decision active epoch {}",
            root_control.active_epoch, decision.active_epoch
        ));
    }
    if active_epoch_manifest.epoch != decision.active_epoch {
        return Err(format!(
            "ec_spire scheduled replacement manifest epoch {} does not match decision active epoch {}",
            active_epoch_manifest.epoch, decision.active_epoch
        ));
    }
    if active_epoch_manifest.state != SpireEpochState::Published {
        return Err(format!(
            "ec_spire scheduled replacement active epoch must be published, got {:?}",
            active_epoch_manifest.state
        ));
    }
    if pid_plan.reuses_existing_pid {
        return Err(
            "ec_spire scheduled replacement publish plan requires fresh replacement pids"
                .to_owned(),
        );
    }
    if pid_plan.replacement_pids.len() != decision.replacement_leaf_count {
        return Err(format!(
            "ec_spire scheduled replacement publish plan pid count {} does not match decision replacement count {}",
            pid_plan.replacement_pids.len(),
            decision.replacement_leaf_count
        ));
    }
    let mut seen_replacement_pids = HashSet::new();
    if let Some(duplicate_pid) = pid_plan
        .replacement_pids
        .iter()
        .copied()
        .find(|pid| !seen_replacement_pids.insert(*pid))
    {
        return Err(format!(
            "ec_spire scheduled replacement publish plan replacement pid {duplicate_pid} appears more than once"
        ));
    }
    if let Some(stale_pid) = pid_plan
        .replacement_pids
        .iter()
        .copied()
        .find(|pid| *pid < root_control.next_pid)
    {
        return Err(format!(
            "ec_spire scheduled replacement pid {stale_pid} is behind root/control next_pid {}",
            root_control.next_pid
        ));
    }
    if let Some(unadvanced_pid) = pid_plan
        .replacement_pids
        .iter()
        .copied()
        .find(|pid| *pid >= pid_plan.next_pid)
    {
        return Err(format!(
            "ec_spire scheduled replacement pid plan next_pid {} does not advance past replacement pid {unadvanced_pid}",
            pid_plan.next_pid
        ));
    }
    if pid_plan.next_pid < root_control.next_pid {
        return Err(format!(
            "ec_spire scheduled replacement pid plan next_pid {} is behind root/control next_pid {}",
            pid_plan.next_pid, root_control.next_pid
        ));
    }
    let epoch = decision
        .active_epoch
        .checked_add(1)
        .ok_or_else(|| "ec_spire scheduled replacement publish epoch overflow".to_owned())?;
    Ok(SpireScheduledReplacementPublishPlan {
        epoch,
        consistency_mode: active_epoch_manifest.consistency_mode,
        next_pid: pid_plan.next_pid,
        next_local_vec_seq: root_control.next_local_vec_seq,
    })
}

pub(super) fn plan_scheduled_replacement_publish_lock(
    root_control: &SpireRootControlState,
    active_epoch_manifest: &SpireEpochManifest,
    decision: &SpireLeafReplacementScheduleDecision,
    pid_allocator: &mut SpirePidAllocator,
) -> Result<SpireScheduledReplacementPublishLockPlan, String> {
    let mut planned_pid_allocator = *pid_allocator;
    let pid_plan = plan_scheduled_leaf_replacement_pids(decision, &mut planned_pid_allocator)?;
    let publish_plan = plan_scheduled_replacement_publish_epoch(
        root_control,
        active_epoch_manifest,
        decision,
        &pid_plan,
    )?;
    *pid_allocator = planned_pid_allocator;
    Ok(SpireScheduledReplacementPublishLockPlan {
        pid_plan,
        publish_plan,
    })
}

pub(super) fn plan_rechecked_scheduled_replacement_publish_lock(
    rows: &[SpireIndexLeafSnapshotRow],
    root_control: &SpireRootControlState,
    active_epoch_manifest: &SpireEpochManifest,
    decision: &SpireLeafReplacementScheduleDecision,
    pid_allocator: &mut SpirePidAllocator,
) -> Result<SpireScheduledReplacementPublishLockPlan, String> {
    recheck_leaf_replacement_schedule_decision(rows, decision)?;
    plan_scheduled_replacement_publish_lock(
        root_control,
        active_epoch_manifest,
        decision,
        pid_allocator,
    )
}

pub(super) fn choose_scheduled_replacement_publish_lock_plan(
    rows: &[SpireIndexLeafSnapshotRow],
    root_control: &SpireRootControlState,
    active_epoch_manifest: &SpireEpochManifest,
    pid_allocator: &mut SpirePidAllocator,
) -> Result<Option<SpireSelectedScheduledReplacementPublishLockPlan>, String> {
    let Some(decision) = choose_leaf_replacement_schedule(rows)? else {
        return Ok(None);
    };
    let lock_plan = plan_rechecked_scheduled_replacement_publish_lock(
        rows,
        root_control,
        active_epoch_manifest,
        &decision,
        pid_allocator,
    )?;
    Ok(Some(SpireSelectedScheduledReplacementPublishLockPlan {
        decision,
        lock_plan,
    }))
}

pub(super) fn build_local_scheduled_replacement_execution_input_from_publish_plan(
    publish_plan: &SpireScheduledReplacementPublishPlan,
    pid_plan: &SpireLeafReplacementPidPlan,
    decision: &SpireLeafReplacementScheduleDecision,
    parts: SpireLocalScheduledReplacementExecutionParts,
) -> Result<SpireLocalScheduledReplacementExecutionInput, String> {
    validate_scheduled_replacement_execution_publish_plan_parts(
        publish_plan,
        pid_plan,
        decision,
        &parts.replacement_parent,
        &parts.replacement_children,
        parts.leaf_object_version,
        parts.published_at_micros,
        &parts.leaf_inputs,
    )?;

    Ok(SpireLocalScheduledReplacementExecutionInput {
        epoch: publish_plan.epoch,
        published_at_micros: parts.published_at_micros,
        retain_until_micros: parts.retain_until_micros,
        consistency_mode: publish_plan.consistency_mode,
        replacement_parent: parts.replacement_parent,
        replacement_children: parts.replacement_children,
        leaf_object_version: parts.leaf_object_version,
        leaf_inputs: parts.leaf_inputs,
        placement_write_evidence: parts.placement_write_evidence,
        next_local_vec_seq: publish_plan.next_local_vec_seq,
    })
}

pub(super) fn build_relation_scheduled_replacement_execution_input_from_publish_plan(
    publish_plan: &SpireScheduledReplacementPublishPlan,
    pid_plan: &SpireLeafReplacementPidPlan,
    decision: &SpireLeafReplacementScheduleDecision,
    parts: SpireRelationScheduledReplacementExecutionParts,
) -> Result<SpireRelationScheduledReplacementExecutionInput, String> {
    validate_scheduled_replacement_execution_publish_plan_parts(
        publish_plan,
        pid_plan,
        decision,
        &parts.replacement_parent,
        &parts.replacement_children,
        parts.leaf_object_version,
        parts.published_at_micros,
        &parts.leaf_inputs,
    )?;

    Ok(SpireRelationScheduledReplacementExecutionInput {
        epoch: publish_plan.epoch,
        published_at_micros: parts.published_at_micros,
        retain_until_micros: parts.retain_until_micros,
        consistency_mode: publish_plan.consistency_mode,
        replacement_parent: parts.replacement_parent,
        replacement_children: parts.replacement_children,
        leaf_object_version: parts.leaf_object_version,
        leaf_inputs: parts.leaf_inputs,
        next_local_vec_seq: publish_plan.next_local_vec_seq,
    })
}

pub(super) fn validate_relation_scheduled_replacement_execution_publish_plan(
    publish_plan: &SpireScheduledReplacementPublishPlan,
    pid_plan: &SpireLeafReplacementPidPlan,
    decision: &SpireLeafReplacementScheduleDecision,
    input: &SpireRelationScheduledReplacementExecutionInput,
) -> Result<(), String> {
    if input.epoch != publish_plan.epoch {
        return Err(format!(
            "ec_spire relation scheduled replacement execution epoch {} does not match publish plan epoch {}",
            input.epoch, publish_plan.epoch
        ));
    }
    if input.consistency_mode != publish_plan.consistency_mode {
        return Err(format!(
            "ec_spire relation scheduled replacement execution consistency mode {:?} does not match publish plan consistency mode {:?}",
            input.consistency_mode, publish_plan.consistency_mode
        ));
    }
    if input.next_local_vec_seq != publish_plan.next_local_vec_seq {
        return Err(format!(
            "ec_spire relation scheduled replacement execution next_local_vec_seq {} does not match publish plan next_local_vec_seq {}",
            input.next_local_vec_seq, publish_plan.next_local_vec_seq
        ));
    }
    validate_scheduled_replacement_execution_publish_plan_parts(
        publish_plan,
        pid_plan,
        decision,
        &input.replacement_parent,
        &input.replacement_children,
        input.leaf_object_version,
        input.published_at_micros,
        &input.leaf_inputs,
    )
}

pub(super) fn validate_relation_selected_scheduled_replacement_execution_publish_plan(
    selected: &SpireSelectedScheduledReplacementPublishLockPlan,
    input: &SpireRelationScheduledReplacementExecutionInput,
) -> Result<(), String> {
    validate_relation_scheduled_replacement_execution_publish_plan(
        &selected.lock_plan.publish_plan,
        &selected.lock_plan.pid_plan,
        &selected.decision,
        input,
    )
}

pub(super) fn validate_local_scheduled_replacement_execution_publish_plan(
    publish_plan: &SpireScheduledReplacementPublishPlan,
    pid_plan: &SpireLeafReplacementPidPlan,
    decision: &SpireLeafReplacementScheduleDecision,
    input: &SpireLocalScheduledReplacementExecutionInput,
) -> Result<(), String> {
    if input.epoch != publish_plan.epoch {
        return Err(format!(
            "ec_spire local scheduled replacement execution epoch {} does not match publish plan epoch {}",
            input.epoch, publish_plan.epoch
        ));
    }
    if input.consistency_mode != publish_plan.consistency_mode {
        return Err(format!(
            "ec_spire local scheduled replacement execution consistency mode {:?} does not match publish plan consistency mode {:?}",
            input.consistency_mode, publish_plan.consistency_mode
        ));
    }
    if input.next_local_vec_seq != publish_plan.next_local_vec_seq {
        return Err(format!(
            "ec_spire local scheduled replacement execution next_local_vec_seq {} does not match publish plan next_local_vec_seq {}",
            input.next_local_vec_seq, publish_plan.next_local_vec_seq
        ));
    }
    validate_scheduled_replacement_execution_publish_plan_parts(
        publish_plan,
        pid_plan,
        decision,
        &input.replacement_parent,
        &input.replacement_children,
        input.leaf_object_version,
        input.published_at_micros,
        &input.leaf_inputs,
    )
}

pub(super) fn validate_local_selected_scheduled_replacement_execution_publish_plan(
    selected: &SpireSelectedScheduledReplacementPublishLockPlan,
    input: &SpireLocalScheduledReplacementExecutionInput,
) -> Result<(), String> {
    validate_local_scheduled_replacement_execution_publish_plan(
        &selected.lock_plan.publish_plan,
        &selected.lock_plan.pid_plan,
        &selected.decision,
        input,
    )
}

fn validate_scheduled_replacement_execution_publish_plan_parts(
    publish_plan: &SpireScheduledReplacementPublishPlan,
    pid_plan: &SpireLeafReplacementPidPlan,
    decision: &SpireLeafReplacementScheduleDecision,
    replacement_parent: &SpireRoutingPartitionObject,
    replacement_children: &[SpireRoutingReplacementChild],
    leaf_object_version: u64,
    published_at_micros: i64,
    leaf_inputs: &[SpireReplacementLeafObjectInput],
) -> Result<(), String> {
    validate_leaf_replacement_schedule_decision_shape(decision)?;
    let expected_epoch = decision.active_epoch.checked_add(1).ok_or_else(|| {
        "ec_spire scheduled replacement execution publish epoch overflow".to_owned()
    })?;
    if publish_plan.epoch != expected_epoch {
        return Err(format!(
            "ec_spire scheduled replacement execution publish plan epoch {} must be the immediate successor of active epoch {}",
            publish_plan.epoch, decision.active_epoch
        ));
    }
    if replacement_parent.header.pid != decision.replaced_parent_pid {
        return Err(format!(
            "ec_spire scheduled replacement execution parent pid {} does not match decision parent pid {}",
            replacement_parent.header.pid, decision.replaced_parent_pid
        ));
    }
    if replacement_children.len() != decision.replacement_leaf_count {
        return Err(format!(
            "ec_spire scheduled replacement execution child count {} does not match decision replacement count {}",
            replacement_children.len(),
            decision.replacement_leaf_count
        ));
    }
    validate_scheduled_replacement_parent_contents(
        replacement_parent,
        decision,
        replacement_children,
    )?;
    if leaf_object_version == 0 {
        return Err(
            "ec_spire scheduled replacement execution leaf object_version 0 is invalid".to_owned(),
        );
    }
    if published_at_micros <= 0 {
        return Err(
            "ec_spire scheduled replacement execution requires a publish timestamp".to_owned(),
        );
    }
    if pid_plan.reuses_existing_pid {
        return Err(
            "ec_spire scheduled replacement execution input requires fresh replacement pids"
                .to_owned(),
        );
    }
    if publish_plan.next_pid != pid_plan.next_pid {
        return Err(format!(
            "ec_spire scheduled replacement execution input next_pid {} does not match pid plan next_pid {}",
            publish_plan.next_pid, pid_plan.next_pid
        ));
    }
    let child_pids = replacement_children
        .iter()
        .map(|child| child.child_pid)
        .collect::<Vec<_>>();
    if child_pids != pid_plan.replacement_pids {
        return Err(format!(
            "ec_spire scheduled replacement execution child pids {:?} do not match pid plan {:?}",
            child_pids, pid_plan.replacement_pids
        ));
    }
    validate_replacement_leaf_object_inputs(replacement_children, leaf_inputs)
}

fn validate_scheduled_replacement_parent_contents(
    replacement_parent: &SpireRoutingPartitionObject,
    decision: &SpireLeafReplacementScheduleDecision,
    replacement_children: &[SpireRoutingReplacementChild],
) -> Result<(), String> {
    let parent_child_pids = replacement_parent
        .children()
        .map(|child| child.child_pid)
        .collect::<HashSet<_>>();
    for child in replacement_children {
        if !parent_child_pids.contains(&child.child_pid) {
            return Err(format!(
                "ec_spire scheduled replacement execution parent is missing replacement child pid {}",
                child.child_pid
            ));
        }
    }
    for affected_pid in &decision.affected_leaf_pids {
        if parent_child_pids.contains(affected_pid) {
            return Err(format!(
                "ec_spire scheduled replacement execution parent still contains affected leaf pid {affected_pid}"
            ));
        }
    }
    Ok(())
}

fn validate_scheduled_replacement_execution_snapshot(
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    decision: &SpireLeafReplacementScheduleDecision,
    publish_plan: &SpireScheduledReplacementPublishPlan,
) -> Result<(), String> {
    if snapshot.epoch_manifest.epoch != decision.active_epoch {
        return Err(format!(
            "ec_spire scheduled replacement execution snapshot epoch {} does not match decision active epoch {}",
            snapshot.epoch_manifest.epoch, decision.active_epoch
        ));
    }
    if publish_plan.consistency_mode != snapshot.epoch_manifest.consistency_mode {
        return Err(format!(
            "ec_spire scheduled replacement execution publish plan consistency mode {:?} does not match active snapshot consistency mode {:?}",
            publish_plan.consistency_mode, snapshot.epoch_manifest.consistency_mode
        ));
    }
    Ok(())
}

pub(super) fn validate_selected_scheduled_replacement_execution_snapshot(
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    selected: &SpireSelectedScheduledReplacementPublishLockPlan,
) -> Result<(), String> {
    validate_scheduled_replacement_execution_snapshot(
        snapshot,
        &selected.decision,
        &selected.lock_plan.publish_plan,
    )
}

pub(super) fn validate_relation_selected_scheduled_replacement_publish_inputs(
    previous_epoch_manifest: &SpireEpochManifest,
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    selected: &SpireSelectedScheduledReplacementPublishLockPlan,
    input: &SpireRelationScheduledReplacementExecutionInput,
) -> Result<(), String> {
    if previous_epoch_manifest != snapshot.epoch_manifest {
        return Err(format!(
            "ec_spire selected scheduled replacement publish previous epoch manifest mismatch: got {}, expected {}",
            previous_epoch_manifest.epoch, snapshot.epoch_manifest.epoch
        ));
    }
    validate_relation_selected_scheduled_replacement_execution_publish_plan(selected, input)?;
    validate_selected_scheduled_replacement_execution_snapshot(snapshot, selected)
}

pub(super) fn validate_local_selected_scheduled_replacement_draft_inputs(
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    selected: &SpireSelectedScheduledReplacementPublishLockPlan,
    input: &SpireLocalScheduledReplacementExecutionInput,
) -> Result<(), String> {
    validate_local_selected_scheduled_replacement_execution_publish_plan(selected, input)?;
    validate_selected_scheduled_replacement_execution_snapshot(snapshot, selected)
}

pub(super) fn build_local_scheduled_replacement_epoch_draft(
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    decision: &SpireLeafReplacementScheduleDecision,
    pid_plan: &SpireLeafReplacementPidPlan,
    publish_plan: &SpireScheduledReplacementPublishPlan,
    input: SpireLocalScheduledReplacementExecutionInput,
    object_store: &mut SpireLocalObjectStore,
) -> Result<SpireReplacementEpochDraft, String> {
    validate_local_scheduled_replacement_execution_publish_plan(
        publish_plan,
        pid_plan,
        decision,
        &input,
    )?;
    validate_scheduled_replacement_execution_snapshot(snapshot, decision, publish_plan)?;
    let replacement_object_placements = write_local_scheduled_replacement_objects(
        input.epoch,
        &input.replacement_parent,
        decision,
        &input.replacement_children,
        input.leaf_object_version,
        input.leaf_inputs,
        object_store,
    )?;
    validate_scheduled_replacement_pid_plan_output(
        decision,
        pid_plan,
        &replacement_object_placements,
        pid_plan.next_pid,
    )?;
    build_scheduled_replacement_epoch_draft_from_object_placements(
        snapshot,
        object_store,
        decision,
        SpireScheduledReplacementEpochObjectPlacementInput {
            epoch: input.epoch,
            published_at_micros: input.published_at_micros,
            retain_until_micros: input.retain_until_micros,
            consistency_mode: input.consistency_mode,
            replacement_object_placements,
            placement_write_evidence: input.placement_write_evidence,
            next_pid: pid_plan.next_pid,
            next_local_vec_seq: input.next_local_vec_seq,
        },
    )
}

pub(super) fn build_local_selected_scheduled_replacement_epoch_draft(
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    selected: &SpireSelectedScheduledReplacementPublishLockPlan,
    input: SpireLocalScheduledReplacementExecutionInput,
    object_store: &mut SpireLocalObjectStore,
) -> Result<SpireReplacementEpochDraft, String> {
    validate_local_selected_scheduled_replacement_draft_inputs(snapshot, selected, &input)?;
    build_local_scheduled_replacement_epoch_draft(
        snapshot,
        &selected.decision,
        &selected.lock_plan.pid_plan,
        &selected.lock_plan.publish_plan,
        input,
        object_store,
    )
}

pub(super) fn build_local_selected_scheduled_split_replacement_epoch_draft(
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    selected: &SpireSelectedScheduledReplacementPublishLockPlan,
    parent: &SpireRoutingPartitionObject,
    centroids: Vec<Vec<f32>>,
    routed_leaf_inputs: Vec<SpireReplacementLeafObjectInput>,
    parent_object_version: u64,
    leaf_object_version: u64,
    published_at_micros: i64,
    retain_until_micros: i64,
    placement_write_evidence: Vec<SpirePublishPlacementWriteEvidence>,
    object_store: &mut SpireLocalObjectStore,
) -> Result<SpireReplacementEpochDraft, String> {
    let input = build_local_selected_scheduled_split_replacement_execution_input(
        selected,
        parent,
        centroids,
        routed_leaf_inputs,
        parent_object_version,
        leaf_object_version,
        published_at_micros,
        retain_until_micros,
        placement_write_evidence,
    )?;
    build_local_selected_scheduled_replacement_epoch_draft(snapshot, selected, input, object_store)
}

pub(super) fn build_local_selected_scheduled_split_replacement_epoch_draft_from_snapshot(
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    selected: &SpireSelectedScheduledReplacementPublishLockPlan,
    centroids: Vec<Vec<f32>>,
    routed_leaf_inputs: Vec<SpireReplacementLeafObjectInput>,
    parent_object_version: u64,
    leaf_object_version: u64,
    published_at_micros: i64,
    retain_until_micros: i64,
    placement_write_evidence: Vec<SpirePublishPlacementWriteEvidence>,
    object_store: &mut SpireLocalObjectStore,
) -> Result<SpireReplacementEpochDraft, String> {
    let parent =
        load_selected_scheduled_replacement_parent_routing(snapshot, object_store, selected)?;
    build_local_selected_scheduled_split_replacement_epoch_draft(
        snapshot,
        selected,
        &parent,
        centroids,
        routed_leaf_inputs,
        parent_object_version,
        leaf_object_version,
        published_at_micros,
        retain_until_micros,
        placement_write_evidence,
        object_store,
    )
}

pub(super) fn build_local_selected_scheduled_merge_replacement_epoch_draft(
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    selected: &SpireSelectedScheduledReplacementPublishLockPlan,
    parent: &SpireRoutingPartitionObject,
    rows: &[SpireIndexLeafSnapshotRow],
    replacement_leaf_rows: Vec<SpireReplacementLeafRows>,
    parent_object_version: u64,
    leaf_object_version: u64,
    published_at_micros: i64,
    retain_until_micros: i64,
    placement_write_evidence: Vec<SpirePublishPlacementWriteEvidence>,
    object_store: &mut SpireLocalObjectStore,
) -> Result<SpireReplacementEpochDraft, String> {
    let input = build_local_selected_scheduled_merge_replacement_execution_input(
        selected,
        parent,
        rows,
        replacement_leaf_rows,
        parent_object_version,
        leaf_object_version,
        published_at_micros,
        retain_until_micros,
        placement_write_evidence,
    )?;
    build_local_selected_scheduled_replacement_epoch_draft(snapshot, selected, input, object_store)
}

pub(super) fn build_local_selected_scheduled_merge_replacement_epoch_draft_from_snapshot(
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    selected: &SpireSelectedScheduledReplacementPublishLockPlan,
    rows: &[SpireIndexLeafSnapshotRow],
    parent_object_version: u64,
    leaf_object_version: u64,
    published_at_micros: i64,
    retain_until_micros: i64,
    placement_write_evidence: Vec<SpirePublishPlacementWriteEvidence>,
    object_store: &mut SpireLocalObjectStore,
) -> Result<SpireReplacementEpochDraft, String> {
    let parent =
        load_selected_scheduled_replacement_parent_routing(snapshot, object_store, selected)?;
    let replacement_leaf_rows =
        collect_selected_scheduled_replacement_leaf_rows(snapshot, object_store, selected)?;
    build_local_selected_scheduled_merge_replacement_epoch_draft(
        snapshot,
        selected,
        &parent,
        rows,
        replacement_leaf_rows,
        parent_object_version,
        leaf_object_version,
        published_at_micros,
        retain_until_micros,
        placement_write_evidence,
        object_store,
    )
}

pub(super) unsafe fn publish_relation_scheduled_replacement_epoch(
    index_relation: pgrx::pg_sys::Relation,
    previous_epoch_manifest: SpireEpochManifest,
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    decision: &SpireLeafReplacementScheduleDecision,
    pid_plan: &SpireLeafReplacementPidPlan,
    publish_plan: &SpireScheduledReplacementPublishPlan,
    input: SpireRelationScheduledReplacementExecutionInput,
    object_store: &mut SpireRelationObjectStore,
) -> Result<SpireReplacementEpochDraft, String> {
    if &previous_epoch_manifest != snapshot.epoch_manifest {
        return Err(format!(
            "ec_spire scheduled replacement publish previous epoch manifest mismatch: got {}, expected {}",
            previous_epoch_manifest.epoch, snapshot.epoch_manifest.epoch
        ));
    }
    validate_relation_scheduled_replacement_execution_publish_plan(
        publish_plan,
        pid_plan,
        decision,
        &input,
    )?;
    validate_scheduled_replacement_execution_snapshot(snapshot, decision, publish_plan)?;
    let replacement_object_placements = unsafe {
        write_relation_scheduled_replacement_objects(
            input.epoch,
            &input.replacement_parent,
            decision,
            &input.replacement_children,
            input.leaf_object_version,
            input.leaf_inputs,
            object_store,
        )?
    };
    validate_scheduled_replacement_pid_plan_output(
        decision,
        pid_plan,
        &replacement_object_placements,
        pid_plan.next_pid,
    )?;
    let placement_directory = replacement_placement_directory_from_object_placements(
        snapshot,
        object_store,
        input.epoch,
        decision.replaced_parent_pid,
        decision.affected_leaf_pids.clone(),
        replacement_object_placements.clone(),
    )?;
    let placement_write_evidence =
        unsafe { write_placement_entries_to_relation(index_relation, &placement_directory)? };
    let draft = build_scheduled_replacement_epoch_draft_from_object_placements(
        snapshot,
        object_store,
        decision,
        SpireScheduledReplacementEpochObjectPlacementInput {
            epoch: input.epoch,
            published_at_micros: input.published_at_micros,
            retain_until_micros: input.retain_until_micros,
            consistency_mode: input.consistency_mode,
            replacement_object_placements,
            placement_write_evidence,
            next_pid: pid_plan.next_pid,
            next_local_vec_seq: input.next_local_vec_seq,
        },
    )?;
    unsafe {
        publish_replacement_epoch_to_relation(
            index_relation,
            previous_epoch_manifest,
            draft.publish_input(),
        )?;
    }
    Ok(draft)
}

pub(super) unsafe fn publish_relation_selected_scheduled_replacement_epoch(
    index_relation: pgrx::pg_sys::Relation,
    previous_epoch_manifest: SpireEpochManifest,
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    selected: &SpireSelectedScheduledReplacementPublishLockPlan,
    input: SpireRelationScheduledReplacementExecutionInput,
    object_store: &mut SpireRelationObjectStore,
) -> Result<SpireReplacementEpochDraft, String> {
    validate_relation_selected_scheduled_replacement_publish_inputs(
        &previous_epoch_manifest,
        snapshot,
        selected,
        &input,
    )?;
    unsafe {
        publish_relation_scheduled_replacement_epoch(
            index_relation,
            previous_epoch_manifest,
            snapshot,
            &selected.decision,
            &selected.lock_plan.pid_plan,
            &selected.lock_plan.publish_plan,
            input,
            object_store,
        )
    }
}

pub(super) unsafe fn publish_relation_replacement_epoch_from_object_placements(
    index_relation: pgrx::pg_sys::Relation,
    previous_epoch_manifest: SpireEpochManifest,
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    object_store: &impl SpireObjectReader,
    input: SpireRelationReplacementEpochObjectPlacementInput,
) -> Result<SpireReplacementEpochDraft, String> {
    if &previous_epoch_manifest != snapshot.epoch_manifest {
        return Err(format!(
            "ec_spire replacement publish previous epoch manifest mismatch: got {}, expected {}",
            previous_epoch_manifest.epoch, snapshot.epoch_manifest.epoch
        ));
    }
    let placement_directory = replacement_placement_directory_from_object_placements(
        snapshot,
        object_store,
        input.epoch,
        input.replaced_parent_pid,
        input.affected_leaf_pids,
        input.replacement_object_placements,
    )?;
    let placement_write_evidence =
        unsafe { write_placement_entries_to_relation(index_relation, &placement_directory)? };
    let draft = build_replacement_epoch_draft(SpireReplacementEpochInput {
        epoch: input.epoch,
        published_at_micros: input.published_at_micros,
        retain_until_micros: input.retain_until_micros,
        consistency_mode: input.consistency_mode,
        placement_directory,
        placement_write_evidence,
        next_pid: input.next_pid,
        next_local_vec_seq: input.next_local_vec_seq,
    })?;
    unsafe {
        publish_replacement_epoch_to_relation(
            index_relation,
            previous_epoch_manifest,
            draft.publish_input(),
        )?;
    }
    Ok(draft)
}

fn replacement_placement_directory_from_object_placements(
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    object_store: &impl SpireObjectReader,
    epoch: u64,
    replaced_parent_pid: u64,
    affected_leaf_pids: Vec<u64>,
    replacement_object_placements: SpireReplacementObjectPlacements,
) -> Result<SpirePlacementDirectory, String> {
    plan_replacement_epoch_placement_directory(
        snapshot,
        object_store,
        epoch,
        replaced_parent_pid,
        replacement_object_placements.parent_placement,
        &affected_leaf_pids,
        replacement_object_placements.leaf_placements,
    )
}

pub(super) fn validate_replacement_leaf_object_inputs(
    replacement_children: &[SpireRoutingReplacementChild],
    leaf_inputs: &[SpireReplacementLeafObjectInput],
) -> Result<(), String> {
    if replacement_children.is_empty() {
        return Err("ec_spire replacement leaf object inputs require children".to_owned());
    }
    if replacement_children.len() != leaf_inputs.len() {
        return Err(format!(
            "ec_spire replacement leaf object input count {} does not match replacement child count {}",
            leaf_inputs.len(),
            replacement_children.len()
        ));
    }

    let mut child_pids = HashSet::new();
    for child in replacement_children {
        if child.child_pid == 0 {
            return Err("ec_spire replacement child pid 0 is invalid".to_owned());
        }
        if !child_pids.insert(child.child_pid) {
            return Err("ec_spire replacement child pids must be unique".to_owned());
        }
    }

    let mut input_pids = HashSet::new();
    let mut vec_ids = HashSet::new();
    for input in leaf_inputs {
        if input.pid == 0 {
            return Err("ec_spire replacement leaf object input pid 0 is invalid".to_owned());
        }
        if !input_pids.insert(input.pid) {
            return Err("ec_spire replacement leaf object input pids must be unique".to_owned());
        }
        if !child_pids.contains(&input.pid) {
            return Err(format!(
                "ec_spire replacement leaf object input pid {} has no replacement routing child",
                input.pid
            ));
        }
        for row in &input.rows {
            if !is_visible_primary_assignment(row) {
                return Err(format!(
                    "ec_spire replacement leaf object input pid {} contains a non-visible-primary row",
                    input.pid
                ));
            }
            if row.flags & SPIRE_ASSIGNMENT_FLAG_DELTA_INSERT != 0 {
                return Err(format!(
                    "ec_spire replacement leaf object input pid {} must not contain delta-insert rows",
                    input.pid
                ));
            }
            if !vec_ids.insert(row.vec_id.clone()) {
                return Err(
                    "ec_spire replacement leaf object inputs contain duplicate vec_id rows"
                        .to_owned(),
                );
            }
        }
    }

    for child_pid in child_pids {
        if !input_pids.contains(&child_pid) {
            return Err(format!(
                "ec_spire replacement routing child pid {child_pid} has no leaf object input"
            ));
        }
    }
    Ok(())
}

pub(super) fn write_local_replacement_objects(
    epoch: u64,
    replacement_parent: &SpireRoutingPartitionObject,
    replacement_children: &[SpireRoutingReplacementChild],
    leaf_object_version: u64,
    leaf_inputs: Vec<SpireReplacementLeafObjectInput>,
    object_store: &mut SpireLocalObjectStore,
) -> Result<SpireReplacementObjectPlacements, String> {
    write_replacement_objects_with_writer(
        epoch,
        replacement_parent,
        replacement_children,
        leaf_object_version,
        leaf_inputs,
        object_store,
    )
}

pub(super) unsafe fn write_relation_replacement_objects(
    epoch: u64,
    replacement_parent: &SpireRoutingPartitionObject,
    replacement_children: &[SpireRoutingReplacementChild],
    leaf_object_version: u64,
    leaf_inputs: Vec<SpireReplacementLeafObjectInput>,
    object_store: &mut SpireRelationObjectStore,
) -> Result<SpireReplacementObjectPlacements, String> {
    write_replacement_objects_with_writer(
        epoch,
        replacement_parent,
        replacement_children,
        leaf_object_version,
        leaf_inputs,
        object_store,
    )
}

pub(super) fn write_local_scheduled_replacement_objects(
    epoch: u64,
    replacement_parent: &SpireRoutingPartitionObject,
    decision: &SpireLeafReplacementScheduleDecision,
    replacement_children: &[SpireRoutingReplacementChild],
    leaf_object_version: u64,
    leaf_inputs: Vec<SpireReplacementLeafObjectInput>,
    object_store: &mut SpireLocalObjectStore,
) -> Result<SpireReplacementObjectPlacements, String> {
    write_scheduled_replacement_objects_with_writer(
        epoch,
        replacement_parent,
        decision,
        replacement_children,
        leaf_object_version,
        leaf_inputs,
        object_store,
    )
}

pub(super) unsafe fn write_relation_scheduled_replacement_objects(
    epoch: u64,
    replacement_parent: &SpireRoutingPartitionObject,
    decision: &SpireLeafReplacementScheduleDecision,
    replacement_children: &[SpireRoutingReplacementChild],
    leaf_object_version: u64,
    leaf_inputs: Vec<SpireReplacementLeafObjectInput>,
    object_store: &mut SpireRelationObjectStore,
) -> Result<SpireReplacementObjectPlacements, String> {
    write_scheduled_replacement_objects_with_writer(
        epoch,
        replacement_parent,
        decision,
        replacement_children,
        leaf_object_version,
        leaf_inputs,
        object_store,
    )
}

fn write_scheduled_replacement_objects_with_writer(
    epoch: u64,
    replacement_parent: &SpireRoutingPartitionObject,
    decision: &SpireLeafReplacementScheduleDecision,
    replacement_children: &[SpireRoutingReplacementChild],
    leaf_object_version: u64,
    leaf_inputs: Vec<SpireReplacementLeafObjectInput>,
    object_store: &mut impl SpireReplacementObjectWriter,
) -> Result<SpireReplacementObjectPlacements, String> {
    validate_leaf_replacement_schedule_decision_shape(decision)?;
    let expected_epoch = decision
        .active_epoch
        .checked_add(1)
        .ok_or_else(|| "ec_spire scheduled replacement object writer epoch overflow".to_owned())?;
    if epoch != expected_epoch {
        return Err(format!(
            "ec_spire scheduled replacement object writer epoch {epoch} must be the immediate successor of active epoch {}",
            decision.active_epoch
        ));
    }
    if replacement_parent.header.pid != decision.replaced_parent_pid {
        return Err(format!(
            "ec_spire scheduled replacement object writer parent pid {} does not match decision parent pid {}",
            replacement_parent.header.pid, decision.replaced_parent_pid
        ));
    }
    if replacement_children.len() != decision.replacement_leaf_count {
        return Err(format!(
            "ec_spire scheduled replacement object writer child count {} does not match decision replacement count {}",
            replacement_children.len(),
            decision.replacement_leaf_count
        ));
    }
    write_replacement_objects_with_writer(
        epoch,
        replacement_parent,
        replacement_children,
        leaf_object_version,
        leaf_inputs,
        object_store,
    )
}

fn write_replacement_objects_with_writer(
    epoch: u64,
    replacement_parent: &SpireRoutingPartitionObject,
    replacement_children: &[SpireRoutingReplacementChild],
    leaf_object_version: u64,
    leaf_inputs: Vec<SpireReplacementLeafObjectInput>,
    object_store: &mut impl SpireReplacementObjectWriter,
) -> Result<SpireReplacementObjectPlacements, String> {
    if epoch == 0 {
        return Err("ec_spire replacement object epoch 0 is invalid".to_owned());
    }
    if leaf_object_version == 0 {
        return Err("ec_spire replacement leaf object_version 0 is invalid".to_owned());
    }
    match replacement_parent.header.kind {
        SpirePartitionObjectKind::Root | SpirePartitionObjectKind::Internal => {}
        other => {
            return Err(format!(
                "ec_spire replacement parent must be Root or Internal, got {other:?}"
            ));
        }
    }
    validate_replacement_leaf_object_inputs(replacement_children, &leaf_inputs)?;

    let parent_placement =
        object_store.write_replacement_parent_object(epoch, replacement_parent)?;
    let inputs_by_pid = leaf_inputs
        .into_iter()
        .map(|input| (input.pid, input))
        .collect::<HashMap<_, _>>();
    let mut leaf_placements = Vec::with_capacity(replacement_children.len());
    for child in replacement_children {
        let input = inputs_by_pid.get(&child.child_pid).ok_or_else(|| {
            format!(
                "ec_spire replacement child pid {} has no leaf input",
                child.child_pid
            )
        })?;
        leaf_placements.push(object_store.write_replacement_leaf_object_v2_from_rows(
            epoch,
            input.pid,
            leaf_object_version,
            replacement_parent.header.pid,
            &input.rows,
        )?);
    }

    Ok(SpireReplacementObjectPlacements {
        parent_placement,
        leaf_placements,
    })
}
