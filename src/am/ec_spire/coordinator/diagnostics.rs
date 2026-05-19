impl SpireActiveSnapshotDiagnostics {
    fn empty(root_control: meta::SpireRootControlState) -> Self {
        Self {
            active_epoch: root_control.active_epoch,
            next_pid: root_control.next_pid,
            next_local_vec_seq: root_control.next_local_vec_seq,
            consistency_mode: "none",
            object_count: 0,
            placement_count: 0,
            local_store_count: 0,
            available_placement_count: 0,
            stale_placement_count: 0,
            unavailable_placement_count: 0,
            skipped_placement_count: 0,
            root_object_count: 0,
            internal_object_count: 0,
            leaf_object_count: 0,
            delta_object_count: 0,
            routing_child_count: 0,
            leaf_assignment_count: 0,
            delta_assignment_count: 0,
            available_object_bytes: 0,
            routing_object_bytes: 0,
            leaf_object_bytes: 0,
            delta_object_bytes: 0,
        }
    }
}

fn health_snapshot_from_diagnostics(
    diagnostics: &SpireActiveSnapshotDiagnostics,
) -> SpireIndexHealthSnapshot {
    let has_no_active_epoch = diagnostics.active_epoch == 0;
    let (status, healthy, recommendation, compaction_recommended) = if has_no_active_epoch {
        (
            "empty",
            true,
            "build or insert rows to publish the first SPIRE epoch",
            false,
        )
    } else if diagnostics.unavailable_placement_count > 0 {
        (
            "unavailable_placements",
            false,
            "restore unavailable local placements before relying on this index",
            false,
        )
    } else if diagnostics.stale_placement_count > 0 {
        (
            "stale_placements",
            false,
            "publish a cleanup epoch to remove stale placements",
            false,
        )
    } else if diagnostics.skipped_placement_count > 0 {
        (
            "skipped_placements",
            false,
            "inspect skipped placements before enabling degraded reads",
            false,
        )
    } else if diagnostics.delta_object_count > 0 {
        (
            "maintenance_recommended",
            true,
            "run VACUUM to compact active delta objects into V2 base leaves",
            true,
        )
    } else if diagnostics.consistency_mode == "degraded" {
        (
            "degraded_consistency",
            true,
            "verify degraded-read policy before relying on strict local semantics",
            false,
        )
    } else {
        ("ok", true, "none", false)
    };

    SpireIndexHealthSnapshot {
        active_epoch: diagnostics.active_epoch,
        consistency_mode: diagnostics.consistency_mode,
        status,
        healthy,
        recommendation,
        compaction_recommended,
        object_count: diagnostics.object_count,
        leaf_assignment_count: diagnostics.leaf_assignment_count,
        delta_assignment_count: diagnostics.delta_assignment_count,
        delta_object_count: diagnostics.delta_object_count,
        available_placement_count: diagnostics.available_placement_count,
        stale_placement_count: diagnostics.stale_placement_count,
        unavailable_placement_count: diagnostics.unavailable_placement_count,
        skipped_placement_count: diagnostics.skipped_placement_count,
    }
}

// Pure helpers shared with the hardening shadow crate live in
// `diagnostics_helpers.rs` so the careful crate can `include!` them
// directly under shimmed type names. The set must stay free of pgrx
// FFI or `pg_sys` calls.
include!("diagnostics_helpers.rs");

#[derive(Debug, Default)]
struct BoundaryReplicaIdentityAccumulator {
    assignment_count: u64,
    primary_assignment_count: u64,
    boundary_replica_assignment_count: u64,
    delta_insert_assignment_count: u64,
    leaf_pids: BTreeSet<u64>,
    node_ids: BTreeSet<u32>,
    local_store_ids: BTreeSet<u32>,
}

#[derive(Debug, Default)]
struct BoundaryReplicaPlacementAccumulator {
    assignment_count: u64,
    primary_assignment_count: u64,
    boundary_replica_assignment_count: u64,
    stale_boundary_replica_count: u64,
    unavailable_boundary_replica_count: u64,
    skipped_boundary_replica_count: u64,
    node_ids: BTreeSet<u32>,
}

