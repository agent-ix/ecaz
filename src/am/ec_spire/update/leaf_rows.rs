fn merge_pair_sort_key(
    pair: (&SpireIndexLeafSnapshotRow, &SpireIndexLeafSnapshotRow),
) -> (u64, u64, u64, u64) {
    (
        pair.0
            .effective_assignment_count
            .saturating_add(pair.1.effective_assignment_count),
        pair.0.effective_assignment_count,
        pair.0.leaf_pid,
        pair.1.leaf_pid,
    )
}

pub(super) fn collect_replacement_leaf_rows(
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    object_store: &impl SpireObjectReader,
    affected_leaf_pids: &[u64],
) -> Result<Vec<SpireReplacementLeafRows>, String> {
    validate_affected_leaf_pids(affected_leaf_pids)?;
    let affected: HashSet<u64> = affected_leaf_pids.iter().copied().collect();
    let snapshot = SpireValidatedEpochSnapshot::from_snapshot(*snapshot)?;
    validate_delta_base_snapshot_placements_available(&snapshot)?;
    let deleted_by_base_pid = collect_delete_vec_ids_by_base_pid(&snapshot, object_store)?;
    let mut rows_by_base_pid: HashMap<u64, Vec<SpireLeafAssignmentRow>> = HashMap::new();
    let mut active_leaf_pids = HashSet::new();
    let mut visible_vec_ids = HashSet::new();

    for manifest_entry in &snapshot.object_manifest().entries {
        let lookup = snapshot.require_lookup(manifest_entry.pid, "replacement leaf row")?;
        let placement = lookup.placement;
        let header = object_store.read_object_header(placement)?;
        match header.kind {
            SpirePartitionObjectKind::Leaf if affected.contains(&manifest_entry.pid) => {
                active_leaf_pids.insert(manifest_entry.pid);
                let deleted = deleted_by_base_pid.get(&manifest_entry.pid);
                for assignment in read_leaf_assignments_for_replacement(object_store, placement)? {
                    push_visible_replacement_row(
                        &mut rows_by_base_pid,
                        &mut visible_vec_ids,
                        manifest_entry.pid,
                        assignment,
                        deleted,
                    )?;
                }
            }
            SpirePartitionObjectKind::Delta if affected.contains(&header.parent_pid) => {
                let deleted = deleted_by_base_pid.get(&header.parent_pid);
                let delta = object_store.read_delta_object(placement)?;
                for assignment in delta.assignments {
                    push_visible_replacement_row(
                        &mut rows_by_base_pid,
                        &mut visible_vec_ids,
                        header.parent_pid,
                        assignment,
                        deleted,
                    )?;
                }
            }
            SpirePartitionObjectKind::Root
            | SpirePartitionObjectKind::Internal
            | SpirePartitionObjectKind::Leaf
            | SpirePartitionObjectKind::Delta
            | SpirePartitionObjectKind::TopGraph => {}
        }
    }

    if active_leaf_pids.len() != affected.len() {
        let mut missing = affected
            .difference(&active_leaf_pids)
            .copied()
            .collect::<Vec<_>>();
        missing.sort_unstable();
        return Err(format!(
            "ec_spire replacement leaf rows require active leaf pids for all affected pids: missing {missing:?}"
        ));
    }
    for base_pid in &affected {
        rows_by_base_pid.entry(*base_pid).or_default();
    }

    let mut folded = rows_by_base_pid
        .into_iter()
        .map(|(base_pid, rows)| SpireReplacementLeafRows { base_pid, rows })
        .collect::<Vec<_>>();
    folded.sort_by_key(|entry| entry.base_pid);
    Ok(folded)
}

pub(super) fn collect_selected_scheduled_replacement_leaf_rows(
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    object_store: &impl SpireObjectReader,
    selected: &SpireSelectedScheduledReplacementPublishLockPlan,
) -> Result<Vec<SpireReplacementLeafRows>, String> {
    validate_selected_scheduled_replacement_execution_snapshot(snapshot, selected)?;
    collect_replacement_leaf_rows(
        snapshot,
        object_store,
        &selected.decision.affected_leaf_pids,
    )
}

