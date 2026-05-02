use std::collections::HashSet;

use super::meta::{SpireConsistencyMode, SpirePlacementState, SpirePublishedEpochSnapshot};
use super::storage::{
    SpireLeafAssignmentRow, SpireLocalObjectStore, SpirePartitionObjectKind,
    SPIRE_ASSIGNMENT_FLAG_BOUNDARY_REPLICA, SPIRE_ASSIGNMENT_FLAG_DELTA_DELETE,
    SPIRE_ASSIGNMENT_FLAG_PRIMARY, SPIRE_ASSIGNMENT_FLAG_STALE_LOCATOR,
    SPIRE_ASSIGNMENT_FLAG_TOMBSTONE,
};
use pgrx::pg_sys;

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SpireLeafScanRow {
    pub(super) pid: u64,
    pub(super) object_version: u64,
    pub(super) row_index: u32,
    pub(super) assignment: SpireLeafAssignmentRow,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SpireDeltaScanRow {
    pub(super) pid: u64,
    pub(super) object_version: u64,
    pub(super) row_index: u32,
    pub(super) assignment: SpireLeafAssignmentRow,
}

pub(super) fn collect_snapshot_leaf_rows(
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    object_store: &SpireLocalObjectStore,
) -> Result<Vec<SpireLeafScanRow>, String> {
    SpirePublishedEpochSnapshot::new(
        snapshot.epoch_manifest,
        snapshot.object_manifest,
        snapshot.placement_directory,
    )?;

    let mut rows = Vec::new();
    for manifest_entry in &snapshot.object_manifest.entries {
        let placement = snapshot
            .placement_directory
            .get(manifest_entry.pid)
            .ok_or_else(|| {
                format!(
                    "ec_spire scan snapshot missing placement for pid {}",
                    manifest_entry.pid
                )
            })?;

        if should_skip_placement(snapshot.epoch_manifest.consistency_mode, placement.state)? {
            continue;
        }

        let header = object_store.read_object_header(placement)?;
        if header.kind != SpirePartitionObjectKind::Leaf {
            continue;
        }

        let leaf_object = object_store.read_leaf_object(placement)?;
        for (row_index, assignment) in leaf_object.assignments.into_iter().enumerate() {
            let row_index = u32::try_from(row_index)
                .map_err(|_| "ec_spire scan row index exceeds u32".to_owned())?;
            rows.push(SpireLeafScanRow {
                pid: manifest_entry.pid,
                object_version: manifest_entry.object_version,
                row_index,
                assignment,
            });
        }
    }
    Ok(rows)
}

pub(super) fn collect_snapshot_delta_rows(
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    object_store: &SpireLocalObjectStore,
) -> Result<Vec<SpireDeltaScanRow>, String> {
    SpirePublishedEpochSnapshot::new(
        snapshot.epoch_manifest,
        snapshot.object_manifest,
        snapshot.placement_directory,
    )?;

    let mut rows = Vec::new();
    for manifest_entry in &snapshot.object_manifest.entries {
        let placement = snapshot
            .placement_directory
            .get(manifest_entry.pid)
            .ok_or_else(|| {
                format!(
                    "ec_spire scan snapshot missing placement for pid {}",
                    manifest_entry.pid
                )
            })?;

        if should_skip_placement(snapshot.epoch_manifest.consistency_mode, placement.state)? {
            continue;
        }

        let header = object_store.read_object_header(placement)?;
        if header.kind != SpirePartitionObjectKind::Delta {
            continue;
        }

        let delta_object = object_store.read_delta_object(placement)?;
        for (row_index, assignment) in delta_object.assignments.into_iter().enumerate() {
            let row_index = u32::try_from(row_index)
                .map_err(|_| "ec_spire scan row index exceeds u32".to_owned())?;
            rows.push(SpireDeltaScanRow {
                pid: manifest_entry.pid,
                object_version: manifest_entry.object_version,
                row_index,
                assignment,
            });
        }
    }
    Ok(rows)
}

pub(super) fn collect_snapshot_visible_primary_rows(
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    object_store: &SpireLocalObjectStore,
) -> Result<Vec<SpireLeafScanRow>, String> {
    let delta_rows = collect_snapshot_delta_rows(snapshot, object_store)?;
    let deleted_vec_ids: HashSet<_> = delta_rows
        .iter()
        .filter(|row| is_delete_delta_assignment(&row.assignment))
        .map(|row| row.assignment.vec_id.clone())
        .collect();

    let mut visible_rows = Vec::new();
    visible_rows.extend(
        collect_snapshot_leaf_rows(snapshot, object_store)?
            .into_iter()
            .filter(|row| {
                is_visible_primary_assignment(&row.assignment)
                    && !deleted_vec_ids.contains(&row.assignment.vec_id)
            }),
    );
    visible_rows.extend(delta_rows.into_iter().filter_map(|row| {
        if is_visible_primary_assignment(&row.assignment)
            && !deleted_vec_ids.contains(&row.assignment.vec_id)
        {
            Some(SpireLeafScanRow {
                pid: row.pid,
                object_version: row.object_version,
                row_index: row.row_index,
                assignment: row.assignment,
            })
        } else {
            None
        }
    }));

    let mut visible_vec_ids = HashSet::new();
    for row in &visible_rows {
        if !visible_vec_ids.insert(row.assignment.vec_id.clone()) {
            return Err(
                "ec_spire visible snapshot contains duplicate primary vec_id assignments"
                    .to_owned(),
            );
        }
    }

    Ok(visible_rows)
}

fn is_visible_primary_assignment(assignment: &SpireLeafAssignmentRow) -> bool {
    let blocked_flags = SPIRE_ASSIGNMENT_FLAG_BOUNDARY_REPLICA
        | SPIRE_ASSIGNMENT_FLAG_TOMBSTONE
        | SPIRE_ASSIGNMENT_FLAG_DELTA_DELETE
        | SPIRE_ASSIGNMENT_FLAG_STALE_LOCATOR;
    assignment.flags & SPIRE_ASSIGNMENT_FLAG_PRIMARY != 0 && assignment.flags & blocked_flags == 0
}

fn is_delete_delta_assignment(assignment: &SpireLeafAssignmentRow) -> bool {
    assignment.flags & SPIRE_ASSIGNMENT_FLAG_DELTA_DELETE != 0
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

pub(super) unsafe extern "C-unwind" fn ec_spire_ambeginscan(
    _index_relation: pg_sys::Relation,
    _nkeys: std::ffi::c_int,
    _norderbys: std::ffi::c_int,
) -> pg_sys::IndexScanDesc {
    unsafe { pgrx::pgrx_extern_c_guard(|| super::not_implemented("ambeginscan")) }
}

pub(super) unsafe extern "C-unwind" fn ec_spire_amrescan(
    _scan: pg_sys::IndexScanDesc,
    _keys: pg_sys::ScanKey,
    _nkeys: std::ffi::c_int,
    _orderbys: pg_sys::ScanKey,
    _norderbys: std::ffi::c_int,
) {
    unsafe { pgrx::pgrx_extern_c_guard(|| super::not_implemented("amrescan")) }
}

pub(super) unsafe extern "C-unwind" fn ec_spire_amgettuple(
    _scan: pg_sys::IndexScanDesc,
    _direction: pg_sys::ScanDirection::Type,
) -> bool {
    unsafe { pgrx::pgrx_extern_c_guard(|| super::not_implemented("amgettuple")) }
}

pub(super) unsafe extern "C-unwind" fn ec_spire_amendscan(_scan: pg_sys::IndexScanDesc) {
    unsafe { pgrx::pgrx_extern_c_guard(|| super::not_implemented("amendscan")) }
}

#[cfg(test)]
mod tests {
    use super::{
        collect_snapshot_delta_rows, collect_snapshot_leaf_rows,
        collect_snapshot_visible_primary_rows,
    };
    use crate::am::ec_spire::assign::{
        SpireDeleteDeltaInput, SpireLeafAssignmentInput, SpireLocalVecIdAllocator,
        SpirePidAllocator, SPIRE_FIRST_PID,
    };
    use crate::am::ec_spire::build::{
        build_single_level_leaf_epoch_draft, SpireSingleLevelBuildInput,
    };
    use crate::am::ec_spire::meta::{
        SpireConsistencyMode, SpireEpochManifest, SpireEpochState, SpireManifestEntry,
        SpireObjectManifest, SpirePlacementDirectory, SpirePlacementEntry, SpirePlacementState,
        SpirePublishedEpochSnapshot,
    };
    use crate::am::ec_spire::storage::SpireLocalObjectStore;
    use crate::am::ec_spire::storage::{
        SpireDeltaPartitionObject, SpireLeafAssignmentRow, SpireLeafPartitionObject, SpireVecId,
        SPIRE_ASSIGNMENT_FLAG_BOUNDARY_REPLICA, SPIRE_ASSIGNMENT_FLAG_DELTA_DELETE,
        SPIRE_ASSIGNMENT_FLAG_DELTA_INSERT, SPIRE_ASSIGNMENT_FLAG_PRIMARY,
        SPIRE_ASSIGNMENT_FLAG_STALE_LOCATOR, SPIRE_ASSIGNMENT_FLAG_TOMBSTONE,
    };
    use crate::am::ec_spire::update::{
        build_delta_epoch_draft_from_snapshot, SpireDeltaEpochInput,
    };
    use crate::storage::page::ItemPointer;

    fn tid(block_number: u32, offset_number: u16) -> ItemPointer {
        ItemPointer {
            block_number,
            offset_number,
        }
    }

    fn assignment_input(block_number: u32, offset_number: u16) -> SpireLeafAssignmentInput {
        SpireLeafAssignmentInput {
            heap_tid: tid(block_number, offset_number),
            payload_format: 1,
            gamma: 0.5,
            encoded_payload: vec![1, 2, 3],
        }
    }

    fn build_input(assignments: Vec<SpireLeafAssignmentInput>) -> SpireSingleLevelBuildInput {
        SpireSingleLevelBuildInput {
            epoch: 7,
            object_version: 1,
            published_at_micros: 1000,
            retain_until_micros: 2000,
            consistency_mode: SpireConsistencyMode::Strict,
            placement_tid: tid(60, 1),
            assignments,
        }
    }

    fn delta_input(
        insert_assignments: Vec<SpireLeafAssignmentInput>,
        delete_assignments: Vec<SpireDeleteDeltaInput>,
    ) -> SpireDeltaEpochInput {
        SpireDeltaEpochInput {
            epoch: 8,
            object_version: 3,
            published_at_micros: 2000,
            retain_until_micros: 3000,
            consistency_mode: SpireConsistencyMode::Strict,
            base_pid: SPIRE_FIRST_PID,
            placement_tid: tid(80, 1),
            insert_assignments,
            delete_assignments,
        }
    }

    fn delete_delta_input(
        vec_seq: u64,
        block_number: u32,
        offset_number: u16,
    ) -> SpireDeleteDeltaInput {
        SpireDeleteDeltaInput {
            vec_id: SpireVecId::local(vec_seq),
            heap_tid: tid(block_number, offset_number),
        }
    }

    fn assignment_row(flags: u16, offset_number: u16) -> SpireLeafAssignmentRow {
        SpireLeafAssignmentRow {
            flags,
            vec_id: SpireVecId::local(u64::from(offset_number)),
            heap_tid: tid(10, offset_number),
            payload_format: 1,
            gamma: 0.5,
            encoded_payload: vec![1, 2, 3],
        }
    }

    fn delete_assignment_row(
        vec_seq: u64,
        block_number: u32,
        offset_number: u16,
    ) -> SpireLeafAssignmentRow {
        SpireLeafAssignmentRow {
            flags: SPIRE_ASSIGNMENT_FLAG_TOMBSTONE | SPIRE_ASSIGNMENT_FLAG_DELTA_DELETE,
            vec_id: SpireVecId::local(vec_seq),
            heap_tid: tid(block_number, offset_number),
            payload_format: 0,
            gamma: 0.0,
            encoded_payload: Vec::new(),
        }
    }

    fn snapshot_for_placement<'a>(
        epoch_manifest: &'a SpireEpochManifest,
        object_manifest: &'a SpireObjectManifest,
        placement_directory: &'a SpirePlacementDirectory,
    ) -> SpirePublishedEpochSnapshot<'a> {
        SpirePublishedEpochSnapshot::new(epoch_manifest, object_manifest, placement_directory)
            .unwrap()
    }

    #[test]
    fn collect_snapshot_leaf_rows_returns_available_leaf_assignments() {
        let mut pid_allocator = SpirePidAllocator::default();
        let mut local_vec_id_allocator = SpireLocalVecIdAllocator::default();
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let draft = build_single_level_leaf_epoch_draft(
            build_input(vec![assignment_input(10, 1), assignment_input(10, 2)]),
            &mut pid_allocator,
            &mut local_vec_id_allocator,
            &mut object_store,
        )
        .unwrap();
        let snapshot = SpirePublishedEpochSnapshot::new(
            &draft.epoch_manifest,
            &draft.object_manifest,
            &draft.placement_directory,
        )
        .unwrap();

        let rows = collect_snapshot_leaf_rows(&snapshot, &object_store).unwrap();

        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].pid, SPIRE_FIRST_PID);
        assert_eq!(rows[0].object_version, 1);
        assert_eq!(rows[0].row_index, 0);
        assert_eq!(rows[0].assignment.heap_tid, tid(10, 1));
        assert_eq!(rows[1].row_index, 1);
        assert_eq!(rows[1].assignment.heap_tid, tid(10, 2));
    }

    #[test]
    fn collect_snapshot_leaf_rows_skips_degraded_unavailable_or_skipped_placements() {
        for state in [
            SpirePlacementState::Unavailable,
            SpirePlacementState::Skipped,
        ] {
            let epoch_manifest = SpireEpochManifest {
                epoch: 7,
                state: SpireEpochState::Published,
                consistency_mode: SpireConsistencyMode::Degraded,
                published_at_micros: 1000,
                retain_until_micros: 2000,
                active_query_count: 0,
            };
            let object_manifest = SpireObjectManifest::from_entries(
                7,
                vec![SpireManifestEntry {
                    epoch: 7,
                    pid: 11,
                    object_version: 1,
                    placement_tid: tid(60, 1),
                }],
            )
            .unwrap();
            let mut placement =
                SpirePlacementEntry::local_single_store(7, 11, 12345, 1, tid(44, 2), 4096);
            placement.state = state;
            let placement_directory =
                SpirePlacementDirectory::from_entries(7, vec![placement]).unwrap();
            let snapshot = SpirePublishedEpochSnapshot::new(
                &epoch_manifest,
                &object_manifest,
                &placement_directory,
            )
            .unwrap();
            let object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();

            assert!(collect_snapshot_leaf_rows(&snapshot, &object_store)
                .unwrap()
                .is_empty());
        }
    }

    #[test]
    fn collect_snapshot_visible_primary_rows_filters_non_output_assignments() {
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let object = SpireLeafPartitionObject::new(
            11,
            1,
            0,
            vec![
                assignment_row(SPIRE_ASSIGNMENT_FLAG_PRIMARY, 1),
                assignment_row(
                    SPIRE_ASSIGNMENT_FLAG_PRIMARY | SPIRE_ASSIGNMENT_FLAG_BOUNDARY_REPLICA,
                    2,
                ),
                assignment_row(
                    SPIRE_ASSIGNMENT_FLAG_PRIMARY | SPIRE_ASSIGNMENT_FLAG_TOMBSTONE,
                    3,
                ),
                assignment_row(
                    SPIRE_ASSIGNMENT_FLAG_PRIMARY | SPIRE_ASSIGNMENT_FLAG_STALE_LOCATOR,
                    4,
                ),
            ],
        )
        .unwrap();
        let leaf_placement = object_store.insert_leaf_object(7, &object).unwrap();
        let delta_object =
            SpireDeltaPartitionObject::new(12, 1, 11, vec![delete_assignment_row(6, 10, 6)])
                .unwrap();
        let delta_placement = object_store.insert_delta_object(7, &delta_object).unwrap();
        let epoch_manifest = SpireEpochManifest {
            epoch: 7,
            state: SpireEpochState::Published,
            consistency_mode: SpireConsistencyMode::Strict,
            published_at_micros: 1000,
            retain_until_micros: 2000,
            active_query_count: 0,
        };
        let object_manifest = SpireObjectManifest::from_entries(
            7,
            vec![
                SpireManifestEntry {
                    epoch: 7,
                    pid: 11,
                    object_version: 1,
                    placement_tid: tid(60, 1),
                },
                SpireManifestEntry {
                    epoch: 7,
                    pid: 12,
                    object_version: 1,
                    placement_tid: tid(60, 2),
                },
            ],
        )
        .unwrap();
        let placement_directory =
            SpirePlacementDirectory::from_entries(7, vec![leaf_placement, delta_placement])
                .unwrap();
        let snapshot =
            snapshot_for_placement(&epoch_manifest, &object_manifest, &placement_directory);

        let rows = collect_snapshot_visible_primary_rows(&snapshot, &object_store).unwrap();

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].pid, 11);
        assert_eq!(rows[0].row_index, 0);
        assert_eq!(rows[0].assignment.heap_tid, tid(10, 1));
    }

    #[test]
    fn collect_snapshot_visible_primary_rows_rejects_duplicate_primary_vec_ids() {
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let first = SpireLeafPartitionObject::new(
            11,
            1,
            0,
            vec![SpireLeafAssignmentRow {
                flags: SPIRE_ASSIGNMENT_FLAG_PRIMARY,
                vec_id: SpireVecId::local(1),
                heap_tid: tid(10, 1),
                payload_format: 1,
                gamma: 0.5,
                encoded_payload: vec![1, 2, 3],
            }],
        )
        .unwrap();
        let second = SpireLeafPartitionObject::new(
            12,
            1,
            0,
            vec![SpireLeafAssignmentRow {
                flags: SPIRE_ASSIGNMENT_FLAG_PRIMARY,
                vec_id: SpireVecId::local(1),
                heap_tid: tid(20, 1),
                payload_format: 1,
                gamma: 0.75,
                encoded_payload: vec![4, 5, 6],
            }],
        )
        .unwrap();
        let first_placement = object_store.insert_leaf_object(7, &first).unwrap();
        let second_placement = object_store.insert_leaf_object(7, &second).unwrap();
        let epoch_manifest = SpireEpochManifest {
            epoch: 7,
            state: SpireEpochState::Published,
            consistency_mode: SpireConsistencyMode::Strict,
            published_at_micros: 1000,
            retain_until_micros: 2000,
            active_query_count: 0,
        };
        let object_manifest = SpireObjectManifest::from_entries(
            7,
            vec![
                SpireManifestEntry {
                    epoch: 7,
                    pid: 11,
                    object_version: 1,
                    placement_tid: tid(60, 1),
                },
                SpireManifestEntry {
                    epoch: 7,
                    pid: 12,
                    object_version: 1,
                    placement_tid: tid(60, 2),
                },
            ],
        )
        .unwrap();
        let placement_directory =
            SpirePlacementDirectory::from_entries(7, vec![first_placement, second_placement])
                .unwrap();
        let snapshot =
            snapshot_for_placement(&epoch_manifest, &object_manifest, &placement_directory);

        assert!(collect_snapshot_visible_primary_rows(&snapshot, &object_store).is_err());
    }

    #[test]
    fn collect_snapshot_rows_dispatches_leaf_and_delta_objects() {
        let mut pid_allocator = SpirePidAllocator::default();
        let mut local_vec_id_allocator = SpireLocalVecIdAllocator::default();
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let base_draft = build_single_level_leaf_epoch_draft(
            build_input(vec![assignment_input(10, 1)]),
            &mut pid_allocator,
            &mut local_vec_id_allocator,
            &mut object_store,
        )
        .unwrap();
        let base_snapshot = SpirePublishedEpochSnapshot::new(
            &base_draft.epoch_manifest,
            &base_draft.object_manifest,
            &base_draft.placement_directory,
        )
        .unwrap();
        let delta_draft = build_delta_epoch_draft_from_snapshot(
            delta_input(
                vec![assignment_input(20, 1)],
                vec![delete_delta_input(1, 10, 1)],
            ),
            &base_snapshot,
            &mut pid_allocator,
            &mut local_vec_id_allocator,
            &mut object_store,
        )
        .unwrap();
        let snapshot = SpirePublishedEpochSnapshot::new(
            &delta_draft.epoch_manifest,
            &delta_draft.object_manifest,
            &delta_draft.placement_directory,
        )
        .unwrap();

        let leaf_rows = collect_snapshot_leaf_rows(&snapshot, &object_store).unwrap();
        let delta_rows = collect_snapshot_delta_rows(&snapshot, &object_store).unwrap();
        let visible_rows = collect_snapshot_visible_primary_rows(&snapshot, &object_store).unwrap();

        assert_eq!(leaf_rows.len(), 1);
        assert_eq!(leaf_rows[0].pid, SPIRE_FIRST_PID);
        assert_eq!(leaf_rows[0].assignment.heap_tid, tid(10, 1));
        assert_eq!(delta_rows.len(), 2);
        assert_eq!(delta_rows[0].pid, SPIRE_FIRST_PID + 1);
        assert_eq!(
            delta_rows[0].assignment.flags,
            SPIRE_ASSIGNMENT_FLAG_PRIMARY | SPIRE_ASSIGNMENT_FLAG_DELTA_INSERT
        );
        assert_eq!(
            delta_rows[1].assignment.flags,
            SPIRE_ASSIGNMENT_FLAG_TOMBSTONE | SPIRE_ASSIGNMENT_FLAG_DELTA_DELETE
        );
        assert_eq!(visible_rows.len(), 1);
        assert_eq!(visible_rows[0].pid, SPIRE_FIRST_PID + 1);
        assert_eq!(visible_rows[0].assignment.heap_tid, tid(20, 1));
        assert_eq!(visible_rows[0].assignment.vec_id.local_sequence(), Some(2));
    }
}
