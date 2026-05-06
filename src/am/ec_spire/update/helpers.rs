fn manifest_locators_for_validation() -> SpirePublishedManifestLocators {
    SpirePublishedManifestLocators {
        epoch_manifest_tid: ItemPointer {
            block_number: 1,
            offset_number: 1,
        },
        object_manifest_tid: ItemPointer {
            block_number: 1,
            offset_number: 2,
        },
        placement_directory_tid: ItemPointer {
            block_number: 1,
            offset_number: 3,
        },
        local_store_config_tid: ItemPointer {
            block_number: 1,
            offset_number: 4,
        },
    }
}

fn validate_affected_leaf_pids(affected_leaf_pids: &[u64]) -> Result<(), String> {
    if affected_leaf_pids.is_empty() {
        return Err("ec_spire leaf replacement requires at least one affected leaf pid".to_owned());
    }
    let mut seen = HashSet::new();
    for pid in affected_leaf_pids {
        if *pid == 0 {
            return Err("ec_spire leaf replacement affected pid 0 is invalid".to_owned());
        }
        if !seen.insert(*pid) {
            return Err("ec_spire leaf replacement affected leaf pids must be unique".to_owned());
        }
    }
    Ok(())
}

fn validate_replacement_placement(
    epoch: u64,
    expected_pid: u64,
    label: &str,
    placement: &SpirePlacementEntry,
) -> Result<(), String> {
    if placement.epoch != epoch {
        return Err(format!(
            "ec_spire {label} placement epoch {} does not match replacement epoch {epoch}",
            placement.epoch
        ));
    }
    if placement.pid != expected_pid {
        return Err(format!(
            "ec_spire {label} placement pid {} does not match expected pid {expected_pid}",
            placement.pid
        ));
    }
    if placement.state != SpirePlacementState::Available {
        return Err(format!(
            "ec_spire {label} placement must be available, got {:?}",
            placement.state
        ));
    }
    placement.encode()?;
    Ok(())
}

fn validate_replacement_routing_children(
    parent: &SpireRoutingPartitionObject,
    affected_child_pids: &[u64],
    replacement_children: &[SpireRoutingReplacementChild],
) -> Result<(), String> {
    if replacement_children.is_empty() {
        return Err(
            "ec_spire routing replacement requires at least one replacement child".to_owned(),
        );
    }
    let affected: HashSet<u64> = affected_child_pids.iter().copied().collect();
    let mut parent_child_pids = HashSet::new();
    let mut found_affected = HashSet::new();
    for child in parent.children() {
        if !parent_child_pids.insert(child.child_pid) {
            return Err(
                "ec_spire routing replacement parent contains duplicate child pids".to_owned(),
            );
        }
        if affected.contains(&child.child_pid) {
            found_affected.insert(child.child_pid);
        }
    }
    if found_affected.len() != affected.len() {
        let mut missing = affected
            .difference(&found_affected)
            .copied()
            .collect::<Vec<_>>();
        missing.sort_unstable();
        return Err(format!(
            "ec_spire routing replacement affected child pids are missing from parent: {missing:?}"
        ));
    }

    let mut replacement_pids = HashSet::new();
    let dimensions = usize::from(parent.dimensions);
    for replacement in replacement_children {
        if replacement.child_pid == 0 {
            return Err("ec_spire routing replacement child pid 0 is invalid".to_owned());
        }
        if !replacement_pids.insert(replacement.child_pid) {
            return Err("ec_spire routing replacement child pids must be unique".to_owned());
        }
        if !affected.contains(&replacement.child_pid)
            && parent_child_pids.contains(&replacement.child_pid)
        {
            return Err(format!(
                "ec_spire routing replacement child pid {} already exists outside the affected set",
                replacement.child_pid
            ));
        }
        if replacement.centroid.len() != dimensions {
            return Err(format!(
                "ec_spire routing replacement child pid {} centroid dimensions mismatch: got {}, expected {dimensions}",
                replacement.child_pid,
                replacement.centroid.len()
            ));
        }
        if replacement
            .centroid
            .iter()
            .any(|component| !component.is_finite())
        {
            return Err(format!(
                "ec_spire routing replacement child pid {} centroid must be finite",
                replacement.child_pid
            ));
        }
    }
    Ok(())
}

