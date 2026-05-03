use std::ffi::c_void;
use std::ptr;

use super::assign::{
    build_primary_leaf_assignments, SpireLeafAssignmentInput, SpireLocalVecIdAllocator,
    SpirePidAllocator,
};
use super::meta::{
    SpireConsistencyMode, SpireEpochManifest, SpireEpochState, SpireManifestEntry,
    SpireObjectManifest, SpirePlacementDirectory, SpireRootControlState,
    SpireValidatedEpochSnapshot,
};
use super::storage::{
    SpireLeafPartitionObject, SpireLocalObjectStore, SpireRoutingChildEntry,
    SpireRoutingPartitionObject,
};
use super::{options, page};
use crate::am::common::training as common_training;
use crate::storage::page::ItemPointer;
use pgrx::{pg_sys, PgBox};

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
    pub(super) root_placement_tid: ItemPointer,
    pub(super) placement_tids: Vec<ItemPointer>,
    pub(super) assignments: Vec<SpireLeafAssignmentInput>,
    pub(super) centroid_plan: SpireSingleLevelCentroidPlan,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SpirePartitionedSingleLevelBuildDraft {
    pub(super) epoch_manifest: SpireEpochManifest,
    pub(super) object_manifest: SpireObjectManifest,
    pub(super) placement_directory: SpirePlacementDirectory,
    pub(super) route_map: SpireSingleLevelRouteMap,
    pub(super) root_pid: u64,
    pub(super) routing_object: SpireRoutingPartitionObject,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct SpirePublishObjectWriteEvidence {
    pub(super) pid: u64,
    pub(super) object_tid: ItemPointer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct SpirePublishPlacementWriteEvidence {
    pub(super) pid: u64,
    pub(super) placement_tid: ItemPointer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SpirePublishStage {
    WritingObjects,
    WritingPlacements,
    WritingManifest,
    Validating,
    PublishingActiveEpoch,
    Published,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct SpirePublishFailed {
    pub(super) stage: SpirePublishStage,
    pub(super) error: String,
}

impl SpirePublishFailed {
    fn at(stage: SpirePublishStage, error: String) -> Self {
        Self { stage, error }
    }

    fn into_error(self) -> String {
        format!(
            "ec_spire publish coordinator {:?} failed: {}",
            self.stage, self.error
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) struct SpirePublishCoordinatorInput<'a> {
    pub(super) epoch_manifest: &'a SpireEpochManifest,
    pub(super) object_manifest: &'a SpireObjectManifest,
    pub(super) placement_directory: &'a SpirePlacementDirectory,
    pub(super) next_pid: u64,
    pub(super) next_local_vec_seq: u64,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct SpirePublishWritingObjects<'a> {
    input: SpirePublishCoordinatorInput<'a>,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct SpirePublishWritingPlacements<'a> {
    input: SpirePublishCoordinatorInput<'a>,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct SpirePublishWritingManifest<'a> {
    input: SpirePublishCoordinatorInput<'a>,
}

#[derive(Debug, Clone)]
pub(super) struct SpirePublishValidating<'a> {
    input: SpirePublishCoordinatorInput<'a>,
    manifests: SpireEncodedManifestBundle,
}

#[derive(Debug, Clone)]
pub(super) struct SpirePublishPublishingActiveEpoch<'a> {
    input: SpirePublishCoordinatorInput<'a>,
    manifests: SpireEncodedManifestBundle,
    snapshot: SpireValidatedEpochSnapshot<'a>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct SpirePublishPublished {
    pub(super) root_control_state: SpireRootControlState,
    pub(super) bundle: SpireEncodedPublishBundle,
}

impl<'a> SpirePublishWritingObjects<'a> {
    pub(super) fn new(input: SpirePublishCoordinatorInput<'a>) -> Self {
        Self { input }
    }

    pub(super) fn objects_written(
        self,
        evidence: &[SpirePublishObjectWriteEvidence],
    ) -> Result<SpirePublishWritingPlacements<'a>, SpirePublishFailed> {
        validate_object_write_evidence(self.input.placement_directory, evidence)
            .map_err(|error| SpirePublishFailed::at(SpirePublishStage::WritingObjects, error))?;
        Ok(SpirePublishWritingPlacements { input: self.input })
    }
}

impl<'a> SpirePublishWritingPlacements<'a> {
    pub(super) fn placements_written(
        self,
        evidence: &[SpirePublishPlacementWriteEvidence],
    ) -> Result<SpirePublishWritingManifest<'a>, SpirePublishFailed> {
        validate_placement_write_evidence(self.input.object_manifest, evidence)
            .map_err(|error| SpirePublishFailed::at(SpirePublishStage::WritingPlacements, error))?;
        Ok(SpirePublishWritingManifest { input: self.input })
    }
}

impl<'a> SpirePublishWritingManifest<'a> {
    pub(super) fn write_manifests(self) -> Result<SpirePublishValidating<'a>, SpirePublishFailed> {
        let manifests = SpireEncodedManifestBundle {
            epoch_manifest: self.input.epoch_manifest.encode().map_err(|error| {
                SpirePublishFailed::at(SpirePublishStage::WritingManifest, error)
            })?,
            object_manifest: self.input.object_manifest.encode().map_err(|error| {
                SpirePublishFailed::at(SpirePublishStage::WritingManifest, error)
            })?,
            placement_directory: self.input.placement_directory.encode().map_err(|error| {
                SpirePublishFailed::at(SpirePublishStage::WritingManifest, error)
            })?,
        };
        Ok(SpirePublishValidating {
            input: self.input,
            manifests,
        })
    }
}

impl<'a> SpirePublishValidating<'a> {
    pub(super) fn validate(
        self,
    ) -> Result<SpirePublishPublishingActiveEpoch<'a>, SpirePublishFailed> {
        let snapshot = SpireValidatedEpochSnapshot::new(
            self.input.epoch_manifest,
            self.input.object_manifest,
            self.input.placement_directory,
        )
        .map_err(|error| SpirePublishFailed::at(SpirePublishStage::Validating, error))?;
        Ok(SpirePublishPublishingActiveEpoch {
            input: self.input,
            manifests: self.manifests,
            snapshot,
        })
    }
}

impl SpirePublishPublishingActiveEpoch<'_> {
    pub(super) fn manifests(&self) -> &SpireEncodedManifestBundle {
        &self.manifests
    }

    pub(super) fn root_control_state(
        &self,
        locators: SpirePublishedManifestLocators,
    ) -> Result<SpireRootControlState, SpirePublishFailed> {
        SpireRootControlState::published(
            self.snapshot.epoch_manifest().epoch,
            self.input.next_pid,
            self.input.next_local_vec_seq,
            locators.epoch_manifest_tid,
            locators.object_manifest_tid,
            locators.placement_directory_tid,
        )
        .map_err(|error| SpirePublishFailed::at(SpirePublishStage::PublishingActiveEpoch, error))
    }

    pub(super) fn publish_active_epoch(
        self,
        locators: SpirePublishedManifestLocators,
    ) -> Result<SpirePublishPublished, SpirePublishFailed> {
        let root_control_state = self.root_control_state(locators)?;
        let root_control_state_bytes = root_control_state.encode().map_err(|error| {
            SpirePublishFailed::at(SpirePublishStage::PublishingActiveEpoch, error)
        })?;
        Ok(SpirePublishPublished {
            root_control_state,
            bundle: SpireEncodedPublishBundle {
                manifests: self.manifests,
                root_control_state: root_control_state_bytes,
            },
        })
    }
}

