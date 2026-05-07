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
