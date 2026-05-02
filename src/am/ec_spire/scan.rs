use super::meta::{SpireConsistencyMode, SpirePlacementState, SpirePublishedEpochSnapshot};
use super::storage::{SpireLeafAssignmentRow, SpireLocalObjectStore};
use pgrx::pg_sys;

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SpireLeafScanRow {
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
    use super::collect_snapshot_leaf_rows;
    use crate::am::ec_spire::assign::{
        SpireLeafAssignmentInput, SpireLocalVecIdAllocator, SpirePidAllocator, SPIRE_FIRST_PID,
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
}
