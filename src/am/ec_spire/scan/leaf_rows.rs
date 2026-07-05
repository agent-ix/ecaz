fn collect_snapshot_leaf_rows_for_pid(
    snapshot: &SpireValidatedEpochSnapshot<'_>,
    object_store: &impl SpireObjectReader,
    leaf_pid: u64,
    expected_parent_pid: u64,
) -> Result<Vec<SpireLeafScanRow>, String> {
    let lookup = snapshot.require_lookup(leaf_pid, "routed scan leaf")?;
    let manifest_entry = lookup.manifest_entry;
    let placement = lookup.placement;
    if should_skip_placement(snapshot.epoch_manifest().consistency_mode, placement.state)? {
        return Ok(Vec::new());
    }

    let header = object_store.read_object_header(placement)?;
    if header.kind != SpirePartitionObjectKind::Leaf {
        return Err(format!(
            "ec_spire routed scan pid {leaf_pid} is not a leaf object"
        ));
    }
    if header.parent_pid != expected_parent_pid {
        return Err(format!(
            "ec_spire routed scan leaf pid {leaf_pid} parent {} does not match expected parent pid {expected_parent_pid}",
            header.parent_pid,
        ));
    }

    read_leaf_scan_rows(
        object_store,
        placement,
        leaf_pid,
        manifest_entry.object_version,
    )
}

fn read_leaf_scan_rows(
    object_store: &impl SpireObjectReader,
    placement: &super::meta::SpirePlacementEntry,
    pid: u64,
    object_version: u64,
) -> Result<Vec<SpireLeafScanRow>, String> {
    match object_store.read_leaf_object(placement) {
        Ok(leaf_object) => {
            let mut rows = Vec::with_capacity(leaf_object.assignments.len());
            for (row_index, assignment) in leaf_object.assignments.into_iter().enumerate() {
                let row_index = u32::try_from(row_index)
                    .map_err(|_| "ec_spire scan row index exceeds u32".to_owned())?;
                rows.push(SpireLeafScanRow {
                    pid,
                    object_version,
                    row_index,
                    assignment,
                });
            }
            Ok(rows)
        }
        Err(v1_error) => {
            let leaf_object = object_store.read_leaf_object_v2(placement).map_err(|v2_error| {
                format!(
                    "ec_spire scan could not read leaf pid {pid} as V1 or V2: V1 error: {v1_error}; V2 error: {v2_error}"
                )
            })?;
            let assignments = leaf_object.assignment_rows()?;
            let mut rows = Vec::with_capacity(assignments.len());
            for (row_index, assignment) in assignments.into_iter().enumerate() {
                let row_index = u32::try_from(row_index)
                    .map_err(|_| "ec_spire scan row index exceeds u32".to_owned())?;
                rows.push(SpireLeafScanRow {
                    pid,
                    object_version,
                    row_index,
                    assignment,
                });
            }
            Ok(rows)
        }
    }
}

fn should_skip_placement(
    consistency_mode: SpireConsistencyMode,
    state: SpirePlacementState,
) -> Result<bool, String> {
    match (consistency_mode, state) {
        (_, SpirePlacementState::Available) => Ok(false),
        (SpireConsistencyMode::Degraded, SpirePlacementState::Unavailable)
        | (SpireConsistencyMode::Degraded, SpirePlacementState::Skipped) => Ok(true),
        (SpireConsistencyMode::Strict, state) => Err(format!(
            "ec_spire strict scan cannot skip {:?} placement",
            state
        )),
        (SpireConsistencyMode::Degraded, SpirePlacementState::Stale) => {
            Err("ec_spire degraded scan cannot use stale placement".to_owned())
        }
    }
}
