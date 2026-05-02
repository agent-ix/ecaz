use super::assign::{
    build_primary_leaf_assignments, SpireLeafAssignmentInput, SpireLocalVecIdAllocator,
    SpirePidAllocator,
};
use super::meta::{
    SpireConsistencyMode, SpireEpochManifest, SpireEpochState, SpireManifestEntry,
    SpireObjectManifest, SpirePlacementDirectory, SpirePublishedEpochSnapshot,
    SpireRootControlState,
};
use super::storage::{SpireLeafPartitionObject, SpireLocalObjectStore};
use crate::storage::page::ItemPointer;
use pgrx::pg_sys;

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SpireSingleLevelBuildInput {
    pub(super) epoch: u64,
    pub(super) object_version: u64,
    pub(super) published_at_micros: i64,
    pub(super) retain_until_micros: i64,
    pub(super) consistency_mode: SpireConsistencyMode,
    pub(super) placement_tid: ItemPointer,
    pub(super) assignments: Vec<SpireLeafAssignmentInput>,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SpireSingleLevelBuildDraft {
    pub(super) epoch_manifest: SpireEpochManifest,
    pub(super) object_manifest: SpireObjectManifest,
    pub(super) placement_directory: SpirePlacementDirectory,
    pub(super) leaf_object: SpireLeafPartitionObject,
    pub(super) next_pid: u64,
    pub(super) next_local_vec_seq: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct SpireEncodedManifestBundle {
    pub(super) epoch_manifest: Vec<u8>,
    pub(super) object_manifest: Vec<u8>,
    pub(super) placement_directory: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct SpireEncodedPublishBundle {
    pub(super) manifests: SpireEncodedManifestBundle,
    pub(super) root_control_state: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct SpirePublishedManifestLocators {
    pub(super) epoch_manifest_tid: ItemPointer,
    pub(super) object_manifest_tid: ItemPointer,
    pub(super) placement_directory_tid: ItemPointer,
}

impl SpireSingleLevelBuildDraft {
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

pub(super) fn build_single_level_leaf_epoch_draft(
    input: SpireSingleLevelBuildInput,
    pid_allocator: &mut SpirePidAllocator,
    local_vec_id_allocator: &mut SpireLocalVecIdAllocator,
    object_store: &mut SpireLocalObjectStore,
) -> Result<SpireSingleLevelBuildDraft, String> {
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
    let pid = pid_cursor.allocate()?;
    let object_manifest = SpireObjectManifest::from_entries(
        input.epoch,
        vec![SpireManifestEntry {
            epoch: input.epoch,
            pid,
            object_version: input.object_version,
            placement_tid: input.placement_tid,
        }],
    )?;
    let assignments = build_primary_leaf_assignments(&mut local_vec_id_cursor, input.assignments)?;
    let leaf_object = SpireLeafPartitionObject::new(pid, input.object_version, 0, assignments)?;
    let placement = object_store.insert_leaf_object(input.epoch, &leaf_object)?;
    let placement_directory = SpirePlacementDirectory::from_entries(input.epoch, vec![placement])?;

    let draft = SpireSingleLevelBuildDraft {
        epoch_manifest,
        object_manifest,
        placement_directory,
        leaf_object,
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

pub(super) unsafe extern "C-unwind" fn ec_spire_ambuild(
    _heap_relation: pg_sys::Relation,
    _index_relation: pg_sys::Relation,
    _index_info: *mut pg_sys::IndexInfo,
) -> *mut pg_sys::IndexBuildResult {
    unsafe { pgrx::pgrx_extern_c_guard(|| super::not_implemented("ambuild")) }
}

pub(super) unsafe extern "C-unwind" fn ec_spire_ambuildempty(_index_relation: pg_sys::Relation) {
    unsafe { pgrx::pgrx_extern_c_guard(|| super::not_implemented("ambuildempty")) }
}

#[cfg(test)]
mod tests {
    use super::{build_single_level_leaf_epoch_draft, SpireSingleLevelBuildInput};
    use super::{SpirePublishedManifestLocators, SpireSingleLevelBuildDraft};
    use crate::am::ec_spire::assign::{
        SpireLeafAssignmentInput, SpireLocalVecIdAllocator, SpirePidAllocator,
        SPIRE_FIRST_LOCAL_VEC_SEQ, SPIRE_FIRST_PID,
    };
    use crate::am::ec_spire::meta::{SpireConsistencyMode, SpirePublishedEpochSnapshot};
    use crate::am::ec_spire::meta::{
        SpireEpochManifest, SpireObjectManifest, SpirePlacementDirectory, SpireRootControlState,
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

    fn build_valid_draft() -> (
        SpireSingleLevelBuildDraft,
        SpirePidAllocator,
        SpireLocalVecIdAllocator,
        SpireLocalObjectStore,
    ) {
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

        (draft, pid_allocator, local_vec_id_allocator, object_store)
    }

    fn manifest_locators() -> SpirePublishedManifestLocators {
        SpirePublishedManifestLocators {
            epoch_manifest_tid: tid(70, 1),
            object_manifest_tid: tid(70, 2),
            placement_directory_tid: tid(70, 3),
        }
    }

    #[test]
    fn single_level_draft_builds_leaf_object_and_published_snapshot() {
        let (draft, pid_allocator, local_vec_id_allocator, object_store) = build_valid_draft();

        let placement = draft.placement_directory.get(SPIRE_FIRST_PID).unwrap();
        let stored_leaf = object_store.read_leaf_object(placement).unwrap();

        assert_eq!(draft.epoch_manifest.epoch, 7);
        assert_eq!(draft.leaf_object.header.pid, SPIRE_FIRST_PID);
        assert_eq!(draft.leaf_object.header.object_version, 1);
        assert_eq!(draft.leaf_object.assignments.len(), 2);
        assert_eq!(stored_leaf, draft.leaf_object);
        assert_eq!(
            draft
                .object_manifest
                .get(SPIRE_FIRST_PID)
                .unwrap()
                .placement_tid,
            tid(60, 1)
        );
        SpirePublishedEpochSnapshot::new(
            &draft.epoch_manifest,
            &draft.object_manifest,
            &draft.placement_directory,
        )
        .unwrap();
        assert_eq!(draft.next_pid, SPIRE_FIRST_PID + 1);
        assert_eq!(draft.next_local_vec_seq, SPIRE_FIRST_LOCAL_VEC_SEQ + 2);
        assert_eq!(pid_allocator.next_pid(), draft.next_pid);
        assert_eq!(
            local_vec_id_allocator.next_local_vec_seq(),
            draft.next_local_vec_seq
        );
    }

    #[test]
    fn single_level_draft_builds_root_control_state_from_manifest_locators() {
        let (draft, _, _, _) = build_valid_draft();
        let root_control = draft.root_control_state(manifest_locators()).unwrap();

        assert_eq!(root_control.active_epoch, draft.epoch_manifest.epoch);
        assert_eq!(root_control.next_pid, draft.next_pid);
        assert_eq!(root_control.next_local_vec_seq, draft.next_local_vec_seq);
        assert_eq!(root_control.epoch_manifest_tid, tid(70, 1));
        assert_eq!(root_control.object_manifest_tid, tid(70, 2));
        assert_eq!(root_control.placement_directory_tid, tid(70, 3));
    }

    #[test]
    fn single_level_draft_rejects_invalid_root_control_manifest_locator() {
        let (draft, _, _, _) = build_valid_draft();
        let mut locators = manifest_locators();
        locators.object_manifest_tid = ItemPointer::INVALID;

        assert!(draft.root_control_state(locators).is_err());
    }

    #[test]
    fn single_level_draft_encodes_manifest_bundle() {
        let (draft, _, _, _) = build_valid_draft();

        let encoded = draft.encode_manifest_bundle().unwrap();

        assert_eq!(
            SpireEpochManifest::decode(&encoded.epoch_manifest).unwrap(),
            draft.epoch_manifest
        );
        assert_eq!(
            SpireObjectManifest::decode(&encoded.object_manifest).unwrap(),
            draft.object_manifest
        );
        assert_eq!(
            SpirePlacementDirectory::decode(&encoded.placement_directory).unwrap(),
            draft.placement_directory
        );
    }

    #[test]
    fn single_level_draft_encodes_publish_bundle() {
        let (draft, _, _, _) = build_valid_draft();

        let encoded = draft.encode_publish_bundle(manifest_locators()).unwrap();
        let root_control = SpireRootControlState::decode(&encoded.root_control_state).unwrap();

        assert_eq!(
            SpireEpochManifest::decode(&encoded.manifests.epoch_manifest).unwrap(),
            draft.epoch_manifest
        );
        assert_eq!(root_control.active_epoch, draft.epoch_manifest.epoch);
        assert_eq!(root_control.next_pid, draft.next_pid);
        assert_eq!(root_control.next_local_vec_seq, draft.next_local_vec_seq);
        assert_eq!(root_control.epoch_manifest_tid, tid(70, 1));
        assert_eq!(root_control.object_manifest_tid, tid(70, 2));
        assert_eq!(root_control.placement_directory_tid, tid(70, 3));
    }

    #[test]
    fn single_level_draft_rejects_invalid_assignment_without_advancing_allocators() {
        let mut pid_allocator = SpirePidAllocator::default();
        let mut local_vec_id_allocator = SpireLocalVecIdAllocator::default();
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let mut bad_assignment = assignment_input(10, 1);
        bad_assignment.heap_tid = ItemPointer::INVALID;

        assert!(build_single_level_leaf_epoch_draft(
            build_input(vec![bad_assignment]),
            &mut pid_allocator,
            &mut local_vec_id_allocator,
            &mut object_store,
        )
        .is_err());
        assert_eq!(pid_allocator.next_pid(), SPIRE_FIRST_PID);
        assert_eq!(
            local_vec_id_allocator.next_local_vec_seq(),
            SPIRE_FIRST_LOCAL_VEC_SEQ
        );
    }

    #[test]
    fn single_level_draft_rejects_invalid_manifest_locator_without_advancing_allocators() {
        let mut pid_allocator = SpirePidAllocator::default();
        let mut local_vec_id_allocator = SpireLocalVecIdAllocator::default();
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let mut input = build_input(vec![assignment_input(10, 1)]);
        input.placement_tid = ItemPointer::INVALID;

        assert!(build_single_level_leaf_epoch_draft(
            input,
            &mut pid_allocator,
            &mut local_vec_id_allocator,
            &mut object_store,
        )
        .is_err());
        assert_eq!(pid_allocator.next_pid(), SPIRE_FIRST_PID);
        assert_eq!(
            local_vec_id_allocator.next_local_vec_seq(),
            SPIRE_FIRST_LOCAL_VEC_SEQ
        );
    }

    #[test]
    fn single_level_draft_rejects_invalid_publish_timestamp_without_advancing_allocators() {
        let mut pid_allocator = SpirePidAllocator::default();
        let mut local_vec_id_allocator = SpireLocalVecIdAllocator::default();
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let mut input = build_input(vec![assignment_input(10, 1)]);
        input.published_at_micros = 0;

        assert!(build_single_level_leaf_epoch_draft(
            input,
            &mut pid_allocator,
            &mut local_vec_id_allocator,
            &mut object_store,
        )
        .is_err());
        assert_eq!(pid_allocator.next_pid(), SPIRE_FIRST_PID);
        assert_eq!(
            local_vec_id_allocator.next_local_vec_seq(),
            SPIRE_FIRST_LOCAL_VEC_SEQ
        );
    }
}
