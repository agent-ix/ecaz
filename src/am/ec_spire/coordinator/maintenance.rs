fn no_maintenance_plan_snapshot(
    root_control: meta::SpireRootControlState,
    active_epoch: u64,
    planned_reason: &'static str,
    planner_message: &'static str,
) -> SpireIndexMaintenancePlanSnapshot {
    SpireIndexMaintenancePlanSnapshot {
        active_epoch,
        planner_status: "no_action",
        planned_action: "none",
        planned_reason,
        replaced_parent_pid: 0,
        affected_leaf_pids: Vec::new(),
        replacement_leaf_count: 0,
        replacement_leaf_pids: Vec::new(),
        publish_epoch: 0,
        next_pid: root_control.next_pid,
        next_local_vec_seq: root_control.next_local_vec_seq,
        planner_message,
    }
}

fn no_maintenance_run_result(
    root_control: meta::SpireRootControlState,
    active_epoch: u64,
    planned_reason: &'static str,
    maintenance_message: &'static str,
) -> SpireIndexMaintenanceRunResult {
    SpireIndexMaintenanceRunResult {
        active_epoch_before: active_epoch,
        active_epoch_after: active_epoch,
        maintenance_status: "no_action",
        planned_action: "none",
        planned_reason,
        replaced_parent_pid: 0,
        affected_leaf_pids: Vec::new(),
        replacement_leaf_count: 0,
        replacement_leaf_pids: Vec::new(),
        publish_epoch: 0,
        next_pid: root_control.next_pid,
        next_local_vec_seq: root_control.next_local_vec_seq,
        published: false,
        maintenance_message,
    }
}

const RECURSIVE_MAINTENANCE_DEFERRED_MESSAGE: &str =
    "ec_spire maintenance split/merge is deferred for recursive SPIRE indexes until recursive update propagation lands";

fn reject_recursive_maintenance_until_update_propagation<R: storage::SpireObjectReader>(
    snapshot: &meta::SpireValidatedEpochSnapshot<'_>,
    object_store: &R,
) -> Result<(), String> {
    for entry in &snapshot.object_manifest().entries {
        let lookup = snapshot.require_lookup(entry.pid, "recursive maintenance guard")?;
        if lookup.placement.state != meta::SpirePlacementState::Available {
            continue;
        }
        let header = object_store.read_object_header(lookup.placement)?;
        if header.kind == storage::SpirePartitionObjectKind::Internal
            || (header.kind == storage::SpirePartitionObjectKind::Root && header.level > 1)
        {
            return Err(RECURSIVE_MAINTENANCE_DEFERRED_MESSAGE.to_owned());
        }
    }
    Ok(())
}

fn selected_maintenance_run_result(
    selected: update::SpireSelectedScheduledReplacementPublishLockPlan,
    maintenance_status: &'static str,
    published: bool,
    maintenance_message: &'static str,
) -> Result<SpireIndexMaintenanceRunResult, String> {
    let planned_action = match selected.decision.mode {
        update::SpireLeafReplacementScheduleMode::Split => "split",
        update::SpireLeafReplacementScheduleMode::Merge => "merge",
    };
    let replacement_leaf_count = u64::try_from(selected.decision.replacement_leaf_count)
        .map_err(|_| "ec_spire maintenance run replacement leaf count exceeds u64".to_owned())?;

    Ok(SpireIndexMaintenanceRunResult {
        active_epoch_before: selected.decision.active_epoch,
        active_epoch_after: if published {
            selected.lock_plan.publish_plan.epoch
        } else {
            selected.decision.active_epoch
        },
        maintenance_status,
        planned_action,
        planned_reason: selected.decision.reason,
        replaced_parent_pid: selected.decision.replaced_parent_pid,
        affected_leaf_pids: selected.decision.affected_leaf_pids,
        replacement_leaf_count,
        replacement_leaf_pids: selected.lock_plan.pid_plan.replacement_pids,
        publish_epoch: selected.lock_plan.publish_plan.epoch,
        next_pid: selected.lock_plan.publish_plan.next_pid,
        next_local_vec_seq: selected.lock_plan.publish_plan.next_local_vec_seq,
        published,
        maintenance_message,
    })
}

fn next_spire_object_version(current: u64, label: &str, pid: u64) -> Result<u64, String> {
    if current == 0 {
        return Err(format!(
            "ec_spire {label} object_version 0 is invalid for pid {pid}"
        ));
    }
    current.checked_add(1).ok_or_else(|| {
        format!("ec_spire {label} object_version overflow for pid {pid}: current {current}")
    })
}

