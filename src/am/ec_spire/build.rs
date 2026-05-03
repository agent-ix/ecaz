use std::ffi::{c_void, CStr};
use std::mem::size_of;
use std::ptr;

use pgrx::{itemptr::item_pointer_get_both, pg_sys, PgBox, PgTupleDesc};

use super::assign::{
    build_primary_leaf_assignments, SpireLeafAssignmentInput, SpireLocalVecIdAllocator,
    SpirePidAllocator,
};
use super::meta::{
    SpireConsistencyMode, SpireEpochManifest, SpireEpochState, SpireManifestEntry,
    SpireObjectManifest, SpirePlacementDirectory, SpireRootControlState,
    SpireValidatedEpochSnapshot, SPIRE_MIN_EPOCH_RETENTION_SECS,
};
use super::storage::{
    SpireLeafPartitionObject, SpireLocalObjectStore, SpireRelationObjectStore,
    SpireRoutingChildEntry, SpireRoutingPartitionObject,
};
use super::{options, page};
use super::{quantizer, quantizer::SpireAssignmentPayloadFormat};
use crate::am::common::training as common_training;
use crate::quant::prod::ProdQuantizer;
use crate::storage::page::ItemPointer;

const SPIRE_DEFAULT_KMEANS_ITERATIONS: usize = 8;
const SPIRE_DEFAULT_AUTO_TRAINING_SAMPLE_ROWS: usize = 10_000;
const SPIRE_INITIAL_EPOCH: u64 = 1;
const SPIRE_INITIAL_OBJECT_VERSION: u64 = 1;
const MICROS_PER_SECOND: i64 = 1_000_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SpireIndexedVectorKind {
    Ecvector,
    Tqvector,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SpireBuildTuple {
    pub(super) heap_tid: ItemPointer,
    pub(super) dimensions: u16,
    pub(super) assignment: SpireLeafAssignmentInput,
    pub(super) source_vector: Vec<f32>,
}

struct SpireBuildState {
    options: options::EcSpireOptions,
    indexed_vector_kind: SpireIndexedVectorKind,
    scanned_tuples: usize,
    tuples: Vec<SpireBuildTuple>,
    dimensions: Option<u16>,
}

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

pub(super) fn object_manifest_from_placement_writes(
    epoch: u64,
    placement_directory: &SpirePlacementDirectory,
    evidence: &[SpirePublishPlacementWriteEvidence],
) -> Result<SpireObjectManifest, String> {
    if epoch == 0 {
        return Err("ec_spire object manifest epoch 0 is invalid".to_owned());
    }
    if placement_directory.epoch != epoch {
        return Err(format!(
            "ec_spire object manifest placement directory epoch mismatch: got {}, expected {epoch}",
            placement_directory.epoch
        ));
    }
    if evidence.len() != placement_directory.entries.len() {
        return Err(format!(
            "ec_spire placement write evidence count mismatch: got {}, expected {}",
            evidence.len(),
            placement_directory.entries.len()
        ));
    }

    let mut sorted = evidence.to_vec();
    sorted.sort_by_key(|entry| entry.pid);
    let mut previous_pid = None;
    for entry in &sorted {
        if entry.pid == 0 {
            return Err("ec_spire placement write evidence pid 0 is invalid".to_owned());
        }
        if entry.placement_tid == ItemPointer::INVALID {
            return Err(format!(
                "ec_spire placement write evidence for pid {} has invalid placement_tid",
                entry.pid
            ));
        }
        if Some(entry.pid) == previous_pid {
            return Err(format!(
                "ec_spire placement write evidence duplicate pid {}",
                entry.pid
            ));
        }
        previous_pid = Some(entry.pid);
    }

    let mut entries = Vec::with_capacity(placement_directory.entries.len());
    for (placement, write) in placement_directory.entries.iter().zip(sorted.iter()) {
        if placement.pid != write.pid {
            return Err(format!(
                "ec_spire placement write evidence pid mismatch: got {}, expected {}",
                write.pid, placement.pid
            ));
        }
        entries.push(SpireManifestEntry {
            epoch,
            pid: placement.pid,
            object_version: placement.object_version,
            placement_tid: write.placement_tid,
        });
    }
    SpireObjectManifest::from_entries(epoch, entries)
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

pub(super) unsafe fn write_placement_entries_to_relation(
    index_relation: pg_sys::Relation,
    placement_directory: &SpirePlacementDirectory,
) -> Result<Vec<SpirePublishPlacementWriteEvidence>, String> {
    let mut evidence = Vec::with_capacity(placement_directory.entries.len());
    for entry in &placement_directory.entries {
        let encoded = entry.encode()?;
        let placement_tid = unsafe { page::append_object_tuple(index_relation, &encoded)? };
        evidence.push(SpirePublishPlacementWriteEvidence {
            pid: entry.pid,
            placement_tid,
        });
    }
    Ok(evidence)
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

impl SpireBuildState {
    fn new(options: options::EcSpireOptions, indexed_vector_kind: SpireIndexedVectorKind) -> Self {
        Self {
            options,
            indexed_vector_kind,
            scanned_tuples: 0,
            tuples: Vec::new(),
            dimensions: None,
        }
    }

    fn push(&mut self, tuple: SpireBuildTuple) {
        self.try_push(tuple)
            .unwrap_or_else(|e| pgrx::error!("ec_spire ambuild found invalid indexed tuple: {e}"));
    }

    fn try_push(&mut self, tuple: SpireBuildTuple) -> Result<(), String> {
        if tuple.heap_tid == ItemPointer::INVALID {
            return Err("heap tid must be valid".to_owned());
        }
        if tuple.assignment.heap_tid != tuple.heap_tid {
            return Err("assignment heap tid must match build tuple heap tid".to_owned());
        }
        if SpireAssignmentPayloadFormat::from_tag(tuple.assignment.payload_format)?
            != self.options.assignment_payload_format()
        {
            return Err("assignment payload format does not match build options".to_owned());
        }
        if tuple.source_vector.len() != usize::from(tuple.dimensions) {
            return Err(format!(
                "source dimensions mismatch: source dim {} vs indexed dim {}",
                tuple.source_vector.len(),
                tuple.dimensions
            ));
        }
        common_training::normalize_vector(
            "ec_spire",
            &tuple.source_vector,
            usize::from(tuple.dimensions),
        )?;

        match self.dimensions {
            None => self.dimensions = Some(tuple.dimensions),
            Some(dimensions) if dimensions == tuple.dimensions => {}
            Some(dimensions) => {
                return Err(format!(
                    "dimension mismatch: saw {} after {}",
                    tuple.dimensions, dimensions
                ));
            }
        }

        self.scanned_tuples += 1;
        self.tuples.push(tuple);
        Ok(())
    }

    fn training_sample_count(&self) -> usize {
        resolve_training_sample_count(self.options.training_sample_rows, self.tuples.len())
    }

    fn training_sample_vectors(&self) -> Vec<&[f32]> {
        let indices = common_training::deterministic_sample_indices(
            self.tuples.len(),
            self.training_sample_count(),
            self.options.seed as u64,
        );
        indices
            .into_iter()
            .map(|index| self.tuples[index].source_vector.as_slice())
            .collect()
    }

    fn assignment_inputs(&self) -> Vec<SpireLeafAssignmentInput> {
        self.tuples
            .iter()
            .map(|tuple| tuple.assignment.clone())
            .collect()
    }

    fn train_centroid_plan(&self) -> Result<SpireSingleLevelCentroidPlan, String> {
        let dimensions = self
            .dimensions
            .ok_or_else(|| "ec_spire centroid training requires at least one tuple".to_owned())?;
        let requested_nlists = u32::try_from(self.options.nlists)
            .map_err(|_| "ec_spire nlists reloption must be non-negative".to_owned())?;
        let nlists = common_training::resolve_auto_nlists(requested_nlists, self.tuples.len());
        let sample_vectors = self.training_sample_vectors();
        let model = common_training::train_spherical_kmeans(
            "ec_spire",
            &sample_vectors,
            usize::from(dimensions),
            nlists,
            self.options.seed as u64,
            SPIRE_DEFAULT_KMEANS_ITERATIONS,
        )?;
        let mut assignment_indexes = Vec::with_capacity(self.tuples.len());
        for tuple in &self.tuples {
            let centroid_index = common_training::assign_vector_to_centroid(
                "ec_spire",
                &tuple.source_vector,
                &model,
            )?;
            assignment_indexes.push(
                u32::try_from(centroid_index)
                    .map_err(|_| "ec_spire centroid assignment index exceeds u32".to_owned())?,
            );
        }

        Ok(SpireSingleLevelCentroidPlan {
            dimensions,
            centroids: model.centroids,
            assignment_indexes,
        })
    }
}

fn resolve_training_sample_count(requested_sample_rows: i32, row_count: usize) -> usize {
    if row_count == 0 {
        return 0;
    }
    if requested_sample_rows > 0 {
        return (requested_sample_rows as usize).min(row_count);
    }
    row_count.min(SPIRE_DEFAULT_AUTO_TRAINING_SAMPLE_ROWS)
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

unsafe fn publish_relation_partitioned_single_level_build(
    index_relation: pg_sys::Relation,
    state: &SpireBuildState,
) -> Result<usize, String> {
    if state.scanned_tuples == 0 {
        return Ok(0);
    }

    let (published_at_micros, retain_until_micros) = unsafe { current_epoch_publish_times()? };
    let epoch_manifest = SpireEpochManifest {
        epoch: SPIRE_INITIAL_EPOCH,
        state: SpireEpochState::Published,
        consistency_mode: SpireConsistencyMode::Strict,
        published_at_micros,
        retain_until_micros,
        active_query_count: 0,
    };
    epoch_manifest.validate()?;

    let centroid_plan = state.train_centroid_plan()?;
    let assignments = state.assignment_inputs();
    let centroid_count = centroid_plan.centroid_count();
    if assignments.len() != centroid_plan.assignment_indexes.len() {
        return Err(format!(
            "ec_spire populated build assignment count {} does not match centroid assignment count {}",
            assignments.len(),
            centroid_plan.assignment_indexes.len()
        ));
    }

    let mut pid_allocator = SpirePidAllocator::default();
    let mut local_vec_id_allocator = SpireLocalVecIdAllocator::default();
    let root_pid = pid_allocator.allocate()?;
    let mut centroid_pids = Vec::with_capacity(centroid_count);
    for _ in 0..centroid_count {
        centroid_pids.push(pid_allocator.allocate()?);
    }
    let route_map = SpireSingleLevelRouteMap::from_centroid_plan(&centroid_plan, &centroid_pids)?;
    let routing_object = SpireRoutingPartitionObject::root(
        root_pid,
        SPIRE_INITIAL_OBJECT_VERSION,
        centroid_plan.dimensions,
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

    let mut assignments_by_centroid = vec![Vec::new(); centroid_count];
    for (assignment, assignment_index) in assignments
        .into_iter()
        .zip(centroid_plan.assignment_indexes.iter().copied())
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

    let mut leaf_assignments_by_centroid = Vec::with_capacity(centroid_count);
    for assignments in assignments_by_centroid {
        leaf_assignments_by_centroid.push(build_primary_leaf_assignments(
            &mut local_vec_id_allocator,
            assignments,
        )?);
    }

    let store = unsafe { SpireRelationObjectStore::for_index_relation(index_relation)? };
    let mut placements = Vec::with_capacity(centroid_count + 1);
    placements.push(unsafe { store.insert_routing_object(SPIRE_INITIAL_EPOCH, &routing_object)? });
    for (pid, assignments) in centroid_pids
        .iter()
        .copied()
        .zip(leaf_assignments_by_centroid.iter())
    {
        placements.push(unsafe {
            store.insert_leaf_object_v2_from_rows(
                SPIRE_INITIAL_EPOCH,
                pid,
                SPIRE_INITIAL_OBJECT_VERSION,
                root_pid,
                assignments,
            )?
        });
    }
    let placement_directory =
        SpirePlacementDirectory::from_entries(SPIRE_INITIAL_EPOCH, placements)?;
    let placement_evidence =
        unsafe { write_placement_entries_to_relation(index_relation, &placement_directory)? };
    let object_manifest = object_manifest_from_placement_writes(
        SPIRE_INITIAL_EPOCH,
        &placement_directory,
        &placement_evidence,
    )?;

    let input = SpirePublishCoordinatorInput {
        epoch_manifest: &epoch_manifest,
        object_manifest: &object_manifest,
        placement_directory: &placement_directory,
        next_pid: pid_allocator.next_pid(),
        next_local_vec_seq: local_vec_id_allocator.next_local_vec_seq(),
    };
    let manifests = encode_manifest_bundle_for_publish(input)?;
    let locators = unsafe { write_manifest_bundle_to_relation(index_relation, &manifests)? };
    let root_control = root_control_state_for_publish(input, locators)?;
    unsafe { page::initialize_root_control_page(index_relation, root_control) };
    Ok(state.scanned_tuples)
}

pub(super) unsafe fn current_epoch_publish_times() -> Result<(i64, i64), String> {
    let published_at_micros = unsafe { pg_sys::GetCurrentTimestamp() };
    let retention_micros = i64::from(SPIRE_MIN_EPOCH_RETENTION_SECS)
        .checked_mul(MICROS_PER_SECOND)
        .ok_or_else(|| "ec_spire epoch retention micros overflow".to_owned())?;
    let retain_until_micros = published_at_micros
        .checked_add(retention_micros)
        .ok_or_else(|| "ec_spire epoch retain_until timestamp overflow".to_owned())?;
    Ok((published_at_micros, retain_until_micros))
}

pub(super) unsafe fn build_spire_index_tuple(
    values: *mut pg_sys::Datum,
    isnull: *mut bool,
    heap_tid: ItemPointer,
    indexed_vector_kind: SpireIndexedVectorKind,
    payload_format: SpireAssignmentPayloadFormat,
    context: &str,
) -> SpireBuildTuple {
    if values.is_null() || isnull.is_null() {
        pgrx::error!("ec_spire {context} received null tuple value arrays");
    }
    if unsafe { *isnull } {
        pgrx::error!("ec_spire does not support NULL indexed values");
    }

    let datum = unsafe { *values };
    if datum.is_null() {
        pgrx::error!("ec_spire {context} received a null indexed datum");
    }

    let bytes = unsafe { detoasted_varlena_bytes(datum, "indexed vector column") };
    match indexed_vector_kind {
        SpireIndexedVectorKind::Ecvector => {
            build_spire_ecvector_tuple(heap_tid, &bytes, payload_format, context)
        }
        SpireIndexedVectorKind::Tqvector => {
            build_spire_tqvector_tuple(heap_tid, &bytes, payload_format, context)
        }
    }
}

fn build_spire_ecvector_tuple(
    heap_tid: ItemPointer,
    bytes: &[u8],
    payload_format: SpireAssignmentPayloadFormat,
    context: &str,
) -> SpireBuildTuple {
    let source_vector = crate::unpack_raw_f32(bytes, "ec_spire indexed ecvector column")
        .unwrap_or_else(|e| pgrx::error!("ec_spire {context} found invalid indexed ecvector: {e}"));
    let dimensions = build_dimensions(source_vector.len(), context, "ecvector");
    let assignment = quantizer::encode_assignment_input(payload_format, heap_tid, &source_vector)
        .unwrap_or_else(|e| pgrx::error!("ec_spire {context} found invalid indexed ecvector: {e}"));
    SpireBuildTuple {
        heap_tid,
        dimensions,
        assignment,
        source_vector,
    }
}

fn build_spire_tqvector_tuple(
    heap_tid: ItemPointer,
    bytes: &[u8],
    payload_format: SpireAssignmentPayloadFormat,
    context: &str,
) -> SpireBuildTuple {
    let (dimensions, bits, seed, gamma, code) = crate::unpack(bytes)
        .unwrap_or_else(|e| pgrx::error!("ec_spire {context} found invalid indexed tqvector: {e}"));
    let mut full_payload = Vec::with_capacity(size_of::<f32>() + code.len());
    full_payload.extend_from_slice(&gamma.to_le_bytes());
    full_payload.extend_from_slice(code);
    let quantizer = ProdQuantizer::cached(usize::from(dimensions), bits, seed);
    let source_vector = quantizer.decode_approximate(&full_payload);
    let assignment = quantizer::encode_assignment_input(payload_format, heap_tid, &source_vector)
        .unwrap_or_else(|e| pgrx::error!("ec_spire {context} found invalid indexed tqvector: {e}"));
    SpireBuildTuple {
        heap_tid,
        dimensions,
        assignment,
        source_vector,
    }
}

fn build_dimensions(dimensions: usize, context: &str, label: &str) -> u16 {
    u16::try_from(dimensions).unwrap_or_else(|_| {
        pgrx::error!(
            "ec_spire {context} found invalid indexed {label}: embedding dimension {dimensions} exceeds maximum 65535"
        )
    })
}

unsafe fn detoasted_varlena_bytes(datum: pg_sys::Datum, label: &str) -> Vec<u8> {
    let original = datum.cast_mut_ptr::<c_void>().cast::<pg_sys::varlena>();
    let varlena = unsafe { pg_sys::pg_detoast_datum_packed(original.cast()) };
    if varlena.is_null() {
        pgrx::error!("ec_spire could not detoast {label}");
    }
    let owned = !ptr::eq(varlena, original);
    let bytes = unsafe { pgrx::varlena::varlena_to_byte_slice(varlena) }.to_vec();
    if owned {
        unsafe { pg_sys::pfree(varlena.cast()) };
    }
    bytes
}

pub(super) unsafe fn decode_heap_tid(tid: pg_sys::ItemPointer, context: &str) -> ItemPointer {
    if tid.is_null() {
        pgrx::error!("ec_spire {context} received a null heap tid");
    }
    let (block_number, offset_number) = item_pointer_get_both(unsafe { *tid });
    ItemPointer {
        block_number,
        offset_number,
    }
}

pub(super) unsafe fn resolve_indexed_vector_kind(
    heap_relation: pg_sys::Relation,
    index_info: *mut pg_sys::IndexInfo,
    context: &str,
) -> SpireIndexedVectorKind {
    if index_info.is_null() {
        pgrx::error!("ec_spire {context} received a null IndexInfo");
    }
    let index_info = unsafe { &*index_info };
    if index_info.ii_NumIndexAttrs != 1 || index_info.ii_NumIndexKeyAttrs != 1 {
        pgrx::error!("ec_spire currently supports single-column indexes only");
    }
    if !index_info.ii_Expressions.is_null() {
        pgrx::error!("ec_spire does not support expression indexes yet");
    }
    if !index_info.ii_Predicate.is_null() {
        pgrx::error!("ec_spire does not support partial indexes yet");
    }

    let attnum = i32::from(index_info.ii_IndexAttrNumbers[0]);
    if attnum <= 0 {
        pgrx::error!("ec_spire requires a base heap column index key");
    }

    let tuple_desc = unsafe { PgTupleDesc::from_pg_copy((*heap_relation).rd_att) };
    let att = tuple_desc
        .get(attnum as usize - 1)
        .expect("resolved indexed attribute should exist");
    if att.attisdropped {
        pgrx::error!("ec_spire indexed column references a dropped column");
    }
    unsafe { resolve_indexed_vector_kind_from_type(att.atttypid) }
        .unwrap_or_else(|| pgrx::error!("ec_spire indexed column must be ecvector or tqvector"))
}

unsafe fn resolve_indexed_vector_kind_from_type(
    type_oid: pg_sys::Oid,
) -> Option<SpireIndexedVectorKind> {
    let base_type_oid = unsafe { pg_sys::getBaseType(type_oid) };
    let formatted = unsafe { pg_sys::format_type_be(base_type_oid) };
    if formatted.is_null() {
        return None;
    }
    let name = unsafe { CStr::from_ptr(formatted) }
        .to_string_lossy()
        .into_owned();
    unsafe { pg_sys::pfree(formatted.cast()) };
    let type_name = name.rsplit('.').next().unwrap_or(&name).trim_matches('"');
    match type_name {
        "ecvector" => Some(SpireIndexedVectorKind::Ecvector),
        "tqvector" => Some(SpireIndexedVectorKind::Tqvector),
        _ => None,
    }
}

pub(super) unsafe extern "C-unwind" fn ec_spire_ambuild(
    heap_relation: pg_sys::Relation,
    index_relation: pg_sys::Relation,
    index_info: *mut pg_sys::IndexInfo,
) -> *mut pg_sys::IndexBuildResult {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            let options = options::relation_options(index_relation);
            page::initialize_root_control_page(index_relation, SpireRootControlState::empty());
            let indexed_vector_kind =
                resolve_indexed_vector_kind(heap_relation, index_info, "ambuild");
            let mut state = SpireBuildState::new(options, indexed_vector_kind);
            let heap_tuples = pg_sys::table_index_build_scan(
                heap_relation,
                index_relation,
                index_info,
                false,
                false,
                Some(ec_spire_build_callback),
                (&mut state as *mut SpireBuildState).cast(),
                ptr::null_mut(),
            );
            let index_tuples = if state.scanned_tuples == 0 {
                0.0
            } else {
                publish_relation_partitioned_single_level_build(index_relation, &state)
                    .unwrap_or_else(|e| pgrx::error!("ec_spire populated ambuild failed: {e}"))
                    as f64
            };

            let mut result = PgBox::<pg_sys::IndexBuildResult>::alloc0();
            result.heap_tuples = heap_tuples;
            result.index_tuples = index_tuples;
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

unsafe extern "C-unwind" fn ec_spire_build_callback(
    _index: pg_sys::Relation,
    tid: pg_sys::ItemPointer,
    values: *mut pg_sys::Datum,
    isnull: *mut bool,
    _tuple_is_alive: bool,
    state: *mut c_void,
) {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            let state = &mut *state.cast::<SpireBuildState>();
            let heap_tid = decode_heap_tid(tid, "ambuild");
            let tuple = build_spire_index_tuple(
                values,
                isnull,
                heap_tid,
                state.indexed_vector_kind,
                state.options.assignment_payload_format(),
                "ambuild",
            );
            state.push(tuple);
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{
        build_partitioned_single_level_leaf_epoch_draft, build_single_level_leaf_epoch_draft,
        object_manifest_from_placement_writes, object_write_evidence_from_placement_directory,
        placement_write_evidence_from_object_manifest, resolve_training_sample_count,
        train_single_level_centroid_plan, SpireBuildState, SpireBuildTuple, SpireIndexedVectorKind,
        SpirePartitionedSingleLevelBuildInput, SpirePublishPlacementWriteEvidence,
        SpirePublishStage, SpirePublishWritingObjects, SpireSingleLevelBuildInput,
        SpireSingleLevelCentroidPlan, SpireSingleLevelRouteMap,
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
    use crate::am::ec_spire::quantizer::{self, SpireAssignmentPayloadFormat};
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

    fn options(training_sample_rows: i32) -> super::options::EcSpireOptions {
        super::options::EcSpireOptions {
            nlists: 2,
            nprobe: 0,
            rerank_width: 0,
            training_sample_rows,
            seed: 7,
            pq_group_size: 0,
            storage_format: super::options::SpireStorageFormat::TurboQuant,
        }
    }

    fn build_tuple(offset_number: u16, source_vector: Vec<f32>) -> SpireBuildTuple {
        let heap_tid = tid(10, offset_number);
        let assignment = quantizer::encode_assignment_input(
            SpireAssignmentPayloadFormat::TurboQuant,
            heap_tid,
            &source_vector,
        )
        .unwrap();
        SpireBuildTuple {
            heap_tid,
            dimensions: source_vector.len() as u16,
            assignment,
            source_vector,
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
    fn build_state_collects_assignments_and_training_sample() {
        let mut state = SpireBuildState::new(options(1), SpireIndexedVectorKind::Ecvector);

        state.try_push(build_tuple(1, vec![1.0, 0.0])).unwrap();
        state.try_push(build_tuple(2, vec![0.0, 1.0])).unwrap();

        assert_eq!(state.scanned_tuples, 2);
        assert_eq!(state.dimensions, Some(2));
        assert_eq!(state.tuples.len(), 2);
        assert_eq!(state.training_sample_count(), 1);
        assert_eq!(state.training_sample_vectors().len(), 1);
        assert_eq!(resolve_training_sample_count(0, 12_000), 10_000);
    }

    #[test]
    fn build_state_trains_centroid_plan_for_all_collected_rows() {
        let mut state = SpireBuildState::new(options(1), SpireIndexedVectorKind::Ecvector);
        state.try_push(build_tuple(1, vec![1.0, 0.0])).unwrap();
        state.try_push(build_tuple(2, vec![0.0, 1.0])).unwrap();
        state.try_push(build_tuple(3, vec![-1.0, 0.0])).unwrap();

        let plan = state.train_centroid_plan().unwrap();

        assert_eq!(plan.dimensions, 2);
        assert_eq!(plan.centroid_count(), 2);
        assert_eq!(plan.assignment_indexes.len(), 3);
    }

    #[test]
    fn build_state_rejects_invalid_tuple_without_advancing() {
        let mut state = SpireBuildState::new(options(0), SpireIndexedVectorKind::Ecvector);
        state.try_push(build_tuple(1, vec![1.0, 0.0])).unwrap();
        let mut bad = build_tuple(2, vec![0.0, 1.0]);
        bad.dimensions = 3;

        let error = state.try_push(bad).unwrap_err();

        assert!(error.contains("source dimensions mismatch"));
        assert_eq!(state.scanned_tuples, 1);
        assert_eq!(state.tuples.len(), 1);
    }

    #[test]
    fn build_state_rejects_payload_format_mismatch() {
        let mut state = SpireBuildState::new(options(0), SpireIndexedVectorKind::Ecvector);
        let mut bad = build_tuple(1, vec![1.0, 0.0]);
        bad.assignment.payload_format = SpireAssignmentPayloadFormat::RaBitQ.tag();

        let error = state.try_push(bad).unwrap_err();

        assert!(error.contains("payload format"));
        assert_eq!(state.scanned_tuples, 0);
    }

    #[test]
    fn build_state_rejects_zero_vectors() {
        let mut state = SpireBuildState::new(options(0), SpireIndexedVectorKind::Ecvector);
        let mut bad = build_tuple(1, vec![1.0, 0.0]);
        bad.source_vector = vec![0.0, 0.0];

        let error = state.try_push(bad).unwrap_err();

        assert!(error.contains("non-zero"));
        assert_eq!(state.scanned_tuples, 0);
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
    fn object_manifest_from_placement_writes_uses_durable_placement_tids() {
        let (draft, _, _, _) = build_valid_draft();
        let evidence = vec![SpirePublishPlacementWriteEvidence {
            pid: SPIRE_FIRST_PID,
            placement_tid: tid(90, 7),
        }];

        let manifest = object_manifest_from_placement_writes(
            draft.epoch_manifest.epoch,
            &draft.placement_directory,
            &evidence,
        )
        .unwrap();

        let entry = manifest.get(SPIRE_FIRST_PID).unwrap();
        assert_eq!(
            entry.object_version,
            draft.leaf_object.header.object_version
        );
        assert_eq!(entry.placement_tid, tid(90, 7));
    }

    #[test]
    fn object_manifest_from_placement_writes_rejects_missing_or_duplicate_evidence() {
        let (draft, _, _, _) = build_valid_draft();

        assert!(object_manifest_from_placement_writes(
            draft.epoch_manifest.epoch,
            &draft.placement_directory,
            &[],
        )
        .unwrap_err()
        .contains("count mismatch"));

        let duplicate = vec![
            SpirePublishPlacementWriteEvidence {
                pid: SPIRE_FIRST_PID,
                placement_tid: tid(90, 7),
            },
            SpirePublishPlacementWriteEvidence {
                pid: SPIRE_FIRST_PID,
                placement_tid: tid(90, 8),
            },
        ];
        let mut duplicate_directory = draft.placement_directory.clone();
        duplicate_directory
            .entries
            .push(duplicate_directory.entries[0]);
        assert!(object_manifest_from_placement_writes(
            draft.epoch_manifest.epoch,
            &duplicate_directory,
            &duplicate,
        )
        .unwrap_err()
        .contains("duplicate pid"));
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
