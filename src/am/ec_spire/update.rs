//! Epoch-published insert/delete, split, merge, and cleanup mechanics live here.

use std::collections::HashSet;

use super::assign::{
    build_delete_delta_assignments, build_insert_delta_assignments, SpireDeleteDeltaInput,
    SpireLeafAssignmentInput, SpireLocalVecIdAllocator, SpirePidAllocator,
};
use super::build::{
    SpireEncodedManifestBundle, SpireEncodedPublishBundle, SpirePublishedManifestLocators,
};
use super::meta::{
    SpireConsistencyMode, SpireEpochManifest, SpireEpochState, SpireManifestEntry,
    SpireObjectManifest, SpirePlacementDirectory, SpirePlacementEntry, SpirePublishedEpochSnapshot,
    SpireRootControlState,
};
use super::scan::collect_snapshot_visible_primary_rows;
use super::storage::{
    SpireDeltaPartitionObject, SpireLocalObjectStore, SpirePartitionObjectKind, SpireVecId,
};
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

impl SpireDeltaEpochDraft {
    pub(super) fn encode_manifest_bundle(&self) -> Result<SpireEncodedManifestBundle, String> {
        SpirePublishedEpochSnapshot::new(
            &self.epoch_manifest,
            &self.object_manifest,
            &self.placement_directory,
        )?;
        Ok(SpireEncodedManifestBundle {
            epoch_manifest: self.epoch_manifest.encode()?,
            object_manifest: self.object_manifest.encode()?,
            placement_directory: self.placement_directory.encode()?,
        })
    }

    pub(super) fn root_control_state(
        &self,
        locators: SpirePublishedManifestLocators,
    ) -> Result<SpireRootControlState, String> {
        SpirePublishedEpochSnapshot::new(
            &self.epoch_manifest,
            &self.object_manifest,
            &self.placement_directory,
        )?;
        SpireRootControlState::published(
            self.epoch_manifest.epoch,
            self.next_pid,
            self.next_local_vec_seq,
            locators.epoch_manifest_tid,
            locators.object_manifest_tid,
            locators.placement_directory_tid,
        )
    }

    pub(super) fn encode_publish_bundle(
        &self,
        locators: SpirePublishedManifestLocators,
    ) -> Result<SpireEncodedPublishBundle, String> {
        let manifests = self.encode_manifest_bundle()?;
        let root_control_state = self.root_control_state(locators)?.encode()?;
        Ok(SpireEncodedPublishBundle {
            manifests,
            root_control_state,
        })
    }
}

pub(super) fn build_delta_epoch_draft(
    input: SpireDeltaEpochInput,
    pid_allocator: &mut SpirePidAllocator,
    local_vec_id_allocator: &mut SpireLocalVecIdAllocator,
    object_store: &mut SpireLocalObjectStore,
) -> Result<SpireDeltaEpochDraft, String> {
    build_delta_epoch_draft_with_carried_entries(
        input,
        Vec::new(),
        Vec::new(),
        Vec::new(),
        pid_allocator,
        local_vec_id_allocator,
        object_store,
    )
}

pub(super) fn build_delta_epoch_draft_from_snapshot(
    input: SpireDeltaEpochInput,
    base_snapshot: &SpirePublishedEpochSnapshot<'_>,
    pid_allocator: &mut SpirePidAllocator,
    local_vec_id_allocator: &mut SpireLocalVecIdAllocator,
    object_store: &mut SpireLocalObjectStore,
) -> Result<SpireDeltaEpochDraft, String> {
    SpirePublishedEpochSnapshot::new(
        base_snapshot.epoch_manifest,
        base_snapshot.object_manifest,
        base_snapshot.placement_directory,
    )?;
    if input.epoch <= base_snapshot.epoch_manifest.epoch {
        return Err(format!(
            "ec_spire delta epoch {} must be newer than base epoch {}",
            input.epoch, base_snapshot.epoch_manifest.epoch
        ));
    }
    if base_snapshot.object_manifest.get(input.base_pid).is_none() {
        return Err(format!(
            "ec_spire delta epoch base_pid {} is not present in the base snapshot",
            input.base_pid
        ));
    }
    let epoch = input.epoch;
    let carried_manifest_entries = base_snapshot
        .object_manifest
        .entries
        .iter()
        .cloned()
        .map(|mut entry| {
            entry.epoch = epoch;
            entry
        })
        .collect();
    let carried_placement_entries = base_snapshot
        .placement_directory
        .entries
        .iter()
        .cloned()
        .map(|mut entry| {
            entry.epoch = epoch;
            entry
        })
        .collect();
    let observed_vec_ids = collect_snapshot_assignment_vec_ids(base_snapshot, object_store)?;
    let visible_vec_ids = collect_snapshot_visible_vec_ids(base_snapshot, object_store)?;
    validate_delete_delta_targets(&input.delete_assignments, &visible_vec_ids)?;

    build_delta_epoch_draft_with_carried_entries(
        input,
        carried_manifest_entries,
        carried_placement_entries,
        observed_vec_ids,
        pid_allocator,
        local_vec_id_allocator,
        object_store,
    )
}