fn read_leaf_assignment_rows(
    object_store: &impl storage::SpireObjectReader,
    placement: &meta::SpirePlacementEntry,
) -> Result<Vec<storage::SpireLeafAssignmentRow>, String> {
    match object_store.read_leaf_object_v2(placement) {
        Ok(object) => object.assignment_rows(),
        Err(v2_error) => object_store
            .read_leaf_object(placement)
            .map_err(|v1_error| {
                format!(
                    "ec_spire boundary identity could not read leaf pid {} as V2 or V1: V2 error: {v2_error}; V1 error: {v1_error}",
                    placement.pid
                )
            })
            .map(|object| object.assignments),
    }
}

fn coordinator_metadata_read_available_placement(
    placement: &meta::SpirePlacementEntry,
) -> meta::SpirePlacementEntry {
    let mut placement = coordinator_metadata_read_placement(placement);
    placement.state = meta::SpirePlacementState::Available;
    placement
}

unsafe fn load_relation_epoch_manifests_for_boundary_placement_diagnostics(
    index_relation: pg_sys::Relation,
    root_control: meta::SpireRootControlState,
) -> Result<
    (
        meta::SpireEpochManifest,
        meta::SpireObjectManifest,
        meta::SpirePlacementDirectory,
    ),
    String,
> {
    if root_control.active_epoch == 0 {
        return Err("ec_spire cannot load manifests for empty active epoch".to_owned());
    }
    // SAFETY: index_relation is the open SPIRE index relation, and the TID was
    // read from its root/control page for the active epoch manifest.
    let epoch_bytes =
        unsafe { page::read_object_tuple(index_relation, root_control.epoch_manifest_tid)? };
    // SAFETY: index_relation is the open SPIRE index relation, and the TID was
    // read from its root/control page for the active object manifest.
    let object_bytes =
        unsafe { page::read_object_tuple(index_relation, root_control.object_manifest_tid)? };
    // SAFETY: index_relation is the open SPIRE index relation, and the TID was
    // read from its root/control page for the active placement directory.
    let placement_bytes =
        unsafe { page::read_object_tuple(index_relation, root_control.placement_directory_tid)? };
    let epoch_manifest = meta::SpireEpochManifest::decode(&epoch_bytes)?;
    let object_manifest = meta::SpireObjectManifest::decode(&object_bytes)?;
    let placement_directory = meta::SpirePlacementDirectory::decode(&placement_bytes)?;
    if epoch_manifest.epoch != root_control.active_epoch {
        return Err(format!(
            "ec_spire root/control active epoch {} does not match epoch manifest {}",
            root_control.active_epoch, epoch_manifest.epoch
        ));
    }
    if object_manifest.epoch != epoch_manifest.epoch {
        return Err(format!(
            "ec_spire boundary placement diagnostic object manifest epoch mismatch: epoch {}, manifest {}",
            epoch_manifest.epoch, object_manifest.epoch
        ));
    }
    if placement_directory.epoch != epoch_manifest.epoch {
        return Err(format!(
            "ec_spire boundary placement diagnostic placement directory epoch mismatch: epoch {}, directory {}",
            epoch_manifest.epoch, placement_directory.epoch
        ));
    }
    Ok((epoch_manifest, object_manifest, placement_directory))
}

