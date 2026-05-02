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
use crate::am::common::training as common_training;
use crate::storage::page::ItemPointer;
use pgrx::pg_sys;

const SPIRE_DEFAULT_KMEANS_ITERATIONS: usize = 8;

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

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SpirePartitionedSingleLevelBuildInput {
    pub(super) epoch: u64,
    pub(super) object_version: u64,
    pub(super) published_at_micros: i64,
    pub(super) retain_until_micros: i64,
    pub(super) consistency_mode: SpireConsistencyMode,
    pub(super) placement_tids: Vec<ItemPointer>,
    pub(super) assignments: Vec<SpireLeafAssignmentInput>,
    pub(super) centroid_plan: SpireSingleLevelCentroidPlan,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SpirePartitionedSingleLevelBuildDraft {
    pub(super) epoch_manifest: SpireEpochManifest,
    pub(super) object_manifest: SpireObjectManifest,
    pub(super) placement_directory: SpirePlacementDirectory,
    pub(super) centroid_pids: Vec<u64>,
    pub(super) leaf_objects: Vec<SpireLeafPartitionObject>,
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

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SpireSingleLevelCentroidPlan {
    pub(super) dimensions: u16,
    pub(super) centroids: Vec<Vec<f32>>,
    pub(super) assignment_indexes: Vec<u32>,
}

impl SpireSingleLevelCentroidPlan {
    pub(super) fn centroid_count(&self) -> usize {
        self.centroids.len()
    }

    fn validate(&self) -> Result<(), String> {
        if self.dimensions == 0 {
            return Err("ec_spire centroid plan requires dimensions > 0".to_owned());
        }
        if self.centroids.is_empty() {
            return Err("ec_spire centroid plan requires at least one centroid".to_owned());
        }
        let dimensions = usize::from(self.dimensions);
        for (index, centroid) in self.centroids.iter().enumerate() {
            if centroid.len() != dimensions {
                return Err(format!(
                    "ec_spire centroid {index} dimensions mismatch: got {}, expected {dimensions}",
                    centroid.len()
                ));
            }
            if centroid.iter().any(|component| !component.is_finite()) {
                return Err(format!("ec_spire centroid {index} must be finite"));
            }
        }
        for &assignment_index in &self.assignment_indexes {
            let centroid_index = usize::try_from(assignment_index)
                .map_err(|_| "ec_spire centroid assignment index exceeds usize".to_owned())?;
            if centroid_index >= self.centroids.len() {
                return Err(format!(
                    "ec_spire centroid assignment index {centroid_index} exceeds centroid count {}",
                    self.centroids.len()
                ));
            }
        }
        Ok(())
    }
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

impl SpirePartitionedSingleLevelBuildDraft {
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

pub(super) fn train_single_level_centroid_plan(
    dimensions: u16,
    source_vectors: &[Vec<f32>],
    requested_nlists: u32,
    seed: u64,
) -> Result<SpireSingleLevelCentroidPlan, String> {
    if dimensions == 0 {
        return Err("ec_spire centroid plan requires dimensions > 0".to_owned());
    }
    let nlists = common_training::resolve_auto_nlists(requested_nlists, source_vectors.len());
    let source_refs = source_vectors.iter().map(Vec::as_slice).collect::<Vec<_>>();
    let model = common_training::train_spherical_kmeans(
        "ec_spire",
        &source_refs,
        usize::from(dimensions),
        nlists,
        seed,
        SPIRE_DEFAULT_KMEANS_ITERATIONS,
    )?;
    let mut assignment_indexes = Vec::with_capacity(source_vectors.len());
    for source in source_vectors {
        let assignment_index =
            common_training::assign_vector_to_centroid("ec_spire", source, &model)?;
        assignment_indexes.push(
            u32::try_from(assignment_index)
                .map_err(|_| "ec_spire centroid assignment index exceeds u32".to_owned())?,
        );
    }

    Ok(SpireSingleLevelCentroidPlan {
        dimensions,
        centroids: model.centroids,
        assignment_indexes,
    })
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

pub(super) fn build_partitioned_single_level_leaf_epoch_draft(
    input: SpirePartitionedSingleLevelBuildInput,
    pid_allocator: &mut SpirePidAllocator,
    local_vec_id_allocator: &mut SpireLocalVecIdAllocator,
    object_store: &mut SpireLocalObjectStore,
) -> Result<SpirePartitionedSingleLevelBuildDraft, String> {
    let epoch_manifest = SpireEpochManifest {
        epoch: input.epoch,
        state: SpireEpochState::Published,
        consistency_mode: input.consistency_mode,
        published_at_micros: input.published_at_micros,
        retain_until_micros: input.retain_until_micros,
        active_query_count: 0,
    };
    epoch_manifest.validate()?;

    input.centroid_plan.validate()?;
    let centroid_count = input.centroid_plan.centroid_count();
    if input.assignments.len() != input.centroid_plan.assignment_indexes.len() {
        return Err(format!(
            "ec_spire partitioned build assignment count {} does not match centroid assignment count {}",
            input.assignments.len(),
            input.centroid_plan.assignment_indexes.len()
        ));
    }
    if input.placement_tids.len() != centroid_count {
        return Err(format!(
            "ec_spire partitioned build placement count {} does not match centroid count {centroid_count}",
            input.placement_tids.len()
        ));
    }

    let mut assignments_by_centroid = vec![Vec::new(); centroid_count];
    for (assignment, assignment_index) in input
        .assignments
        .into_iter()
        .zip(input.centroid_plan.assignment_indexes.into_iter())
    {
        let centroid_index = usize::try_from(assignment_index)
            .map_err(|_| "ec_spire centroid assignment index exceeds usize".to_owned())?;
        let assignments = assignments_by_centroid.get_mut(centroid_index).ok_or_else(|| {
            format!(
                "ec_spire centroid assignment index {centroid_index} exceeds centroid count {centroid_count}"
            )
        })?;
        assignments.push(assignment);
    }

    let mut pid_cursor = *pid_allocator;
    let mut local_vec_id_cursor = *local_vec_id_allocator;
    let mut centroid_pids = Vec::with_capacity(centroid_count);
    for _ in 0..centroid_count {
        centroid_pids.push(pid_cursor.allocate()?);
    }

    let object_manifest = SpireObjectManifest::from_entries(
        input.epoch,
        centroid_pids
            .iter()
            .zip(input.placement_tids.iter())
            .map(|(&pid, &placement_tid)| SpireManifestEntry {
                epoch: input.epoch,
                pid,
                object_version: input.object_version,
                placement_tid,
            })
            .collect(),
    )?;

    let mut leaf_objects = Vec::with_capacity(centroid_count);
    for (pid, assignments) in centroid_pids
        .iter()
        .copied()
        .zip(assignments_by_centroid.into_iter())
    {
        let assignments = build_primary_leaf_assignments(&mut local_vec_id_cursor, assignments)?;
        let leaf_object = SpireLeafPartitionObject::new(pid, input.object_version, 0, assignments)?;
        leaf_objects.push(leaf_object);
    }
    let mut placements = Vec::with_capacity(centroid_count);
    for leaf_object in &leaf_objects {
        placements.push(object_store.insert_leaf_object(input.epoch, leaf_object)?);
    }
    let placement_directory = SpirePlacementDirectory::from_entries(input.epoch, placements)?;

    let draft = SpirePartitionedSingleLevelBuildDraft {
        epoch_manifest,
        object_manifest,
        placement_directory,
        centroid_pids,
        leaf_objects,
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
    use super::{
        build_partitioned_single_level_leaf_epoch_draft, build_single_level_leaf_epoch_draft,
        train_single_level_centroid_plan, SpirePartitionedSingleLevelBuildInput,
        SpireSingleLevelBuildInput, SpireSingleLevelCentroidPlan,
    };
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

    fn partitioned_build_input(
        assignments: Vec<SpireLeafAssignmentInput>,
        centroid_plan: SpireSingleLevelCentroidPlan,
    ) -> SpirePartitionedSingleLevelBuildInput {
        SpirePartitionedSingleLevelBuildInput {
            epoch: 7,
            object_version: 1,
            published_at_micros: 1000,
            retain_until_micros: 2000,
            consistency_mode: SpireConsistencyMode::Strict,
            placement_tids: vec![tid(60, 1), tid(60, 2)],
            assignments,
            centroid_plan,
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
    fn single_level_centroid_plan_routes_vectors_with_common_training() {
        let source_vectors = vec![vec![1.0, 0.0], vec![-1.0, 0.0]];

        let plan = train_single_level_centroid_plan(2, &source_vectors, 2, 42).unwrap();

        assert_eq!(plan.dimensions, 2);
        assert_eq!(plan.centroid_count(), 2);
        assert_eq!(plan.assignment_indexes.len(), source_vectors.len());
        assert_ne!(plan.assignment_indexes[0], plan.assignment_indexes[1]);
    }

    #[test]
    fn single_level_centroid_plan_resolves_auto_nlists_and_rejects_bad_vectors() {
        let source_vectors = vec![vec![1.0, 0.0], vec![0.0, 1.0]];
        let plan = train_single_level_centroid_plan(2, &source_vectors, 0, 42).unwrap();

        assert_eq!(plan.centroid_count(), 2);

        assert!(train_single_level_centroid_plan(2, &[vec![1.0]], 1, 42)
            .unwrap_err()
            .contains("dimensions mismatch"));
        assert!(
            train_single_level_centroid_plan(2, &[vec![0.0, 0.0]], 1, 42)
                .unwrap_err()
                .contains("non-zero")
        );
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

    #[test]
    fn partitioned_single_level_draft_writes_leaf_objects_per_centroid() {
        let source_vectors = vec![vec![1.0, 0.0], vec![-1.0, 0.0]];
        let centroid_plan = train_single_level_centroid_plan(2, &source_vectors, 2, 42).unwrap();
        let mut pid_allocator = SpirePidAllocator::default();
        let mut local_vec_id_allocator = SpireLocalVecIdAllocator::default();
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();

        let draft = build_partitioned_single_level_leaf_epoch_draft(
            partitioned_build_input(
                vec![assignment_input(10, 1), assignment_input(10, 2)],
                centroid_plan,
            ),
            &mut pid_allocator,
            &mut local_vec_id_allocator,
            &mut object_store,
        )
        .unwrap();

        assert_eq!(
            draft.centroid_pids,
            vec![SPIRE_FIRST_PID, SPIRE_FIRST_PID + 1]
        );
        assert_eq!(draft.leaf_objects.len(), 2);
        assert_eq!(draft.object_manifest.entries.len(), 2);
        assert_eq!(draft.placement_directory.entries.len(), 2);
        for leaf_object in &draft.leaf_objects {
            assert!(draft.object_manifest.get(leaf_object.header.pid).is_some());
            let placement = draft
                .placement_directory
                .get(leaf_object.header.pid)
                .unwrap();
            assert_eq!(
                object_store.read_leaf_object(placement).unwrap(),
                *leaf_object
            );
        }
        assert_eq!(
            draft
                .leaf_objects
                .iter()
                .map(|object| object.assignments.len())
                .sum::<usize>(),
            2
        );
        SpirePublishedEpochSnapshot::new(
            &draft.epoch_manifest,
            &draft.object_manifest,
            &draft.placement_directory,
        )
        .unwrap();
        assert_eq!(draft.next_pid, SPIRE_FIRST_PID + 2);
        assert_eq!(draft.next_local_vec_seq, SPIRE_FIRST_LOCAL_VEC_SEQ + 2);
        assert_eq!(pid_allocator.next_pid(), draft.next_pid);
        assert_eq!(
            local_vec_id_allocator.next_local_vec_seq(),
            draft.next_local_vec_seq
        );
    }

    #[test]
    fn partitioned_single_level_draft_preserves_empty_centroid_leaf() {
        let centroid_plan = SpireSingleLevelCentroidPlan {
            dimensions: 2,
            centroids: vec![vec![1.0, 0.0], vec![-1.0, 0.0]],
            assignment_indexes: vec![0],
        };
        let mut pid_allocator = SpirePidAllocator::default();
        let mut local_vec_id_allocator = SpireLocalVecIdAllocator::default();
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();

        let draft = build_partitioned_single_level_leaf_epoch_draft(
            partitioned_build_input(vec![assignment_input(10, 1)], centroid_plan),
            &mut pid_allocator,
            &mut local_vec_id_allocator,
            &mut object_store,
        )
        .unwrap();

        assert_eq!(draft.leaf_objects.len(), 2);
        assert_eq!(draft.leaf_objects[0].assignments.len(), 1);
        assert!(draft.leaf_objects[1].assignments.is_empty());
        assert_eq!(draft.leaf_objects[1].header.pid, SPIRE_FIRST_PID + 1);
        assert!(draft.object_manifest.get(SPIRE_FIRST_PID + 1).is_some());
        assert!(draft.placement_directory.get(SPIRE_FIRST_PID + 1).is_some());
        assert_eq!(draft.next_pid, SPIRE_FIRST_PID + 2);
        assert_eq!(draft.next_local_vec_seq, SPIRE_FIRST_LOCAL_VEC_SEQ + 1);
    }

    #[test]
    fn partitioned_single_level_draft_rejects_bad_plan_without_advancing_allocators() {
        let centroid_plan = SpireSingleLevelCentroidPlan {
            dimensions: 2,
            centroids: vec![vec![1.0, 0.0], vec![-1.0, 0.0]],
            assignment_indexes: vec![2],
        };
        let mut pid_allocator = SpirePidAllocator::default();
        let mut local_vec_id_allocator = SpireLocalVecIdAllocator::default();
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let initial_page_count = object_store.page_count();

        assert!(build_partitioned_single_level_leaf_epoch_draft(
            partitioned_build_input(vec![assignment_input(10, 1)], centroid_plan),
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
        assert_eq!(object_store.page_count(), initial_page_count);
    }

    #[test]
    fn partitioned_single_level_draft_rejects_late_bad_assignment_without_store_write() {
        let centroid_plan = SpireSingleLevelCentroidPlan {
            dimensions: 2,
            centroids: vec![vec![1.0, 0.0], vec![-1.0, 0.0]],
            assignment_indexes: vec![0, 1],
        };
        let mut pid_allocator = SpirePidAllocator::default();
        let mut local_vec_id_allocator = SpireLocalVecIdAllocator::default();
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let initial_page_count = object_store.page_count();
        let mut bad_assignment = assignment_input(10, 2);
        bad_assignment.heap_tid = ItemPointer::INVALID;

        assert!(build_partitioned_single_level_leaf_epoch_draft(
            partitioned_build_input(vec![assignment_input(10, 1), bad_assignment], centroid_plan),
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
        assert_eq!(object_store.page_count(), initial_page_count);
    }
}
