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

pub(crate) unsafe fn active_epoch(index_relation: pg_sys::Relation) -> u64 {
    unsafe { page::read_root_control_page(index_relation).active_epoch }
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
                level_nprobe_resolution(
                    level,
                    active_leaf_count,
                    entry.routing_child_count,
                    relation_options,
                )?;
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
            "run ec_spire_index_epoch_cleanup_run(index_oid) after retention and active-query checks permit cleanup"
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
            physical_cleanup_supported: true,
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

fn collect_epoch_manifests_for_cleanup(
    index_relation: pg_sys::Relation,
) -> Result<Vec<(crate::storage::page::ItemPointer, meta::SpireEpochManifest)>, String> {
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
    Ok(manifests)
}

fn latest_epoch_manifests(
    manifests: &[(crate::storage::page::ItemPointer, meta::SpireEpochManifest)],
) -> Vec<meta::SpireEpochManifest> {
    let mut latest_tid_by_epoch = HashMap::new();
    for (tid, manifest) in manifests {
        latest_tid_by_epoch
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
    manifests
        .iter()
        .filter_map(|(tid, manifest)| {
            latest_tid_by_epoch
                .get(&manifest.epoch)
                .is_some_and(|latest_tid| latest_tid == tid)
                .then_some(*manifest)
        })
        .collect()
}

fn protect_tuple(
    protected: &mut HashSet<(u32, crate::storage::page::ItemPointer)>,
    relid: u32,
    tid: crate::storage::page::ItemPointer,
) {
    if tid != crate::storage::page::ItemPointer::INVALID {
        protected.insert((relid, tid));
    }
}

fn collect_physical_cleanup_candidates(
    index_relation: pg_sys::Relation,
    root_control: meta::SpireRootControlState,
    now_micros: i64,
) -> Result<(
    HashSet<u64>,
    HashSet<(u32, crate::storage::page::ItemPointer)>,
    BTreeMap<u32, Vec<crate::storage::page::ItemPointer>>,
), String> {
    let index_relid: u32 = unsafe { (*index_relation).rd_id }.into();
    let manifests = collect_epoch_manifests_for_cleanup(index_relation)?;
    let latest_manifests = latest_epoch_manifests(&manifests);
    let cleanup_epochs: HashSet<u64> =
        meta::plan_epoch_cleanup(&latest_manifests, root_control.active_epoch, now_micros)?
            .cleanup_epochs
            .into_iter()
            .collect();

    let mut protected = HashSet::<(u32, crate::storage::page::ItemPointer)>::new();
    let mut storage_relids = HashSet::from([index_relid]);
    let mut protected_directories = Vec::new();
    protect_tuple(&mut protected, index_relid, root_control.epoch_manifest_tid);
    protect_tuple(&mut protected, index_relid, root_control.object_manifest_tid);
    protect_tuple(
        &mut protected,
        index_relid,
        root_control.placement_directory_tid,
    );
    protect_tuple(
        &mut protected,
        index_relid,
        root_control.local_store_config_tid,
    );

    unsafe {
        page::scan_object_tuples(index_relation, |tid, tuple| {
            if let Ok(manifest) = meta::SpireEpochManifest::decode(tuple) {
                if !cleanup_epochs.contains(&manifest.epoch) {
                    protect_tuple(&mut protected, index_relid, tid);
                }
                return Ok(());
            }
            if let Ok(manifest) = meta::SpireObjectManifest::decode(tuple) {
                if !cleanup_epochs.contains(&manifest.epoch) {
                    protect_tuple(&mut protected, index_relid, tid);
                }
                return Ok(());
            }
            if let Ok(directory) = meta::SpirePlacementDirectory::decode(tuple) {
                for placement in &directory.entries {
                    storage_relids.insert(placement.store_relid);
                }
                if !cleanup_epochs.contains(&directory.epoch) {
                    protect_tuple(&mut protected, index_relid, tid);
                    protected_directories.push(directory);
                }
                return Ok(());
            }
            if meta::SpireLocalStoreConfig::decode(tuple).is_ok() {
                protect_tuple(&mut protected, index_relid, tid);
            }
            Ok(())
        })?
    };

    for directory in &protected_directories {
        let object_store = unsafe {
            storage::SpireRelationObjectStoreSet::for_index_relation_and_placements(
                index_relation,
                directory,
                pg_sys::AccessShareLock as pg_sys::LOCKMODE,
            )?
        };
        for placement in &directory.entries {
            protect_tuple(&mut protected, placement.store_relid, placement.object_tid);
            for tid in unsafe { object_store.active_object_tuple_locators(placement)? } {
                protect_tuple(&mut protected, placement.store_relid, tid);
            }
        }
    }

    let mut candidates_by_relid = BTreeMap::<u32, Vec<crate::storage::page::ItemPointer>>::new();
    let mut sorted_storage_relids = storage_relids.into_iter().collect::<Vec<_>>();
    sorted_storage_relids.sort_unstable();
    for storage_relid in sorted_storage_relids {
        let (storage_relation, opened) = if storage_relid == index_relid {
            (index_relation, false)
        } else {
            let relation = unsafe {
                pg_sys::relation_open(
                    pg_sys::Oid::from(storage_relid),
                    pg_sys::RowExclusiveLock as pg_sys::LOCKMODE,
                )
            };
            if relation.is_null() {
                return Err(format!(
                    "ec_spire failed to open local store relation {storage_relid}"
                ));
            }
            (relation, true)
        };
        let mut candidates = Vec::new();
        let scan_result = unsafe {
            page::scan_object_tuples(storage_relation, |tid, _tuple| {
                if !protected.contains(&(storage_relid, tid)) {
                    candidates.push(tid);
                }
                Ok(())
            })
        };
        if opened {
            unsafe {
                pg_sys::relation_close(
                    storage_relation,
                    pg_sys::RowExclusiveLock as pg_sys::LOCKMODE,
                )
            };
        }
        scan_result?;
        if !candidates.is_empty() {
            candidates_by_relid.insert(storage_relid, candidates);
        }
    }

    Ok((cleanup_epochs, protected, candidates_by_relid))
}

pub(crate) unsafe fn index_epoch_cleanup_run(
    index_relation: pg_sys::Relation,
) -> SpireIndexEpochCleanupRunResult {
    let _guard = unsafe { lock_publish_relation(index_relation) };
    let result = (|| -> Result<SpireIndexEpochCleanupRunResult, String> {
        let root_control = unsafe { page::read_root_control_page(index_relation) };
        if root_control.active_epoch == 0 {
            return Ok(SpireIndexEpochCleanupRunResult {
                active_epoch: 0,
                cleanup_epoch_count: 0,
                protected_tuple_count: 0,
                removed_tuple_count: 0,
                removed_tuple_bytes: 0,
                physical_cleanup_status: "not_required",
                cleanup_message: "build or insert rows to publish the first SPIRE epoch",
            });
        }

        let now_micros = unsafe { pg_sys::GetCurrentTimestamp() };
        let (cleanup_epochs, protected, candidates_by_relid) =
            collect_physical_cleanup_candidates(index_relation, root_control, now_micros)?;
        let cleanup_epoch_count = u64::try_from(cleanup_epochs.len())
            .map_err(|_| "ec_spire cleanup epoch count exceeds u64".to_owned())?;
        if cleanup_epochs.is_empty() {
            return Ok(SpireIndexEpochCleanupRunResult {
                active_epoch: root_control.active_epoch,
                cleanup_epoch_count: 0,
                protected_tuple_count: u64::try_from(protected.len())
                    .map_err(|_| "ec_spire protected tuple count exceeds u64".to_owned())?,
                removed_tuple_count: 0,
                removed_tuple_bytes: 0,
                physical_cleanup_status: "blocked_by_retention",
                cleanup_message: "no epochs are cleanup-eligible after retention and active-query checks",
            });
        }

        let index_relid: u32 = unsafe { (*index_relation).rd_id }.into();
        let mut removed_tuple_count = 0_u64;
        let mut removed_tuple_bytes = 0_u64;
        for (storage_relid, tids) in candidates_by_relid {
            let (storage_relation, opened) = if storage_relid == index_relid {
                (index_relation, false)
            } else {
                let relation = unsafe {
                    pg_sys::relation_open(
                        pg_sys::Oid::from(storage_relid),
                        pg_sys::RowExclusiveLock as pg_sys::LOCKMODE,
                    )
                };
                if relation.is_null() {
                    return Err(format!(
                        "ec_spire failed to open local store relation {storage_relid}"
                    ));
                }
                (relation, true)
            };
            let delete_result =
                unsafe { page::delete_object_tuples_no_compact(storage_relation, &tids) };
            if opened {
                unsafe {
                    pg_sys::relation_close(
                        storage_relation,
                        pg_sys::RowExclusiveLock as pg_sys::LOCKMODE,
                    )
                };
            }
            let (deleted_count, deleted_bytes) = delete_result?;
            removed_tuple_count = removed_tuple_count
                .checked_add(deleted_count)
                .ok_or_else(|| "ec_spire removed tuple count overflow".to_owned())?;
            removed_tuple_bytes = removed_tuple_bytes
                .checked_add(deleted_bytes)
                .ok_or_else(|| "ec_spire removed tuple bytes overflow".to_owned())?;
        }

        let physical_cleanup_status = if removed_tuple_count > 0 {
            "reclaimed"
        } else {
            "no_candidates"
        };
        Ok(SpireIndexEpochCleanupRunResult {
            active_epoch: root_control.active_epoch,
            cleanup_epoch_count,
            protected_tuple_count: u64::try_from(protected.len())
                .map_err(|_| "ec_spire protected tuple count exceeds u64".to_owned())?,
            removed_tuple_count,
            removed_tuple_bytes,
            physical_cleanup_status,
            cleanup_message: "removed unprotected object tuples with no-compaction line-pointer deletion",
        })
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

        let remote_node_ids = rows_by_node
            .keys()
            .copied()
            .filter(|node_id| *node_id != meta::SPIRE_LOCAL_NODE_ID)
            .collect::<Vec<_>>();
        let descriptors =
            load_remote_node_descriptor_rows(unsafe { (*index_relation).rd_id }, &remote_node_ids)?;
        for descriptor in descriptors {
            if let Some(row) = rows_by_node.get_mut(&descriptor.node_id) {
                apply_remote_node_descriptor(row, descriptor);
            }
        }

        Ok(rows_by_node.into_values().collect())
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

#[derive(Debug, Clone)]
struct SpireRemoteNodeDescriptorRow {
    node_id: u32,
    descriptor_generation: u64,
    descriptor_state: &'static str,
    last_seen_at_micros: i64,
    last_served_epoch: u64,
    min_retained_epoch: u64,
    extension_version: String,
    last_error: String,
}

fn load_remote_node_descriptor_rows(
    index_relid: pg_sys::Oid,
    remote_node_ids: &[u32],
) -> Result<Vec<SpireRemoteNodeDescriptorRow>, String> {
    if remote_node_ids.is_empty() {
        return Ok(Vec::new());
    }

    let node_id_list = remote_node_ids
        .iter()
        .map(u32::to_string)
        .collect::<Vec<_>>()
        .join(",");
    let sql = format!(
        "SELECT node_id::int4, \
                descriptor_generation::bigint, \
                descriptor_state, \
                (EXTRACT(EPOCH FROM last_seen_at) * 1000000)::bigint AS last_seen_at_micros, \
                last_served_epoch::bigint, \
                min_retained_epoch::bigint, \
                extension_version, \
                last_error \
           FROM ec_spire_remote_node_descriptor \
          WHERE coordinator_index_oid = '{}'::oid \
            AND node_id = ANY (ARRAY[{}]::integer[])",
        u32::from(index_relid),
        node_id_list
    );

    Spi::connect(|client| {
        client
            .select(sql.as_str(), None, &[])
            .map_err(|e| format!("ec_spire remote node descriptor catalog read failed: {e}"))?
            .map(|row| {
                let node_id = row["node_id"]
                    .value::<i32>()
                    .map_err(|e| format!("ec_spire remote node descriptor node_id decode failed: {e}"))?
                    .ok_or_else(|| "ec_spire remote node descriptor node_id is null".to_owned())
                    .and_then(|value| {
                        u32::try_from(value)
                            .map_err(|_| "ec_spire remote node descriptor node_id is negative".to_owned())
                    })?;
                let descriptor_generation = row["descriptor_generation"]
                    .value::<i64>()
                    .map_err(|e| format!("ec_spire remote node descriptor generation decode failed: {e}"))?
                    .ok_or_else(|| "ec_spire remote node descriptor generation is null".to_owned())
                    .and_then(|value| {
                        u64::try_from(value)
                            .map_err(|_| "ec_spire remote node descriptor generation is negative".to_owned())
                    })?;
                let descriptor_state_value = row["descriptor_state"]
                    .value::<String>()
                    .map_err(|e| format!("ec_spire remote node descriptor state decode failed: {e}"))?
                    .ok_or_else(|| "ec_spire remote node descriptor state is null".to_owned())?;
                let descriptor_state =
                    remote_node_descriptor_state_name(&descriptor_state_value)?;
                let last_seen_at_micros = row["last_seen_at_micros"]
                    .value::<i64>()
                    .map_err(|e| format!("ec_spire remote node descriptor last_seen_at decode failed: {e}"))?
                    .ok_or_else(|| "ec_spire remote node descriptor last_seen_at is null".to_owned())?;
                let last_served_epoch = row["last_served_epoch"]
                    .value::<i64>()
                    .map_err(|e| format!("ec_spire remote node descriptor served epoch decode failed: {e}"))?
                    .ok_or_else(|| "ec_spire remote node descriptor served epoch is null".to_owned())
                    .and_then(|value| {
                        u64::try_from(value)
                            .map_err(|_| "ec_spire remote node descriptor served epoch is negative".to_owned())
                    })?;
                let min_retained_epoch = row["min_retained_epoch"]
                    .value::<i64>()
                    .map_err(|e| format!("ec_spire remote node descriptor retained epoch decode failed: {e}"))?
                    .ok_or_else(|| "ec_spire remote node descriptor retained epoch is null".to_owned())
                    .and_then(|value| {
                        u64::try_from(value)
                            .map_err(|_| "ec_spire remote node descriptor retained epoch is negative".to_owned())
                    })?;
                let extension_version = row["extension_version"]
                    .value::<String>()
                    .map_err(|e| format!("ec_spire remote node descriptor extension version decode failed: {e}"))?
                    .ok_or_else(|| "ec_spire remote node descriptor extension version is null".to_owned())?;
                let last_error = row["last_error"]
                    .value::<String>()
                    .map_err(|e| format!("ec_spire remote node descriptor last_error decode failed: {e}"))?
                    .ok_or_else(|| "ec_spire remote node descriptor last_error is null".to_owned())?;

                Ok(SpireRemoteNodeDescriptorRow {
                    node_id,
                    descriptor_generation,
                    descriptor_state,
                    last_seen_at_micros,
                    last_served_epoch,
                    min_retained_epoch,
                    extension_version,
                    last_error,
                })
            })
            .collect::<Result<Vec<_>, String>>()
    })
}

fn remote_node_descriptor_state_name(state: &str) -> Result<&'static str, String> {
    match state {
        SPIRE_REMOTE_DESCRIPTOR_STATE_ACTIVE => Ok(SPIRE_REMOTE_DESCRIPTOR_STATE_ACTIVE),
        SPIRE_REMOTE_DESCRIPTOR_STATE_DRAINING => Ok(SPIRE_REMOTE_DESCRIPTOR_STATE_DRAINING),
        SPIRE_REMOTE_DESCRIPTOR_STATE_DISABLED => Ok(SPIRE_REMOTE_DESCRIPTOR_STATE_DISABLED),
        SPIRE_REMOTE_DESCRIPTOR_STATE_FAILED => Ok(SPIRE_REMOTE_DESCRIPTOR_STATE_FAILED),
        other => Err(format!(
            "ec_spire remote node descriptor has unsupported descriptor_state '{other}'"
        )),
    }
}

fn apply_remote_node_descriptor(
    row: &mut SpireRemoteNodeSnapshotRow,
    descriptor: SpireRemoteNodeDescriptorRow,
) {
    let (status, recommendation) = remote_node_descriptor_status(descriptor.descriptor_state);
    row.descriptor_generation = descriptor.descriptor_generation;
    row.descriptor_state = descriptor.descriptor_state;
    row.last_seen_at_micros = descriptor.last_seen_at_micros;
    row.last_served_epoch = descriptor.last_served_epoch;
    row.min_retained_epoch = descriptor.min_retained_epoch;
    row.extension_version = descriptor.extension_version;
    row.last_error = descriptor.last_error;
    row.status = status;
    row.recommendation = recommendation;
}

fn remote_node_descriptor_status(
    descriptor_state: &'static str,
) -> (&'static str, &'static str) {
    match descriptor_state {
        SPIRE_REMOTE_DESCRIPTOR_STATE_ACTIVE | SPIRE_REMOTE_DESCRIPTOR_STATE_DRAINING => {
            (SPIRE_REMOTE_STATUS_READY, SPIRE_REMOTE_NONE)
        }
        SPIRE_REMOTE_DESCRIPTOR_STATE_DISABLED => (
            "disabled_remote_node",
            "enable or replace remote node descriptor before libpq fanout execution",
        ),
        SPIRE_REMOTE_DESCRIPTOR_STATE_FAILED => (
            "failed_remote_node",
            "repair remote node descriptor before libpq fanout execution",
        ),
        _ => (
            SPIRE_REMOTE_STATUS_REQUIRES_DESCRIPTOR,
            "register remote node descriptor before libpq fanout execution",
        ),
    }
}

pub(crate) fn remote_node_descriptor_contract_rows(
) -> Vec<SpireRemoteNodeDescriptorContractRow> {
    vec![
        SpireRemoteNodeDescriptorContractRow {
            field_ordinal: 1,
            field_name: "coordinator_index_oid",
            pg_type: "oid",
            semantic_role: "coordinator_index_identity",
            required: true,
            validator: "must_equal_local_index_oid",
        },
        SpireRemoteNodeDescriptorContractRow {
            field_ordinal: 2,
            field_name: "node_id",
            pg_type: "integer",
            semantic_role: "coordinator_scoped_node",
            required: true,
            validator: "must_be_nonzero_remote_node",
        },
        SpireRemoteNodeDescriptorContractRow {
            field_ordinal: 3,
            field_name: "generation",
            pg_type: "bigint",
            semantic_role: "membership_generation",
            required: true,
            validator: "must_match_epoch_node_generation",
        },
        SpireRemoteNodeDescriptorContractRow {
            field_ordinal: 4,
            field_name: "conninfo_secret_name",
            pg_type: "text",
            semantic_role: "indirect_connection_secret",
            required: true,
            validator: "must_be_nonempty_noncolliding_secret_reference",
        },
        SpireRemoteNodeDescriptorContractRow {
            field_ordinal: 5,
            field_name: "remote_index_identity",
            pg_type: "bytea",
            semantic_role: "remote_index_identity",
            required: true,
            validator: "must_match_remote_capability_echo",
        },
        SpireRemoteNodeDescriptorContractRow {
            field_ordinal: 6,
            field_name: "remote_index_regclass",
            pg_type: "text",
            semantic_role: "remote_index_locator",
            required: true,
            // Registration resolves this in the remote node catalog, not in the coordinator.
            validator: "must_resolve_on_remote_node",
        },
        SpireRemoteNodeDescriptorContractRow {
            field_ordinal: 7,
            field_name: "state",
            pg_type: "text",
            semantic_role: "remote_node_policy_state",
            required: true,
            validator: "must_be_active_or_draining_for_reads",
        },
        SpireRemoteNodeDescriptorContractRow {
            field_ordinal: 8,
            field_name: "last_seen_at",
            pg_type: "timestamptz",
            semantic_role: "health_check_timestamp",
            required: false,
            validator: "diagnostic_only",
        },
        SpireRemoteNodeDescriptorContractRow {
            field_ordinal: 9,
            field_name: "last_served_epoch",
            pg_type: "bigint",
            semantic_role: "max_served_epoch",
            required: true,
            validator: "must_cover_requested_epoch",
        },
        SpireRemoteNodeDescriptorContractRow {
            field_ordinal: 10,
            field_name: "min_retained_epoch",
            pg_type: "bigint",
            semantic_role: "retention_floor",
            required: true,
            validator: "must_not_exceed_requested_epoch",
        },
        SpireRemoteNodeDescriptorContractRow {
            field_ordinal: 11,
            field_name: "extension_version",
            pg_type: "text",
            semantic_role: "remote_extension_version",
            required: true,
            validator: "must_match_required_extension_version",
        },
        SpireRemoteNodeDescriptorContractRow {
            field_ordinal: 12,
            field_name: "last_error",
            pg_type: "text",
            semantic_role: "last_health_or_search_error",
            required: false,
            validator: "diagnostic_only",
        },
    ]
}

pub(crate) fn remote_node_descriptor_state_contract_rows(
) -> Vec<SpireRemoteNodeDescriptorStateContractRow> {
    vec![
        SpireRemoteNodeDescriptorStateContractRow {
            state_ordinal: 1,
            descriptor_state: SPIRE_REMOTE_DESCRIPTOR_STATE_ACTIVE,
            state_source: "catalog",
            read_eligible: true,
            snapshot_status: SPIRE_REMOTE_STATUS_READY,
            recommendation: SPIRE_REMOTE_NONE,
        },
        SpireRemoteNodeDescriptorStateContractRow {
            state_ordinal: 2,
            descriptor_state: SPIRE_REMOTE_DESCRIPTOR_STATE_DRAINING,
            state_source: "catalog",
            read_eligible: true,
            snapshot_status: SPIRE_REMOTE_STATUS_READY,
            recommendation: SPIRE_REMOTE_NONE,
        },
        SpireRemoteNodeDescriptorStateContractRow {
            state_ordinal: 3,
            descriptor_state: SPIRE_REMOTE_DESCRIPTOR_STATE_DISABLED,
            state_source: "catalog",
            read_eligible: false,
            snapshot_status: "disabled_remote_node",
            recommendation: "enable or replace remote node descriptor before libpq fanout execution",
        },
        SpireRemoteNodeDescriptorStateContractRow {
            state_ordinal: 4,
            descriptor_state: SPIRE_REMOTE_DESCRIPTOR_STATE_FAILED,
            state_source: "catalog",
            read_eligible: false,
            snapshot_status: "failed_remote_node",
            recommendation: "repair remote node descriptor before libpq fanout execution",
        },
        SpireRemoteNodeDescriptorStateContractRow {
            state_ordinal: 5,
            descriptor_state: SPIRE_REMOTE_DESCRIPTOR_STATE_MISSING,
            state_source: "synthetic",
            read_eligible: false,
            snapshot_status: SPIRE_REMOTE_STATUS_REQUIRES_DESCRIPTOR,
            recommendation: "register remote node descriptor before libpq fanout execution",
        },
    ]
}

pub(crate) fn remote_node_descriptor_catalog_state_is_supported(state: &str) -> bool {
    remote_node_descriptor_state_contract_rows()
        .into_iter()
        .any(|row| row.state_source == "catalog" && row.descriptor_state == state)
}

pub(crate) fn remote_node_descriptor_registration_contract_rows(
) -> Vec<SpireRemoteNodeDescriptorRegistrationContractRow> {
    vec![
        SpireRemoteNodeDescriptorRegistrationContractRow {
            step_ordinal: 1,
            step_name: "bind_coordinator_index",
            input_field: "coordinator_index_oid",
            semantic_role: "coordinator_index_identity",
            validator: "must_equal_local_index_oid",
            persistence_action: "bind_descriptor_to_index",
            failure_status: "invalid_descriptor",
        },
        SpireRemoteNodeDescriptorRegistrationContractRow {
            step_ordinal: 2,
            step_name: "validate_remote_node",
            input_field: "node_id",
            semantic_role: "coordinator_scoped_node",
            validator: "must_be_nonzero_remote_node",
            persistence_action: "upsert_node_descriptor_key",
            failure_status: "invalid_descriptor",
        },
        SpireRemoteNodeDescriptorRegistrationContractRow {
            step_ordinal: 3,
            step_name: "record_secret_reference",
            input_field: "conninfo_secret_name",
            semantic_role: "indirect_connection_secret",
            validator: "must_be_nonempty_noncolliding_secret_reference",
            persistence_action: "persist_secret_reference_only",
            failure_status: "invalid_descriptor",
        },
        SpireRemoteNodeDescriptorRegistrationContractRow {
            step_ordinal: 4,
            step_name: "resolve_remote_index",
            input_field: "remote_index_regclass",
            semantic_role: "remote_index_locator",
            validator: "must_resolve_on_remote_node",
            persistence_action: "persist_remote_index_locator",
            failure_status: "remote_capability_failed",
        },
        SpireRemoteNodeDescriptorRegistrationContractRow {
            step_ordinal: 5,
            step_name: "verify_remote_identity",
            input_field: "remote_index_identity",
            semantic_role: "remote_index_identity",
            validator: "must_match_remote_capability_echo",
            persistence_action: "persist_remote_identity",
            failure_status: "remote_capability_failed",
        },
        SpireRemoteNodeDescriptorRegistrationContractRow {
            step_ordinal: 6,
            step_name: "verify_epoch_window",
            input_field: "last_served_epoch,min_retained_epoch",
            semantic_role: "served_epoch_window",
            validator: "must_cover_active_epoch_and_retention_floor",
            persistence_action: "persist_served_retained_epochs",
            failure_status: "remote_epoch_not_served",
        },
        SpireRemoteNodeDescriptorRegistrationContractRow {
            step_ordinal: 7,
            step_name: "verify_extension_version",
            input_field: "extension_version",
            semantic_role: "remote_extension_version",
            validator: "must_match_required_extension_version",
            persistence_action: "persist_capability_version",
            failure_status: "incompatible_extension_version",
        },
        SpireRemoteNodeDescriptorRegistrationContractRow {
            step_ordinal: 8,
            step_name: "apply_policy_state",
            input_field: "state",
            semantic_role: "remote_node_policy_state",
            validator: "must_be_active_or_draining_for_reads",
            persistence_action: "persist_descriptor_state",
            failure_status: "disabled_remote_node",
        },
        SpireRemoteNodeDescriptorRegistrationContractRow {
            step_ordinal: 9,
            step_name: "publish_generation",
            input_field: "generation",
            semantic_role: "membership_generation",
            validator: "must_advance_descriptor_generation",
            persistence_action: "atomically_replace_descriptor",
            failure_status: "stale_descriptor_generation",
        },
    ]
}

pub(crate) unsafe fn remote_node_descriptor_readiness(
    index_relation: pg_sys::Relation,
) -> Vec<SpireRemoteNodeDescriptorReadinessRow> {
    let mut rows = Vec::new();
    let contract_rows = remote_node_descriptor_contract_rows();
    for node in unsafe { remote_node_snapshot(index_relation) } {
        if node.node_id == meta::SPIRE_LOCAL_NODE_ID {
            continue;
        }
        for contract in &contract_rows {
            let (status, recommendation) =
                if node.descriptor_state == SPIRE_REMOTE_DESCRIPTOR_STATE_MISSING {
                    if contract.required {
                        (
                            SPIRE_REMOTE_STATUS_MISSING_DESCRIPTOR,
                            "register remote node descriptor before libpq fanout execution",
                        )
                    } else {
                        (
                            SPIRE_REMOTE_STATUS_OPTIONAL_DESCRIPTOR_MISSING,
                            SPIRE_REMOTE_NONE,
                        )
                    }
                } else if node.status == SPIRE_REMOTE_STATUS_REQUIRES_DESCRIPTOR {
                    (
                        SPIRE_REMOTE_STATUS_REQUIRES_DESCRIPTOR,
                        "repair remote node descriptor before libpq fanout execution",
                    )
                } else {
                    (SPIRE_REMOTE_STATUS_READY, SPIRE_REMOTE_NONE)
                };
            rows.push(SpireRemoteNodeDescriptorReadinessRow {
                active_epoch: node.active_epoch,
                node_id: node.node_id,
                field_ordinal: contract.field_ordinal,
                field_name: contract.field_name,
                semantic_role: contract.semantic_role,
                required: contract.required,
                validator: contract.validator,
                descriptor_state: node.descriptor_state,
                status,
                recommendation,
            });
        }
    }
    rows
}

pub(crate) unsafe fn remote_node_descriptor_readiness_summary(
    index_relation: pg_sys::Relation,
) -> SpireRemoteNodeDescriptorReadinessSummaryRow {
    let root_control = unsafe { page::read_root_control_page(index_relation) };
    let mut summary = SpireRemoteNodeDescriptorReadinessSummaryRow {
        active_epoch: root_control.active_epoch,
        remote_node_count: 0,
        descriptor_field_count: 0,
        required_field_count: 0,
        ready_field_count: 0,
        blocked_field_count: 0,
        missing_required_field_count: 0,
        status: "empty",
        recommendation: "build index before remote descriptor readiness check",
    };
    if root_control.active_epoch == 0 {
        return summary;
    }

    let mut seen_nodes = HashSet::new();
    for row in unsafe { remote_node_descriptor_readiness(index_relation) } {
        if seen_nodes.insert(row.node_id) {
            summary.remote_node_count =
                summary.remote_node_count.checked_add(1).unwrap_or_else(|| {
                    pgrx::error!("ec_spire remote node descriptor readiness node count overflow")
                });
        }
        summary.descriptor_field_count = summary
            .descriptor_field_count
            .checked_add(1)
            .unwrap_or_else(|| {
                pgrx::error!("ec_spire remote node descriptor readiness field count overflow")
            });
        if row.required {
            summary.required_field_count = summary
                .required_field_count
                .checked_add(1)
                .unwrap_or_else(|| {
                    pgrx::error!(
                        "ec_spire remote node descriptor readiness required field count overflow"
                    )
                });
        }
        if row.status == SPIRE_REMOTE_STATUS_READY {
            summary.ready_field_count =
                summary.ready_field_count.checked_add(1).unwrap_or_else(|| {
                    pgrx::error!("ec_spire remote node descriptor readiness ready count overflow")
                });
        } else if row.required {
            summary.blocked_field_count =
                summary.blocked_field_count.checked_add(1).unwrap_or_else(|| {
                    pgrx::error!("ec_spire remote node descriptor readiness blocked count overflow")
                });
            if row.status == SPIRE_REMOTE_STATUS_MISSING_DESCRIPTOR {
                summary.missing_required_field_count = summary
                    .missing_required_field_count
                    .checked_add(1)
                    .unwrap_or_else(|| {
                        pgrx::error!(
                            "ec_spire remote node descriptor readiness missing required count overflow"
                        )
                    });
            }
        }
    }

    if summary.blocked_field_count == 0 {
        summary.status = SPIRE_REMOTE_STATUS_READY;
        summary.recommendation = SPIRE_REMOTE_NONE;
    } else {
        summary.status = SPIRE_REMOTE_STATUS_REQUIRES_DESCRIPTOR;
        summary.recommendation = "register remote node descriptors before libpq fanout execution";
    }
    summary
}

pub(crate) unsafe fn remote_node_capability_plan(
    index_relation: pg_sys::Relation,
) -> Vec<SpireRemoteNodeCapabilityPlanRow> {
    unsafe { remote_node_snapshot(index_relation) }
        .into_iter()
        .map(remote_node_capability_plan_row)
        .collect()
}

pub(crate) unsafe fn remote_node_capability_summary(
    index_relation: pg_sys::Relation,
) -> SpireRemoteNodeCapabilitySummaryRow {
    let root_control = unsafe { page::read_root_control_page(index_relation) };
    let mut summary = SpireRemoteNodeCapabilitySummaryRow {
        active_epoch: root_control.active_epoch,
        node_count: 0,
        local_node_count: 0,
        remote_node_count: 0,
        ready_node_count: 0,
        blocked_node_count: 0,
        missing_descriptor_node_count: 0,
        required_candidate_format: "local",
        required_extension_version: env!("CARGO_PKG_VERSION"),
        status: "empty",
        recommendation: "build index before remote capability summary check",
    };
    if root_control.active_epoch == 0 {
        return summary;
    }

    for row in unsafe { remote_node_capability_plan(index_relation) } {
        summary.node_count = summary.node_count.checked_add(1).unwrap_or_else(|| {
            pgrx::error!("ec_spire remote node capability summary node count overflow")
        });
        if row.node_id == meta::SPIRE_LOCAL_NODE_ID {
            summary.local_node_count =
                summary.local_node_count.checked_add(1).unwrap_or_else(|| {
                    pgrx::error!(
                        "ec_spire remote node capability summary local node count overflow"
                    )
                });
        } else {
            summary.remote_node_count =
                summary.remote_node_count.checked_add(1).unwrap_or_else(|| {
                    pgrx::error!(
                        "ec_spire remote node capability summary remote node count overflow"
                    )
                });
            summary.required_candidate_format = SPIRE_REMOTE_CANDIDATE_FORMAT_V1;
        }
        if row.status == SPIRE_REMOTE_STATUS_READY {
            summary.ready_node_count =
                summary.ready_node_count.checked_add(1).unwrap_or_else(|| {
                    pgrx::error!(
                        "ec_spire remote node capability summary ready node count overflow"
                    )
                });
        } else {
            summary.blocked_node_count =
                summary.blocked_node_count.checked_add(1).unwrap_or_else(|| {
                    pgrx::error!(
                        "ec_spire remote node capability summary blocked node count overflow"
                    )
                });
        }
        if row.descriptor_state == SPIRE_REMOTE_DESCRIPTOR_STATE_MISSING
            || row.status == SPIRE_REMOTE_STATUS_REQUIRES_DESCRIPTOR
        {
            summary.missing_descriptor_node_count =
                summary
                    .missing_descriptor_node_count
                    .checked_add(1)
                    .unwrap_or_else(|| {
                        pgrx::error!(
                            "ec_spire remote node capability summary missing descriptor count overflow"
                        )
                    });
        }
    }

    if summary.blocked_node_count == 0 {
        summary.status = SPIRE_REMOTE_STATUS_READY;
        summary.recommendation = SPIRE_REMOTE_NONE;
    } else {
        summary.status = SPIRE_REMOTE_STATUS_REQUIRES_DESCRIPTOR;
        summary.recommendation = "register remote node descriptors before capability checks";
    }
    summary
}

pub(crate) unsafe fn remote_epoch_publish_plan(
    index_relation: pg_sys::Relation,
) -> Vec<SpireRemoteEpochPublishPlanRow> {
    unsafe { remote_node_snapshot(index_relation) }
        .into_iter()
        .filter(|node| node.node_id != meta::SPIRE_LOCAL_NODE_ID)
        .map(remote_epoch_publish_plan_row)
        .collect()
}

pub(crate) unsafe fn remote_epoch_publish_readiness(
    index_relation: pg_sys::Relation,
) -> SpireRemoteEpochPublishReadinessRow {
    let root_control = unsafe { page::read_root_control_page(index_relation) };
    let mut summary = SpireRemoteEpochPublishReadinessRow {
        active_epoch: root_control.active_epoch,
        remote_node_count: 0,
        remote_placement_count: 0,
        remote_available_placement_count: 0,
        remote_unavailable_placement_count: 0,
        remote_skipped_placement_count: 0,
        ready_remote_node_count: 0,
        blocked_remote_node_count: 0,
        missing_descriptor_node_count: 0,
        status: "empty",
        recommendation: "build index before remote epoch publish readiness check",
    };
    if root_control.active_epoch == 0 {
        return summary;
    }

    for row in unsafe { remote_epoch_publish_plan(index_relation) } {
        summary.remote_node_count =
            summary.remote_node_count.checked_add(1).unwrap_or_else(|| {
                pgrx::error!("ec_spire remote epoch publish readiness node count overflow")
            });
        summary.remote_placement_count = summary
            .remote_placement_count
            .checked_add(row.placement_count)
            .unwrap_or_else(|| {
                pgrx::error!("ec_spire remote epoch publish readiness placement count overflow")
            });
        summary.remote_available_placement_count = summary
            .remote_available_placement_count
            .checked_add(row.available_placement_count)
            .unwrap_or_else(|| {
                pgrx::error!(
                    "ec_spire remote epoch publish readiness available placement count overflow"
                )
            });
        summary.remote_unavailable_placement_count = summary
            .remote_unavailable_placement_count
            .checked_add(row.unavailable_placement_count)
            .unwrap_or_else(|| {
                pgrx::error!(
                    "ec_spire remote epoch publish readiness unavailable placement count overflow"
                )
            });
        summary.remote_skipped_placement_count = summary
            .remote_skipped_placement_count
            .checked_add(row.skipped_placement_count)
            .unwrap_or_else(|| {
                pgrx::error!(
                    "ec_spire remote epoch publish readiness skipped placement count overflow"
                )
            });
        if row.status == SPIRE_REMOTE_STATUS_READY {
            summary.ready_remote_node_count =
                summary
                    .ready_remote_node_count
                    .checked_add(1)
                    .unwrap_or_else(|| {
                        pgrx::error!(
                            "ec_spire remote epoch publish readiness ready node count overflow"
                        )
                    });
        } else {
            summary.blocked_remote_node_count =
                summary
                    .blocked_remote_node_count
                    .checked_add(1)
                    .unwrap_or_else(|| {
                        pgrx::error!(
                            "ec_spire remote epoch publish readiness blocked node count overflow"
                        )
                    });
        }
        if row.descriptor_state == SPIRE_REMOTE_DESCRIPTOR_STATE_MISSING
            || row.status == SPIRE_REMOTE_STATUS_REQUIRES_DESCRIPTOR
        {
            summary.missing_descriptor_node_count =
                summary
                    .missing_descriptor_node_count
                    .checked_add(1)
                    .unwrap_or_else(|| {
                        pgrx::error!(
                            "ec_spire remote epoch publish readiness missing descriptor count overflow"
                        )
                    });
        }
    }

    if summary.blocked_remote_node_count == 0 {
        summary.status = SPIRE_REMOTE_STATUS_READY;
        summary.recommendation = SPIRE_REMOTE_NONE;
    } else if summary.missing_descriptor_node_count > 0 {
        summary.status = SPIRE_REMOTE_STATUS_REQUIRES_DESCRIPTOR;
        summary.recommendation =
            "register remote node descriptors before publishing distributed epochs";
    } else {
        summary.status = "remote_epoch_window";
        summary.recommendation =
            "refresh remote node served epoch window before publishing distributed epochs";
    }
    summary
}

pub(crate) unsafe fn remote_epoch_publish_gate_summary(
    index_relation: pg_sys::Relation,
) -> SpireRemoteEpochPublishGateSummaryRow {
    let readiness = unsafe { remote_epoch_publish_readiness(index_relation) };
    let (publish_scope, publish_decision, next_blocker, recommendation) =
        if readiness.active_epoch == 0 {
            (
                "empty",
                "build_required",
                "build_index",
                readiness.recommendation,
            )
        } else if readiness.remote_node_count == 0 {
            (
                "local_only",
                "publish_local_epoch",
                SPIRE_REMOTE_NONE,
                SPIRE_REMOTE_NONE,
            )
        } else if readiness.blocked_remote_node_count > 0 {
            let next_blocker = if readiness.missing_descriptor_node_count > 0 {
                "remote_node_descriptor"
            } else {
                "remote_epoch_window"
            };
            (
                "distributed",
                "block_publish",
                next_blocker,
                readiness.recommendation,
            )
        } else {
            (
                "distributed",
                "publish_distributed_epoch",
                SPIRE_REMOTE_NONE,
                SPIRE_REMOTE_NONE,
            )
        };

    SpireRemoteEpochPublishGateSummaryRow {
        active_epoch: readiness.active_epoch,
        publish_scope,
        publish_decision,
        remote_node_count: readiness.remote_node_count,
        remote_placement_count: readiness.remote_placement_count,
        ready_remote_node_count: readiness.ready_remote_node_count,
        blocked_remote_node_count: readiness.blocked_remote_node_count,
        missing_descriptor_node_count: readiness.missing_descriptor_node_count,
        policy_contract: "ec_spire_remote_degradation_policy_contract",
        status: readiness.status,
        next_blocker,
        recommendation,
    }
}

pub(crate) unsafe fn remote_epoch_manifest_plan(
    index_relation: pg_sys::Relation,
) -> Vec<SpireRemoteEpochManifestPlanRow> {
    unsafe { remote_epoch_publish_plan(index_relation) }
        .into_iter()
        .map(remote_epoch_manifest_plan_row)
        .collect()
}

pub(crate) unsafe fn remote_epoch_manifest_summary(
    index_relation: pg_sys::Relation,
) -> SpireRemoteEpochManifestSummaryRow {
    let gate = unsafe { remote_epoch_publish_gate_summary(index_relation) };
    let (manifest_decision, recommendation) = if gate.active_epoch == 0 {
        ("build_required", gate.recommendation)
    } else if gate.publish_decision == "block_publish" {
        ("block_manifest", gate.recommendation)
    } else if gate.publish_scope == "local_only" {
        ("emit_local_epoch_manifest", SPIRE_REMOTE_NONE)
    } else {
        ("emit_distributed_epoch_manifest", SPIRE_REMOTE_NONE)
    };

    SpireRemoteEpochManifestSummaryRow {
        active_epoch: gate.active_epoch,
        manifest_scope: gate.publish_scope,
        manifest_decision,
        manifest_entry_count: gate.remote_node_count,
        included_remote_node_count: gate.ready_remote_node_count,
        blocked_remote_node_count: gate.blocked_remote_node_count,
        remote_placement_count: gate.remote_placement_count,
        publish_decision: gate.publish_decision,
        next_blocker: gate.next_blocker,
        status: gate.status,
        recommendation,
    }
}

pub(crate) fn remote_degradation_policy_contract_rows(
) -> Vec<SpireRemoteDegradationPolicyContractRow> {
    vec![
        SpireRemoteDegradationPolicyContractRow {
            consistency_mode: "strict",
            placement_state: "available",
            search_action: "dispatch",
            publish_action: "publish",
            status: SPIRE_REMOTE_STATUS_READY,
            recommendation: SPIRE_REMOTE_NONE,
        },
        SpireRemoteDegradationPolicyContractRow {
            consistency_mode: "strict",
            placement_state: "stale",
            search_action: "fail_closed",
            publish_action: "block_publish",
            status: "requires_fresh_epoch",
            recommendation: "refresh stale placement before strict search or epoch publish",
        },
        SpireRemoteDegradationPolicyContractRow {
            consistency_mode: "strict",
            placement_state: "unavailable",
            search_action: "fail_closed",
            publish_action: "block_publish",
            status: "requires_available_placement",
            recommendation: "restore unavailable placement before strict search or epoch publish",
        },
        SpireRemoteDegradationPolicyContractRow {
            consistency_mode: "strict",
            placement_state: "skipped",
            search_action: "fail_closed",
            publish_action: "block_publish",
            status: "requires_available_placement",
            recommendation: "remove skipped placement from strict epoch manifests",
        },
        SpireRemoteDegradationPolicyContractRow {
            consistency_mode: "degraded",
            placement_state: "available",
            search_action: "dispatch",
            publish_action: "publish",
            status: SPIRE_REMOTE_STATUS_READY,
            recommendation: SPIRE_REMOTE_NONE,
        },
        SpireRemoteDegradationPolicyContractRow {
            consistency_mode: "degraded",
            placement_state: "stale",
            search_action: "fail_closed",
            publish_action: "block_publish",
            status: "requires_fresh_epoch",
            recommendation: "do not serve stale placements in degraded mode",
        },
        SpireRemoteDegradationPolicyContractRow {
            consistency_mode: "degraded",
            placement_state: "unavailable",
            search_action: "skip_and_report",
            publish_action: "publish_degraded",
            status: SPIRE_REMOTE_STATUS_DEGRADED_SKIPPED,
            recommendation: "report skipped unavailable placement in degraded search results",
        },
        SpireRemoteDegradationPolicyContractRow {
            consistency_mode: "degraded",
            placement_state: "skipped",
            search_action: "skip_and_report",
            publish_action: "publish_degraded",
            status: SPIRE_REMOTE_STATUS_DEGRADED_SKIPPED,
            recommendation: "report skipped placement in degraded search results",
        },
    ]
}

pub(crate) fn remote_epoch_manifest_publication_contract_rows(
) -> Vec<SpireRemoteEpochManifestPublicationContractRow> {
    vec![
        SpireRemoteEpochManifestPublicationContractRow {
            step_ordinal: 1,
            prerequisite: "remote_epoch_publish_gate",
            publication_action: "block_manifest_publication",
            required_status: SPIRE_REMOTE_STATUS_READY,
            validator: "must_allow_distributed_epoch_publish",
            failure_status: "remote_epoch_publish_gate_blocked",
        },
        SpireRemoteEpochManifestPublicationContractRow {
            step_ordinal: 2,
            prerequisite: "remote_epoch_manifest_catalog_summary",
            publication_action: "persist_remote_epoch_manifest",
            required_status: "ready",
            validator: "must_have_persisted_current_manifest_header",
            failure_status: "requires_remote_epoch_manifest_persistence",
        },
        SpireRemoteEpochManifestPublicationContractRow {
            step_ordinal: 3,
            prerequisite: "remote_epoch_manifest_entry_catalog",
            publication_action: "refresh_remote_epoch_manifest",
            required_status: "ready",
            validator: "persisted_entries_must_match_current_manifest_plan",
            failure_status: "stale_remote_epoch_manifest",
        },
        SpireRemoteEpochManifestPublicationContractRow {
            step_ordinal: 4,
            prerequisite: "remote_epoch_manifest_publication_plan",
            publication_action: "publish_remote_epoch_manifest",
            required_status: "ready",
            validator: "all_publication_rows_must_be_ready",
            failure_status: "block_manifest_publication",
        },
        SpireRemoteEpochManifestPublicationContractRow {
            step_ordinal: 5,
            prerequisite: "remote_epoch_manifest_transport",
            publication_action: "publish_remote_epoch_manifest",
            required_status: "libpq_pipeline",
            validator: "future_executor_must_use_libpq_pipeline",
            failure_status: "requires_libpq_executor",
        },
    ]
}

fn remote_node_capability_plan_row(
    node: SpireRemoteNodeSnapshotRow,
) -> SpireRemoteNodeCapabilityPlanRow {
    // Until remote descriptors carry binary metadata, version checks report the
    // coordinator build's Cargo version as the required storage-node version.
    if node.node_id == meta::SPIRE_LOCAL_NODE_ID {
        SpireRemoteNodeCapabilityPlanRow {
            active_epoch: node.active_epoch,
            node_id: node.node_id,
            node_kind: node.node_kind,
            descriptor_generation: node.descriptor_generation,
            descriptor_state: node.descriptor_state,
            required_last_served_epoch: node.active_epoch,
            required_min_retained_epoch: node.active_epoch,
            required_candidate_format: SPIRE_REMOTE_CANDIDATE_FORMAT_LOCAL,
            required_extension_version: env!("CARGO_PKG_VERSION"),
            conninfo_source: "local",
            remote_index_identity_status: "not_required",
            epoch_window_status: SPIRE_REMOTE_STATUS_READY,
            candidate_format_status: "not_required",
            extension_version_status: SPIRE_REMOTE_STATUS_READY,
            status: SPIRE_REMOTE_STATUS_READY,
            recommendation: SPIRE_REMOTE_NONE,
        }
    } else if node.descriptor_state == SPIRE_REMOTE_DESCRIPTOR_STATE_MISSING {
        SpireRemoteNodeCapabilityPlanRow {
            active_epoch: node.active_epoch,
            node_id: node.node_id,
            node_kind: node.node_kind,
            descriptor_generation: node.descriptor_generation,
            descriptor_state: node.descriptor_state,
            required_last_served_epoch: node.active_epoch,
            required_min_retained_epoch: node.active_epoch,
            required_candidate_format: SPIRE_REMOTE_CANDIDATE_FORMAT_V1,
            required_extension_version: env!("CARGO_PKG_VERSION"),
            conninfo_source: SPIRE_REMOTE_DESCRIPTOR_SOURCE,
            remote_index_identity_status: SPIRE_REMOTE_STATUS_MISSING_DESCRIPTOR,
            epoch_window_status: SPIRE_REMOTE_STATUS_MISSING_DESCRIPTOR,
            candidate_format_status: SPIRE_REMOTE_STATUS_MISSING_DESCRIPTOR,
            extension_version_status: SPIRE_REMOTE_STATUS_MISSING_DESCRIPTOR,
            status: SPIRE_REMOTE_STATUS_REQUIRES_DESCRIPTOR,
            recommendation: "register remote node descriptor before capability check",
        }
    } else {
        let required_last_served_epoch = node.active_epoch;
        let required_min_retained_epoch = node.active_epoch;
        let epoch_window_status = if node.last_served_epoch < required_last_served_epoch {
            "stale_epoch"
        } else if node.min_retained_epoch > required_min_retained_epoch {
            "retention_gap"
        } else {
            SPIRE_REMOTE_STATUS_READY
        };
        let extension_version_status = if node.extension_version == env!("CARGO_PKG_VERSION") {
            SPIRE_REMOTE_STATUS_READY
        } else {
            "incompatible_extension_version"
        };
        let (status, recommendation) = if node.status != SPIRE_REMOTE_STATUS_READY {
            (node.status, node.recommendation)
        } else if epoch_window_status != SPIRE_REMOTE_STATUS_READY {
            (
                epoch_window_status,
                "refresh remote node served epoch window before capability check",
            )
        } else if extension_version_status != SPIRE_REMOTE_STATUS_READY {
            (
                extension_version_status,
                "upgrade remote node extension before libpq fanout execution",
            )
        } else {
            (SPIRE_REMOTE_STATUS_READY, SPIRE_REMOTE_NONE)
        };

        SpireRemoteNodeCapabilityPlanRow {
            active_epoch: node.active_epoch,
            node_id: node.node_id,
            node_kind: node.node_kind,
            descriptor_generation: node.descriptor_generation,
            descriptor_state: node.descriptor_state,
            required_last_served_epoch,
            required_min_retained_epoch,
            required_candidate_format: SPIRE_REMOTE_CANDIDATE_FORMAT_V1,
            required_extension_version: env!("CARGO_PKG_VERSION"),
            conninfo_source: SPIRE_REMOTE_DESCRIPTOR_SOURCE,
            remote_index_identity_status: SPIRE_REMOTE_STATUS_READY,
            epoch_window_status,
            candidate_format_status: SPIRE_REMOTE_STATUS_READY,
            extension_version_status,
            status,
            recommendation,
        }
    }
}

fn remote_epoch_publish_plan_row(
    node: SpireRemoteNodeSnapshotRow,
) -> SpireRemoteEpochPublishPlanRow {
    let required_last_served_epoch = node.active_epoch;
    let required_min_retained_epoch = node.active_epoch;
    let epoch_window_status = if node.descriptor_state == SPIRE_REMOTE_DESCRIPTOR_STATE_MISSING {
        SPIRE_REMOTE_STATUS_MISSING_DESCRIPTOR
    } else if node.last_served_epoch < required_last_served_epoch {
        "stale_epoch"
    } else if node.min_retained_epoch > required_min_retained_epoch {
        "retention_gap"
    } else {
        SPIRE_REMOTE_STATUS_READY
    };
    let (status, recommendation) = if node.status != SPIRE_REMOTE_STATUS_READY {
        (
            node.status,
            "register remote node descriptor before publishing distributed epochs",
        )
    } else if epoch_window_status != SPIRE_REMOTE_STATUS_READY {
        (
            epoch_window_status,
            "refresh remote node served epoch window before distributed epoch publish",
        )
    } else {
        (SPIRE_REMOTE_STATUS_READY, SPIRE_REMOTE_NONE)
    };

    SpireRemoteEpochPublishPlanRow {
        active_epoch: node.active_epoch,
        node_id: node.node_id,
        descriptor_state: node.descriptor_state,
        placement_count: node.placement_count,
        available_placement_count: node.available_placement_count,
        stale_placement_count: node.stale_placement_count,
        unavailable_placement_count: node.unavailable_placement_count,
        skipped_placement_count: node.skipped_placement_count,
        required_last_served_epoch,
        required_min_retained_epoch,
        last_served_epoch: node.last_served_epoch,
        min_retained_epoch: node.min_retained_epoch,
        epoch_window_status,
        status,
        recommendation,
    }
}

fn remote_epoch_manifest_plan_row(
    row: SpireRemoteEpochPublishPlanRow,
) -> SpireRemoteEpochManifestPlanRow {
    let manifest_action = if row.status == SPIRE_REMOTE_STATUS_READY {
        "include_remote_node"
    } else {
        "block_manifest"
    };

    SpireRemoteEpochManifestPlanRow {
        active_epoch: row.active_epoch,
        node_id: row.node_id,
        descriptor_state: row.descriptor_state,
        placement_count: row.placement_count,
        required_last_served_epoch: row.required_last_served_epoch,
        required_min_retained_epoch: row.required_min_retained_epoch,
        last_served_epoch: row.last_served_epoch,
        min_retained_epoch: row.min_retained_epoch,
        epoch_window_status: row.epoch_window_status,
        manifest_action,
        status: row.status,
        recommendation: row.recommendation,
    }
}

fn remote_node_snapshot_empty_row(active_epoch: u64, node_id: u32) -> SpireRemoteNodeSnapshotRow {
    if node_id == meta::SPIRE_LOCAL_NODE_ID {
        // A local empty index is ready but has no dispatchable placements.
        // Executors that need work items must still gate on placement_count.
        SpireRemoteNodeSnapshotRow {
            active_epoch,
            node_id,
            node_kind: "local",
            descriptor_generation: 0,
            descriptor_state: SPIRE_REMOTE_DESCRIPTOR_STATE_ACTIVE,
            placement_count: 0,
            available_placement_count: 0,
            stale_placement_count: 0,
            unavailable_placement_count: 0,
            skipped_placement_count: 0,
            local_store_count: 0,
            last_seen_at_micros: 0,
            last_served_epoch: active_epoch,
            min_retained_epoch: active_epoch,
            extension_version: env!("CARGO_PKG_VERSION").to_owned(),
            last_error: SPIRE_REMOTE_NONE.to_owned(),
            status: SPIRE_REMOTE_STATUS_READY,
            recommendation: SPIRE_REMOTE_NONE,
        }
    } else {
        SpireRemoteNodeSnapshotRow {
            active_epoch,
            node_id,
            node_kind: "remote",
            descriptor_generation: 0,
            descriptor_state: SPIRE_REMOTE_DESCRIPTOR_STATE_MISSING,
            placement_count: 0,
            available_placement_count: 0,
            stale_placement_count: 0,
            unavailable_placement_count: 0,
            skipped_placement_count: 0,
            local_store_count: 0,
            last_seen_at_micros: 0,
            last_served_epoch: 0,
            min_retained_epoch: 0,
            extension_version: "unknown".to_owned(),
            last_error: "missing_remote_node_descriptor".to_owned(),
            status: SPIRE_REMOTE_STATUS_REQUIRES_DESCRIPTOR,
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