fn publish_through_validation(
    input: SpirePublishCoordinatorInput<'_>,
) -> Result<SpirePublishPublishingActiveEpoch<'_>, SpirePublishFailed> {
    let object_evidence = object_write_evidence_from_placement_directory(input.placement_directory);
    let placement_evidence = placement_write_evidence_from_object_manifest(input.object_manifest);
    SpirePublishWritingObjects::new(input)
        .objects_written(&object_evidence)?
        .placements_written(&placement_evidence)?
        .write_manifests()?
        .validate()
}

pub(super) fn object_write_evidence_from_placement_directory(
    placement_directory: &SpirePlacementDirectory,
) -> Vec<SpirePublishObjectWriteEvidence> {
    placement_directory
        .entries
        .iter()
        .map(|entry| SpirePublishObjectWriteEvidence {
            pid: entry.pid,
            object_tid: entry.object_tid,
        })
        .collect()
}

pub(super) fn placement_write_evidence_from_object_manifest(
    object_manifest: &SpireObjectManifest,
) -> Vec<SpirePublishPlacementWriteEvidence> {
    object_manifest
        .entries
        .iter()
        .map(|entry| SpirePublishPlacementWriteEvidence {
            pid: entry.pid,
            placement_tid: entry.placement_tid,
        })
        .collect()
}

