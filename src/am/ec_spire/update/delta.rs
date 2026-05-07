pub(super) fn build_delta_epoch_draft(
    input: SpireDeltaEpochInput,
    pid_allocator: &mut SpirePidAllocator,
    local_vec_id_allocator: &mut SpireLocalVecIdAllocator,
    object_store: &mut SpireLocalObjectStore,
) -> Result<SpireDeltaEpochDraft, String> {
    build_delta_epoch_draft_with_carried_entries(
        input,
        Vec::new(),
        Vec::new(),
        Vec::new(),
        pid_allocator,
        local_vec_id_allocator,
        object_store,
    )
}

pub(super) fn build_delta_epoch_draft_from_snapshot(
    input: SpireDeltaEpochInput,
    base_snapshot: &SpirePublishedEpochSnapshot<'_>,
    pid_allocator: &mut SpirePidAllocator,
    local_vec_id_allocator: &mut SpireLocalVecIdAllocator,
    object_store: &mut SpireLocalObjectStore,
) -> Result<SpireDeltaEpochDraft, String> {
    let base_snapshot = SpireValidatedEpochSnapshot::from_snapshot(*base_snapshot)?;
    validate_delta_base_snapshot_placements_available(&base_snapshot)?;
    if input.epoch <= base_snapshot.epoch_manifest().epoch {
        return Err(format!(
            "ec_spire delta epoch {} must be newer than base epoch {}",
            input.epoch,
            base_snapshot.epoch_manifest().epoch
        ));
    }
    let base_lookup = base_snapshot.require_lookup(input.base_pid, "delta epoch base pid")?;
    let base_header = object_store.read_object_header(base_lookup.placement)?;
    if base_header.kind != SpirePartitionObjectKind::Leaf {
        return Err(format!(
            "ec_spire delta epoch base_pid {} must reference a leaf partition object",
            input.base_pid
        ));
    }
    let epoch = input.epoch;
    let carried_manifest_entries = base_snapshot
        .object_manifest()
        .entries
        .iter()
        .cloned()
        .map(|mut entry| {
            entry.epoch = epoch;
            entry
        })
        .collect();
    let carried_placement_entries = base_snapshot
        .placement_directory()
        .entries
        .iter()
        .cloned()
        .map(|mut entry| {
            entry.epoch = epoch;
            entry
        })
        .collect();
    let observed_vec_ids = collect_snapshot_assignment_vec_ids(&base_snapshot, object_store)?;
    let visible_rows =
        collect_validated_snapshot_visible_primary_rows(&base_snapshot, object_store)?;
    validate_delete_delta_targets(&input.delete_assignments, &visible_rows)?;

    build_delta_epoch_draft_with_carried_entries(
        input,
        carried_manifest_entries,
        carried_placement_entries,
        observed_vec_ids,
        pid_allocator,
        local_vec_id_allocator,
        object_store,
    )
}

fn build_delta_epoch_draft_with_carried_entries(
    input: SpireDeltaEpochInput,
    mut object_entries: Vec<SpireManifestEntry>,
    mut placement_entries: Vec<SpirePlacementEntry>,
    observed_vec_ids: Vec<SpireVecId>,
    pid_allocator: &mut SpirePidAllocator,
    local_vec_id_allocator: &mut SpireLocalVecIdAllocator,
    object_store: &mut SpireLocalObjectStore,
) -> Result<SpireDeltaEpochDraft, String> {
    if input.insert_assignments.is_empty() && input.delete_assignments.is_empty() {
        return Err("ec_spire delta epoch draft must contain at least one assignment".to_owned());
    }

    let epoch_manifest = SpireEpochManifest {
        epoch: input.epoch,
        state: SpireEpochState::Published,
        consistency_mode: input.consistency_mode,
        published_at_micros: input.published_at_micros,
        retain_until_micros: input.retain_until_micros,
        active_query_count: 0,
    };
    epoch_manifest.validate()?;

    let mut pid_cursor = *pid_allocator;
    let mut local_vec_id_cursor = *local_vec_id_allocator;
    for entry in &object_entries {
        pid_cursor.observe(entry.pid)?;
    }
    for vec_id in &observed_vec_ids {
        local_vec_id_cursor.observe(vec_id)?;
    }
    let delta_pid = pid_cursor.allocate()?;
    object_entries.push(SpireManifestEntry {
        epoch: input.epoch,
        pid: delta_pid,
        object_version: input.object_version,
        placement_tid: input.placement_tid,
    });
    let object_manifest = SpireObjectManifest::from_entries(input.epoch, object_entries)?;

    let mut assignments =
        build_insert_delta_assignments(&mut local_vec_id_cursor, input.insert_assignments)?;
    assignments.extend(build_delete_delta_assignments(input.delete_assignments)?);
    let delta_object = SpireDeltaPartitionObject::new(
        delta_pid,
        input.object_version,
        input.base_pid,
        assignments,
    )?;
    let placement = object_store.insert_delta_object(input.epoch, &delta_object)?;
    placement_entries.push(placement);
    let placement_directory =
        SpirePlacementDirectory::from_entries(input.epoch, placement_entries)?;

    let draft = SpireDeltaEpochDraft {
        epoch_manifest,
        object_manifest,
        placement_directory,
        delta_object,
        next_pid: pid_cursor.next_pid(),
        next_local_vec_seq: local_vec_id_cursor.next_local_vec_seq(),
    };
    SpireValidatedEpochSnapshot::new(
        &draft.epoch_manifest,
        &draft.object_manifest,
        &draft.placement_directory,
    )?;

    *pid_allocator = pid_cursor;
    *local_vec_id_allocator = local_vec_id_cursor;
    Ok(draft)
}