fn append_replacement_routing_children(
    children: &mut Vec<SpireRoutingChildEntry>,
    replacement_children: &[SpireRoutingReplacementChild],
) -> Result<(), String> {
    for replacement in replacement_children {
        let centroid_index = u32::try_from(children.len())
            .map_err(|_| "ec_spire routing replacement child count exceeds u32".to_owned())?;
        children.push(SpireRoutingChildEntry {
            centroid_index,
            child_pid: replacement.child_pid,
            centroid: replacement.centroid.clone(),
        });
    }
    Ok(())
}

fn allocate_replacement_pids(
    pid_allocator: &mut SpirePidAllocator,
    count: usize,
) -> Result<Vec<u64>, String> {
    let mut pids = Vec::with_capacity(count);
    for _ in 0..count {
        pids.push(pid_allocator.allocate()?);
    }
    Ok(pids)
}

fn push_visible_replacement_row(
    rows_by_base_pid: &mut HashMap<u64, Vec<SpireLeafAssignmentRow>>,
    visible_vec_ids: &mut HashSet<SpireVecId>,
    base_pid: u64,
    mut assignment: SpireLeafAssignmentRow,
    deleted: Option<&HashSet<SpireVecId>>,
) -> Result<(), String> {
    if !is_visible_primary_assignment(&assignment) {
        return Ok(());
    }
    if deleted.is_some_and(|deleted| deleted.contains(&assignment.vec_id)) {
        return Ok(());
    }
    assignment.flags &= !SPIRE_ASSIGNMENT_FLAG_DELTA_INSERT;
    if !visible_vec_ids.insert(assignment.vec_id.clone()) {
        return Err(
            "ec_spire replacement leaf rows contain duplicate visible vec_id assignments"
                .to_owned(),
        );
    }
    rows_by_base_pid
        .entry(base_pid)
        .or_default()
        .push(assignment);
    Ok(())
}

fn collect_delete_vec_ids_by_base_pid(
    snapshot: &SpireValidatedEpochSnapshot<'_>,
    object_store: &impl SpireObjectReader,
) -> Result<HashMap<u64, HashSet<SpireVecId>>, String> {
    let mut deleted_by_base_pid: HashMap<u64, HashSet<SpireVecId>> = HashMap::new();
    for manifest_entry in &snapshot.object_manifest().entries {
        let lookup =
            snapshot.require_lookup(manifest_entry.pid, "replacement delete assignment")?;
        let placement = lookup.placement;
        let header = object_store.read_object_header(placement)?;
        if header.kind != SpirePartitionObjectKind::Delta {
            continue;
        }
        let delta = object_store.read_delta_object(placement)?;
        for assignment in delta.assignments {
            if is_delete_delta_assignment(&assignment) {
                deleted_by_base_pid
                    .entry(header.parent_pid)
                    .or_default()
                    .insert(assignment.vec_id);
            }
        }
    }
    Ok(deleted_by_base_pid)
}

fn read_leaf_assignments_for_replacement(
    object_store: &impl SpireObjectReader,
    placement: &SpirePlacementEntry,
) -> Result<Vec<SpireLeafAssignmentRow>, String> {
    match object_store.read_leaf_object(placement) {
        Ok(object) => Ok(object.assignments),
        Err(v1_error) => object_store
            .read_leaf_object_v2(placement)
            .map_err(|v2_error| {
                format!(
                    "ec_spire replacement leaf rows could not read leaf pid {} as V1 or V2: V1 error: {v1_error}; V2 error: {v2_error}",
                    placement.pid
                )
            })?
            .assignment_rows(),
    }
}