fn scheduled_replacement_object_version_plan(
    selected: &update::SpireSelectedScheduledReplacementPublishLockPlan,
    parent_object_version: u64,
    rows: &[SpireIndexLeafSnapshotRow],
) -> Result<SpireScheduledReplacementObjectVersionPlan, String> {
    let replacement_parent_object_version = next_spire_object_version(
        parent_object_version,
        "scheduled replacement parent",
        selected.decision.replaced_parent_pid,
    )?;
    let affected_leaf_pids = selected
        .decision
        .affected_leaf_pids
        .iter()
        .copied()
        .collect::<HashSet<_>>();
    let mut seen_leaf_pids = HashSet::with_capacity(affected_leaf_pids.len());
    let mut max_leaf_object_version = None;
    for row in rows {
        if !affected_leaf_pids.contains(&row.leaf_pid) {
            continue;
        }
        if !seen_leaf_pids.insert(row.leaf_pid) {
            return Err(format!(
                "ec_spire scheduled replacement saw duplicate affected leaf pid {}",
                row.leaf_pid
            ));
        }
        let leaf_object_version = next_spire_object_version(
            row.object_version,
            "scheduled replacement leaf",
            row.leaf_pid,
        )?;
        max_leaf_object_version = Some(
            max_leaf_object_version
                .unwrap_or(leaf_object_version)
                .max(leaf_object_version),
        );
    }
    if seen_leaf_pids.len() != affected_leaf_pids.len() {
        let missing = affected_leaf_pids
            .difference(&seen_leaf_pids)
            .copied()
            .collect::<Vec<_>>();
        return Err(format!(
            "ec_spire scheduled replacement object version plan missing affected leaf rows: {missing:?}"
        ));
    }
    let leaf_object_version = max_leaf_object_version.ok_or_else(|| {
        "ec_spire scheduled replacement object version plan requires affected leaf rows".to_owned()
    })?;

    Ok(SpireScheduledReplacementObjectVersionPlan {
        parent_object_version: replacement_parent_object_version,
        leaf_object_version,
    })
}

fn maintenance_run_result_from_rows(
    root_control: meta::SpireRootControlState,
    active_epoch_manifest: &meta::SpireEpochManifest,
    rows: &[SpireIndexLeafSnapshotRow],
) -> Result<SpireIndexMaintenanceRunResult, String> {
    if root_control.active_epoch == 0 {
        return Ok(no_maintenance_run_result(
            root_control,
            0,
            "empty_index",
            "build or insert rows to publish the first SPIRE epoch",
        ));
    }

    let mut pid_allocator = assign::SpirePidAllocator::new(root_control.next_pid)?;
    let Some(selected) = update::choose_scheduled_replacement_publish_lock_plan(
        rows,
        &root_control,
        active_epoch_manifest,
        &mut pid_allocator,
    )?
    else {
        return Ok(no_maintenance_run_result(
            root_control,
            active_epoch_manifest.epoch,
            "no_candidate",
            "active leaves are within split/merge thresholds",
        ));
    };

    selected_maintenance_run_result(
        selected,
        "planned",
        false,
        "scheduled replacement candidate selected under publish lock; no epoch was published",
    )
}