fn validate_delta_base_snapshot_placements_available(
    snapshot: &SpireValidatedEpochSnapshot<'_>,
) -> Result<(), String> {
    for placement in &snapshot.placement_directory().entries {
        if placement.state != SpirePlacementState::Available {
            return Err(format!(
                "ec_spire delta epoch base snapshot requires available placement for pid {}: got {:?}",
                placement.pid, placement.state
            ));
        }
    }
    Ok(())
}

fn collect_snapshot_assignment_vec_ids(
    snapshot: &SpireValidatedEpochSnapshot<'_>,
    object_store: &impl SpireObjectReader,
) -> Result<Vec<SpireVecId>, String> {
    let mut vec_ids = Vec::new();
    for manifest_entry in &snapshot.object_manifest().entries {
        let placement = snapshot
            .require_lookup(
                manifest_entry.pid,
                "delta draft assignment vec_id collection",
            )?
            .placement;
        let header = object_store.read_object_header(placement)?;
        match header.kind {
            SpirePartitionObjectKind::Leaf => {
                let assignments = match object_store.read_leaf_object(placement) {
                    Ok(object) => object.assignments,
                    Err(v1_error) => object_store
                        .read_leaf_object_v2(placement)
                        .map_err(|v2_error| {
                            format!(
                                "ec_spire delta draft could not read leaf pid {} as V1 or V2: V1 error: {v1_error}; V2 error: {v2_error}",
                                placement.pid
                            )
                        })?
                        .assignment_rows()?,
                };
                vec_ids.extend(assignments.into_iter().map(|assignment| assignment.vec_id));
            }
            SpirePartitionObjectKind::Delta => {
                let object = object_store.read_delta_object(placement)?;
                vec_ids.extend(
                    object
                        .assignments
                        .into_iter()
                        .map(|assignment| assignment.vec_id),
                );
            }
            SpirePartitionObjectKind::Root | SpirePartitionObjectKind::Internal => {}
        }
    }
    Ok(vec_ids)
}

fn validate_delete_delta_targets(
    delete_assignments: &[SpireDeleteDeltaInput],
    visible_rows: &[SpireLeafScanRow],
) -> Result<(), String> {
    let mut visible_targets = HashMap::new();
    for row in visible_rows {
        if visible_targets
            .insert(row.assignment.vec_id.clone(), row.assignment.heap_tid)
            .is_some()
        {
            return Err(
                "ec_spire visible snapshot contains duplicate vec_id assignments".to_owned(),
            );
        }
    }

    let mut seen_deletes = HashSet::new();
    for assignment in delete_assignments {
        if !seen_deletes.insert(assignment.vec_id.clone()) {
            return Err(
                "ec_spire delete delta vec_id appears more than once in the draft".to_owned(),
            );
        }
        let Some(heap_tid) = visible_targets.get(&assignment.vec_id) else {
            return Err(
                "ec_spire delete delta vec_id is not present in the base snapshot".to_owned(),
            );
        };
        if *heap_tid != assignment.heap_tid {
            return Err(
                "ec_spire delete delta heap_tid does not match the visible base row".to_owned(),
            );
        }
    }
    Ok(())
}