fn build_delta_epoch_draft_with_carried_entries(
    input: SpireDeltaEpochInput,
    mut object_entries: Vec<SpireManifestEntry>,
    mut placement_entries: Vec<SpirePlacementEntry>,
    observed_vec_ids: Vec<SpireVecId>,
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
    for entry in &object_entries {
        pid_cursor.observe(entry.pid)?;
    }
    for vec_id in &observed_vec_ids {
        local_vec_id_cursor.observe(vec_id)?;
    }
    let delta_pid = pid_cursor.allocate()?;
    object_entries.push(SpireManifestEntry {
        epoch: input.epoch,
        pid: delta_pid,
        object_version: input.object_version,
        placement_tid: input.placement_tid,
    });
    let object_manifest = SpireObjectManifest::from_entries(input.epoch, object_entries)?;

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
    placement_entries.push(placement);
    let placement_directory =
        SpirePlacementDirectory::from_entries(input.epoch, placement_entries)?;

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

fn collect_snapshot_assignment_vec_ids(
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    object_store: &SpireLocalObjectStore,
) -> Result<Vec<SpireVecId>, String> {
    let mut vec_ids = Vec::new();
    for manifest_entry in &snapshot.object_manifest.entries {
        let placement = snapshot
            .placement_directory
            .get(manifest_entry.pid)
            .ok_or_else(|| {
                format!(
                    "ec_spire delta draft base snapshot missing placement for pid {}",
                    manifest_entry.pid
                )
            })?;
        let header = object_store.read_object_header(placement)?;
        match header.kind {
            SpirePartitionObjectKind::Leaf => {
                let object = object_store.read_leaf_object(placement)?;
                vec_ids.extend(
                    object
                        .assignments
                        .into_iter()
                        .map(|assignment| assignment.vec_id),
                );
            }
            SpirePartitionObjectKind::Delta => {
                let object = object_store.read_delta_object(placement)?;
                vec_ids.extend(
                    object
                        .assignments
                        .into_iter()
                        .map(|assignment| assignment.vec_id),
                );
            }
            SpirePartitionObjectKind::Root | SpirePartitionObjectKind::Internal => {}
        }
    }
    Ok(vec_ids)
}

fn validate_delete_delta_targets(
    delete_assignments: &[SpireDeleteDeltaInput],
    visible_vec_ids: &[SpireVecId],
) -> Result<(), String> {
    let visible: HashSet<_> = visible_vec_ids.iter().cloned().collect();
    let mut seen_deletes = HashSet::new();
    for assignment in delete_assignments {
        if !seen_deletes.insert(assignment.vec_id.clone()) {
            return Err(
                "ec_spire delete delta vec_id appears more than once in the draft".to_owned(),
            );
        }
        if !visible.contains(&assignment.vec_id) {
            return Err(
                "ec_spire delete delta vec_id is not present in the base snapshot".to_owned(),
            );
        }
    }
    Ok(())
}

fn collect_snapshot_visible_vec_ids(
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    object_store: &SpireLocalObjectStore,
) -> Result<Vec<SpireVecId>, String> {
    Ok(
        collect_snapshot_visible_primary_rows(snapshot, object_store)?
            .into_iter()
            .map(|row| row.assignment.vec_id)
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::{
        build_delta_epoch_draft, build_delta_epoch_draft_from_snapshot, SpireDeltaEpochInput,
    };
    use crate::am::ec_spire::assign::{
        SpireDeleteDeltaInput, SpireLeafAssignmentInput, SpireLocalVecIdAllocator,
        SpirePidAllocator,
    };
    use crate::am::ec_spire::build::{
        build_single_level_leaf_epoch_draft, SpirePublishedManifestLocators,
        SpireSingleLevelBuildInput,
    };
    use crate::am::ec_spire::meta::{
        SpireConsistencyMode, SpireEpochManifest, SpireObjectManifest, SpirePlacementDirectory,
        SpirePublishedEpochSnapshot, SpireRootControlState,
    };
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

    fn base_build_input(assignments: Vec<SpireLeafAssignmentInput>) -> SpireSingleLevelBuildInput {
        SpireSingleLevelBuildInput {
            epoch: 7,
            object_version: 1,
            published_at_micros: 900,
            retain_until_micros: 1900,
            consistency_mode: SpireConsistencyMode::Strict,
            placement_tid: tid(70, 1),
            assignments,
        }
    }

    fn manifest_locators() -> SpirePublishedManifestLocators {
        SpirePublishedManifestLocators {
            epoch_manifest_tid: tid(90, 1),
            object_manifest_tid: tid(90, 2),
            placement_directory_tid: tid(90, 3),
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
    fn delta_epoch_draft_encodes_publish_bundle() {
        let mut pid_allocator = SpirePidAllocator::new(50).unwrap();
        let mut local_vec_id_allocator = SpireLocalVecIdAllocator::default();
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let draft = build_delta_epoch_draft(
            delta_input(
                vec![insert_assignment(20, 1)],
                vec![delete_assignment(99, 21, 1)],
            ),
            &mut pid_allocator,
            &mut local_vec_id_allocator,
            &mut object_store,
        )
        .unwrap();

        let encoded = draft.encode_publish_bundle(manifest_locators()).unwrap();
        let root_control = SpireRootControlState::decode(&encoded.root_control_state).unwrap();

        assert_eq!(
            SpireEpochManifest::decode(&encoded.manifests.epoch_manifest).unwrap(),
            draft.epoch_manifest
        );
        assert_eq!(
            SpireObjectManifest::decode(&encoded.manifests.object_manifest).unwrap(),
            draft.object_manifest
        );
        assert_eq!(
            SpirePlacementDirectory::decode(&encoded.manifests.placement_directory).unwrap(),
            draft.placement_directory
        );
        assert_eq!(root_control.active_epoch, draft.epoch_manifest.epoch);
        assert_eq!(root_control.next_pid, draft.next_pid);
        assert_eq!(root_control.next_local_vec_seq, draft.next_local_vec_seq);
        assert_eq!(root_control.epoch_manifest_tid, tid(90, 1));
        assert_eq!(root_control.object_manifest_tid, tid(90, 2));
        assert_eq!(root_control.placement_directory_tid, tid(90, 3));
    }

    #[test]
    fn delta_epoch_draft_rejects_invalid_root_control_locator() {
        let mut pid_allocator = SpirePidAllocator::new(50).unwrap();
        let mut local_vec_id_allocator = SpireLocalVecIdAllocator::default();
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let draft = build_delta_epoch_draft(
            delta_input(vec![insert_assignment(20, 1)], Vec::new()),
            &mut pid_allocator,
            &mut local_vec_id_allocator,
            &mut object_store,
        )
        .unwrap();
        let mut locators = manifest_locators();
        locators.placement_directory_tid = ItemPointer::INVALID;

        assert!(draft.root_control_state(locators).is_err());
    }

    #[test]
    fn delta_epoch_draft_from_snapshot_carries_base_entries() {
        let mut pid_allocator = SpirePidAllocator::default();
        let mut local_vec_id_allocator = SpireLocalVecIdAllocator::default();
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let base_draft = build_single_level_leaf_epoch_draft(
            base_build_input(vec![insert_assignment(10, 1)]),
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

        let mut input = delta_input(
            vec![insert_assignment(20, 1)],
            vec![delete_assignment(1, 10, 1)],
        );
        input.base_pid = 1;
        let draft = build_delta_epoch_draft_from_snapshot(
            input,
            &base_snapshot,
            &mut pid_allocator,
            &mut local_vec_id_allocator,
            &mut object_store,
        )
        .unwrap();

        let base_entry = draft.object_manifest.get(1).unwrap();
        let delta_entry = draft.object_manifest.get(2).unwrap();
        let base_placement = draft.placement_directory.get(1).unwrap();
        let delta_placement = draft.placement_directory.get(2).unwrap();

        assert_eq!(draft.object_manifest.entries.len(), 2);
        assert_eq!(draft.placement_directory.entries.len(), 2);
        assert_eq!(base_entry.epoch, 8);
        assert_eq!(base_entry.object_version, 1);
        assert_eq!(base_entry.placement_tid, tid(70, 1));
        assert_eq!(delta_entry.object_version, 3);
        assert_eq!(base_placement.epoch, 8);
        assert_eq!(base_placement.object_version, 1);
        assert_eq!(delta_placement.epoch, 8);
        assert_eq!(delta_placement.object_version, 3);
        assert_eq!(
            object_store
                .read_leaf_object(base_placement)
                .unwrap()
                .header
                .pid,
            1
        );
        assert_eq!(
            object_store
                .read_delta_object(delta_placement)
                .unwrap()
                .header
                .pid,
            2
        );
        SpirePublishedEpochSnapshot::new(
            &draft.epoch_manifest,
            &draft.object_manifest,
            &draft.placement_directory,
        )
        .unwrap();
        assert_eq!(draft.next_pid, 3);
        assert_eq!(draft.next_local_vec_seq, 3);
        assert_eq!(pid_allocator.next_pid(), 3);
        assert_eq!(local_vec_id_allocator.next_local_vec_seq(), 3);
    }

    #[test]
    fn delta_epoch_draft_from_snapshot_observes_existing_vec_ids_before_allocating() {
        let mut pid_allocator = SpirePidAllocator::default();
        let mut local_vec_id_allocator = SpireLocalVecIdAllocator::default();
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let base_draft = build_single_level_leaf_epoch_draft(
            base_build_input(vec![insert_assignment(10, 1)]),
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
        let mut stale_pid_allocator = SpirePidAllocator::default();
        let mut stale_local_vec_id_allocator = SpireLocalVecIdAllocator::default();
        let mut input = delta_input(vec![insert_assignment(20, 1)], Vec::new());
        input.base_pid = 1;

        let draft = build_delta_epoch_draft_from_snapshot(
            input,
            &base_snapshot,
            &mut stale_pid_allocator,
            &mut stale_local_vec_id_allocator,
            &mut object_store,
        )
        .unwrap();

        assert_eq!(draft.delta_object.header.pid, 2);
        assert_eq!(
            draft.delta_object.assignments[0].vec_id.local_sequence(),
            Some(2)
        );
        assert_eq!(draft.next_pid, 3);
        assert_eq!(draft.next_local_vec_seq, 3);
        assert_eq!(stale_pid_allocator.next_pid(), 3);
        assert_eq!(stale_local_vec_id_allocator.next_local_vec_seq(), 3);
    }

    #[test]
    fn delta_epoch_draft_from_snapshot_rejects_missing_base_pid() {
        let mut pid_allocator = SpirePidAllocator::default();
        let mut local_vec_id_allocator = SpireLocalVecIdAllocator::default();
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let base_draft = build_single_level_leaf_epoch_draft(
            base_build_input(vec![insert_assignment(10, 1)]),
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
        let initial_page_count = object_store.page_count();
        let mut input = delta_input(vec![insert_assignment(20, 1)], Vec::new());
        input.base_pid = 99;

        assert!(build_delta_epoch_draft_from_snapshot(
            input,
            &base_snapshot,
            &mut pid_allocator,
            &mut local_vec_id_allocator,
            &mut object_store,
        )
        .is_err());
        assert_eq!(pid_allocator.next_pid(), 2);
        assert_eq!(local_vec_id_allocator.next_local_vec_seq(), 2);
        assert_eq!(object_store.page_count(), initial_page_count);
    }

    #[test]
    fn delta_epoch_draft_from_snapshot_rejects_unknown_delete_vec_id() {
        let mut pid_allocator = SpirePidAllocator::default();
        let mut local_vec_id_allocator = SpireLocalVecIdAllocator::default();
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let base_draft = build_single_level_leaf_epoch_draft(
            base_build_input(vec![insert_assignment(10, 1)]),
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
        let initial_page_count = object_store.page_count();
        let mut input = delta_input(
            vec![insert_assignment(20, 1)],
            vec![delete_assignment(99, 10, 1)],
        );
        input.base_pid = 1;

        assert!(build_delta_epoch_draft_from_snapshot(
            input,
            &base_snapshot,
            &mut pid_allocator,
            &mut local_vec_id_allocator,
            &mut object_store,
        )
        .is_err());
        assert_eq!(pid_allocator.next_pid(), 2);
        assert_eq!(local_vec_id_allocator.next_local_vec_seq(), 2);
        assert_eq!(object_store.page_count(), initial_page_count);
    }

    #[test]
    fn delta_epoch_draft_from_snapshot_rejects_duplicate_delete_vec_id() {
        let mut pid_allocator = SpirePidAllocator::default();
        let mut local_vec_id_allocator = SpireLocalVecIdAllocator::default();
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let base_draft = build_single_level_leaf_epoch_draft(
            base_build_input(vec![insert_assignment(10, 1)]),
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
        let initial_page_count = object_store.page_count();
        let mut input = delta_input(
            Vec::new(),
            vec![delete_assignment(1, 10, 1), delete_assignment(1, 10, 1)],
        );
        input.base_pid = 1;

        assert!(build_delta_epoch_draft_from_snapshot(
            input,
            &base_snapshot,
            &mut pid_allocator,
            &mut local_vec_id_allocator,
            &mut object_store,
        )
        .is_err());
        assert_eq!(pid_allocator.next_pid(), 2);
        assert_eq!(local_vec_id_allocator.next_local_vec_seq(), 2);
        assert_eq!(object_store.page_count(), initial_page_count);
    }

    #[test]
    fn delta_epoch_draft_from_snapshot_rejects_already_deleted_vec_id() {
        let mut pid_allocator = SpirePidAllocator::default();
        let mut local_vec_id_allocator = SpireLocalVecIdAllocator::default();
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let base_draft = build_single_level_leaf_epoch_draft(
            base_build_input(vec![insert_assignment(10, 1)]),
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
        let mut first_delete = delta_input(Vec::new(), vec![delete_assignment(1, 10, 1)]);
        first_delete.base_pid = 1;
        let first_delta = build_delta_epoch_draft_from_snapshot(
            first_delete,
            &base_snapshot,
            &mut pid_allocator,
            &mut local_vec_id_allocator,
            &mut object_store,
        )
        .unwrap();
        let deleted_snapshot = SpirePublishedEpochSnapshot::new(
            &first_delta.epoch_manifest,
            &first_delta.object_manifest,
            &first_delta.placement_directory,
        )
        .unwrap();
        let initial_page_count = object_store.page_count();
        let mut duplicate_delete = delta_input(Vec::new(), vec![delete_assignment(1, 10, 1)]);
        duplicate_delete.epoch = 9;
        duplicate_delete.base_pid = 1;

        assert!(build_delta_epoch_draft_from_snapshot(
            duplicate_delete,
            &deleted_snapshot,
            &mut pid_allocator,
            &mut local_vec_id_allocator,
            &mut object_store,
        )
        .is_err());
        assert_eq!(pid_allocator.next_pid(), 3);
        assert_eq!(local_vec_id_allocator.next_local_vec_seq(), 2);
        assert_eq!(object_store.page_count(), initial_page_count);
    }

    #[test]
    fn delta_epoch_draft_from_snapshot_rejects_non_newer_epoch() {
        let mut pid_allocator = SpirePidAllocator::default();
        let mut local_vec_id_allocator = SpireLocalVecIdAllocator::default();
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let base_draft = build_single_level_leaf_epoch_draft(
            base_build_input(vec![insert_assignment(10, 1)]),
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
        let initial_page_count = object_store.page_count();
        let mut input = delta_input(vec![insert_assignment(20, 1)], Vec::new());
        input.epoch = base_draft.epoch_manifest.epoch;
        input.base_pid = 1;

        assert!(build_delta_epoch_draft_from_snapshot(
            input,
            &base_snapshot,
            &mut pid_allocator,
            &mut local_vec_id_allocator,
            &mut object_store,
        )
        .is_err());
        assert_eq!(pid_allocator.next_pid(), 2);
        assert_eq!(local_vec_id_allocator.next_local_vec_seq(), 2);
        assert_eq!(object_store.page_count(), initial_page_count);
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
