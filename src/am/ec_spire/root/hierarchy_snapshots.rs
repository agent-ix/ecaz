pub(crate) unsafe fn index_insert_debt_snapshot(
    index_relation: pg_sys::Relation,
) -> SpireIndexInsertDebtSnapshot {
    let root_control = unsafe { page::read_root_control_page(index_relation) };
    let leaf_rows = unsafe { index_leaf_snapshot(index_relation) };
    let active_leaf_count = u64::try_from(leaf_rows.len())
        .unwrap_or_else(|_| pgrx::error!("ec_spire leaf row count exceeds u64"));
    let leaf_count_with_deltas = leaf_rows
        .iter()
        .filter(|row| row.delta_object_count > 0)
        .count()
        .try_into()
        .unwrap_or_else(|_| pgrx::error!("ec_spire leaf delta row count exceeds u64"));
    let delta_object_count = leaf_rows
        .iter()
        .map(|row| row.delta_object_count)
        .sum::<u64>();
    let delta_insert_assignment_count = leaf_rows
        .iter()
        .map(|row| row.delta_insert_assignment_count)
        .sum::<u64>();
    let max_delta_objects_per_leaf = leaf_rows
        .iter()
        .map(|row| row.delta_object_count)
        .max()
        .unwrap_or(0);
    let batching_recommended =
        max_delta_objects_per_leaf > 1 || delta_object_count > active_leaf_count;
    let recommendation = if batching_recommended {
        "batch post-build inserts by routed base leaf before publishing replacement epochs"
    } else {
        "none"
    };

    SpireIndexInsertDebtSnapshot {
        active_epoch: root_control.active_epoch,
        active_leaf_count,
        leaf_count_with_deltas,
        delta_object_count,
        delta_insert_assignment_count,
        max_delta_objects_per_leaf,
        insert_batching_supported: false,
        batching_recommended,
        recommendation,
    }
}