pub(super) fn rewrite_routing_partition_for_leaf_replacement(
    parent: &SpireRoutingPartitionObject,
    affected_child_pids: &[u64],
    replacement_children: Vec<SpireRoutingReplacementChild>,
    object_version: u64,
) -> Result<SpireRoutingPartitionObject, String> {
    validate_affected_leaf_pids(affected_child_pids)?;
    validate_replacement_routing_children(parent, affected_child_pids, &replacement_children)?;

    let affected: HashSet<u64> = affected_child_pids.iter().copied().collect();
    let mut inserted_replacements = false;
    let mut children = Vec::with_capacity(
        parent
            .child_count()
            .saturating_sub(affected.len())
            .saturating_add(replacement_children.len()),
    );
    for child in parent.children() {
        if affected.contains(&child.child_pid) {
            if !inserted_replacements {
                append_replacement_routing_children(&mut children, &replacement_children)?;
                inserted_replacements = true;
            }
            continue;
        }
        let centroid_index = u32::try_from(children.len())
            .map_err(|_| "ec_spire routing replacement child count exceeds u32".to_owned())?;
        children.push(SpireRoutingChildEntry {
            centroid_index,
            child_pid: child.child_pid,
            centroid: child.centroid.to_vec(),
        });
    }

    if !inserted_replacements {
        return Err("ec_spire routing replacement did not find any affected child pid".to_owned());
    }

    match parent.header.kind {
        SpirePartitionObjectKind::Root => SpireRoutingPartitionObject::root(
            parent.header.pid,
            object_version,
            parent.dimensions,
            children,
        ),
        SpirePartitionObjectKind::Internal => SpireRoutingPartitionObject::internal(
            parent.header.pid,
            object_version,
            parent.header.level,
            parent.header.parent_pid,
            parent.dimensions,
            children,
        ),
        other => Err(format!(
            "ec_spire routing replacement parent must be Root or Internal, got {other:?}"
        )),
    }
}

pub(super) fn plan_replacement_epoch_placement_directory(
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    object_store: &impl SpireObjectReader,
    new_epoch: u64,
    replaced_parent_pid: u64,
    replacement_parent_placement: SpirePlacementEntry,
    affected_leaf_pids: &[u64],
    replacement_leaf_placements: Vec<SpirePlacementEntry>,
) -> Result<SpirePlacementDirectory, String> {
    if new_epoch == 0 {
        return Err("ec_spire replacement placement epoch 0 is invalid".to_owned());
    }
    if replaced_parent_pid == 0 {
        return Err("ec_spire replacement parent pid 0 is invalid".to_owned());
    }
    validate_affected_leaf_pids(affected_leaf_pids)?;
    let snapshot = SpireValidatedEpochSnapshot::from_snapshot(*snapshot)?;
    if new_epoch <= snapshot.epoch_manifest().epoch {
        return Err(format!(
            "ec_spire replacement placement epoch {new_epoch} must be newer than base epoch {}",
            snapshot.epoch_manifest().epoch
        ));
    }
    validate_delta_base_snapshot_placements_available(&snapshot)?;
    validate_replacement_placement(
        new_epoch,
        replaced_parent_pid,
        "replacement parent",
        &replacement_parent_placement,
    )?;
    let affected: HashSet<u64> = affected_leaf_pids.iter().copied().collect();
    let mut replacement_leaf_pids = HashSet::new();
    for placement in &replacement_leaf_placements {
        validate_replacement_placement(new_epoch, placement.pid, "replacement leaf", placement)?;
        if placement.pid == replaced_parent_pid {
            return Err(
                "ec_spire replacement leaf placement cannot use the parent routing pid".to_owned(),
            );
        }
        if !replacement_leaf_pids.insert(placement.pid) {
            return Err("ec_spire replacement leaf placements must have unique pids".to_owned());
        }
    }

    let mut entries = Vec::with_capacity(
        snapshot
            .placement_directory()
            .entries
            .len()
            .saturating_add(1)
            .saturating_add(replacement_leaf_placements.len()),
    );
    let mut active_affected_leaves = HashSet::new();
    for manifest_entry in &snapshot.object_manifest().entries {
        let lookup = snapshot.require_lookup(manifest_entry.pid, "replacement placement")?;
        let placement = lookup.placement;
        let header = object_store.read_object_header(placement)?;
        match header.kind {
            SpirePartitionObjectKind::Root | SpirePartitionObjectKind::Internal
                if manifest_entry.pid == replaced_parent_pid =>
            {
                continue;
            }
            SpirePartitionObjectKind::Leaf if affected.contains(&manifest_entry.pid) => {
                active_affected_leaves.insert(manifest_entry.pid);
                continue;
            }
            SpirePartitionObjectKind::Delta if affected.contains(&header.parent_pid) => {
                continue;
            }
            SpirePartitionObjectKind::Root
            | SpirePartitionObjectKind::Internal
            | SpirePartitionObjectKind::Leaf
            | SpirePartitionObjectKind::Delta
            | SpirePartitionObjectKind::TopGraph => {
                let mut carried = *placement;
                carried.epoch = new_epoch;
                entries.push(carried);
            }
        }
    }

    if active_affected_leaves.len() != affected.len() {
        let mut missing = affected
            .difference(&active_affected_leaves)
            .copied()
            .collect::<Vec<_>>();
        missing.sort_unstable();
        return Err(format!(
            "ec_spire replacement placement requires active leaf pids for all affected pids: missing {missing:?}"
        ));
    }

    entries.push(replacement_parent_placement);
    entries.extend(replacement_leaf_placements);
    SpirePlacementDirectory::from_entries(new_epoch, entries)
}