unsafe fn build_relation_selected_scheduled_maintenance_input(
    index_relation: pg_sys::Relation,
    snapshot: &meta::SpirePublishedEpochSnapshot<'_>,
    object_store: &storage::SpireRelationObjectStore,
    selected: &update::SpireSelectedScheduledReplacementPublishLockPlan,
    rows: &[SpireIndexLeafSnapshotRow],
) -> Result<update::SpireRelationScheduledReplacementExecutionInput, String> {
    let parent = update::load_selected_scheduled_replacement_parent_routing(
        snapshot,
        object_store,
        selected,
    )?;
    let object_versions =
        scheduled_replacement_object_version_plan(selected, parent.header.object_version, rows)?;
    let (published_at_micros, retain_until_micros) =
        unsafe { build::current_epoch_publish_times()? };

    match selected.decision.mode {
        update::SpireLeafReplacementScheduleMode::Split => {
            let heap_relation = unsafe { open_spire_heap_relation_for_index(index_relation)? };
            let heap_snapshot = unsafe { active_spire_maintenance_snapshot()? };
            let indexed_attribute = unsafe {
                crate::am::ec_hnsw::source::resolve_indexed_vector_attribute(
                    heap_relation.as_ptr(),
                    index_relation,
                    "ec_spire maintenance split replacement source vector",
                )
            };
            let slot = crate::storage::slot_guard::TupleTableSlotGuard::single_for_heap(
                heap_relation.as_ptr(),
            )
            .ok_or_else(|| {
                "ec_spire maintenance failed to allocate a heap tuple slot".to_owned()
            })?;
            let relation_options = options::relation_options(index_relation);
            unsafe {
                update::build_relation_selected_scheduled_split_replacement_execution_input_from_heap_sources(
                    heap_relation.as_ptr(),
                    heap_snapshot,
                    slot.as_ptr(),
                    indexed_attribute,
                    snapshot,
                    object_store,
                    selected,
                    u32::try_from(relation_options.boundary_replica_count).map_err(
                        |_| {
                            "ec_spire boundary_replica_count reloption must be non-negative"
                                .to_owned()
                        },
                    )?,
                    usize::from(parent.dimensions),
                    relation_options.seed as u64,
                    build::SPIRE_DEFAULT_KMEANS_ITERATIONS,
                    object_versions.parent_object_version,
                    object_versions.leaf_object_version,
                    published_at_micros,
                    retain_until_micros,
                )
            }
        }
        update::SpireLeafReplacementScheduleMode::Merge => {
            update::build_relation_selected_scheduled_merge_replacement_execution_input_from_snapshot(
                snapshot,
                object_store,
                selected,
                rows,
                object_versions.parent_object_version,
                object_versions.leaf_object_version,
                published_at_micros,
                retain_until_micros,
            )
        }
    }
}

fn maintenance_plan_snapshot_from_rows(
    root_control: meta::SpireRootControlState,
    active_epoch_manifest: &meta::SpireEpochManifest,
    rows: &[SpireIndexLeafSnapshotRow],
) -> Result<SpireIndexMaintenancePlanSnapshot, String> {
    if root_control.active_epoch == 0 {
        return Ok(no_maintenance_plan_snapshot(
            root_control,
            0,
            "empty_index",
            "build or insert rows to publish the first SPIRE epoch",
        ));
    }

    let mut pid_allocator = assign::SpirePidAllocator::new(root_control.next_pid)?;
    let Some(selected) = update::choose_scheduled_replacement_publish_lock_plan(
        rows,
        &root_control,
        active_epoch_manifest,
        &mut pid_allocator,
    )?
    else {
        return Ok(no_maintenance_plan_snapshot(
            root_control,
            active_epoch_manifest.epoch,
            "no_candidate",
            "active leaves are within split/merge thresholds",
        ));
    };

    let planned_action = match selected.decision.mode {
        update::SpireLeafReplacementScheduleMode::Split => "split",
        update::SpireLeafReplacementScheduleMode::Merge => "merge",
    };
    let replacement_leaf_count = u64::try_from(selected.decision.replacement_leaf_count)
        .map_err(|_| "ec_spire maintenance plan replacement leaf count exceeds u64".to_owned())?;

    Ok(SpireIndexMaintenancePlanSnapshot {
        active_epoch: selected.decision.active_epoch,
        planner_status: "planned",
        planned_action,
        planned_reason: selected.decision.reason,
        replaced_parent_pid: selected.decision.replaced_parent_pid,
        affected_leaf_pids: selected.decision.affected_leaf_pids,
        replacement_leaf_count,
        replacement_leaf_pids: selected.lock_plan.pid_plan.replacement_pids,
        publish_epoch: selected.lock_plan.publish_plan.epoch,
        next_pid: selected.lock_plan.publish_plan.next_pid,
        next_local_vec_seq: selected.lock_plan.publish_plan.next_local_vec_seq,
        planner_message: "scheduled replacement candidate selected; publish_epoch, next_pid, and next_local_vec_seq are projected and not advanced",
    })
}