fn validate_object_write_evidence(
    placement_directory: &SpirePlacementDirectory,
    evidence: &[SpirePublishObjectWriteEvidence],
) -> Result<(), String> {
    if evidence.len() != placement_directory.entries.len() {
        return Err(format!(
            "ec_spire publish object write evidence count mismatch: got {}, expected {}",
            evidence.len(),
            placement_directory.entries.len()
        ));
    }

    let mut sorted = evidence.to_vec();
    sorted.sort_by_key(|entry| entry.pid);
    let mut previous_pid = None;
    for entry in &sorted {
        if entry.pid == 0 {
            return Err("ec_spire publish object write evidence pid 0 is invalid".to_owned());
        }
        if entry.object_tid == ItemPointer::INVALID {
            return Err(format!(
                "ec_spire publish object write evidence for pid {} has invalid object_tid",
                entry.pid
            ));
        }
        if Some(entry.pid) == previous_pid {
            return Err(format!(
                "ec_spire publish object write evidence duplicate pid {}",
                entry.pid
            ));
        }
        previous_pid = Some(entry.pid);
    }

    for (expected, actual) in placement_directory.entries.iter().zip(sorted.iter()) {
        if expected.pid != actual.pid {
            return Err(format!(
                "ec_spire publish object write evidence pid mismatch: got {}, expected {}",
                actual.pid, expected.pid
            ));
        }
        if expected.object_tid != actual.object_tid {
            return Err(format!(
                "ec_spire publish object write evidence object_tid mismatch for pid {}",
                expected.pid
            ));
        }
    }
    Ok(())
}

fn validate_placement_write_evidence(
    object_manifest: &SpireObjectManifest,
    evidence: &[SpirePublishPlacementWriteEvidence],
) -> Result<(), String> {
    if evidence.len() != object_manifest.entries.len() {
        return Err(format!(
            "ec_spire publish placement write evidence count mismatch: got {}, expected {}",
            evidence.len(),
            object_manifest.entries.len()
        ));
    }

    let mut sorted = evidence.to_vec();
    sorted.sort_by_key(|entry| entry.pid);
    let mut previous_pid = None;
    for entry in &sorted {
        if entry.pid == 0 {
            return Err("ec_spire publish placement write evidence pid 0 is invalid".to_owned());
        }
        if entry.placement_tid == ItemPointer::INVALID {
            return Err(format!(
                "ec_spire publish placement write evidence for pid {} has invalid placement_tid",
                entry.pid
            ));
        }
        if Some(entry.pid) == previous_pid {
            return Err(format!(
                "ec_spire publish placement write evidence duplicate pid {}",
                entry.pid
            ));
        }
        previous_pid = Some(entry.pid);
    }

    for (expected, actual) in object_manifest.entries.iter().zip(sorted.iter()) {
        if expected.pid != actual.pid {
            return Err(format!(
                "ec_spire publish placement write evidence pid mismatch: got {}, expected {}",
                actual.pid, expected.pid
            ));
        }
        if expected.placement_tid != actual.placement_tid {
            return Err(format!(
                "ec_spire publish placement write evidence placement_tid mismatch for pid {}",
                expected.pid
            ));
        }
    }
    Ok(())
}

