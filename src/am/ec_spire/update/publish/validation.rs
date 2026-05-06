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

