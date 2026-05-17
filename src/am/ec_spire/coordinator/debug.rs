#[cfg(any(test, feature = "pg_test"))]
use crate::storage::relation_guard::IndexRelationGuard;

fn not_implemented(callback: &str) -> ! {
    pgrx::error!("ec_spire {callback} is not implemented yet")
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_spire_relation_object_tuple_roundtrip(
    index_oid: pg_sys::Oid,
) -> (u32, u16, u64, u32, u64, u64, u32, u64) {
    let lockmode = pg_sys::RowExclusiveLock as pg_sys::LOCKMODE;
    let index_relation = IndexRelationGuard::open(index_oid, lockmode, "ec_spire debug");
    let result = (|| -> Result<(u32, u16, u64, u32, u64, u64, u32, u64), String> {
        let store = unsafe {
            storage::SpireRelationObjectStore::for_index_relation(index_relation.as_ptr())?
        };
        let object = storage::SpireRoutingPartitionObject::root(
            10,
            1,
            2,
            vec![storage::SpireRoutingChildEntry {
                centroid_index: 0,
                child_pid: 11,
                centroid: vec![1.0, 0.0],
            }],
        )?;

        let placement = store.insert_routing_object(1, &object)?;
        let root_control = unsafe { page::read_root_control_page(index_relation.as_ptr()) };
        let decoded = unsafe { store.read_routing_object(&placement)? };
        let child = decoded
            .children()
            .next()
            .ok_or_else(|| "ec_spire debug routing object lost its child".to_owned())?;

        Ok((
            placement.object_tid.block_number,
            placement.object_tid.offset_number,
            root_control.active_epoch,
            placement.store_relid,
            decoded.header.pid,
            decoded.header.object_version,
            decoded.header.child_count,
            child.child_pid,
        ))
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_spire_relation_leaf_v2_roundtrip(
    index_oid: pg_sys::Oid,
) -> (u32, u16, u32, u32, u64, u32) {
    let lockmode = pg_sys::RowExclusiveLock as pg_sys::LOCKMODE;
    let index_relation = IndexRelationGuard::open(index_oid, lockmode, "ec_spire debug");
    let result = (|| -> Result<(u32, u16, u32, u32, u64, u32), String> {
        let store = unsafe {
            storage::SpireRelationObjectStore::for_index_relation(index_relation.as_ptr())?
        };
        let assignments = vec![
            storage::SpireLeafAssignmentRow {
                flags: storage::SPIRE_ASSIGNMENT_FLAG_PRIMARY,
                vec_id: storage::SpireVecId::local(1),
                heap_tid: crate::storage::page::ItemPointer {
                    block_number: 42,
                    offset_number: 1,
                },
                payload_format: storage::SPIRE_PAYLOAD_FORMAT_TURBOQUANT,
                gamma: 0.5,
                encoded_payload: vec![1, 2, 3, 4],
            },
            storage::SpireLeafAssignmentRow {
                flags: storage::SPIRE_ASSIGNMENT_FLAG_PRIMARY,
                vec_id: storage::SpireVecId::local(2),
                heap_tid: crate::storage::page::ItemPointer {
                    block_number: 43,
                    offset_number: 2,
                },
                payload_format: storage::SPIRE_PAYLOAD_FORMAT_TURBOQUANT,
                gamma: 0.75,
                encoded_payload: vec![5, 6, 7, 8],
            },
        ];

        let placement =
            store.insert_leaf_object_v2_from_rows(1, 20, 1, 10, &assignments)?;
        let leaf = unsafe { store.read_leaf_object_v2(&placement)? };
        let rows = leaf.assignment_rows()?;
        let first_row = rows
            .first()
            .ok_or_else(|| "ec_spire debug leaf V2 lost its first row".to_owned())?;

        Ok((
            placement.object_tid.block_number,
            placement.object_tid.offset_number,
            leaf.meta.header.assignment_count,
            leaf.meta.segment_count,
            first_row
                .vec_id
                .local_sequence()
                .ok_or_else(|| "ec_spire debug leaf V2 first row lost local vec_id".to_owned())?,
            first_row.heap_tid.block_number,
        ))
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_spire_empty_manifest_publish_roundtrip(
    index_oid: pg_sys::Oid,
) -> (u64, u64, u64, u32, u16, u32, u16, u32, u16) {
    let lockmode = pg_sys::RowExclusiveLock as pg_sys::LOCKMODE;
    let index_relation = IndexRelationGuard::open(index_oid, lockmode, "ec_spire debug");
    let result = (|| -> Result<(u64, u64, u64, u32, u16, u32, u16, u32, u16), String> {
        let epoch_manifest = meta::SpireEpochManifest {
            epoch: 1,
            state: meta::SpireEpochState::Published,
            consistency_mode: meta::SpireConsistencyMode::Strict,
            published_at_micros: 1,
            retain_until_micros: 600_000_001,
            active_query_count: 0,
        };
        let object_manifest = meta::SpireObjectManifest::from_entries(1, Vec::new())?;
        let placement_directory = meta::SpirePlacementDirectory::from_entries(1, Vec::new())?;
        let input = build::SpirePublishCoordinatorInput {
            epoch_manifest: &epoch_manifest,
            object_manifest: &object_manifest,
            placement_directory: &placement_directory,
            local_store_config: meta::SpireLocalStoreConfig::embedded_single_store(
                unsafe { (*index_relation.as_ptr()).rd_id }.into(),
                unsafe { (*(*index_relation.as_ptr()).rd_rel).reltablespace }.into(),
            )?,
            next_pid: assign::SPIRE_FIRST_PID,
            next_local_vec_seq: assign::SPIRE_FIRST_LOCAL_VEC_SEQ,
        };
        let manifests = build::encode_manifest_bundle_for_publish(input.clone())?;
        let locators = unsafe {
            build::write_manifest_bundle_to_relation(index_relation.as_ptr(), &manifests)?
        };
        let root_control = build::root_control_state_for_publish(input, locators)?;
        unsafe { page::initialize_root_control_page(index_relation.as_ptr(), root_control) };
        let persisted = unsafe { page::read_root_control_page(index_relation.as_ptr()) };

        Ok((
            persisted.active_epoch,
            persisted.next_pid,
            persisted.next_local_vec_seq,
            persisted.epoch_manifest_tid.block_number,
            persisted.epoch_manifest_tid.offset_number,
            persisted.object_manifest_tid.block_number,
            persisted.object_manifest_tid.offset_number,
            persisted.placement_directory_tid.block_number,
            persisted.placement_directory_tid.offset_number,
        ))
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_spire_age_retired_epoch_manifests(
    index_oid: pg_sys::Oid,
    retain_until_micros: i64,
) -> u64 {
    let lockmode = pg_sys::RowExclusiveLock as pg_sys::LOCKMODE;
    let index_relation = IndexRelationGuard::open(index_oid, lockmode, "ec_spire debug");
    let result = (|| -> Result<u64, String> {
        let mut rewrites = Vec::new();
        unsafe {
            page::scan_object_tuples(index_relation.as_ptr(), |tid, tuple| {
                if tuple.len() != meta::SpireEpochManifest::encoded_len() {
                    return Ok(());
                }
                let Ok(mut manifest) = meta::SpireEpochManifest::decode(tuple) else {
                    return Ok(());
                };
                if manifest.state != meta::SpireEpochState::Retired {
                    return Ok(());
                }
                manifest.published_at_micros = retain_until_micros;
                manifest.retain_until_micros = retain_until_micros;
                manifest.active_query_count = 0;
                rewrites.push((tid, manifest.encode()?));
                Ok(())
            })?
        };
        for (tid, payload) in &rewrites {
            unsafe {
                page::rewrite_object_tuple_same_len(index_relation.as_ptr(), *tid, payload)?
            };
        }
        u64::try_from(rewrites.len())
            .map_err(|_| "ec_spire debug retired epoch rewrite count exceeds u64".to_owned())
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_spire_relation_two_store_scan_roundtrip(
    root_index_oid: pg_sys::Oid,
    aux_store_oid: pg_sys::Oid,
) -> (u32, u32, u32, u32, u64, u64) {
    let lockmode = pg_sys::RowExclusiveLock as pg_sys::LOCKMODE;
    let root_relation = IndexRelationGuard::open(root_index_oid, lockmode, "ec_spire debug");
    let aux_relation = IndexRelationGuard::open(aux_store_oid, lockmode, "ec_spire debug");
    let result = (|| -> Result<(u32, u32, u32, u32, u64, u64), String> {
        let root_relid: u32 = root_index_oid.into();
        let aux_relid: u32 = aux_store_oid.into();
        let root_store = storage::SpireRelationObjectStore::for_store_relation_id(
            root_relation.as_ptr(),
            meta::SPIRE_SINGLE_LOCAL_STORE_ID,
            root_relid,
        );
        let aux_store =
            storage::SpireRelationObjectStore::for_store_relation_id(
                aux_relation.as_ptr(),
                1,
                aux_relid,
            );

        let root_pid = assign::SPIRE_FIRST_PID;
        let left_leaf_pid = assign::SPIRE_FIRST_PID + 1;
        let right_leaf_pid = assign::SPIRE_FIRST_PID + 8;
        let root_object = storage::SpireRoutingPartitionObject::root(
            root_pid,
            1,
            2,
            vec![
                storage::SpireRoutingChildEntry {
                    centroid_index: 0,
                    child_pid: left_leaf_pid,
                    centroid: vec![1.0, 0.0],
                },
                storage::SpireRoutingChildEntry {
                    centroid_index: 1,
                    child_pid: right_leaf_pid,
                    centroid: vec![-1.0, 0.0],
                },
            ],
        )?;
        let left_assignment = quantizer::encode_assignment_input(
            quantizer::SpireAssignmentPayloadFormat::TurboQuant,
            crate::storage::page::ItemPointer {
                block_number: 10,
                offset_number: 1,
            },
            &[1.0, 0.0],
        )?;
        let right_assignment = quantizer::encode_assignment_input(
            quantizer::SpireAssignmentPayloadFormat::TurboQuant,
            crate::storage::page::ItemPointer {
                block_number: 10,
                offset_number: 2,
            },
            &[-1.0, 0.0],
        )?;
        let left_row = storage::SpireLeafAssignmentRow {
            flags: storage::SPIRE_ASSIGNMENT_FLAG_PRIMARY,
            vec_id: storage::SpireVecId::local(1),
            heap_tid: left_assignment.heap_tid,
            payload_format: left_assignment.payload_format,
            gamma: left_assignment.gamma,
            encoded_payload: left_assignment.encoded_payload,
        };
        let right_row = storage::SpireLeafAssignmentRow {
            flags: storage::SPIRE_ASSIGNMENT_FLAG_PRIMARY,
            vec_id: storage::SpireVecId::local(2),
            heap_tid: right_assignment.heap_tid,
            payload_format: right_assignment.payload_format,
            gamma: right_assignment.gamma,
            encoded_payload: right_assignment.encoded_payload,
        };

        let placements = vec![
            aux_store.insert_routing_object(1, &root_object)?,
            root_store.insert_leaf_object_v2_from_rows(
                1,
                left_leaf_pid,
                1,
                root_pid,
                &[left_row],
            )?,
            aux_store.insert_leaf_object_v2_from_rows(
                1,
                right_leaf_pid,
                1,
                root_pid,
                &[right_row],
            )?,
        ];
        let placement_directory = meta::SpirePlacementDirectory::from_entries(1, placements)?;
        let placement_evidence = unsafe {
            build::write_placement_entries_to_relation(root_relation.as_ptr(), &placement_directory)?
        };
        let object_manifest = build::object_manifest_from_placement_writes(
            1,
            &placement_directory,
            &placement_evidence,
        )?;
        let epoch_manifest = meta::SpireEpochManifest {
            epoch: 1,
            state: meta::SpireEpochState::Published,
            consistency_mode: meta::SpireConsistencyMode::Strict,
            published_at_micros: 1,
            retain_until_micros: 600_000_001,
            active_query_count: 0,
        };
        let input = build::SpirePublishCoordinatorInput {
            epoch_manifest: &epoch_manifest,
            object_manifest: &object_manifest,
            placement_directory: &placement_directory,
            local_store_config: meta::SpireLocalStoreConfig::from_placement_directory(
                epoch_manifest.epoch,
                &placement_directory,
            )?,
            next_pid: assign::SPIRE_FIRST_PID + 9,
            next_local_vec_seq: assign::SPIRE_FIRST_LOCAL_VEC_SEQ + 2,
        };
        let manifests = build::encode_manifest_bundle_for_publish(input.clone())?;
        let locators = unsafe {
            build::write_manifest_bundle_to_relation(root_relation.as_ptr(), &manifests)?
        };
        let root_control = build::root_control_state_for_publish(input, locators)?;
        unsafe { page::initialize_root_control_page(root_relation.as_ptr(), root_control) };

        let snapshot = meta::SpirePublishedEpochSnapshot::new(
            &epoch_manifest,
            &object_manifest,
            &placement_directory,
        )?;
        let relation_store_set = unsafe {
            storage::SpireRelationObjectStoreSet::for_index_relation_and_placements(
                root_relation.as_ptr(),
                &placement_directory,
                pg_sys::AccessShareLock as pg_sys::LOCKMODE,
            )?
        };
        let candidates = scan::collect_quantized_routed_probe_candidates(
            &snapshot,
            &relation_store_set,
            &[1.0, 0.0],
            2,
            quantizer::SpireAssignmentPayloadFormat::TurboQuant,
            options::SpireCandidateDedupeMode::NoReplicaDedupeDisabled,
            Some(2),
        )?;
        if candidates.len() != 2 {
            return Err(format!(
                "ec_spire debug two-store scan expected 2 candidates, got {}",
                candidates.len()
            ));
        }
        let mut candidate_store_ids = candidates
            .iter()
            .map(|candidate| {
                placement_directory
                    .get(candidate.pid)
                    .ok_or_else(|| {
                        format!(
                            "ec_spire debug candidate pid {} missing placement",
                            candidate.pid
                        )
                    })
                    .map(|placement| placement.local_store_id)
            })
            .collect::<Result<Vec<_>, _>>()?;
        candidate_store_ids.sort_unstable();
        candidate_store_ids.dedup();
        if candidate_store_ids != [0, 1] {
            return Err(format!(
                "ec_spire debug two-store scan touched stores {:?}",
                candidate_store_ids
            ));
        }
        let root_placement = placement_directory
            .get(root_pid)
            .ok_or_else(|| "ec_spire debug root placement missing".to_owned())?;
        let left_placement = placement_directory
            .get(left_leaf_pid)
            .ok_or_else(|| "ec_spire debug left leaf placement missing".to_owned())?;
        let right_placement = placement_directory
            .get(right_leaf_pid)
            .ok_or_else(|| "ec_spire debug right leaf placement missing".to_owned())?;

        Ok((
            root_placement.local_store_id,
            left_placement.local_store_id,
            right_placement.local_store_id,
            u32::try_from(candidates.len())
                .map_err(|_| "ec_spire debug candidate count exceeds u32".to_owned())?,
            candidates[0]
                .vec_id
                .local_sequence()
                .ok_or_else(|| "ec_spire debug first candidate lost vec_id".to_owned())?,
            candidates[1]
                .vec_id
                .local_sequence()
                .ok_or_else(|| "ec_spire debug second candidate lost vec_id".to_owned())?,
        ))
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_spire_root_control(index_oid: pg_sys::Oid) -> (u64, u64, u64) {
    let lockmode = pg_sys::AccessShareLock as pg_sys::LOCKMODE;
    let index_relation = IndexRelationGuard::open(index_oid, lockmode, "ec_spire debug");
    let root_control = unsafe { page::read_root_control_page(index_relation.as_ptr()) };
    (
        root_control.active_epoch,
        root_control.next_pid,
        root_control.next_local_vec_seq,
    )
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_spire_rewrite_placement_state(
    index_oid: pg_sys::Oid,
    pid: u64,
    state: &str,
) -> u64 {
    let lockmode = pg_sys::RowExclusiveLock as pg_sys::LOCKMODE;
    let index_relation = IndexRelationGuard::open(index_oid, lockmode, "ec_spire debug");
    let result = (|| -> Result<u64, String> {
        let root_control = unsafe { page::read_root_control_page(index_relation.as_ptr()) };
        let local_store_config =
            unsafe { scan::load_relation_local_store_config(index_relation.as_ptr(), root_control)? };
        let epoch_bytes =
            unsafe { page::read_object_tuple(index_relation.as_ptr(), root_control.epoch_manifest_tid)? };
        let object_bytes =
            unsafe { page::read_object_tuple(index_relation.as_ptr(), root_control.object_manifest_tid)? };
        let placement_bytes = unsafe {
            page::read_object_tuple(index_relation.as_ptr(), root_control.placement_directory_tid)?
        };
        let epoch_manifest = meta::SpireEpochManifest::decode(&epoch_bytes)?;
        let object_manifest = meta::SpireObjectManifest::decode(&object_bytes)?;
        let mut placement_directory = meta::SpirePlacementDirectory::decode(&placement_bytes)?;
        if epoch_manifest.epoch != root_control.active_epoch {
            return Err(format!(
                "ec_spire debug placement rewrite active epoch {} does not match epoch manifest {}",
                root_control.active_epoch, epoch_manifest.epoch
            ));
        }
        let placement = placement_directory
            .entries
            .iter_mut()
            .find(|entry| entry.pid == pid)
            .ok_or_else(|| format!("ec_spire debug placement rewrite missing pid {pid}"))?;
        placement.state = match state {
            "available" => meta::SpirePlacementState::Available,
            "stale" => meta::SpirePlacementState::Stale,
            "unavailable" => meta::SpirePlacementState::Unavailable,
            "skipped" => meta::SpirePlacementState::Skipped,
            other => {
                return Err(format!(
                    "ec_spire debug placement rewrite unknown state '{other}'"
                ))
            }
        };

        let manifests = build::SpireEncodedManifestBundle {
            epoch_manifest: epoch_manifest.encode()?,
            object_manifest: object_manifest.encode()?,
            placement_directory: placement_directory.encode()?,
            local_store_config: local_store_config.encode()?,
        };
        let locators = unsafe {
            build::write_manifest_bundle_to_relation(index_relation.as_ptr(), &manifests)?
        };
        let root_control = meta::SpireRootControlState::published_with_store_config(
            root_control.active_epoch,
            root_control.next_pid,
            root_control.next_local_vec_seq,
            locators.epoch_manifest_tid,
            locators.object_manifest_tid,
            locators.placement_directory_tid,
            locators.local_store_config_tid,
        )?;
        unsafe { page::initialize_root_control_page(index_relation.as_ptr(), root_control) };
        Ok(root_control.active_epoch)
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_spire_rewrite_placement_node(
    index_oid: pg_sys::Oid,
    pid: u64,
    node_id: u32,
) -> u64 {
    unsafe { debug_spire_rewrite_placement_nodes(index_oid, &[(pid, node_id)]) }
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_spire_rewrite_placement_nodes(
    index_oid: pg_sys::Oid,
    rewrites: &[(u64, u32)],
) -> u64 {
    let lockmode = pg_sys::RowExclusiveLock as pg_sys::LOCKMODE;
    let index_relation = IndexRelationGuard::open(index_oid, lockmode, "ec_spire debug");
    let result = (|| -> Result<u64, String> {
        let root_control = unsafe { page::read_root_control_page(index_relation.as_ptr()) };
        let local_store_config =
            unsafe { scan::load_relation_local_store_config(index_relation.as_ptr(), root_control)? };
        let (epoch_manifest, object_manifest, mut placement_directory) = unsafe {
            load_relation_epoch_manifests_for_coordinator_fanout(
                index_relation.as_ptr(),
                root_control,
            )?
        };
        for (pid, node_id) in rewrites {
            let placement = placement_directory
                .entries
                .iter_mut()
                .find(|entry| entry.pid == *pid)
                .ok_or_else(|| {
                    format!("ec_spire debug placement node rewrite missing pid {pid}")
                })?;
            placement.node_id = *node_id;
            placement.local_store_id = *node_id;
        }

        let manifests = build::SpireEncodedManifestBundle {
            epoch_manifest: epoch_manifest.encode()?,
            object_manifest: object_manifest.encode()?,
            placement_directory: placement_directory.encode()?,
            local_store_config: local_store_config.encode()?,
        };
        let locators = unsafe {
            build::write_manifest_bundle_to_relation(index_relation.as_ptr(), &manifests)?
        };
        let root_control = meta::SpireRootControlState::published_with_store_config(
            root_control.active_epoch,
            root_control.next_pid,
            root_control.next_local_vec_seq,
            locators.epoch_manifest_tid,
            locators.object_manifest_tid,
            locators.placement_directory_tid,
            locators.local_store_config_tid,
        )?;
        unsafe { page::initialize_root_control_page(index_relation.as_ptr(), root_control) };
        Ok(root_control.active_epoch)
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_spire_rewrite_consistency_mode(
    index_oid: pg_sys::Oid,
    mode: &str,
) -> u64 {
    let lockmode = pg_sys::RowExclusiveLock as pg_sys::LOCKMODE;
    let index_relation = IndexRelationGuard::open(index_oid, lockmode, "ec_spire debug");
    let result = (|| -> Result<u64, String> {
        let root_control = unsafe { page::read_root_control_page(index_relation.as_ptr()) };
        let local_store_config =
            unsafe { scan::load_relation_local_store_config(index_relation.as_ptr(), root_control)? };
        let (mut epoch_manifest, object_manifest, placement_directory) = unsafe {
            load_relation_epoch_manifests_for_coordinator_fanout(
                index_relation.as_ptr(),
                root_control,
            )?
        };
        epoch_manifest.consistency_mode = match mode {
            "strict" => meta::SpireConsistencyMode::Strict,
            "degraded" => meta::SpireConsistencyMode::Degraded,
            other => {
                return Err(format!(
                    "ec_spire debug consistency mode rewrite unknown mode '{other}'"
                ))
            }
        };

        let manifests = build::SpireEncodedManifestBundle {
            epoch_manifest: epoch_manifest.encode()?,
            object_manifest: object_manifest.encode()?,
            placement_directory: placement_directory.encode()?,
            local_store_config: local_store_config.encode()?,
        };
        let locators = unsafe {
            build::write_manifest_bundle_to_relation(index_relation.as_ptr(), &manifests)?
        };
        let root_control = meta::SpireRootControlState::published_with_store_config(
            root_control.active_epoch,
            root_control.next_pid,
            root_control.next_local_vec_seq,
            locators.epoch_manifest_tid,
            locators.object_manifest_tid,
            locators.placement_directory_tid,
            locators.local_store_config_tid,
        )?;
        unsafe { page::initialize_root_control_page(index_relation.as_ptr(), root_control) };
        Ok(root_control.active_epoch)
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

#[cfg(any(test, feature = "pg_test"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SpireDebugSnapshotDiagnostics {
    pub(crate) epoch: u64,
    pub(crate) object_count: u64,
    pub(crate) placement_count: u64,
    pub(crate) local_store_count: u64,
    pub(crate) available_placement_count: u64,
    pub(crate) root_object_count: u64,
    pub(crate) leaf_object_count: u64,
    pub(crate) routing_child_count: u64,
    pub(crate) leaf_assignment_count: u64,
    pub(crate) available_object_bytes: u64,
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_spire_active_snapshot_diagnostics(
    index_oid: pg_sys::Oid,
) -> SpireDebugSnapshotDiagnostics {
    let lockmode = pg_sys::AccessShareLock as pg_sys::LOCKMODE;
    let index_relation = IndexRelationGuard::open(index_oid, lockmode, "ec_spire debug");
    let result = (|| -> Result<SpireDebugSnapshotDiagnostics, String> {
        let root_control = unsafe { page::read_root_control_page(index_relation.as_ptr()) };
        let (epoch_manifest, object_manifest, placement_directory) =
            unsafe { scan::load_relation_epoch_manifests(index_relation.as_ptr(), root_control)? };
        let snapshot = meta::SpirePublishedEpochSnapshot::new(
            &epoch_manifest,
            &object_manifest,
            &placement_directory,
        )?;
        let object_store = unsafe {
            storage::SpireRelationObjectStore::for_index_relation(index_relation.as_ptr())?
        };
        let diagnostics = diagnostics::collect_snapshot_diagnostics(&snapshot, &object_store)?;

        Ok(SpireDebugSnapshotDiagnostics {
            epoch: diagnostics.epoch,
            object_count: diagnostics.object_count as u64,
            placement_count: diagnostics.placement_count as u64,
            local_store_count: diagnostics.local_store_count as u64,
            available_placement_count: diagnostics.available_placement_count as u64,
            root_object_count: diagnostics.root_object_count as u64,
            leaf_object_count: diagnostics.leaf_object_count as u64,
            routing_child_count: diagnostics.routing_child_count as u64,
            leaf_assignment_count: diagnostics.leaf_assignment_count as u64,
            available_object_bytes: diagnostics.available_object_bytes,
        })
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}