pub(super) fn build_replacement_epoch_draft(
    input: SpireReplacementEpochInput,
) -> Result<SpireReplacementEpochDraft, String> {
    let epoch_manifest = SpireEpochManifest {
        epoch: input.epoch,
        state: SpireEpochState::Published,
        consistency_mode: input.consistency_mode,
        published_at_micros: input.published_at_micros,
        retain_until_micros: input.retain_until_micros,
        active_query_count: 0,
    };
    epoch_manifest.validate()?;
    let object_manifest = object_manifest_from_placement_writes(
        input.epoch,
        &input.placement_directory,
        &input.placement_write_evidence,
    )?;
    let draft = SpireReplacementEpochDraft {
        epoch_manifest,
        object_manifest,
        placement_directory: input.placement_directory,
        next_pid: input.next_pid,
        next_local_vec_seq: input.next_local_vec_seq,
    };
    SpireValidatedEpochSnapshot::new(
        &draft.epoch_manifest,
        &draft.object_manifest,
        &draft.placement_directory,
    )?;
    root_control_state_for_publish(draft.publish_input(), manifest_locators_for_validation())?;
    Ok(draft)
}

pub(super) fn build_replacement_epoch_draft_from_object_placements(
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    object_store: &impl SpireObjectReader,
    input: SpireReplacementEpochObjectPlacementInput,
) -> Result<SpireReplacementEpochDraft, String> {
    let placement_directory = replacement_placement_directory_from_object_placements(
        snapshot,
        object_store,
        input.epoch,
        input.replaced_parent_pid,
        input.affected_leaf_pids,
        input.replacement_object_placements,
    )?;

    build_replacement_epoch_draft(SpireReplacementEpochInput {
        epoch: input.epoch,
        published_at_micros: input.published_at_micros,
        retain_until_micros: input.retain_until_micros,
        consistency_mode: input.consistency_mode,
        placement_directory,
        placement_write_evidence: input.placement_write_evidence,
        next_pid: input.next_pid,
        next_local_vec_seq: input.next_local_vec_seq,
    })
}

pub(super) fn build_scheduled_replacement_epoch_draft_from_object_placements(
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    object_store: &impl SpireObjectReader,
    decision: &SpireLeafReplacementScheduleDecision,
    input: SpireScheduledReplacementEpochObjectPlacementInput,
) -> Result<SpireReplacementEpochDraft, String> {
    validate_leaf_replacement_schedule_decision_shape(decision)?;
    if snapshot.epoch_manifest.epoch != decision.active_epoch {
        return Err(format!(
            "ec_spire scheduled replacement publish snapshot epoch {} does not match decision active epoch {}",
            snapshot.epoch_manifest.epoch, decision.active_epoch
        ));
    }
    let expected_epoch = decision
        .active_epoch
        .checked_add(1)
        .ok_or_else(|| "ec_spire scheduled replacement publish epoch overflow".to_owned())?;
    if input.epoch != expected_epoch {
        return Err(format!(
            "ec_spire scheduled replacement publish epoch {} must be the immediate successor of active epoch {}",
            input.epoch, decision.active_epoch
        ));
    }
    if input.consistency_mode != snapshot.epoch_manifest.consistency_mode {
        return Err(format!(
            "ec_spire scheduled replacement consistency mode {:?} does not match active epoch consistency mode {:?}",
            input.consistency_mode, snapshot.epoch_manifest.consistency_mode
        ));
    }
    if input.replacement_object_placements.leaf_placements.len() != decision.replacement_leaf_count
    {
        return Err(format!(
            "ec_spire scheduled replacement publish leaf placement count {} does not match decision replacement count {}",
            input.replacement_object_placements.leaf_placements.len(),
            decision.replacement_leaf_count
        ));
    }
    build_replacement_epoch_draft_from_object_placements(
        snapshot,
        object_store,
        SpireReplacementEpochObjectPlacementInput {
            epoch: input.epoch,
            published_at_micros: input.published_at_micros,
            retain_until_micros: input.retain_until_micros,
            consistency_mode: input.consistency_mode,
            replaced_parent_pid: decision.replaced_parent_pid,
            affected_leaf_pids: decision.affected_leaf_pids.clone(),
            replacement_object_placements: input.replacement_object_placements,
            placement_write_evidence: input.placement_write_evidence,
            next_pid: input.next_pid,
            next_local_vec_seq: input.next_local_vec_seq,
        },
    )
}
