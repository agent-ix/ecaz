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
    let root_control = unsafe { page::read_root_control_page(index_relation) };
    if root_control.active_epoch != previous_epoch_manifest.epoch {
        return Err(format!(
            "ec_spire scheduled replacement publish root/control epoch {} does not match previous epoch {}",
            root_control.active_epoch, previous_epoch_manifest.epoch
        ));
    }
    let local_store_config =
        unsafe { load_relation_local_store_config(index_relation, root_control)? };
    unsafe {
        publish_replacement_epoch_to_relation(
            index_relation,
            previous_epoch_manifest,
            draft.publish_input_with_local_store_config(local_store_config),
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
    let root_control = unsafe { page::read_root_control_page(index_relation) };
    if root_control.active_epoch != previous_epoch_manifest.epoch {
        return Err(format!(
            "ec_spire replacement publish root/control epoch {} does not match previous epoch {}",
            root_control.active_epoch, previous_epoch_manifest.epoch
        ));
    }
    let local_store_config =
        unsafe { load_relation_local_store_config(index_relation, root_control)? };
    unsafe {
        publish_replacement_epoch_to_relation(
            index_relation,
            previous_epoch_manifest,
            draft.publish_input_with_local_store_config(local_store_config),
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
    let mut vec_id_roles: HashMap<SpireVecId, (usize, usize)> = HashMap::new();
    let mut vec_id_leaf_locations = HashSet::new();
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
            if !is_visible_scored_assignment(row) {
                return Err(format!(
                    "ec_spire replacement leaf object input pid {} contains a non-visible-scored row",
                    input.pid
                ));
            }
            if row.flags & SPIRE_ASSIGNMENT_FLAG_DELTA_INSERT != 0 {
                return Err(format!(
                    "ec_spire replacement leaf object input pid {} must not contain delta-insert rows",
                    input.pid
                ));
            }
            let scored_roles =
                row.flags & (SPIRE_ASSIGNMENT_FLAG_PRIMARY | SPIRE_ASSIGNMENT_FLAG_BOUNDARY_REPLICA);
            if scored_roles.count_ones() != 1 {
                return Err(format!(
                    "ec_spire replacement leaf object input pid {} must set exactly one primary/boundary role",
                    input.pid
                ));
            }
            if !vec_id_leaf_locations.insert((input.pid, row.vec_id.clone())) {
                return Err(
                    "ec_spire replacement leaf object inputs contain duplicate vec_id rows in one leaf"
                        .to_owned(),
                );
            }
            let entry = vec_id_roles.entry(row.vec_id.clone()).or_default();
            if row.flags & SPIRE_ASSIGNMENT_FLAG_PRIMARY != 0 {
                entry.0 += 1;
            }
            if row.flags & SPIRE_ASSIGNMENT_FLAG_BOUNDARY_REPLICA != 0 {
                entry.1 += 1;
            }
        }
    }
    for (vec_id, (primary_count, _replica_count)) in vec_id_roles {
        if primary_count != 1 {
            return Err(format!(
                "ec_spire replacement leaf object inputs vec_id {:?} must have exactly one primary row",
                vec_id
            ));
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