pub(super) fn encode_manifest_bundle_for_publish(
    input: SpirePublishCoordinatorInput<'_>,
) -> Result<SpireEncodedManifestBundle, String> {
    let publish = publish_through_validation(input).map_err(SpirePublishFailed::into_error)?;
    Ok(publish.manifests().clone())
}

pub(super) fn root_control_state_for_publish(
    input: SpirePublishCoordinatorInput<'_>,
    locators: SpirePublishedManifestLocators,
) -> Result<SpireRootControlState, String> {
    publish_through_validation(input)
        .and_then(|publish| publish.root_control_state(locators))
        .map_err(SpirePublishFailed::into_error)
}

pub(super) fn encode_publish_bundle_for_publish(
    input: SpirePublishCoordinatorInput<'_>,
    locators: SpirePublishedManifestLocators,
) -> Result<SpireEncodedPublishBundle, String> {
    publish_through_validation(input)
        .and_then(|publish| publish.publish_active_epoch(locators))
        .map(|published| published.bundle)
        .map_err(SpirePublishFailed::into_error)
}

pub(super) unsafe fn write_manifest_bundle_to_relation(
    index_relation: pg_sys::Relation,
    manifests: &SpireEncodedManifestBundle,
) -> Result<SpirePublishedManifestLocators, String> {
    let epoch_manifest_tid =
        unsafe { page::append_object_tuple(index_relation, &manifests.epoch_manifest)? };
    let object_manifest_tid =
        unsafe { page::append_object_tuple(index_relation, &manifests.object_manifest)? };
    let placement_directory_tid =
        unsafe { page::append_object_tuple(index_relation, &manifests.placement_directory)? };
    Ok(SpirePublishedManifestLocators {
        epoch_manifest_tid,
        object_manifest_tid,
        placement_directory_tid,
    })
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SpireSingleLevelCentroidPlan {
    pub(super) dimensions: u16,
    pub(super) centroids: Vec<Vec<f32>>,
    pub(super) assignment_indexes: Vec<u32>,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SpireSingleLevelRouteEntry {
    pub(super) centroid_index: u32,
    pub(super) pid: u64,
    pub(super) centroid: Vec<f32>,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SpireSingleLevelRouteMap {
    pub(super) dimensions: u16,
    pub(super) entries: Vec<SpireSingleLevelRouteEntry>,
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

impl SpireSingleLevelRouteMap {
    pub(super) fn from_centroid_plan(
        plan: &SpireSingleLevelCentroidPlan,
        centroid_pids: &[u64],
    ) -> Result<Self, String> {
        plan.validate()?;
        if centroid_pids.len() != plan.centroid_count() {
            return Err(format!(
                "ec_spire route map pid count {} does not match centroid count {}",
                centroid_pids.len(),
                plan.centroid_count()
            ));
        }

        let mut entries = Vec::with_capacity(plan.centroid_count());
        for (centroid_index, (centroid, &pid)) in
            plan.centroids.iter().zip(centroid_pids.iter()).enumerate()
        {
            if pid == 0 {
                return Err("ec_spire route map pid 0 is invalid".to_owned());
            }
            entries.push(SpireSingleLevelRouteEntry {
                centroid_index: u32::try_from(centroid_index)
                    .map_err(|_| "ec_spire route map centroid index exceeds u32".to_owned())?,
                pid,
                centroid: centroid.clone(),
            });
        }

        let route_map = Self {
            dimensions: plan.dimensions,
            entries,
        };
        route_map.validate()?;
        Ok(route_map)
    }

    pub(super) fn route_pid_for_vector(&self, vector: &[f32]) -> Result<u64, String> {
        self.validate()?;
        let model = common_training::SphericalKMeansModel {
            dimensions: usize::from(self.dimensions),
            centroids: self
                .entries
                .iter()
                .map(|entry| entry.centroid.clone())
                .collect(),
        };
        let centroid_index =
            common_training::assign_vector_to_centroid("ec_spire", vector, &model)?;
        Ok(self.entries[centroid_index].pid)
    }

    pub(super) fn get(&self, centroid_index: u32) -> Option<&SpireSingleLevelRouteEntry> {
        self.entries
            .get(usize::try_from(centroid_index).ok()?)
            .filter(|entry| entry.centroid_index == centroid_index)
    }

    fn validate(&self) -> Result<(), String> {
        if self.dimensions == 0 {
            return Err("ec_spire route map requires dimensions > 0".to_owned());
        }
        if self.entries.is_empty() {
            return Err("ec_spire route map requires at least one entry".to_owned());
        }
        let dimensions = usize::from(self.dimensions);
        for (expected_index, entry) in self.entries.iter().enumerate() {
            let expected_index = u32::try_from(expected_index)
                .map_err(|_| "ec_spire route map centroid index exceeds u32".to_owned())?;
            if entry.centroid_index != expected_index {
                return Err(format!(
                    "ec_spire route map centroid index mismatch: got {}, expected {expected_index}",
                    entry.centroid_index
                ));
            }
            if entry.pid == 0 {
                return Err("ec_spire route map pid 0 is invalid".to_owned());
            }
            if entry.centroid.len() != dimensions {
                return Err(format!(
                    "ec_spire route map centroid {} dimensions mismatch: got {}, expected {dimensions}",
                    entry.centroid_index,
                    entry.centroid.len()
                ));
            }
            if entry
                .centroid
                .iter()
                .any(|component| !component.is_finite())
            {
                return Err(format!(
                    "ec_spire route map centroid {} must be finite",
                    entry.centroid_index
                ));
            }
        }
        Ok(())
    }
}

impl SpireSingleLevelBuildDraft {
    fn publish_input(&self) -> SpirePublishCoordinatorInput<'_> {
        SpirePublishCoordinatorInput {
            epoch_manifest: &self.epoch_manifest,
            object_manifest: &self.object_manifest,
            placement_directory: &self.placement_directory,
            next_pid: self.next_pid,
            next_local_vec_seq: self.next_local_vec_seq,
        }
    }

    pub(super) fn encode_manifest_bundle(&self) -> Result<SpireEncodedManifestBundle, String> {
        encode_manifest_bundle_for_publish(self.publish_input())
    }

    pub(super) fn root_control_state(
        &self,
        locators: SpirePublishedManifestLocators,
    ) -> Result<SpireRootControlState, String> {
        root_control_state_for_publish(self.publish_input(), locators)
    }

    pub(super) fn encode_publish_bundle(
        &self,
        locators: SpirePublishedManifestLocators,
    ) -> Result<SpireEncodedPublishBundle, String> {
        encode_publish_bundle_for_publish(self.publish_input(), locators)
    }
}

impl SpirePartitionedSingleLevelBuildDraft {
    fn publish_input(&self) -> SpirePublishCoordinatorInput<'_> {
        SpirePublishCoordinatorInput {
            epoch_manifest: &self.epoch_manifest,
            object_manifest: &self.object_manifest,
            placement_directory: &self.placement_directory,
            next_pid: self.next_pid,
            next_local_vec_seq: self.next_local_vec_seq,
        }
    }

    pub(super) fn encode_manifest_bundle(&self) -> Result<SpireEncodedManifestBundle, String> {
        encode_manifest_bundle_for_publish(self.publish_input())
    }

    pub(super) fn root_control_state(
        &self,
        locators: SpirePublishedManifestLocators,
    ) -> Result<SpireRootControlState, String> {
        root_control_state_for_publish(self.publish_input(), locators)
    }

    pub(super) fn encode_publish_bundle(
        &self,
        locators: SpirePublishedManifestLocators,
    ) -> Result<SpireEncodedPublishBundle, String> {
        encode_publish_bundle_for_publish(self.publish_input(), locators)
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
    let placement = object_store.insert_leaf_object_v2_from_rows(
        input.epoch,
        leaf_object.header.pid,
        leaf_object.header.object_version,
        leaf_object.header.parent_pid,
        &leaf_object.assignments,
    )?;
    let placement_directory = SpirePlacementDirectory::from_entries(input.epoch, vec![placement])?;

    let draft = SpireSingleLevelBuildDraft {
        epoch_manifest,
        object_manifest,
        placement_directory,
        leaf_object,
        next_pid: pid_cursor.next_pid(),
        next_local_vec_seq: local_vec_id_cursor.next_local_vec_seq(),
    };
    SpireValidatedEpochSnapshot::new(
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
        .zip(input.centroid_plan.assignment_indexes.iter().copied())
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
    let root_pid = pid_cursor.allocate()?;
    let mut centroid_pids = Vec::with_capacity(centroid_count);
    for _ in 0..centroid_count {
        centroid_pids.push(pid_cursor.allocate()?);
    }
    let route_map =
        SpireSingleLevelRouteMap::from_centroid_plan(&input.centroid_plan, &centroid_pids)?;

    let routing_object = SpireRoutingPartitionObject::root(
        root_pid,
        input.object_version,
        input.centroid_plan.dimensions,
        route_map
            .entries
            .iter()
            .map(|entry| SpireRoutingChildEntry {
                centroid_index: entry.centroid_index,
                child_pid: entry.pid,
                centroid: entry.centroid.clone(),
            })
            .collect(),
    )?;

    let mut manifest_entries = Vec::with_capacity(centroid_count + 1);
    manifest_entries.push(SpireManifestEntry {
        epoch: input.epoch,
        pid: root_pid,
        object_version: input.object_version,
        placement_tid: input.root_placement_tid,
    });
    manifest_entries.extend(centroid_pids.iter().zip(input.placement_tids.iter()).map(
        |(&pid, &placement_tid)| SpireManifestEntry {
            epoch: input.epoch,
            pid,
            object_version: input.object_version,
            placement_tid,
        },
    ));
    let object_manifest = SpireObjectManifest::from_entries(input.epoch, manifest_entries)?;

    let mut leaf_objects = Vec::with_capacity(centroid_count);
    for (pid, assignments) in centroid_pids
        .iter()
        .copied()
        .zip(assignments_by_centroid.into_iter())
    {
        let assignments = build_primary_leaf_assignments(&mut local_vec_id_cursor, assignments)?;
        let leaf_object =
            SpireLeafPartitionObject::new(pid, input.object_version, root_pid, assignments)?;
        leaf_objects.push(leaf_object);
    }
    let mut placements = Vec::with_capacity(centroid_count + 1);
    placements.push(object_store.insert_routing_object(input.epoch, &routing_object)?);
    for leaf_object in &leaf_objects {
        placements.push(object_store.insert_leaf_object_v2_from_rows(
            input.epoch,
            leaf_object.header.pid,
            leaf_object.header.object_version,
            leaf_object.header.parent_pid,
            &leaf_object.assignments,
        )?);
    }
    let placement_directory = SpirePlacementDirectory::from_entries(input.epoch, placements)?;

    let draft = SpirePartitionedSingleLevelBuildDraft {
        epoch_manifest,
        object_manifest,
        placement_directory,
        route_map,
        root_pid,
        routing_object,
        centroid_pids,
        leaf_objects,
        next_pid: pid_cursor.next_pid(),
        next_local_vec_seq: local_vec_id_cursor.next_local_vec_seq(),
    };
    SpireValidatedEpochSnapshot::new(
        &draft.epoch_manifest,
        &draft.object_manifest,
        &draft.placement_directory,
    )?;

    *pid_allocator = pid_cursor;
    *local_vec_id_allocator = local_vec_id_cursor;
    Ok(draft)
}

pub(super) unsafe extern "C-unwind" fn ec_spire_ambuild(
    heap_relation: pg_sys::Relation,
    index_relation: pg_sys::Relation,
    index_info: *mut pg_sys::IndexInfo,
) -> *mut pg_sys::IndexBuildResult {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            let _options = options::relation_options(index_relation);
            page::initialize_root_control_page(index_relation, SpireRootControlState::empty());
            let heap_tuples = pg_sys::table_index_build_scan(
                heap_relation,
                index_relation,
                index_info,
                false,
                false,
                Some(ec_spire_empty_build_callback),
                ptr::null_mut(),
                ptr::null_mut(),
            );

            let mut result = PgBox::<pg_sys::IndexBuildResult>::alloc0();
            result.heap_tuples = heap_tuples;
            result.index_tuples = 0.0;
            result.into_pg()
        })
    }
}

pub(super) unsafe extern "C-unwind" fn ec_spire_ambuildempty(_index_relation: pg_sys::Relation) {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            page::initialize_root_control_page(_index_relation, SpireRootControlState::empty());
        })
    }
}