pub(crate) unsafe fn index_boundary_replica_identity_snapshot(
    index_relation: pg_sys::Relation,
) -> Vec<SpireBoundaryReplicaIdentitySnapshotRow> {
    let result = (|| -> Result<Vec<SpireBoundaryReplicaIdentitySnapshotRow>, String> {
        // SAFETY: index_relation is the open SPIRE index relation inspected by
        // this read-only diagnostic.
        let root_control = unsafe { page::read_root_control_page(index_relation) };
        if root_control.active_epoch == 0 {
            return Ok(Vec::new());
        }
        // SAFETY: root_control was read from the same open relation and names
        // the active coordinator fanout manifest set.
        let (_epoch_manifest, _object_manifest, placement_directory) =
            unsafe { load_relation_epoch_manifests_for_coordinator_fanout(index_relation, root_control)? };
        // SAFETY: placement_directory was decoded from the active relation
        // manifests and is used to open the corresponding local object stores.
        let object_store = unsafe {
            storage::SpireRelationObjectStoreSet::for_index_relation_and_placements(
                index_relation,
                &placement_directory,
                pg_sys::AccessShareLock as pg_sys::LOCKMODE,
            )?
        };
        let mut groups = BTreeMap::<Vec<u8>, BoundaryReplicaIdentityAccumulator>::new();

        for placement in &placement_directory.entries {
            if placement.state != meta::SpirePlacementState::Available {
                continue;
            }
            let metadata_placement = coordinator_metadata_read_placement(placement);
            let header = object_store.read_object_header(&metadata_placement)?;
            let assignments = match header.kind {
                storage::SpirePartitionObjectKind::Leaf => {
                    read_leaf_assignment_rows(&object_store, &metadata_placement)?
                }
                storage::SpirePartitionObjectKind::Delta => {
                    object_store
                        .read_delta_object(&metadata_placement)?
                        .assignments
                }
                _ => continue,
            };

            for assignment in assignments {
                let group = groups
                    .entry(assignment.vec_id.as_bytes().to_vec())
                    .or_default();
                group.assignment_count = group
                    .assignment_count
                    .checked_add(1)
                    .ok_or_else(|| "ec_spire boundary identity assignment count overflow".to_owned())?;
                if assignment.flags & storage::SPIRE_ASSIGNMENT_FLAG_PRIMARY != 0 {
                    group.primary_assignment_count =
                        group.primary_assignment_count.checked_add(1).ok_or_else(|| {
                            "ec_spire boundary identity primary count overflow".to_owned()
                        })?;
                }
                if assignment.flags & storage::SPIRE_ASSIGNMENT_FLAG_BOUNDARY_REPLICA != 0 {
                    group.boundary_replica_assignment_count =
                        group.boundary_replica_assignment_count.checked_add(1).ok_or_else(|| {
                            "ec_spire boundary identity replica count overflow".to_owned()
                        })?;
                }
                if assignment.flags & storage::SPIRE_ASSIGNMENT_FLAG_DELTA_INSERT != 0 {
                    group.delta_insert_assignment_count =
                        group.delta_insert_assignment_count.checked_add(1).ok_or_else(|| {
                            "ec_spire boundary identity delta count overflow".to_owned()
                        })?;
                }
                group.leaf_pids.insert(placement.pid);
                group.node_ids.insert(placement.node_id);
                group.local_store_ids.insert(placement.local_store_id);
            }
        }

        groups
            .into_iter()
            .filter(|(_, group)| group.boundary_replica_assignment_count > 0)
            .map(|(vec_id, group)| {
                let vec_id_scope = boundary_replica_identity_scope(&vec_id);
                let node_count = u64::try_from(group.node_ids.len())
                    .map_err(|_| "ec_spire boundary identity node count overflow")?;
                let (status, recommendation) = boundary_replica_identity_status(
                    vec_id_scope,
                    group.primary_assignment_count,
                    group.boundary_replica_assignment_count,
                    node_count,
                );
                Ok(SpireBoundaryReplicaIdentitySnapshotRow {
                    active_epoch: root_control.active_epoch,
                    vec_id,
                    vec_id_scope,
                    assignment_count: group.assignment_count,
                    primary_assignment_count: group.primary_assignment_count,
                    boundary_replica_assignment_count: group.boundary_replica_assignment_count,
                    delta_insert_assignment_count: group.delta_insert_assignment_count,
                    leaf_pid_count: u64::try_from(group.leaf_pids.len())
                        .map_err(|_| "ec_spire boundary identity leaf count overflow")?,
                    node_count,
                    local_store_count: u64::try_from(group.local_store_ids.len())
                        .map_err(|_| "ec_spire boundary identity store count overflow")?,
                    min_node_id: group.node_ids.first().copied().unwrap_or(0),
                    max_node_id: group.node_ids.last().copied().unwrap_or(0),
                    status,
                    recommendation,
                })
            })
            .collect()
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

pub(crate) unsafe fn index_boundary_replica_placement_diagnostics(
    index_relation: pg_sys::Relation,
) -> Vec<SpireBoundaryReplicaPlacementDiagnosticRow> {
    let result = (|| -> Result<Vec<SpireBoundaryReplicaPlacementDiagnosticRow>, String> {
        // SAFETY: index_relation is the open SPIRE index relation inspected by
        // this read-only diagnostic.
        let root_control = unsafe { page::read_root_control_page(index_relation) };
        if root_control.active_epoch == 0 {
            return Ok(Vec::new());
        }
        // SAFETY: root_control was read from the same open relation and names
        // the active manifest set for boundary placement diagnostics.
        let (_epoch_manifest, _object_manifest, placement_directory) = unsafe {
            load_relation_epoch_manifests_for_boundary_placement_diagnostics(
                index_relation,
                root_control,
            )?
        };
        // SAFETY: placement_directory was decoded from the active relation
        // manifests and is used to open the corresponding local object stores.
        let object_store = unsafe {
            storage::SpireRelationObjectStoreSet::for_index_relation_and_placements(
                index_relation,
                &placement_directory,
                pg_sys::AccessShareLock as pg_sys::LOCKMODE,
            )?
        };
        let mut groups = BTreeMap::<Vec<u8>, BoundaryReplicaPlacementAccumulator>::new();

        for placement in &placement_directory.entries {
            let metadata_placement = coordinator_metadata_read_available_placement(placement);
            let header = object_store.read_object_header(&metadata_placement)?;
            let assignments = match header.kind {
                storage::SpirePartitionObjectKind::Leaf => {
                    read_leaf_assignment_rows(&object_store, &metadata_placement)?
                }
                storage::SpirePartitionObjectKind::Delta => {
                    object_store
                        .read_delta_object(&metadata_placement)?
                        .assignments
                }
                _ => continue,
            };

            for assignment in assignments {
                let group = groups
                    .entry(assignment.vec_id.as_bytes().to_vec())
                    .or_default();
                group.assignment_count = group.assignment_count.checked_add(1).ok_or_else(|| {
                    "ec_spire boundary placement assignment count overflow".to_owned()
                })?;
                if assignment.flags & storage::SPIRE_ASSIGNMENT_FLAG_PRIMARY != 0 {
                    group.primary_assignment_count =
                        group.primary_assignment_count.checked_add(1).ok_or_else(|| {
                            "ec_spire boundary placement primary count overflow".to_owned()
                        })?;
                }
                if assignment.flags & storage::SPIRE_ASSIGNMENT_FLAG_BOUNDARY_REPLICA != 0 {
                    group.boundary_replica_assignment_count =
                        group.boundary_replica_assignment_count.checked_add(1).ok_or_else(|| {
                            "ec_spire boundary placement replica count overflow".to_owned()
                        })?;
                    match placement.state {
                        meta::SpirePlacementState::Available => {}
                        meta::SpirePlacementState::Stale => {
                            group.stale_boundary_replica_count =
                                group.stale_boundary_replica_count.checked_add(1).ok_or_else(
                                    || {
                                        "ec_spire boundary placement stale count overflow"
                                            .to_owned()
                                    },
                                )?;
                        }
                        meta::SpirePlacementState::Unavailable => {
                            group.unavailable_boundary_replica_count = group
                                .unavailable_boundary_replica_count
                                .checked_add(1)
                                .ok_or_else(|| {
                                    "ec_spire boundary placement unavailable count overflow"
                                        .to_owned()
                                })?;
                        }
                        meta::SpirePlacementState::Skipped => {
                            group.skipped_boundary_replica_count = group
                                .skipped_boundary_replica_count
                                .checked_add(1)
                                .ok_or_else(|| {
                                    "ec_spire boundary placement skipped count overflow".to_owned()
                                })?;
                        }
                    }
                }
                group.node_ids.insert(placement.node_id);
            }
        }

        groups
            .into_iter()
            .filter(|(_, group)| {
                group.primary_assignment_count > 0 || group.boundary_replica_assignment_count > 0
            })
            .map(|(vec_id, group)| {
                let vec_id_scope = boundary_replica_identity_scope(&vec_id);
                let (status, degraded_mode_action, recommendation) =
                    boundary_replica_placement_status(
                        group.primary_assignment_count,
                        group.boundary_replica_assignment_count,
                        group.stale_boundary_replica_count,
                        group.unavailable_boundary_replica_count,
                        group.skipped_boundary_replica_count,
                    );
                Ok(SpireBoundaryReplicaPlacementDiagnosticRow {
                    active_epoch: root_control.active_epoch,
                    vec_id,
                    vec_id_scope,
                    assignment_count: group.assignment_count,
                    primary_assignment_count: group.primary_assignment_count,
                    boundary_replica_assignment_count: group.boundary_replica_assignment_count,
                    stale_boundary_replica_count: group.stale_boundary_replica_count,
                    unavailable_boundary_replica_count: group.unavailable_boundary_replica_count,
                    skipped_boundary_replica_count: group.skipped_boundary_replica_count,
                    node_count: u64::try_from(group.node_ids.len())
                        .map_err(|_| "ec_spire boundary placement node count overflow")?,
                    min_node_id: group.node_ids.first().copied().unwrap_or(0),
                    max_node_id: group.node_ids.last().copied().unwrap_or(0),
                    status,
                    degraded_mode_action,
                    recommendation,
                })
            })
            .collect()
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

fn count_snapshot_options_leaf_pids(
    snapshot: &meta::SpirePublishedEpochSnapshot<'_>,
    object_store: &impl storage::SpireObjectReader,
    recursive_build_enabled: bool,
) -> Result<u32, String> {
    if recursive_build_enabled {
        scan::count_snapshot_recursive_leaf_pids(snapshot, object_store)
    } else {
        scan::count_snapshot_single_level_leaf_pids(snapshot, object_store)
    }
}

fn level_target_fanout(
    level: u16,
    relation_options: &options::EcSpireOptions,
    observed_child_count: u64,
) -> Result<u32, String> {
    if level == 1 && relation_options.nlists > 0 {
        return u32::try_from(relation_options.nlists)
            .map_err(|_| "ec_spire nlists reloption must be non-negative".to_owned());
    }
    if level > 1 {
        if let Some(recursive_fanout) = relation_options.recursive_fanout() {
            return Ok(recursive_fanout);
        }
    }
    u32::try_from(observed_child_count)
        .map_err(|_| "ec_spire observed routing child count exceeds u32".to_owned())
}

fn level_nprobe_resolution(
    level: u16,
    leaf_count: u32,
    observed_child_count: u64,
    relation_options: &options::EcSpireOptions,
) -> Result<(Option<i32>, u32, &'static str, &'static str), String> {
    let relation_nprobe = u32::try_from(relation_options.nprobe)
        .map_err(|_| "ec_spire nprobe reloption must be non-negative".to_owned())?;
    if level <= 1 {
        let resolved = options::resolve_scan_nprobe(leaf_count, relation_nprobe);
        let session_nprobe = resolved
            .session_nprobe
            .map(|value| i32::try_from(value).expect("session nprobe should fit in i32"));
        return Ok((
            session_nprobe,
            resolved.effective_nprobe,
            resolved.source,
            "relation_or_session_leaf_level",
        ));
    }
    let nprobe_policy = options::SpireRecursiveNprobePolicy::from_level_values(
        options::resolve_scan_nprobe(leaf_count, relation_nprobe).effective_nprobe,
        relation_options.nprobe_per_level_values(),
    )?;
    if let Some(configured_nprobe) = nprobe_policy.configured_nprobe_for_level(level) {
        let observed_child_count = u32::try_from(observed_child_count)
            .map_err(|_| "ec_spire observed routing child count exceeds u32".to_owned())?;
        return Ok((
            None,
            configured_nprobe.clamp(1, observed_child_count.max(1)),
            "relation_per_level",
            "configured_above_level_1",
        ));
    }
    Ok((
        None,
        1,
        "conservative_upper_level",
        "one_child_above_level_1",
    ))
}

fn epoch_snapshot_rows_from_manifests(
    root_control: meta::SpireRootControlState,
    mut manifests: Vec<(crate::storage::page::ItemPointer, meta::SpireEpochManifest)>,
    now_micros: i64,
) -> Result<Vec<SpireIndexEpochSnapshotRow>, String> {
    manifests.sort_by_key(|(tid, manifest)| (manifest.epoch, tid.block_number, tid.offset_number));

    let mut latest_manifest_tid_by_epoch = HashMap::new();
    for (tid, manifest) in &manifests {
        latest_manifest_tid_by_epoch
            .entry(manifest.epoch)
            .and_modify(|latest_tid: &mut crate::storage::page::ItemPointer| {
                if (tid.block_number, tid.offset_number)
                    > (latest_tid.block_number, latest_tid.offset_number)
                {
                    *latest_tid = *tid;
                }
            })
            .or_insert(*tid);
    }
    let latest_manifests = manifests
        .iter()
        .filter_map(|(tid, manifest)| {
            let latest_tid = latest_manifest_tid_by_epoch.get(&manifest.epoch)?;
            if latest_tid == tid {
                Some(*manifest)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    let cleanup_plan =
        meta::plan_epoch_cleanup(&latest_manifests, root_control.active_epoch, now_micros)?;
    let cleanup_epochs: HashSet<u64> = cleanup_plan.cleanup_epochs.into_iter().collect();
    let retained_retired_epochs: HashSet<u64> =
        cleanup_plan.retained_retired_epochs.into_iter().collect();

    Ok(manifests
        .into_iter()
        .map(|(tid, manifest)| {
            let is_latest_manifest = latest_manifest_tid_by_epoch
                .get(&manifest.epoch)
                .is_some_and(|latest_tid| latest_tid == &tid);
            let is_active_root_manifest = root_control.active_epoch == manifest.epoch
                && root_control.epoch_manifest_tid == tid;
            let cleanup_eligible_now =
                is_latest_manifest && cleanup_epochs.contains(&manifest.epoch);
            let retained_retired = retained_retired_epochs.contains(&manifest.epoch);
            let cleanup_blocked_reason = if is_active_root_manifest {
                "active_root_manifest"
            } else if is_latest_manifest {
                epoch_cleanup_blocked_reason(
                    &manifest,
                    now_micros,
                    false,
                    retained_retired,
                    cleanup_eligible_now,
                )
            } else {
                "superseded_manifest"
            };
            SpireIndexEpochSnapshotRow {
                active_epoch: root_control.active_epoch,
                epoch: manifest.epoch,
                state: epoch_state_name(manifest.state),
                consistency_mode: consistency_mode_name(manifest.consistency_mode),
                published_at_micros: manifest.published_at_micros,
                retain_until_micros: manifest.retain_until_micros,
                active_query_count: manifest.active_query_count,
                manifest_block: tid.block_number,
                manifest_offset: tid.offset_number,
                is_active_root_manifest,
                cleanup_eligible_now,
                cleanup_blocked_reason,
            }
        })
        .collect())
}

fn apply_leaf_snapshot_base_row(
    rows_by_leaf_pid: &mut HashMap<u64, SpireIndexLeafSnapshotRow>,
    active_epoch: u64,
    header: &storage::SpirePartitionObjectHeader,
    placement: &meta::SpirePlacementEntry,
    base_primary_assignment_count: u64,
    base_boundary_replica_assignment_count: u64,
) {
    let row = rows_by_leaf_pid
        .entry(header.pid)
        .or_insert_with(|| SpireIndexLeafSnapshotRow {
            active_epoch,
            leaf_pid: header.pid,
            parent_pid: header.parent_pid,
            object_version: header.object_version,
            node_id: placement.node_id,
            local_store_id: placement.local_store_id,
            placement_state: placement_state_name(placement.state),
            base_assignment_count: 0,
            base_primary_assignment_count: 0,
            base_boundary_replica_assignment_count: 0,
            delta_object_count: 0,
            delta_insert_assignment_count: 0,
            delta_boundary_replica_insert_assignment_count: 0,
            delta_delete_assignment_count: 0,
            effective_assignment_count: 0,
            effective_boundary_replica_assignment_count: 0,
            split_assignment_threshold: 0,
            merge_assignment_threshold: 0,
            split_recommended: false,
            merge_recommended: false,
            maintenance_action: "none",
            maintenance_reason: "not_evaluated",
            leaf_object_bytes: 0,
            delta_object_bytes: 0,
        });

    row.active_epoch = active_epoch;
    row.leaf_pid = header.pid;
    row.parent_pid = header.parent_pid;
    row.object_version = header.object_version;
    row.node_id = placement.node_id;
    row.local_store_id = placement.local_store_id;
    row.placement_state = placement_state_name(placement.state);
    row.base_assignment_count = u64::from(header.assignment_count);
    row.base_primary_assignment_count = base_primary_assignment_count;
    row.base_boundary_replica_assignment_count = base_boundary_replica_assignment_count;
    row.effective_assignment_count = u64::from(header.assignment_count);
    row.effective_boundary_replica_assignment_count = base_boundary_replica_assignment_count;
    row.maintenance_action = "none";
    row.maintenance_reason = "not_evaluated";
    row.leaf_object_bytes = u64::from(placement.object_bytes);
}