pub(crate) unsafe fn index_maintenance_plan_snapshot(
    index_relation: pg_sys::Relation,
) -> SpireIndexMaintenancePlanSnapshot {
    let result = (|| -> Result<SpireIndexMaintenancePlanSnapshot, String> {
        let root_control = unsafe { page::read_root_control_page(index_relation) };
        if root_control.active_epoch == 0 {
            return Ok(no_maintenance_plan_snapshot(
                root_control,
                0,
                "empty_index",
                "build or insert rows to publish the first SPIRE epoch",
            ));
        }

        let (epoch_manifest, object_manifest, placement_directory) =
            unsafe { scan::load_relation_epoch_manifests(index_relation, root_control)? };
        let snapshot = meta::SpireValidatedEpochSnapshot::new(
            &epoch_manifest,
            &object_manifest,
            &placement_directory,
        )?;
        let object_store =
            unsafe { storage::SpireRelationObjectStore::for_index_relation(index_relation)? };
        reject_recursive_maintenance_until_update_propagation(&snapshot, &object_store)?;
        let rows = collect_leaf_snapshot_rows(root_control, &snapshot, &object_store)?;
        maintenance_plan_snapshot_from_rows(root_control, &epoch_manifest, &rows)
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

pub(crate) unsafe fn index_locked_maintenance_plan_snapshot(
    index_relation: pg_sys::Relation,
) -> SpireIndexMaintenancePlanSnapshot {
    let _guard = unsafe { lock_publish_relation(index_relation) };
    unsafe { index_maintenance_plan_snapshot(index_relation) }
}

pub(crate) unsafe fn index_locked_maintenance_run_plan(
    index_relation: pg_sys::Relation,
) -> SpireIndexMaintenanceRunResult {
    let _guard = unsafe { lock_publish_relation(index_relation) };
    let result = (|| -> Result<SpireIndexMaintenanceRunResult, String> {
        let root_control = unsafe { page::read_root_control_page(index_relation) };
        if root_control.active_epoch == 0 {
            return Ok(no_maintenance_run_result(
                root_control,
                0,
                "empty_index",
                "build or insert rows to publish the first SPIRE epoch",
            ));
        }

        let (epoch_manifest, object_manifest, placement_directory) =
            unsafe { scan::load_relation_epoch_manifests(index_relation, root_control)? };
        let snapshot = meta::SpireValidatedEpochSnapshot::new(
            &epoch_manifest,
            &object_manifest,
            &placement_directory,
        )?;
        let object_store =
            unsafe { storage::SpireRelationObjectStore::for_index_relation(index_relation)? };
        reject_recursive_maintenance_until_update_propagation(&snapshot, &object_store)?;
        let rows = collect_leaf_snapshot_rows(root_control, &snapshot, &object_store)?;
        maintenance_run_result_from_rows(root_control, &epoch_manifest, &rows)
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

pub(crate) unsafe fn index_maintenance_run(
    index_relation: pg_sys::Relation,
) -> SpireIndexMaintenanceRunResult {
    let _guard = unsafe { lock_publish_relation(index_relation) };
    let result = (|| -> Result<SpireIndexMaintenanceRunResult, String> {
        let root_control = unsafe { page::read_root_control_page(index_relation) };
        if root_control.active_epoch == 0 {
            return Ok(no_maintenance_run_result(
                root_control,
                0,
                "empty_index",
                "build or insert rows to publish the first SPIRE epoch",
            ));
        }

        let (epoch_manifest, object_manifest, placement_directory) =
            unsafe { scan::load_relation_epoch_manifests(index_relation, root_control)? };
        let published_snapshot = meta::SpirePublishedEpochSnapshot::new(
            &epoch_manifest,
            &object_manifest,
            &placement_directory,
        )?;
        let validated_snapshot =
            meta::SpireValidatedEpochSnapshot::from_snapshot(published_snapshot)?;
        let mut object_store =
            unsafe { storage::SpireRelationObjectStore::for_index_relation(index_relation)? };
        reject_recursive_maintenance_until_update_propagation(&validated_snapshot, &object_store)?;
        let rows = collect_leaf_snapshot_rows(root_control, &validated_snapshot, &object_store)?;
        let mut pid_allocator = assign::SpirePidAllocator::new(root_control.next_pid)?;
        let Some(selected) = update::choose_scheduled_replacement_publish_lock_plan(
            &rows,
            &root_control,
            &epoch_manifest,
            &mut pid_allocator,
        )?
        else {
            return Ok(no_maintenance_run_result(
                root_control,
                epoch_manifest.epoch,
                "no_candidate",
                "active leaves are within split/merge thresholds",
            ));
        };
        let input = unsafe {
            build_relation_selected_scheduled_maintenance_input(
                index_relation,
                &published_snapshot,
                &object_store,
                &selected,
                &rows,
            )?
        };
        unsafe {
            update::publish_relation_selected_scheduled_replacement_epoch(
                index_relation,
                epoch_manifest,
                &published_snapshot,
                &selected,
                input,
                &mut object_store,
            )?;
        }

        selected_maintenance_run_result(
            selected,
            "published",
            true,
            "scheduled replacement epoch was published",
        )
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}