unsafe extern "C-unwind" fn ec_spire_empty_build_callback(
    _index: pg_sys::Relation,
    _tid: pg_sys::ItemPointer,
    _values: *mut pg_sys::Datum,
    _isnull: *mut bool,
    _tuple_is_alive: bool,
    _state: *mut c_void,
) {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            pgrx::error!(
                "ec_spire populated ambuild is not implemented yet; create the index on an empty relation for the current persistence checkpoint"
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{
        build_partitioned_single_level_leaf_epoch_draft, build_single_level_leaf_epoch_draft,
        object_write_evidence_from_placement_directory,
        placement_write_evidence_from_object_manifest, train_single_level_centroid_plan,
        SpirePartitionedSingleLevelBuildInput, SpirePublishStage, SpirePublishWritingObjects,
        SpireSingleLevelBuildInput, SpireSingleLevelCentroidPlan, SpireSingleLevelRouteMap,
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
            root_placement_tid: tid(60, 3),
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
    fn single_level_route_map_routes_query_to_centroid_pid() {
        let centroid_plan = SpireSingleLevelCentroidPlan {
            dimensions: 2,
            centroids: vec![vec![1.0, 0.0], vec![-1.0, 0.0]],
            assignment_indexes: Vec::new(),
        };
        let route_map =
            SpireSingleLevelRouteMap::from_centroid_plan(&centroid_plan, &[11, 12]).unwrap();

        assert_eq!(route_map.get(0).unwrap().pid, 11);
        assert_eq!(route_map.get(1).unwrap().pid, 12);
        assert_eq!(route_map.route_pid_for_vector(&[1.0, 0.0]).unwrap(), 11);
        assert_eq!(route_map.route_pid_for_vector(&[-1.0, 0.0]).unwrap(), 12);
        assert!(route_map.route_pid_for_vector(&[1.0]).is_err());
    }

    #[test]
    fn single_level_route_map_rejects_pid_count_mismatch() {
        let centroid_plan = SpireSingleLevelCentroidPlan {
            dimensions: 2,
            centroids: vec![vec![1.0, 0.0], vec![-1.0, 0.0]],
            assignment_indexes: Vec::new(),
        };

        assert!(SpireSingleLevelRouteMap::from_centroid_plan(&centroid_plan, &[11]).is_err());
    }

    #[test]
    fn single_level_draft_builds_leaf_object_and_published_snapshot() {
        let (draft, pid_allocator, local_vec_id_allocator, object_store) = build_valid_draft();

        let placement = draft.placement_directory.get(SPIRE_FIRST_PID).unwrap();
        let stored_leaf = object_store.read_leaf_object_v2(placement).unwrap();

        assert_eq!(draft.epoch_manifest.epoch, 7);
        assert_eq!(draft.leaf_object.header.pid, SPIRE_FIRST_PID);
        assert_eq!(draft.leaf_object.header.object_version, 1);
        assert_eq!(draft.leaf_object.assignments.len(), 2);
        assert_eq!(stored_leaf.meta.header.pid, draft.leaf_object.header.pid);
        assert_eq!(
            stored_leaf.meta.header.object_version,
            draft.leaf_object.header.object_version
        );
        assert_eq!(
            stored_leaf.meta.header.assignment_count,
            draft.leaf_object.assignments.len() as u32
        );
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
    fn publish_coordinator_validates_before_active_epoch_publish() {
        let (mut draft, _, _, _) = build_valid_draft();
        draft.placement_directory.entries[0].object_version = 99;

        let error = draft
            .encode_publish_bundle(manifest_locators())
            .unwrap_err();

        assert!(error.contains("Validating failed"));
        assert!(error.contains("object_version mismatch"));
    }

    #[test]
    fn publish_coordinator_rejects_missing_object_write_evidence() {
        let (draft, _, _, _) = build_valid_draft();
        let error = SpirePublishWritingObjects::new(draft.publish_input())
            .objects_written(&[])
            .unwrap_err();

        assert_eq!(error.stage, SpirePublishStage::WritingObjects);
        assert!(error.error.contains("object write evidence count mismatch"));
    }

    #[test]
    fn publish_coordinator_rejects_mismatched_placement_write_evidence() {
        let (draft, _, _, _) = build_valid_draft();
        let object_evidence =
            object_write_evidence_from_placement_directory(&draft.placement_directory);
        let mut placement_evidence =
            placement_write_evidence_from_object_manifest(&draft.object_manifest);
        placement_evidence[0].placement_tid = tid(99, 9);

        let error = SpirePublishWritingObjects::new(draft.publish_input())
            .objects_written(&object_evidence)
            .unwrap()
            .placements_written(&placement_evidence)
            .unwrap_err();

        assert_eq!(error.stage, SpirePublishStage::WritingPlacements);
        assert!(error.error.contains("placement_tid mismatch"));
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
            vec![SPIRE_FIRST_PID + 1, SPIRE_FIRST_PID + 2]
        );
        assert_eq!(draft.root_pid, SPIRE_FIRST_PID);
        assert_eq!(draft.routing_object.header.pid, SPIRE_FIRST_PID);
        assert_eq!(draft.routing_object.header.child_count, 2);
        assert_eq!(draft.leaf_objects.len(), 2);
        assert_eq!(draft.route_map.entries.len(), 2);
        assert_eq!(draft.object_manifest.entries.len(), 3);
        assert_eq!(draft.placement_directory.entries.len(), 3);
        let root_placement = draft.placement_directory.get(draft.root_pid).unwrap();
        let mut expected_routing_object = draft.routing_object.clone();
        expected_routing_object.header.published_epoch_backref = draft.epoch_manifest.epoch;
        assert_eq!(
            object_store.read_routing_object(root_placement).unwrap(),
            expected_routing_object
        );
        for &pid in &draft.centroid_pids {
            assert!(draft.route_map.entries.iter().any(|entry| entry.pid == pid));
        }
        for leaf_object in &draft.leaf_objects {
            assert_eq!(leaf_object.header.parent_pid, draft.root_pid);
            assert!(draft.object_manifest.get(leaf_object.header.pid).is_some());
            let placement = draft
                .placement_directory
                .get(leaf_object.header.pid)
                .unwrap();
            let stored_leaf = object_store.read_leaf_object_v2(placement).unwrap();
            assert_eq!(stored_leaf.meta.header.pid, leaf_object.header.pid);
            assert_eq!(
                stored_leaf.meta.header.assignment_count,
                leaf_object.assignments.len() as u32
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
        assert_eq!(draft.next_pid, SPIRE_FIRST_PID + 3);
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
        assert_eq!(draft.leaf_objects[1].header.pid, SPIRE_FIRST_PID + 2);
        assert_eq!(draft.route_map.get(1).unwrap().pid, SPIRE_FIRST_PID + 2);
        assert!(draft.object_manifest.get(SPIRE_FIRST_PID + 2).is_some());
        assert!(draft.placement_directory.get(SPIRE_FIRST_PID + 2).is_some());
        assert_eq!(draft.next_pid, SPIRE_FIRST_PID + 3);
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