pub(crate) unsafe fn index_hierarchy_snapshot(
    index_relation: pg_sys::Relation,
) -> SpireIndexHierarchySnapshot {
    let result = (|| -> Result<SpireIndexHierarchySnapshot, String> {
        let root_control = unsafe { page::read_root_control_page(index_relation) };
        if root_control.active_epoch == 0 {
            let (status, recommendation) = hierarchy_snapshot_status(0, 0, 0, true, false);
            return Ok(SpireIndexHierarchySnapshot {
                active_epoch: 0,
                root_pid: 0,
                root_level: 0,
                max_observed_level: 0,
                hierarchy_depth: 0,
                routing_object_count: 0,
                root_routing_object_count: 0,
                internal_routing_object_count: 0,
                leaf_object_count: 0,
                delta_object_count: 0,
                centroid_dimensions: 0,
                root_child_count: 0,
                distinct_leaf_parent_count: 0,
                recursive_routing_supported: false,
                per_level_nprobe_supported: false,
                status,
                recommendation,
            });
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

        let mut root_pid = 0_u64;
        let mut root_level = 0_u16;
        let mut max_observed_level = 0_u16;
        let mut routing_object_count = 0_u64;
        let mut root_routing_object_count = 0_u64;
        let mut internal_routing_object_count = 0_u64;
        let mut leaf_object_count = 0_u64;
        let mut delta_object_count = 0_u64;
        let mut centroid_dimensions = 0_u16;
        let mut root_child_count = 0_u64;
        let mut leaf_parent_pids = HashSet::new();
        let mut hierarchy_objects = Vec::new();

        for manifest_entry in &snapshot.object_manifest().entries {
            let lookup = snapshot.require_lookup(manifest_entry.pid, "hierarchy snapshot")?;
            let placement = lookup.placement;
            if placement.state != meta::SpirePlacementState::Available {
                continue;
            }
            let header = unsafe { object_store.read_object_header(placement)? };
            max_observed_level = max_observed_level.max(header.level);
            match header.kind {
                storage::SpirePartitionObjectKind::Root => {
                    let routing_object = unsafe { object_store.read_routing_object(placement)? };
                    routing_object_count =
                        routing_object_count.checked_add(1).ok_or_else(|| {
                            "ec_spire hierarchy snapshot routing object count overflow".to_owned()
                        })?;
                    root_routing_object_count =
                        root_routing_object_count.checked_add(1).ok_or_else(|| {
                            "ec_spire hierarchy snapshot root object count overflow".to_owned()
                        })?;
                    root_pid = header.pid;
                    root_level = header.level;
                    centroid_dimensions = routing_object.dimensions;
                    hierarchy_objects.push(hierarchy_object_summary(
                        &routing_object.header,
                        routing_object.child_pids.clone(),
                    ));
                    root_child_count =
                        u64::try_from(routing_object.child_count()).map_err(|_| {
                            "ec_spire hierarchy snapshot root child count exceeds u64".to_owned()
                        })?;
                }
                storage::SpirePartitionObjectKind::Internal => {
                    routing_object_count =
                        routing_object_count.checked_add(1).ok_or_else(|| {
                            "ec_spire hierarchy snapshot routing object count overflow".to_owned()
                        })?;
                    internal_routing_object_count = internal_routing_object_count
                        .checked_add(1)
                        .ok_or_else(|| {
                            "ec_spire hierarchy snapshot internal object count overflow".to_owned()
                        })?;
                    let routing_object = unsafe { object_store.read_routing_object(placement)? };
                    hierarchy_objects.push(hierarchy_object_summary(
                        &routing_object.header,
                        routing_object.child_pids.clone(),
                    ));
                }
                storage::SpirePartitionObjectKind::Leaf => {
                    leaf_object_count = leaf_object_count.checked_add(1).ok_or_else(|| {
                        "ec_spire hierarchy snapshot leaf object count overflow".to_owned()
                    })?;
                    leaf_parent_pids.insert(header.parent_pid);
                    hierarchy_objects.push(hierarchy_object_summary(&header, Vec::new()));
                }
                storage::SpirePartitionObjectKind::Delta => {
                    delta_object_count = delta_object_count.checked_add(1).ok_or_else(|| {
                        "ec_spire hierarchy snapshot delta object count overflow".to_owned()
                    })?;
                    hierarchy_objects.push(hierarchy_object_summary(&header, Vec::new()));
                }
            }
        }

        let hierarchy_depth = if root_routing_object_count == 0 {
            0
        } else {
            max_observed_level.max(root_level)
        };
        let hierarchy_shape_valid = validate_recursive_hierarchy_shape(&hierarchy_objects).is_ok();
        let per_level_nprobe_supported = hierarchy_shape_valid && internal_routing_object_count > 0;
        let (status, recommendation) = hierarchy_snapshot_status(
            root_routing_object_count,
            internal_routing_object_count,
            leaf_object_count,
            hierarchy_shape_valid,
            per_level_nprobe_supported,
        );

        Ok(SpireIndexHierarchySnapshot {
            active_epoch: root_control.active_epoch,
            root_pid,
            root_level,
            max_observed_level,
            hierarchy_depth,
            routing_object_count,
            root_routing_object_count,
            internal_routing_object_count,
            leaf_object_count,
            delta_object_count,
            centroid_dimensions,
            root_child_count,
            distinct_leaf_parent_count: u64::try_from(leaf_parent_pids.len()).map_err(|_| {
                "ec_spire hierarchy snapshot leaf parent count exceeds u64".to_owned()
            })?,
            recursive_routing_supported: hierarchy_shape_valid && internal_routing_object_count > 0,
            per_level_nprobe_supported,
            status,
            recommendation,
        })
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

pub(crate) unsafe fn index_object_snapshot(
    index_relation: pg_sys::Relation,
) -> Vec<SpireIndexObjectSnapshotRow> {
    let result = (|| -> Result<Vec<SpireIndexObjectSnapshotRow>, String> {
        let root_control = unsafe { page::read_root_control_page(index_relation) };
        if root_control.active_epoch == 0 {
            return Ok(Vec::new());
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
        let mut rows = Vec::with_capacity(snapshot.object_manifest().entries.len());

        for manifest_entry in &snapshot.object_manifest().entries {
            let lookup = snapshot.require_lookup(manifest_entry.pid, "object snapshot")?;
            let placement = lookup.placement;
            let mut row = SpireIndexObjectSnapshotRow {
                active_epoch: root_control.active_epoch,
                pid: manifest_entry.pid,
                object_kind: "unreadable",
                object_version: manifest_entry.object_version,
                published_epoch_backref: 0,
                level: 0,
                parent_pid: 0,
                child_count: 0,
                assignment_count: 0,
                node_id: placement.node_id,
                local_store_id: placement.local_store_id,
                store_relid: placement.store_relid,
                placement_state: placement_state_name(placement.state),
                object_bytes: u64::from(placement.object_bytes),
                object_readable: false,
            };
            if placement.state == meta::SpirePlacementState::Available {
                let header = unsafe { object_store.read_object_header(placement)? };
                row.object_kind = partition_object_kind_name(header.kind);
                row.object_version = header.object_version;
                row.published_epoch_backref = header.published_epoch_backref;
                row.level = header.level;
                row.parent_pid = header.parent_pid;
                row.child_count = u64::from(header.child_count);
                row.assignment_count = u64::from(header.assignment_count);
                row.object_readable = true;
            }
            rows.push(row);
        }

        rows.sort_by_key(|row| row.pid);
        Ok(rows)
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

pub(crate) unsafe fn index_delta_snapshot(
    index_relation: pg_sys::Relation,
) -> Vec<SpireIndexDeltaSnapshotRow> {
    let result = (|| -> Result<Vec<SpireIndexDeltaSnapshotRow>, String> {
        let root_control = unsafe { page::read_root_control_page(index_relation) };
        if root_control.active_epoch == 0 {
            return Ok(Vec::new());
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
        let mut rows = Vec::new();

        for manifest_entry in &snapshot.object_manifest().entries {
            let lookup = snapshot.require_lookup(manifest_entry.pid, "delta snapshot")?;
            let placement = lookup.placement;
            if placement.state != meta::SpirePlacementState::Available {
                continue;
            }
            let header = unsafe { object_store.read_object_header(placement)? };
            if header.kind != storage::SpirePartitionObjectKind::Delta {
                continue;
            }
            let delta_object = unsafe { object_store.read_delta_object(placement)? };
            let mut insert_assignment_count = 0_u64;
            let mut delete_assignment_count = 0_u64;
            for assignment in &delta_object.assignments {
                if storage::is_delete_delta_assignment(assignment) {
                    delete_assignment_count =
                        delete_assignment_count.checked_add(1).ok_or_else(|| {
                            "ec_spire delta snapshot delete assignment count overflow".to_owned()
                        })?;
                } else if assignment.flags & storage::SPIRE_ASSIGNMENT_FLAG_DELTA_INSERT != 0 {
                    insert_assignment_count =
                        insert_assignment_count.checked_add(1).ok_or_else(|| {
                            "ec_spire delta snapshot insert assignment count overflow".to_owned()
                        })?;
                }
            }
            rows.push(SpireIndexDeltaSnapshotRow {
                active_epoch: root_control.active_epoch,
                delta_pid: header.pid,
                parent_leaf_pid: header.parent_pid,
                object_version: header.object_version,
                published_epoch_backref: header.published_epoch_backref,
                node_id: placement.node_id,
                local_store_id: placement.local_store_id,
                store_relid: placement.store_relid,
                placement_state: placement_state_name(placement.state),
                assignment_count: u64::from(header.assignment_count),
                insert_assignment_count,
                delete_assignment_count,
                object_bytes: u64::from(placement.object_bytes),
            });
        }

        rows.sort_by_key(|row| row.delta_pid);
        Ok(rows)
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

pub(crate) unsafe fn index_scan_placement_snapshot(
    index_relation: pg_sys::Relation,
    query_values: Vec<f32>,
) -> Vec<SpireIndexScanPlacementSnapshotRow> {
    let result = (|| -> Result<Vec<SpireIndexScanPlacementSnapshotRow>, String> {
        let query = scan::SpireScanQuery::new(query_values)?;
        let root_control = unsafe { page::read_root_control_page(index_relation) };
        if root_control.active_epoch == 0 {
            return Ok(Vec::new());
        }

        let (epoch_manifest, object_manifest, placement_directory) =
            unsafe { scan::load_relation_epoch_manifests(index_relation, root_control)? };
        let snapshot = meta::SpirePublishedEpochSnapshot::new(
            &epoch_manifest,
            &object_manifest,
            &placement_directory,
        )?;
        let object_store =
            unsafe { storage::SpireRelationObjectStore::for_index_relation(index_relation)? };
        let diagnostics = scan::collect_single_level_scan_placement_diagnostics(
            &snapshot,
            &object_store,
            &query,
            options::relation_options(index_relation),
        )?;
        let rows = diagnostics
            .stores
            .into_iter()
            .map(|store| SpireIndexScanPlacementSnapshotRow {
                active_epoch: store.epoch,
                effective_nprobe: diagnostics.scan_plan.nprobe,
                effective_nprobe_source: diagnostics.scan_plan.nprobe_source,
                effective_rerank_width: diagnostics.scan_plan.rerank_width as u64,
                effective_rerank_width_source: diagnostics.scan_plan.rerank_width_source,
                node_id: store.node_id,
                local_store_id: store.local_store_id,
                scanned_pid_count: store.scanned_pid_count as u64,
                leaf_pid_count: store.leaf_pid_count as u64,
                delta_pid_count: store.delta_pid_count as u64,
                candidate_row_count: store.candidate_row_count as u64,
                leaf_candidate_row_count: store.leaf_candidate_row_count as u64,
                delta_candidate_row_count: store.delta_candidate_row_count as u64,
                delete_delta_row_count: store.delete_delta_row_count as u64,
            })
            .collect();
        Ok(rows)
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

pub(crate) unsafe fn index_root_routing_snapshot(
    index_relation: pg_sys::Relation,
) -> Vec<SpireIndexRootRoutingSnapshotRow> {
    let result = (|| -> Result<Vec<SpireIndexRootRoutingSnapshotRow>, String> {
        let root_control = unsafe { page::read_root_control_page(index_relation) };
        if root_control.active_epoch == 0 {
            return Ok(Vec::new());
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
        collect_root_routing_snapshot_rows(&snapshot, &object_store)
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

pub(crate) unsafe fn index_routing_centroid_snapshot(
    index_relation: pg_sys::Relation,
) -> Vec<SpireIndexRoutingCentroidSnapshotRow> {
    let result = (|| -> Result<Vec<SpireIndexRoutingCentroidSnapshotRow>, String> {
        let root_control = unsafe { page::read_root_control_page(index_relation) };
        if root_control.active_epoch == 0 {
            return Ok(Vec::new());
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
        collect_routing_centroid_snapshot_rows(&snapshot, &object_store)
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

fn collect_root_routing_snapshot_rows(
    snapshot: &meta::SpireValidatedEpochSnapshot<'_>,
    object_store: &impl storage::SpireObjectReader,
) -> Result<Vec<SpireIndexRootRoutingSnapshotRow>, String> {
    let mut root = None;
    // Walk the full manifest so malformed epochs with multiple roots are reported.
    for manifest_entry in &snapshot.object_manifest().entries {
        let lookup = snapshot.require_lookup(manifest_entry.pid, "root routing snapshot")?;
        let header = object_store.read_object_header(lookup.placement)?;
        if header.kind != storage::SpirePartitionObjectKind::Root {
            continue;
        }
        if root.is_some() {
            return Err("ec_spire root routing snapshot found multiple root objects".to_owned());
        }
        root = Some((
            manifest_entry.pid,
            manifest_entry.object_version,
            object_store.read_routing_object(lookup.placement)?,
        ));
    }

    let Some((root_pid, root_object_version, root_object)) = root else {
        return Err("ec_spire root routing snapshot found no active root object".to_owned());
    };
    let root_child_count = u64::try_from(root_object.child_count())
        .map_err(|_| "ec_spire root routing child count exceeds u64".to_owned())?;
    root_object
        .children()
        .map(|child| {
            let child_lookup = snapshot.require_lookup(child.child_pid, "root routing child")?;
            let child_header = object_store.read_object_header(child_lookup.placement)?;
            Ok(SpireIndexRootRoutingSnapshotRow {
                active_epoch: snapshot.epoch_manifest().epoch,
                root_pid,
                root_object_version,
                root_level: root_object.header.level,
                root_child_count,
                centroid_dimensions: root_object.dimensions,
                centroid_index: child.centroid_index,
                child_pid: child.child_pid,
                child_kind: partition_object_kind_name(child_header.kind),
                child_object_version: child_header.object_version,
                child_level: child_header.level,
                child_parent_pid: child_header.parent_pid,
                child_assignment_count: u64::from(child_header.assignment_count),
                child_node_id: child_lookup.placement.node_id,
                child_local_store_id: child_lookup.placement.local_store_id,
                child_store_relid: child_lookup.placement.store_relid,
                child_placement_state: placement_state_name(child_lookup.placement.state),
                child_object_bytes: u64::from(child_lookup.placement.object_bytes),
            })
        })
        .collect::<Result<Vec<_>, String>>()
}

fn collect_routing_centroid_snapshot_rows(
    snapshot: &meta::SpireValidatedEpochSnapshot<'_>,
    object_store: &impl storage::SpireObjectReader,
) -> Result<Vec<SpireIndexRoutingCentroidSnapshotRow>, String> {
    let mut rows = Vec::new();
    for manifest_entry in &snapshot.object_manifest().entries {
        let lookup =
            snapshot.require_lookup(manifest_entry.pid, "routing centroid snapshot parent")?;
        let parent_header = object_store.read_object_header(lookup.placement)?;
        if parent_header.kind != storage::SpirePartitionObjectKind::Root
            && parent_header.kind != storage::SpirePartitionObjectKind::Internal
        {
            continue;
        }
        let parent = object_store.read_routing_object(lookup.placement)?;
        let parent_child_count = u64::try_from(parent.child_count())
            .map_err(|_| "ec_spire routing centroid child count exceeds u64".to_owned())?;
        for child in parent.children() {
            let child_lookup =
                snapshot.require_lookup(child.child_pid, "routing centroid snapshot child")?;
            let child_header = object_store.read_object_header(child_lookup.placement)?;
            rows.push(SpireIndexRoutingCentroidSnapshotRow {
                active_epoch: snapshot.epoch_manifest().epoch,
                parent_pid: parent.header.pid,
                parent_kind: partition_object_kind_name(parent.header.kind),
                parent_object_version: parent.header.object_version,
                parent_level: parent.header.level,
                parent_child_count,
                centroid_dimensions: parent.dimensions,
                centroid_index: child.centroid_index,
                child_pid: child.child_pid,
                child_kind: partition_object_kind_name(child_header.kind),
                child_object_version: child_header.object_version,
                child_level: child_header.level,
                child_parent_pid: child_header.parent_pid,
                child_assignment_count: u64::from(child_header.assignment_count),
                child_node_id: child_lookup.placement.node_id,
                child_local_store_id: child_lookup.placement.local_store_id,
                child_store_relid: child_lookup.placement.store_relid,
                child_placement_state: placement_state_name(child_lookup.placement.state),
                child_object_bytes: u64::from(child_lookup.placement.object_bytes),
                centroid: child.centroid.to_vec(),
            });
        }
    }
    Ok(rows)
}
