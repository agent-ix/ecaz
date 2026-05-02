//! Epoch-published insert/delete, split, merge, and cleanup mechanics live here.

use super::assign::{
    build_delete_delta_assignments, build_insert_delta_assignments, SpireDeleteDeltaInput,
    SpireLeafAssignmentInput, SpireLocalVecIdAllocator, SpirePidAllocator,
};
use super::meta::{
    SpireConsistencyMode, SpireEpochManifest, SpireEpochState, SpireManifestEntry,
    SpireObjectManifest, SpirePlacementDirectory, SpirePublishedEpochSnapshot,
};
use super::storage::{SpireDeltaPartitionObject, SpireLocalObjectStore};
use crate::storage::page::ItemPointer;

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SpireDeltaEpochInput {
    pub(super) epoch: u64,
    pub(super) object_version: u64,
    pub(super) published_at_micros: i64,
    pub(super) retain_until_micros: i64,
    pub(super) consistency_mode: SpireConsistencyMode,
    pub(super) base_pid: u64,
    pub(super) placement_tid: ItemPointer,
    pub(super) insert_assignments: Vec<SpireLeafAssignmentInput>,
    pub(super) delete_assignments: Vec<SpireDeleteDeltaInput>,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SpireDeltaEpochDraft {
    pub(super) epoch_manifest: SpireEpochManifest,
    pub(super) object_manifest: SpireObjectManifest,
    pub(super) placement_directory: SpirePlacementDirectory,
    pub(super) delta_object: SpireDeltaPartitionObject,
    pub(super) next_pid: u64,
    pub(super) next_local_vec_seq: u64,
}

pub(super) fn build_delta_epoch_draft(
    input: SpireDeltaEpochInput,
    pid_allocator: &mut SpirePidAllocator,
    local_vec_id_allocator: &mut SpireLocalVecIdAllocator,
    object_store: &mut SpireLocalObjectStore,
) -> Result<SpireDeltaEpochDraft, String> {
    if input.insert_assignments.is_empty() && input.delete_assignments.is_empty() {
        return Err("ec_spire delta epoch draft must contain at least one assignment".to_owned());
    }

    let epoch_manifest = SpireEpochManifest {
        epoch: input.epoch,
        state: SpireEpochState::Published,
        consistency_mode: input.consistency_mode,
        published_at_micros: input.published_at_micros,
        retain_until_micros: input.retain_until_micros,
        active_query_count: 0,
    };
    epoch_manifest.validate()?;

    let mut pid_cursor = *pid_allocator;
    let mut local_vec_id_cursor = *local_vec_id_allocator;
    let delta_pid = pid_cursor.allocate()?;
    let object_manifest = SpireObjectManifest::from_entries(
        input.epoch,
        vec![SpireManifestEntry {
            epoch: input.epoch,
            pid: delta_pid,
            object_version: input.object_version,
            placement_tid: input.placement_tid,
        }],
    )?;

    let mut assignments =
        build_insert_delta_assignments(&mut local_vec_id_cursor, input.insert_assignments)?;
    assignments.extend(build_delete_delta_assignments(input.delete_assignments)?);
    let delta_object = SpireDeltaPartitionObject::new(
        delta_pid,
        input.object_version,
        input.base_pid,
        assignments,
    )?;
    let placement = object_store.insert_delta_object(input.epoch, &delta_object)?;
    let placement_directory = SpirePlacementDirectory::from_entries(input.epoch, vec![placement])?;

    let draft = SpireDeltaEpochDraft {
        epoch_manifest,
        object_manifest,
        placement_directory,
        delta_object,
        next_pid: pid_cursor.next_pid(),
        next_local_vec_seq: local_vec_id_cursor.next_local_vec_seq(),
    };
    SpirePublishedEpochSnapshot::new(
        &draft.epoch_manifest,
        &draft.object_manifest,
        &draft.placement_directory,
    )?;

    *pid_allocator = pid_cursor;
    *local_vec_id_allocator = local_vec_id_cursor;
    Ok(draft)
}

#[cfg(test)]
mod tests {
    use super::{build_delta_epoch_draft, SpireDeltaEpochInput};
    use crate::am::ec_spire::assign::{
        SpireDeleteDeltaInput, SpireLeafAssignmentInput, SpireLocalVecIdAllocator,
        SpirePidAllocator,
    };
    use crate::am::ec_spire::meta::{SpireConsistencyMode, SpirePublishedEpochSnapshot};
    use crate::am::ec_spire::storage::{
        SpireLocalObjectStore, SpireVecId, SPIRE_ASSIGNMENT_FLAG_DELTA_DELETE,
        SPIRE_ASSIGNMENT_FLAG_DELTA_INSERT, SPIRE_ASSIGNMENT_FLAG_PRIMARY,
        SPIRE_ASSIGNMENT_FLAG_TOMBSTONE,
    };
    use crate::storage::page::ItemPointer;

    fn tid(block_number: u32, offset_number: u16) -> ItemPointer {
        ItemPointer {
            block_number,
            offset_number,
        }
    }

    fn insert_assignment(block_number: u32, offset_number: u16) -> SpireLeafAssignmentInput {
        SpireLeafAssignmentInput {
            heap_tid: tid(block_number, offset_number),
            payload_format: 1,
            gamma: 0.5,
            encoded_payload: vec![1, 2, 3],
        }
    }

    fn delete_assignment(
        vec_seq: u64,
        block_number: u32,
        offset_number: u16,
    ) -> SpireDeleteDeltaInput {
        SpireDeleteDeltaInput {
            vec_id: SpireVecId::local(vec_seq),
            heap_tid: tid(block_number, offset_number),
        }
    }

    fn delta_input(
        insert_assignments: Vec<SpireLeafAssignmentInput>,
        delete_assignments: Vec<SpireDeleteDeltaInput>,
    ) -> SpireDeltaEpochInput {
        SpireDeltaEpochInput {
            epoch: 8,
            object_version: 3,
            published_at_micros: 1000,
            retain_until_micros: 2000,
            consistency_mode: SpireConsistencyMode::Strict,
            base_pid: 11,
            placement_tid: tid(80, 1),
            insert_assignments,
            delete_assignments,
        }
    }

    #[test]
    fn delta_epoch_draft_writes_delta_object_and_published_snapshot() {
        let mut pid_allocator = SpirePidAllocator::new(50).unwrap();
        let mut local_vec_id_allocator = SpireLocalVecIdAllocator::default();
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();

        let draft = build_delta_epoch_draft(
            delta_input(
                vec![insert_assignment(20, 1), insert_assignment(20, 2)],
                vec![delete_assignment(99, 21, 1)],
            ),
            &mut pid_allocator,
            &mut local_vec_id_allocator,
            &mut object_store,
        )
        .unwrap();

        let placement = draft.placement_directory.get(50).unwrap();
        let stored_delta = object_store.read_delta_object(placement).unwrap();

        assert_eq!(stored_delta, draft.delta_object);
        assert_eq!(draft.epoch_manifest.epoch, 8);
        assert_eq!(draft.delta_object.header.pid, 50);
        assert_eq!(draft.delta_object.header.object_version, 3);
        assert_eq!(draft.delta_object.header.parent_pid, 11);
        assert_eq!(draft.delta_object.assignments.len(), 3);
        assert_eq!(
            draft.delta_object.assignments[0].flags,
            SPIRE_ASSIGNMENT_FLAG_PRIMARY | SPIRE_ASSIGNMENT_FLAG_DELTA_INSERT
        );
        assert_eq!(
            draft.delta_object.assignments[2].flags,
            SPIRE_ASSIGNMENT_FLAG_TOMBSTONE | SPIRE_ASSIGNMENT_FLAG_DELTA_DELETE
        );
        assert_eq!(
            draft.object_manifest.get(50).unwrap().placement_tid,
            tid(80, 1)
        );
        SpirePublishedEpochSnapshot::new(
            &draft.epoch_manifest,
            &draft.object_manifest,
            &draft.placement_directory,
        )
        .unwrap();
        assert_eq!(draft.next_pid, 51);
        assert_eq!(draft.next_local_vec_seq, 3);
        assert_eq!(pid_allocator.next_pid(), draft.next_pid);
        assert_eq!(
            local_vec_id_allocator.next_local_vec_seq(),
            draft.next_local_vec_seq
        );
    }

    #[test]
    fn delta_epoch_draft_rejects_empty_delta_without_advancing_allocators() {
        let mut pid_allocator = SpirePidAllocator::new(50).unwrap();
        let mut local_vec_id_allocator = SpireLocalVecIdAllocator::default();
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let initial_page_count = object_store.page_count();

        assert!(build_delta_epoch_draft(
            delta_input(Vec::new(), Vec::new()),
            &mut pid_allocator,
            &mut local_vec_id_allocator,
            &mut object_store,
        )
        .is_err());
        assert_eq!(pid_allocator.next_pid(), 50);
        assert_eq!(local_vec_id_allocator.next_local_vec_seq(), 1);
        assert_eq!(object_store.page_count(), initial_page_count);
    }

    #[test]
    fn delta_epoch_draft_rejects_invalid_base_pid_without_advancing_allocators() {
        let mut pid_allocator = SpirePidAllocator::new(50).unwrap();
        let mut local_vec_id_allocator = SpireLocalVecIdAllocator::default();
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let initial_page_count = object_store.page_count();
        let mut input = delta_input(vec![insert_assignment(20, 1)], Vec::new());
        input.base_pid = 0;

        assert!(build_delta_epoch_draft(
            input,
            &mut pid_allocator,
            &mut local_vec_id_allocator,
            &mut object_store,
        )
        .is_err());
        assert_eq!(pid_allocator.next_pid(), 50);
        assert_eq!(local_vec_id_allocator.next_local_vec_seq(), 1);
        assert_eq!(object_store.page_count(), initial_page_count);
    }

    #[test]
    fn delta_epoch_draft_rejects_invalid_assignment_without_advancing_allocators() {
        let mut pid_allocator = SpirePidAllocator::new(50).unwrap();
        let mut local_vec_id_allocator = SpireLocalVecIdAllocator::default();
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let initial_page_count = object_store.page_count();
        let mut bad_assignment = insert_assignment(20, 1);
        bad_assignment.heap_tid = ItemPointer::INVALID;

        assert!(build_delta_epoch_draft(
            delta_input(vec![bad_assignment], vec![delete_assignment(99, 21, 1)]),
            &mut pid_allocator,
            &mut local_vec_id_allocator,
            &mut object_store,
        )
        .is_err());
        assert_eq!(pid_allocator.next_pid(), 50);
        assert_eq!(local_vec_id_allocator.next_local_vec_seq(), 1);
        assert_eq!(object_store.page_count(), initial_page_count);
    }
}
