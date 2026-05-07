pub(crate) unsafe fn active_snapshot_diagnostics(
    index_relation: pg_sys::Relation,
) -> SpireActiveSnapshotDiagnostics {
    let result = (|| -> Result<SpireActiveSnapshotDiagnostics, String> {
        let root_control = unsafe { page::read_root_control_page(index_relation) };
        if root_control.active_epoch == 0 {
            return Ok(SpireActiveSnapshotDiagnostics::empty(root_control));
        }

        let (epoch_manifest, object_manifest, placement_directory) =
            unsafe { scan::load_relation_epoch_manifests(index_relation, root_control)? };
        let snapshot = meta::SpirePublishedEpochSnapshot::new(
            &epoch_manifest,
            &object_manifest,
            &placement_directory,
        )?;
        let object_store =
            unsafe {
                storage::SpireRelationObjectStoreSet::for_index_relation_and_placements(
                    index_relation,
                    &placement_directory,
                    pg_sys::AccessShareLock as pg_sys::LOCKMODE,
                )?
            };
        let diagnostics = diagnostics::collect_snapshot_diagnostics(&snapshot, &object_store)?;

        Ok(SpireActiveSnapshotDiagnostics {
            active_epoch: root_control.active_epoch,
            next_pid: root_control.next_pid,
            next_local_vec_seq: root_control.next_local_vec_seq,
            consistency_mode: consistency_mode_name(diagnostics.consistency_mode),
            object_count: diagnostics.object_count as u64,
            placement_count: diagnostics.placement_count as u64,
            local_store_count: diagnostics.local_store_count as u64,
            available_placement_count: diagnostics.available_placement_count as u64,
            stale_placement_count: diagnostics.stale_placement_count as u64,
            unavailable_placement_count: diagnostics.unavailable_placement_count as u64,
            skipped_placement_count: diagnostics.skipped_placement_count as u64,
            root_object_count: diagnostics.root_object_count as u64,
            internal_object_count: diagnostics.internal_object_count as u64,
            leaf_object_count: diagnostics.leaf_object_count as u64,
            delta_object_count: diagnostics.delta_object_count as u64,
            routing_child_count: diagnostics.routing_child_count as u64,
            leaf_assignment_count: diagnostics.leaf_assignment_count as u64,
            delta_assignment_count: diagnostics.delta_assignment_count as u64,
            available_object_bytes: diagnostics.available_object_bytes,
            routing_object_bytes: diagnostics.routing_object_bytes,
            leaf_object_bytes: diagnostics.leaf_object_bytes,
            delta_object_bytes: diagnostics.delta_object_bytes,
        })
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

pub(crate) unsafe fn index_allocator_snapshot(
    index_relation: pg_sys::Relation,
    warn_within: u64,
) -> SpireIndexAllocatorSnapshot {
    let result = (|| -> Result<SpireIndexAllocatorSnapshot, String> {
        let root_control = unsafe { page::read_root_control_page(index_relation) };
        let diagnostics = diagnostics::collect_allocator_diagnostics(&root_control, warn_within)?;
        Ok(SpireIndexAllocatorSnapshot {
            active_epoch: root_control.active_epoch,
            warn_within,
            next_pid: diagnostics.pid.next_value,
            remaining_pid_allocations: diagnostics.pid.remaining_allocations,
            pid_near_exhaustion: diagnostics.pid.near_exhaustion,
            next_local_vec_seq: diagnostics.local_vec_id.next_value,
            remaining_local_vec_id_allocations: diagnostics.local_vec_id.remaining_allocations,
            local_vec_id_near_exhaustion: diagnostics.local_vec_id.near_exhaustion,
        })
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

pub(crate) unsafe fn index_options_snapshot(
    index_relation: pg_sys::Relation,
) -> SpireIndexOptionsSnapshot {
    let result = (|| -> Result<SpireIndexOptionsSnapshot, String> {
        let relation_options = unsafe { options::relation_options(index_relation) };
        let recursive_build_enabled = relation_options.recursive_fanout().is_some();
        let root_control = unsafe { page::read_root_control_page(index_relation) };
        let mut recursive_level_parameters = Vec::new();
        let active_leaf_count = if root_control.active_epoch == 0 {
            0
        } else {
            let (epoch_manifest, object_manifest, placement_directory) =
                unsafe { scan::load_relation_epoch_manifests(index_relation, root_control)? };
            let snapshot = meta::SpirePublishedEpochSnapshot::new(
                &epoch_manifest,
                &object_manifest,
                &placement_directory,
            )?;
            let object_store = unsafe {
                storage::SpireRelationObjectStoreSet::for_index_relation_and_placements(
                    index_relation,
                    &placement_directory,
                    pg_sys::AccessShareLock as pg_sys::LOCKMODE,
                )?
            };
            let active_leaf_count = count_snapshot_options_leaf_pids(
                &snapshot,
                &object_store,
                recursive_build_enabled,
            )?;
            if recursive_build_enabled {
                let validated_snapshot =
                    meta::SpireValidatedEpochSnapshot::from_snapshot(snapshot)?;
                recursive_level_parameters = collect_level_parameter_snapshot_rows(
                    &validated_snapshot,
                    &object_store,
                    &relation_options,
                )?;
            }
            active_leaf_count
        };
        let relation_nprobe = u32::try_from(relation_options.nprobe)
            .map_err(|_| "ec_spire nprobe reloption must be non-negative".to_owned())?;
        let nprobe = options::resolve_scan_nprobe(active_leaf_count, relation_nprobe);
        let (effective_nprobe_per_level, nprobe_policy_per_level) =
            if recursive_level_parameters.is_empty() {
                let per_level = if root_control.active_epoch == 0 {
                    Vec::new()
                } else {
                    vec![nprobe.effective_nprobe]
                };
                let policies = if root_control.active_epoch == 0 {
                    Vec::new()
                } else {
                    vec!["single_level"]
                };
                (per_level, policies)
            } else {
                (
                    recursive_level_parameters
                        .iter()
                        .map(|row| row.effective_nprobe)
                        .collect(),
                    recursive_level_parameters
                        .iter()
                        .map(|row| row.nprobe_policy)
                        .collect(),
                )
            };
        let rerank_width = options::resolve_scan_rerank_width(relation_options.rerank_width);
        let assignment_payload_format = relation_options.assignment_payload_format();
        let (
            assignment_payload_scannable,
            assignment_payload_status,
            assignment_payload_recommendation,
        ) = assignment_payload_scannability(assignment_payload_format);

        Ok(SpireIndexOptionsSnapshot {
            nlists: relation_options.nlists,
            recursive_fanout: relation_options.recursive_fanout,
            recursive_build_enabled,
            local_store_count: relation_options.local_store_count,
            local_store_tablespaces: relation_options.local_store_tablespaces.clone(),
            boundary_replica_count: relation_options.boundary_replica_count,
            boundary_replication_enabled: relation_options.boundary_replica_count > 0,
            scan_dedupe_mode: if relation_options.boundary_replica_count > 0 {
                "vec_id"
            } else {
                "none"
            },
            active_leaf_count,
            relation_nprobe: relation_options.nprobe,
            session_nprobe: nprobe
                .session_nprobe
                .map(|value| i32::try_from(value).expect("session nprobe should fit in i32")),
            effective_nprobe: nprobe.effective_nprobe,
            effective_nprobe_source: nprobe.source,
            effective_nprobe_per_level,
            nprobe_policy_per_level,
            relation_rerank_width: relation_options.rerank_width,
            session_rerank_width: rerank_width.session_rerank_width,
            effective_rerank_width: rerank_width.effective_rerank_width,
            effective_rerank_width_source: rerank_width.source,
            training_sample_rows: relation_options.training_sample_rows,
            seed: relation_options.seed,
            pq_group_size: relation_options.pq_group_size,
            storage_format: relation_options.storage_format.reloption_name(),
            assignment_payload_format: assignment_payload_format_name(assignment_payload_format),
            assignment_payload_scannable,
            assignment_payload_status,
            assignment_payload_recommendation,
        })
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

#[derive(Debug, Clone, Copy, Default)]
struct SpireLevelParameterAccumulator {
    routing_object_count: u64,
    routing_child_count: u64,
    centroid_dimensions: u16,
}

pub(crate) unsafe fn index_level_parameter_snapshot(
    index_relation: pg_sys::Relation,
) -> Vec<SpireIndexLevelParameterSnapshotRow> {
    let result = (|| -> Result<Vec<SpireIndexLevelParameterSnapshotRow>, String> {
        let relation_options = unsafe { options::relation_options(index_relation) };
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
        let object_store = unsafe {
            storage::SpireRelationObjectStoreSet::for_index_relation_and_placements(
                index_relation,
                &placement_directory,
                pg_sys::AccessShareLock as pg_sys::LOCKMODE,
            )?
        };
        collect_level_parameter_snapshot_rows(&snapshot, &object_store, &relation_options)
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

fn collect_level_parameter_snapshot_rows(
    snapshot: &meta::SpireValidatedEpochSnapshot<'_>,
    object_store: &impl storage::SpireObjectReader,
    relation_options: &options::EcSpireOptions,
) -> Result<Vec<SpireIndexLevelParameterSnapshotRow>, String> {
    let mut active_leaf_count = 0_u32;
    let mut levels = BTreeMap::<u16, SpireLevelParameterAccumulator>::new();
    for manifest_entry in &snapshot.object_manifest().entries {
        let lookup = snapshot.require_lookup(manifest_entry.pid, "level parameter snapshot")?;
        if lookup.placement.state != meta::SpirePlacementState::Available {
            continue;
        }
        let header = object_store.read_object_header(lookup.placement)?;
        match header.kind {
            storage::SpirePartitionObjectKind::Root
            | storage::SpirePartitionObjectKind::Internal => {
                let routing_object = object_store.read_routing_object(lookup.placement)?;
                let entry = levels.entry(header.level).or_default();
                entry.routing_object_count =
                    entry.routing_object_count.checked_add(1).ok_or_else(|| {
                        "ec_spire level parameter routing object count overflow".to_owned()
                    })?;
                entry.routing_child_count = entry
                    .routing_child_count
                    .checked_add(u64::try_from(routing_object.child_count()).map_err(|_| {
                        "ec_spire level parameter routing child count exceeds u64".to_owned()
                    })?)
                    .ok_or_else(|| {
                        "ec_spire level parameter routing child count overflow".to_owned()
                    })?;
                entry.centroid_dimensions =
                    entry.centroid_dimensions.max(routing_object.dimensions);
            }
            storage::SpirePartitionObjectKind::Leaf => {
                active_leaf_count = active_leaf_count.checked_add(1).ok_or_else(|| {
                    "ec_spire level parameter active leaf count overflow".to_owned()
                })?;
            }
            storage::SpirePartitionObjectKind::Delta
            | storage::SpirePartitionObjectKind::TopGraph => {}
        }
    }

    let assignment_payload_format = relation_options.assignment_payload_format();
    levels
        .into_iter()
        .map(|(level, entry)| {
            let (session_nprobe, effective_nprobe, effective_nprobe_source, nprobe_policy) =
                level_nprobe_resolution(level, active_leaf_count, relation_options)?;
            Ok(SpireIndexLevelParameterSnapshotRow {
                active_epoch: snapshot.epoch_manifest().epoch,
                level,
                routing_object_count: entry.routing_object_count,
                routing_child_count: entry.routing_child_count,
                target_fanout: level_target_fanout(
                    level,
                    relation_options,
                    entry.routing_child_count,
                )?,
                relation_nprobe: relation_options.nprobe,
                session_nprobe,
                effective_nprobe,
                effective_nprobe_source,
                nprobe_policy,
                training_sample_rows: relation_options.training_sample_rows,
                training_iterations: u64::try_from(build::SPIRE_DEFAULT_KMEANS_ITERATIONS)
                    .expect("kmeans iteration count should fit in u64"),
                centroid_dimensions: entry.centroid_dimensions,
                distance_operator: "inner_product",
                assignment_payload_format: assignment_payload_format_name(
                    assignment_payload_format,
                ),
            })
        })
        .collect()
}

pub(crate) unsafe fn index_scan_sanity_snapshot(
    index_relation: pg_sys::Relation,
) -> SpireIndexScanSanitySnapshot {
    let result = (|| -> Result<SpireIndexScanSanitySnapshot, String> {
        let relation_options = unsafe { options::relation_options(index_relation) };
        let recursive_build_enabled = relation_options.recursive_fanout().is_some();
        let root_control = unsafe { page::read_root_control_page(index_relation) };
        let active_leaf_count = if root_control.active_epoch == 0 {
            0
        } else {
            let (epoch_manifest, object_manifest, placement_directory) =
                unsafe { scan::load_relation_epoch_manifests(index_relation, root_control)? };
            let snapshot = meta::SpirePublishedEpochSnapshot::new(
                &epoch_manifest,
                &object_manifest,
                &placement_directory,
            )?;
            let object_store = unsafe {
                storage::SpireRelationObjectStoreSet::for_index_relation_and_placements(
                    index_relation,
                    &placement_directory,
                    pg_sys::AccessShareLock as pg_sys::LOCKMODE,
                )?
            };
            count_snapshot_options_leaf_pids(&snapshot, &object_store, recursive_build_enabled)?
        };
        let relation_nprobe = u32::try_from(relation_options.nprobe)
            .map_err(|_| "ec_spire nprobe reloption must be non-negative".to_owned())?;
        let nprobe = options::resolve_scan_nprobe(active_leaf_count, relation_nprobe);
        let rerank_width = options::resolve_scan_rerank_width(relation_options.rerank_width);
        let exact_leaf_coverage =
            active_leaf_count > 0 && nprobe.effective_nprobe == active_leaf_count;
        let full_frontier_rerank =
            active_leaf_count > 0 && rerank_width.effective_rerank_width == 0;
        let (recall_sanity_status, latency_risk_status, recommendation) = scan_sanity_status(
            root_control.active_epoch,
            exact_leaf_coverage,
            full_frontier_rerank,
        );

        Ok(SpireIndexScanSanitySnapshot {
            active_epoch: root_control.active_epoch,
            active_leaf_count,
            effective_nprobe: nprobe.effective_nprobe,
            effective_nprobe_source: nprobe.source,
            exact_leaf_coverage,
            effective_rerank_width: rerank_width.effective_rerank_width,
            effective_rerank_width_source: rerank_width.source,
            full_frontier_rerank,
            recall_sanity_status,
            latency_risk_status,
            recommendation,
        })
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

pub(crate) unsafe fn index_health_snapshot(
    index_relation: pg_sys::Relation,
) -> SpireIndexHealthSnapshot {
    let diagnostics = unsafe { active_snapshot_diagnostics(index_relation) };
    health_snapshot_from_diagnostics(&diagnostics)
}

pub(crate) unsafe fn index_relation_storage_snapshot(
    index_relation: pg_sys::Relation,
) -> SpireIndexRelationStorageSnapshot {
    let result = (|| -> Result<SpireIndexRelationStorageSnapshot, String> {
        let index_relid: u32 = unsafe { (*index_relation).rd_id }.into();
        let root_control = unsafe { page::read_root_control_page(index_relation) };
        let mut active_tids = HashSet::<(u32, crate::storage::page::ItemPointer)>::new();
        let mut storage_relids = HashSet::from([index_relid]);
        if root_control.active_epoch != 0 {
            active_tids.insert((index_relid, root_control.epoch_manifest_tid));
            active_tids.insert((index_relid, root_control.object_manifest_tid));
            active_tids.insert((index_relid, root_control.placement_directory_tid));
            active_tids.insert((index_relid, root_control.local_store_config_tid));

            let (_epoch_manifest, object_manifest, placement_directory) =
                unsafe { scan::load_relation_epoch_manifests(index_relation, root_control)? };
            for entry in &object_manifest.entries {
                active_tids.insert((index_relid, entry.placement_tid));
            }
            for placement in &placement_directory.entries {
                storage_relids.insert(placement.store_relid);
            }

            let object_store = unsafe {
                storage::SpireRelationObjectStoreSet::for_index_relation_and_placements(
                    index_relation,
                    &placement_directory,
                    pg_sys::AccessShareLock as pg_sys::LOCKMODE,
                )?
            };
            for placement in &placement_directory.entries {
                for tid in unsafe { object_store.active_object_tuple_locators(placement)? } {
                    active_tids.insert((placement.store_relid, tid));
                }
            }
        }

        let mut sorted_storage_relids = storage_relids.into_iter().collect::<Vec<_>>();
        sorted_storage_relids.sort_unstable();
        let mut relation_block_count = 0_u64;
        let mut relation_object_tuple_count = 0_u64;
        let mut relation_object_tuple_bytes = 0_u64;
        let mut active_referenced_tuple_count = 0_u64;
        let mut active_referenced_tuple_bytes = 0_u64;
        for storage_relid in sorted_storage_relids {
            let (storage_relation, opened) = if storage_relid == index_relid {
                (index_relation, false)
            } else {
                let relation = unsafe {
                    pg_sys::relation_open(
                        pg_sys::Oid::from(storage_relid),
                        pg_sys::AccessShareLock as pg_sys::LOCKMODE,
                    )
                };
                if relation.is_null() {
                    return Err(format!(
                        "ec_spire failed to open local store relation {storage_relid}"
                    ));
                }
                (relation, true)
            };

            let storage_block_count = unsafe {
                pg_sys::RelationGetNumberOfBlocksInFork(
                    storage_relation,
                    pg_sys::ForkNumber::MAIN_FORKNUM,
                )
            };
            relation_block_count = relation_block_count
                .checked_add(u64::from(storage_block_count))
                .ok_or_else(|| "ec_spire relation block count overflow".to_owned())?;

            let scan_result = unsafe { page::scan_object_tuples(storage_relation, |tid, tuple| {
                relation_object_tuple_count = relation_object_tuple_count
                    .checked_add(1)
                    .ok_or_else(|| "ec_spire relation object tuple count overflow".to_owned())?;
                let tuple_bytes = u64::try_from(tuple.len())
                    .map_err(|_| "ec_spire relation object tuple bytes exceed u64".to_owned())?;
                relation_object_tuple_bytes = relation_object_tuple_bytes
                    .checked_add(tuple_bytes)
                    .ok_or_else(|| "ec_spire relation object tuple bytes overflow".to_owned())?;
                if active_tids.contains(&(storage_relid, tid)) {
                    active_referenced_tuple_count = active_referenced_tuple_count
                        .checked_add(1)
                        .ok_or_else(|| {
                            "ec_spire active referenced tuple count overflow".to_owned()
                        })?;
                    active_referenced_tuple_bytes = active_referenced_tuple_bytes
                        .checked_add(tuple_bytes)
                        .ok_or_else(|| {
                            "ec_spire active referenced tuple bytes overflow".to_owned()
                        })?;
                }
                Ok(())
            }) };
            if opened {
                unsafe {
                    pg_sys::relation_close(
                        storage_relation,
                        pg_sys::AccessShareLock as pg_sys::LOCKMODE,
                    )
                };
            }
            scan_result?;
        }

        let cleanup_candidate_tuple_count =
            relation_object_tuple_count.saturating_sub(active_referenced_tuple_count);
        let cleanup_candidate_tuple_bytes =
            relation_object_tuple_bytes.saturating_sub(active_referenced_tuple_bytes);
        let recommendation = if cleanup_candidate_tuple_count > 0 {
            "old relation object tuples are cleanup candidates once physical reclamation is implemented"
        } else {
            "none"
        };

        Ok(SpireIndexRelationStorageSnapshot {
            active_epoch: root_control.active_epoch,
            relation_block_count,
            relation_object_tuple_count,
            relation_object_tuple_bytes,
            active_referenced_tuple_count,
            active_referenced_tuple_bytes,
            cleanup_candidate_tuple_count,
            cleanup_candidate_tuple_bytes,
            physical_cleanup_supported: false,
            recommendation,
        })
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

pub(crate) unsafe fn index_epoch_snapshot(
    index_relation: pg_sys::Relation,
) -> Vec<SpireIndexEpochSnapshotRow> {
    let result = (|| -> Result<Vec<SpireIndexEpochSnapshotRow>, String> {
        let root_control = unsafe { page::read_root_control_page(index_relation) };
        let mut manifests = Vec::new();
        unsafe {
            page::scan_object_tuples(index_relation, |tid, tuple| {
                if tuple.len() != meta::SpireEpochManifest::encoded_len() {
                    return Ok(());
                }
                if let Ok(manifest) = meta::SpireEpochManifest::decode(tuple) {
                    manifests.push((tid, manifest));
                }
                Ok(())
            })?
        };
        let now_micros = unsafe { pg_sys::GetCurrentTimestamp() };
        epoch_snapshot_rows_from_manifests(root_control, manifests, now_micros)
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

pub(crate) unsafe fn index_placement_snapshot(
    index_relation: pg_sys::Relation,
) -> Vec<SpireIndexPlacementSnapshotRow> {
    let result = (|| -> Result<Vec<SpireIndexPlacementSnapshotRow>, String> {
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
        let object_store = unsafe {
            storage::SpireRelationObjectStoreSet::for_index_relation_and_placements(
                index_relation,
                &placement_directory,
                pg_sys::AccessShareLock as pg_sys::LOCKMODE,
            )?
        };
        let rows = diagnostics::collect_store_placement_diagnostics(&snapshot, &object_store)?
            .into_iter()
            .map(|row| SpireIndexPlacementSnapshotRow {
                active_epoch: row.epoch,
                node_id: row.node_id,
                local_store_id: row.local_store_id,
                store_relid: row.store_relid,
                placement_count: row.placement_count as u64,
                available_placement_count: row.available_placement_count as u64,
                stale_placement_count: row.stale_placement_count as u64,
                unavailable_placement_count: row.unavailable_placement_count as u64,
                skipped_placement_count: row.skipped_placement_count as u64,
                object_count: row.object_count as u64,
                root_object_count: row.root_object_count as u64,
                internal_object_count: row.internal_object_count as u64,
                leaf_object_count: row.leaf_object_count as u64,
                delta_object_count: row.delta_object_count as u64,
                routing_child_count: row.routing_child_count as u64,
                assignment_count: row.assignment_count as u64,
                placement_object_bytes: row.placement_object_bytes,
                available_object_bytes: row.available_object_bytes,
                routing_object_bytes: row.routing_object_bytes,
                leaf_object_bytes: row.leaf_object_bytes,
                delta_object_bytes: row.delta_object_bytes,
            })
            .collect();
        Ok(rows)
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

pub(crate) unsafe fn remote_node_snapshot(
    index_relation: pg_sys::Relation,
) -> Vec<SpireRemoteNodeSnapshotRow> {
    let result = (|| -> Result<Vec<SpireRemoteNodeSnapshotRow>, String> {
        let root_control = unsafe { page::read_root_control_page(index_relation) };
        if root_control.active_epoch == 0 {
            return Ok(Vec::new());
        }

        let (_epoch_manifest, _object_manifest, placement_directory) = unsafe {
            load_relation_epoch_manifests_for_coordinator_fanout(index_relation, root_control)?
        };
        let mut rows_by_node = BTreeMap::<u32, SpireRemoteNodeSnapshotRow>::new();
        let mut local_stores_by_node = BTreeMap::<u32, HashSet<u32>>::new();
        for placement in &placement_directory.entries {
            let row =
                rows_by_node
                    .entry(placement.node_id)
                    .or_insert_with(|| remote_node_snapshot_empty_row(
                        root_control.active_epoch,
                        placement.node_id,
                    ));
            row.placement_count = row.placement_count.checked_add(1).ok_or_else(|| {
                "ec_spire remote node snapshot placement count overflow".to_owned()
            })?;
            match placement.state {
                meta::SpirePlacementState::Available => {
                    row.available_placement_count =
                        row.available_placement_count.checked_add(1).ok_or_else(|| {
                            "ec_spire remote node snapshot available placement count overflow"
                                .to_owned()
                        })?;
                }
                meta::SpirePlacementState::Stale => {
                    row.stale_placement_count =
                        row.stale_placement_count.checked_add(1).ok_or_else(|| {
                            "ec_spire remote node snapshot stale placement count overflow"
                                .to_owned()
                        })?;
                }
                meta::SpirePlacementState::Unavailable => {
                    row.unavailable_placement_count =
                        row.unavailable_placement_count
                            .checked_add(1)
                            .ok_or_else(|| {
                                "ec_spire remote node snapshot unavailable placement count overflow"
                                    .to_owned()
                            })?;
                }
                meta::SpirePlacementState::Skipped => {
                    row.skipped_placement_count =
                        row.skipped_placement_count.checked_add(1).ok_or_else(|| {
                            "ec_spire remote node snapshot skipped placement count overflow"
                                .to_owned()
                        })?;
                }
            }
            local_stores_by_node
                .entry(placement.node_id)
                .or_default()
                .insert(placement.local_store_id);
        }

        for (node_id, stores) in local_stores_by_node {
            let row = rows_by_node
                .get_mut(&node_id)
                .expect("node row should exist for local stores");
            row.local_store_count = u64::try_from(stores.len()).map_err(|_| {
                "ec_spire remote node snapshot local store count exceeds u64".to_owned()
            })?;
        }

        Ok(rows_by_node.into_values().collect())
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

fn remote_node_snapshot_empty_row(active_epoch: u64, node_id: u32) -> SpireRemoteNodeSnapshotRow {
    if node_id == meta::SPIRE_LOCAL_NODE_ID {
        SpireRemoteNodeSnapshotRow {
            active_epoch,
            node_id,
            node_kind: "local",
            descriptor_generation: 0,
            descriptor_state: "active",
            placement_count: 0,
            available_placement_count: 0,
            stale_placement_count: 0,
            unavailable_placement_count: 0,
            skipped_placement_count: 0,
            local_store_count: 0,
            last_seen_at_micros: 0,
            last_served_epoch: active_epoch,
            min_retained_epoch: active_epoch,
            extension_version: env!("CARGO_PKG_VERSION"),
            last_error: "none",
            status: "ready",
            recommendation: "none",
        }
    } else {
        SpireRemoteNodeSnapshotRow {
            active_epoch,
            node_id,
            node_kind: "remote",
            descriptor_generation: 0,
            descriptor_state: "missing",
            placement_count: 0,
            available_placement_count: 0,
            stale_placement_count: 0,
            unavailable_placement_count: 0,
            skipped_placement_count: 0,
            local_store_count: 0,
            last_seen_at_micros: 0,
            last_served_epoch: 0,
            min_retained_epoch: 0,
            extension_version: "unknown",
            last_error: "missing_remote_node_descriptor",
            status: "requires_remote_node_descriptor",
            recommendation: "register remote node descriptor before libpq fanout execution",
        }
    }
}

pub(crate) unsafe fn index_leaf_snapshot(
    index_relation: pg_sys::Relation,
) -> Vec<SpireIndexLeafSnapshotRow> {
    let result = (|| -> Result<Vec<SpireIndexLeafSnapshotRow>, String> {
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
        collect_leaf_snapshot_rows(root_control, &snapshot, &object_store)
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

fn collect_leaf_snapshot_rows(
    root_control: meta::SpireRootControlState,
    snapshot: &meta::SpireValidatedEpochSnapshot<'_>,
    object_store: &impl storage::SpireObjectReader,
) -> Result<Vec<SpireIndexLeafSnapshotRow>, String> {
    let mut rows_by_leaf_pid: HashMap<u64, SpireIndexLeafSnapshotRow> = HashMap::new();

    for manifest_entry in &snapshot.object_manifest().entries {
        let lookup = snapshot.require_lookup(manifest_entry.pid, "leaf snapshot")?;
        let placement = lookup.placement;
        if placement.state != meta::SpirePlacementState::Available {
            continue;
        }
        let header = object_store.read_object_header(placement)?;
        match header.kind {
            storage::SpirePartitionObjectKind::Leaf => {
                let (base_row_count, base_primary_count, base_boundary_replica_count) =
                    count_leaf_snapshot_base_assignment_roles(object_store, placement)?;
                let header_assignment_count = u64::from(header.assignment_count);
                if base_row_count != header_assignment_count {
                    return Err(format!(
                        "ec_spire leaf snapshot base row count {base_row_count} does not match header assignment_count {header_assignment_count} for leaf pid {}",
                        header.pid
                    ));
                }
                let role_count = base_primary_count
                    .checked_add(base_boundary_replica_count)
                    .ok_or_else(|| {
                        "ec_spire leaf snapshot base role count overflow".to_owned()
                    })?;
                if role_count != header_assignment_count {
                    return Err(format!(
                        "ec_spire leaf snapshot base role count {role_count} does not match header assignment_count {header_assignment_count} for leaf pid {}",
                        header.pid
                    ));
                }
                apply_leaf_snapshot_base_row(
                    &mut rows_by_leaf_pid,
                    root_control.active_epoch,
                    &header,
                    placement,
                    base_primary_count,
                    base_boundary_replica_count,
                );
            }
            storage::SpirePartitionObjectKind::Delta => {
                let delta_object = object_store.read_delta_object(placement)?;
                let row = rows_by_leaf_pid
                    .entry(header.parent_pid)
                    .or_insert_with(|| SpireIndexLeafSnapshotRow {
                        active_epoch: root_control.active_epoch,
                        leaf_pid: header.parent_pid,
                        parent_pid: 0,
                        object_version: 0,
                        node_id: placement.node_id,
                        local_store_id: placement.local_store_id,
                        placement_state: "missing_base_leaf",
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
                        maintenance_reason: "missing_base_leaf",
                        leaf_object_bytes: 0,
                        delta_object_bytes: 0,
                    });
                row.delta_object_count =
                    row.delta_object_count.checked_add(1).ok_or_else(|| {
                        "ec_spire leaf snapshot delta object count overflow".to_owned()
                    })?;
                row.delta_object_bytes = row
                    .delta_object_bytes
                    .checked_add(u64::from(placement.object_bytes))
                    .ok_or_else(|| {
                        "ec_spire leaf snapshot delta object bytes overflow".to_owned()
                    })?;
                for assignment in &delta_object.assignments {
                    if storage::is_delete_delta_assignment(assignment) {
                        row.delta_delete_assignment_count = row
                            .delta_delete_assignment_count
                            .checked_add(1)
                            .ok_or_else(|| {
                                "ec_spire leaf snapshot delta delete count overflow".to_owned()
                            })?;
                    } else if assignment.flags & storage::SPIRE_ASSIGNMENT_FLAG_DELTA_INSERT != 0 {
                        row.delta_insert_assignment_count = row
                            .delta_insert_assignment_count
                            .checked_add(1)
                            .ok_or_else(|| {
                                "ec_spire leaf snapshot delta insert count overflow".to_owned()
                            })?;
                        if assignment.flags & storage::SPIRE_ASSIGNMENT_FLAG_BOUNDARY_REPLICA != 0 {
                            row.delta_boundary_replica_insert_assignment_count = row
                                .delta_boundary_replica_insert_assignment_count
                                .checked_add(1)
                                .ok_or_else(|| {
                                    "ec_spire leaf snapshot delta boundary replica insert count overflow"
                                        .to_owned()
                                })?;
                        }
                    }
                }
            }
            storage::SpirePartitionObjectKind::Root
            | storage::SpirePartitionObjectKind::Internal
            | storage::SpirePartitionObjectKind::TopGraph => {}
        }
    }

    let mut rows = rows_by_leaf_pid.into_values().collect::<Vec<_>>();
    for row in &mut rows {
        row.effective_assignment_count = row
            .base_assignment_count
            .saturating_add(row.delta_insert_assignment_count)
            .saturating_sub(row.delta_delete_assignment_count);
        row.effective_boundary_replica_assignment_count = row
            .base_boundary_replica_assignment_count
            .saturating_add(row.delta_boundary_replica_insert_assignment_count);
    }
    let effective_total = rows
        .iter()
        .map(|row| row.effective_assignment_count)
        .try_fold(0_u64, |acc, count| {
            acc.checked_add(count).ok_or_else(|| {
                "ec_spire leaf snapshot effective assignment total overflow".to_owned()
            })
        })?;
    let leaf_count = u64::try_from(rows.len())
        .map_err(|_| "ec_spire leaf snapshot row count exceeds u64".to_owned())?;
    let (split_threshold, merge_threshold) =
        leaf_maintenance_thresholds(effective_total, leaf_count);
    for row in &mut rows {
        row.split_assignment_threshold = split_threshold;
        row.merge_assignment_threshold = merge_threshold;
        let (split, merge, action, reason) = leaf_maintenance_labels(
            row.effective_assignment_count,
            split_threshold,
            merge_threshold,
        );
        row.split_recommended = split;
        row.merge_recommended = merge;
        row.maintenance_action = action;
        row.maintenance_reason = reason;
    }
    rows.sort_by_key(|row| row.leaf_pid);
    Ok(rows)
}

fn count_leaf_snapshot_base_assignment_roles(
    object_store: &impl storage::SpireObjectReader,
    placement: &meta::SpirePlacementEntry,
) -> Result<(u64, u64, u64), String> {
    let assignments = match object_store.read_leaf_object_v2(placement) {
        Ok(leaf_object) => leaf_object.assignment_rows()?,
        Err(v2_error) => object_store
            .read_leaf_object(placement)
            .map_err(|v1_error| {
                format!(
                    "ec_spire leaf snapshot failed to read leaf object: v2 error: {v2_error}; v1 error: {v1_error}"
                )
            })?
            .assignments,
    };
    let row_count = u64::try_from(assignments.len())
        .map_err(|_| "ec_spire leaf snapshot base row count exceeds u64".to_owned())?;
    let mut primary_count = 0_u64;
    let mut boundary_replica_count = 0_u64;
    for assignment in assignments {
        if assignment.flags & storage::SPIRE_ASSIGNMENT_FLAG_PRIMARY != 0 {
            primary_count = primary_count.checked_add(1).ok_or_else(|| {
                "ec_spire leaf snapshot base primary count overflow".to_owned()
            })?;
        }
        if assignment.flags & storage::SPIRE_ASSIGNMENT_FLAG_BOUNDARY_REPLICA != 0 {
            boundary_replica_count = boundary_replica_count.checked_add(1).ok_or_else(|| {
                "ec_spire leaf snapshot base boundary replica count overflow".to_owned()
            })?;
        }
    }
    Ok((row_count, primary_count, boundary_replica_count))
}
