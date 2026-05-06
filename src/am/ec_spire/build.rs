use std::collections::{HashMap, HashSet};
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
    SpireObjectManifest, SpirePlacementDirectory, SpirePlacementEntry, SpireRootControlState,
    SpireValidatedEpochSnapshot, SPIRE_MIN_EPOCH_RETENTION_SECS,
};
use super::storage::{
    SpireLeafAssignmentRow, SpireLeafPartitionObject, SpireLocalObjectStore, SpireObjectReader,
    SpirePartitionObjectKind, SpireRelationObjectStore, SpireRoutingChildEntry,
    SpireRoutingPartitionObject,
};
use super::{options, page};
use super::{quantizer, quantizer::SpireAssignmentPayloadFormat};
use crate::am::common::training as common_training;
use crate::quant::prod::ProdQuantizer;
use crate::storage::page::ItemPointer;

pub(super) const SPIRE_DEFAULT_KMEANS_ITERATIONS: usize = 8;
const SPIRE_DEFAULT_AUTO_TRAINING_SAMPLE_ROWS: usize = 10_000;
pub(super) const SPIRE_INITIAL_EPOCH: u64 = 1;
pub(super) const SPIRE_INITIAL_OBJECT_VERSION: u64 = 1;
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

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SpireRecursiveRoutingChildInput {
    pub(super) child_pid: u64,
    pub(super) child_level: u16,
    pub(super) centroid: Vec<f32>,
    pub(super) source_count: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SpireRecursiveRoutingBuildInput {
    pub(super) object_version: u64,
    pub(super) dimensions: u16,
    pub(super) target_fanout: u32,
    pub(super) seed: u64,
    pub(super) children: Vec<SpireRecursiveRoutingChildInput>,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SpireRecursiveRoutingBuildDraft {
    pub(super) root_pid: u64,
    pub(super) root_level: u16,
    pub(super) routing_objects: Vec<SpireRoutingPartitionObject>,
    pub(super) centroid_records: Vec<SpireRecursiveCentroidRecord>,
    pub(super) next_pid: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SpireRecursiveCentroidRecord {
    pub(super) parent_pid: u64,
    pub(super) child_pid: u64,
    pub(super) child_level: u16,
    pub(super) centroid_ordinal: u32,
    pub(super) dimensions: u16,
    pub(super) centroid: Vec<f32>,
    pub(super) source_count: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SpireRecursiveBuildCoordinatorInput {
    pub(super) epoch: u64,
    pub(super) object_version: u64,
    pub(super) published_at_micros: i64,
    pub(super) retain_until_micros: i64,
    pub(super) consistency_mode: SpireConsistencyMode,
    pub(super) target_fanout: u32,
    pub(super) seed: u64,
    pub(super) assignments: Vec<SpireLeafAssignmentInput>,
    pub(super) centroid_plan: SpireSingleLevelCentroidPlan,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SpireRecursiveBuildCoordinatorDraft {
    pub(super) epoch_input: SpireRecursiveRoutingEpochObjectInput,
    pub(super) leaf_pids: Vec<u64>,
    pub(super) next_pid: u64,
    pub(super) next_local_vec_seq: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SpireRecursiveLeafObjectInput {
    pub(super) pid: u64,
    pub(super) object_version: u64,
    pub(super) parent_pid: u64,
    pub(super) rows: Vec<SpireLeafAssignmentRow>,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SpireRecursiveRoutingEpochObjectInput {
    pub(super) epoch: u64,
    pub(super) published_at_micros: i64,
    pub(super) retain_until_micros: i64,
    pub(super) consistency_mode: SpireConsistencyMode,
    pub(super) routing_draft: SpireRecursiveRoutingBuildDraft,
    pub(super) leaf_inputs: Vec<SpireRecursiveLeafObjectInput>,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SpireRecursiveRoutingEpochInput {
    pub(super) epoch: u64,
    pub(super) published_at_micros: i64,
    pub(super) retain_until_micros: i64,
    pub(super) consistency_mode: SpireConsistencyMode,
    pub(super) routing_draft: SpireRecursiveRoutingBuildDraft,
    pub(super) leaf_placements: Vec<SpirePlacementEntry>,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SpireRecursiveRoutingEpochDraft {
    pub(super) epoch_manifest: SpireEpochManifest,
    pub(super) object_manifest: SpireObjectManifest,
    pub(super) placement_directory: SpirePlacementDirectory,
    pub(super) root_pid: u64,
    pub(super) routing_objects: Vec<SpireRoutingPartitionObject>,
    // TODO: these are not persisted separately; diagnostics rebuild them with
    // centroid_records_from_routing until durable centroid objects land.
    pub(super) centroid_records: Vec<SpireRecursiveCentroidRecord>,
    pub(super) next_pid: u64,
}

#[derive(Debug, Clone, PartialEq)]
struct SpireRecursiveDraftInvariants {
    leaf_parent_pids: HashMap<u64, u64>,
}

trait SpireRecursiveRoutingEpochObjectStore: SpireObjectReader {
    fn write_recursive_routing_object(
        &mut self,
        epoch: u64,
        object: &SpireRoutingPartitionObject,
    ) -> Result<SpirePlacementEntry, String>;

    fn write_recursive_leaf_object_v2_from_rows(
        &mut self,
        epoch: u64,
        pid: u64,
        object_version: u64,
        parent_pid: u64,
        rows: &[SpireLeafAssignmentRow],
    ) -> Result<SpirePlacementEntry, String>;
}

impl SpireRecursiveRoutingEpochObjectStore for SpireLocalObjectStore {
    fn write_recursive_routing_object(
        &mut self,
        epoch: u64,
        object: &SpireRoutingPartitionObject,
    ) -> Result<SpirePlacementEntry, String> {
        self.insert_routing_object(epoch, object)
    }

    fn write_recursive_leaf_object_v2_from_rows(
        &mut self,
        epoch: u64,
        pid: u64,
        object_version: u64,
        parent_pid: u64,
        rows: &[SpireLeafAssignmentRow],
    ) -> Result<SpirePlacementEntry, String> {
        self.insert_leaf_object_v2_from_rows(epoch, pid, object_version, parent_pid, rows)
    }
}

impl SpireRecursiveRoutingEpochObjectStore for SpireRelationObjectStore {
    fn write_recursive_routing_object(
        &mut self,
        epoch: u64,
        object: &SpireRoutingPartitionObject,
    ) -> Result<SpirePlacementEntry, String> {
        unsafe { self.insert_routing_object(epoch, object) }
    }

    fn write_recursive_leaf_object_v2_from_rows(
        &mut self,
        epoch: u64,
        pid: u64,
        object_version: u64,
        parent_pid: u64,
        rows: &[SpireLeafAssignmentRow],
    ) -> Result<SpirePlacementEntry, String> {
        unsafe {
            self.insert_leaf_object_v2_from_rows(epoch, pid, object_version, parent_pid, rows)
        }
    }
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

fn retired_epoch_manifest_from(
    previous_epoch_manifest: SpireEpochManifest,
) -> Result<SpireEpochManifest, String> {
    if previous_epoch_manifest.state != SpireEpochState::Published {
        return Err("ec_spire can only retire a previously published epoch manifest".to_owned());
    }
    let retired_epoch_manifest = SpireEpochManifest {
        state: SpireEpochState::Retired,
        active_query_count: 0,
        ..previous_epoch_manifest
    };
    retired_epoch_manifest.validate()?;
    Ok(retired_epoch_manifest)
}

pub(super) unsafe fn write_retired_epoch_manifest_to_relation(
    index_relation: pg_sys::Relation,
    previous_epoch_manifest: SpireEpochManifest,
) -> Result<ItemPointer, String> {
    let retired_epoch_manifest = retired_epoch_manifest_from(previous_epoch_manifest)?;
    let encoded = retired_epoch_manifest.encode()?;
    // Replacement publishes append this retired copy before the new manifest
    // bundle while holding the publish/extension lock, so its TID orders after
    // the original published manifest for snapshot dedupe.
    unsafe { page::append_object_tuple(index_relation, &encoded) }
}

pub(super) unsafe fn publish_replacement_epoch_to_relation(
    index_relation: pg_sys::Relation,
    previous_epoch_manifest: SpireEpochManifest,
    input: SpirePublishCoordinatorInput<'_>,
) -> Result<(), String> {
    let manifests = encode_manifest_bundle_for_publish(input)?;
    unsafe { write_retired_epoch_manifest_to_relation(index_relation, previous_epoch_manifest)? };
    let locators = unsafe { write_manifest_bundle_to_relation(index_relation, &manifests)? };
    let root_control = root_control_state_for_publish(input, locators)?;
    unsafe { page::initialize_root_control_page(index_relation, root_control) };
    Ok(())
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

#[derive(Debug, Clone)]
struct SpirePendingRecursiveRoutingNode {
    pid: u64,
    level: u16,
    centroid: Vec<f32>,
    source_count: u64,
    children: Vec<SpireRecursiveRoutingChildInput>,
}

pub(super) fn build_recursive_routing_hierarchy_draft(
    input: SpireRecursiveRoutingBuildInput,
    pid_allocator: &mut SpirePidAllocator,
) -> Result<SpireRecursiveRoutingBuildDraft, String> {
    input.validate()?;
    let target_fanout = usize::try_from(input.target_fanout)
        .map_err(|_| "ec_spire recursive routing fanout exceeds usize".to_owned())?;
    let mut pid_cursor = *pid_allocator;
    let mut current_children = input.children;
    let mut pending_nodes = Vec::new();

    while current_children.len() > target_fanout {
        let child_level = current_children[0].child_level;
        let parent_level = child_level
            .checked_add(1)
            .ok_or_else(|| "ec_spire recursive routing level overflow".to_owned())?;
        let source_vectors = current_children
            .iter()
            .map(|child| child.centroid.as_slice())
            .collect::<Vec<_>>();
        let model = common_training::train_spherical_kmeans(
            "ec_spire recursive routing",
            &source_vectors,
            usize::from(input.dimensions),
            target_fanout,
            input.seed.wrapping_add(u64::from(parent_level)),
            SPIRE_DEFAULT_KMEANS_ITERATIONS,
        )?;
        let mut grouped_children = vec![Vec::new(); model.centroid_count()];
        for child in current_children {
            let centroid_index = common_training::assign_vector_to_centroid(
                "ec_spire recursive routing",
                &child.centroid,
                &model,
            )?;
            grouped_children[centroid_index].push(child);
        }

        let mut next_children = Vec::new();
        for (centroid_index, children) in grouped_children.into_iter().enumerate() {
            if children.is_empty() {
                continue;
            }
            let pid = pid_cursor.allocate()?;
            let source_count = sum_recursive_source_counts(&children)?;
            let centroid = model.centroids[centroid_index].clone();
            pending_nodes.push(SpirePendingRecursiveRoutingNode {
                pid,
                level: parent_level,
                centroid: centroid.clone(),
                source_count,
                children,
            });
            next_children.push(SpireRecursiveRoutingChildInput {
                child_pid: pid,
                child_level: parent_level,
                centroid,
                source_count,
            });
        }
        current_children = next_children;
    }

    let root_level = current_children[0]
        .child_level
        .checked_add(1)
        .ok_or_else(|| "ec_spire recursive routing root level overflow".to_owned())?;
    let root_pid = pid_cursor.allocate()?;
    let pending_by_pid = pending_nodes
        .iter()
        .map(|node| (node.pid, node))
        .collect::<HashMap<_, _>>();
    let mut routing_objects = Vec::with_capacity(pending_nodes.len() + 1);
    let mut centroid_records = Vec::new();
    routing_objects.push(SpireRoutingPartitionObject::root_at_level(
        root_pid,
        input.object_version,
        root_level,
        input.dimensions,
        routing_child_entries(&current_children)?,
    )?);
    extend_recursive_centroid_records(
        &mut centroid_records,
        root_pid,
        input.dimensions,
        &current_children,
    )?;
    let mut visited_internal_pids = HashSet::with_capacity(pending_nodes.len());
    for child in &current_children {
        materialize_pending_recursive_child(
            child,
            root_pid,
            input.object_version,
            input.dimensions,
            &pending_by_pid,
            &mut visited_internal_pids,
            &mut routing_objects,
            &mut centroid_records,
        )?;
    }
    if visited_internal_pids.len() != pending_nodes.len() {
        return Err("ec_spire recursive routing contains unreachable internal nodes".to_owned());
    }

    let draft = SpireRecursiveRoutingBuildDraft {
        root_pid,
        root_level,
        routing_objects,
        centroid_records,
        next_pid: pid_cursor.next_pid(),
    };
    assert_recursive_draft_invariants(&draft)?;
    *pid_allocator = pid_cursor;
    Ok(draft)
}

pub(super) fn build_recursive_epoch_input_from_centroid_plan(
    input: SpireRecursiveBuildCoordinatorInput,
    pid_allocator: &mut SpirePidAllocator,
    local_vec_id_allocator: &mut SpireLocalVecIdAllocator,
) -> Result<SpireRecursiveBuildCoordinatorDraft, String> {
    input.centroid_plan.validate()?;
    let centroid_count = input.centroid_plan.centroid_count();
    if input.assignments.len() != input.centroid_plan.assignment_indexes.len() {
        return Err(format!(
            "ec_spire recursive build assignment count {} does not match centroid assignment count {}",
            input.assignments.len(),
            input.centroid_plan.assignment_indexes.len()
        ));
    }

    let assignments_by_centroid = group_assignments_by_centroid(
        input.assignments,
        &input.centroid_plan.assignment_indexes,
        centroid_count,
    )?;
    let mut pid_cursor = *pid_allocator;
    let mut local_vec_id_cursor = *local_vec_id_allocator;
    let mut leaf_pids = Vec::with_capacity(centroid_count);
    for _ in 0..centroid_count {
        leaf_pids.push(pid_cursor.allocate()?);
    }
    let routing_draft = build_recursive_routing_hierarchy_draft(
        SpireRecursiveRoutingBuildInput {
            object_version: input.object_version,
            dimensions: input.centroid_plan.dimensions,
            target_fanout: input.target_fanout,
            seed: input.seed,
            children: leaf_pids
                .iter()
                .copied()
                .zip(input.centroid_plan.centroids.iter())
                .map(|(child_pid, centroid)| SpireRecursiveRoutingChildInput {
                    child_pid,
                    child_level: 0,
                    centroid: centroid.clone(),
                    // First-level recursive children are trained leaf centroids, so this counts
                    // one centroid source rather than rows assigned to the eventual leaf object.
                    source_count: 1,
                })
                .collect(),
        },
        &mut pid_cursor,
    )?;
    let leaf_parent_pids = assert_recursive_draft_invariants(&routing_draft)?.leaf_parent_pids;
    let mut leaf_inputs = Vec::with_capacity(centroid_count);
    for (pid, assignments) in leaf_pids
        .iter()
        .copied()
        .zip(assignments_by_centroid.into_iter())
    {
        let parent_pid = *leaf_parent_pids.get(&pid).ok_or_else(|| {
            format!("ec_spire recursive build missing routing parent for leaf pid {pid}")
        })?;
        leaf_inputs.push(SpireRecursiveLeafObjectInput {
            pid,
            object_version: input.object_version,
            parent_pid,
            rows: build_primary_leaf_assignments(&mut local_vec_id_cursor, assignments)?,
        });
    }

    let draft = SpireRecursiveBuildCoordinatorDraft {
        epoch_input: SpireRecursiveRoutingEpochObjectInput {
            epoch: input.epoch,
            published_at_micros: input.published_at_micros,
            retain_until_micros: input.retain_until_micros,
            consistency_mode: input.consistency_mode,
            routing_draft,
            leaf_inputs,
        },
        leaf_pids,
        next_pid: pid_cursor.next_pid(),
        next_local_vec_seq: local_vec_id_cursor.next_local_vec_seq(),
    };
    *pid_allocator = pid_cursor;
    *local_vec_id_allocator = local_vec_id_cursor;
    Ok(draft)
}

impl SpireRecursiveRoutingBuildInput {
    fn validate(&self) -> Result<(), String> {
        if self.object_version == 0 {
            return Err("ec_spire recursive routing object_version 0 is invalid".to_owned());
        }
        if self.dimensions == 0 {
            return Err("ec_spire recursive routing requires dimensions > 0".to_owned());
        }
        if self.target_fanout < 2 {
            return Err("ec_spire recursive routing fanout must be at least 2".to_owned());
        }
        if self.children.is_empty() {
            return Err("ec_spire recursive routing requires at least one child".to_owned());
        }
        let expected_level = self.children[0].child_level;
        let mut child_pids = HashSet::with_capacity(self.children.len());
        for child in &self.children {
            if child.child_pid == 0 {
                return Err("ec_spire recursive routing child pid 0 is invalid".to_owned());
            }
            if !child_pids.insert(child.child_pid) {
                return Err(format!(
                    "ec_spire recursive routing duplicate child pid {}",
                    child.child_pid
                ));
            }
            if child.child_level != expected_level {
                return Err(format!(
                    "ec_spire recursive routing child pid {} level {} does not match expected level {expected_level}",
                    child.child_pid, child.child_level
                ));
            }
            if child.source_count == 0 {
                return Err(format!(
                    "ec_spire recursive routing child pid {} source_count 0 is invalid",
                    child.child_pid
                ));
            }
            validate_recursive_centroid(self.dimensions, child.child_pid, &child.centroid)?;
        }
        Ok(())
    }
}

fn validate_recursive_centroid(
    dimensions: u16,
    child_pid: u64,
    centroid: &[f32],
) -> Result<(), String> {
    let expected = usize::from(dimensions);
    if centroid.len() != expected {
        return Err(format!(
            "ec_spire recursive routing child pid {child_pid} centroid dimensions mismatch: got {}, expected {expected}",
            centroid.len()
        ));
    }
    if centroid.iter().any(|component| !component.is_finite()) {
        return Err(format!(
            "ec_spire recursive routing child pid {child_pid} centroid must be finite"
        ));
    }
    Ok(())
}

fn sum_recursive_source_counts(
    children: &[SpireRecursiveRoutingChildInput],
) -> Result<u64, String> {
    children.iter().try_fold(0_u64, |sum, child| {
        sum.checked_add(child.source_count)
            .ok_or_else(|| "ec_spire recursive routing source_count overflow".to_owned())
    })
}

fn routing_child_entries(
    children: &[SpireRecursiveRoutingChildInput],
) -> Result<Vec<SpireRoutingChildEntry>, String> {
    children
        .iter()
        .enumerate()
        .map(|(index, child)| {
            Ok(SpireRoutingChildEntry {
                centroid_index: u32::try_from(index)
                    .map_err(|_| "ec_spire recursive routing child index exceeds u32".to_owned())?,
                child_pid: child.child_pid,
                centroid: child.centroid.clone(),
            })
        })
        .collect()
}

fn extend_recursive_centroid_records(
    centroid_records: &mut Vec<SpireRecursiveCentroidRecord>,
    parent_pid: u64,
    dimensions: u16,
    children: &[SpireRecursiveRoutingChildInput],
) -> Result<(), String> {
    for (index, child) in children.iter().enumerate() {
        centroid_records.push(SpireRecursiveCentroidRecord {
            parent_pid,
            child_pid: child.child_pid,
            child_level: child.child_level,
            centroid_ordinal: u32::try_from(index)
                .map_err(|_| "ec_spire recursive centroid ordinal exceeds u32".to_owned())?,
            dimensions,
            centroid: child.centroid.clone(),
            source_count: child.source_count,
        });
    }
    Ok(())
}

fn materialize_pending_recursive_child(
    child: &SpireRecursiveRoutingChildInput,
    parent_pid: u64,
    object_version: u64,
    dimensions: u16,
    pending_by_pid: &HashMap<u64, &SpirePendingRecursiveRoutingNode>,
    visited_internal_pids: &mut HashSet<u64>,
    routing_objects: &mut Vec<SpireRoutingPartitionObject>,
    centroid_records: &mut Vec<SpireRecursiveCentroidRecord>,
) -> Result<(), String> {
    if child.child_level == 0 {
        return Ok(());
    }
    let node = pending_by_pid.get(&child.child_pid).ok_or_else(|| {
        format!(
            "ec_spire recursive routing missing internal node pid {}",
            child.child_pid
        )
    })?;
    if node.level != child.child_level {
        return Err(format!(
            "ec_spire recursive routing internal node pid {} level {} does not match child level {}",
            node.pid, node.level, child.child_level
        ));
    }
    if node.source_count != child.source_count {
        return Err(format!(
            "ec_spire recursive routing internal node pid {} source_count {} does not match child source_count {}",
            node.pid, node.source_count, child.source_count
        ));
    }
    if node.centroid != child.centroid {
        return Err(format!(
            "ec_spire recursive routing internal node pid {} centroid drift",
            node.pid
        ));
    }
    if !visited_internal_pids.insert(node.pid) {
        return Err(format!(
            "ec_spire recursive routing internal node pid {} reached twice",
            node.pid
        ));
    }
    routing_objects.push(SpireRoutingPartitionObject::internal(
        node.pid,
        object_version,
        node.level,
        parent_pid,
        dimensions,
        routing_child_entries(&node.children)?,
    )?);
    extend_recursive_centroid_records(centroid_records, node.pid, dimensions, &node.children)?;
    for child in &node.children {
        materialize_pending_recursive_child(
            child,
            node.pid,
            object_version,
            dimensions,
            pending_by_pid,
            visited_internal_pids,
            routing_objects,
            centroid_records,
        )?;
    }
    Ok(())
}

fn validate_recursive_routing_build_draft(
    draft: &SpireRecursiveRoutingBuildDraft,
) -> Result<(), String> {
    // Recursive drafts pass three validation barriers:
    // 1. this in-memory routing-object and centroid-record shape check;
    // 2. epoch leaf-placement validation after object writes;
    // 3. snapshot-time hierarchy validation before scan descent.
    if draft.routing_objects.is_empty() {
        return Err("ec_spire recursive routing draft requires routing objects".to_owned());
    }
    if draft.routing_objects[0].header.kind != super::storage::SpirePartitionObjectKind::Root {
        return Err("ec_spire recursive routing draft first object must be root".to_owned());
    }
    if draft.routing_objects[0].header.pid != draft.root_pid {
        return Err("ec_spire recursive routing draft root pid mismatch".to_owned());
    }
    if draft.routing_objects[0].header.level != draft.root_level {
        return Err("ec_spire recursive routing draft root level mismatch".to_owned());
    }
    let mut pids = HashSet::with_capacity(draft.routing_objects.len());
    for object in &draft.routing_objects {
        if !pids.insert(object.header.pid) {
            return Err(format!(
                "ec_spire recursive routing draft duplicate routing pid {}",
                object.header.pid
            ));
        }
    }
    let mut centroid_keys = HashSet::with_capacity(draft.centroid_records.len());
    let mut centroid_ordinals_by_parent = HashMap::<u64, Vec<u32>>::new();
    for record in &draft.centroid_records {
        validate_recursive_centroid(record.dimensions, record.child_pid, &record.centroid)?;
        if !centroid_keys.insert((record.parent_pid, record.child_pid)) {
            return Err(format!(
                "ec_spire recursive routing draft duplicate centroid record parent {} child {}",
                record.parent_pid, record.child_pid
            ));
        }
        if record.source_count == 0 {
            return Err(format!(
                "ec_spire recursive routing draft centroid record child {} source_count 0",
                record.child_pid
            ));
        }
        centroid_ordinals_by_parent
            .entry(record.parent_pid)
            .or_default()
            .push(record.centroid_ordinal);
    }
    for (parent_pid, mut ordinals) in centroid_ordinals_by_parent {
        ordinals.sort_unstable();
        for (position, ordinal) in ordinals.into_iter().enumerate() {
            let expected = u32::try_from(position).map_err(|_| {
                "ec_spire recursive routing centroid ordinal exceeds u32".to_owned()
            })?;
            if ordinal != expected {
                return Err(format!(
                    "ec_spire recursive routing draft centroid ordinals for parent {parent_pid} are not dense at position {position}: got {ordinal}"
                ));
            }
        }
    }
    Ok(())
}

fn assert_recursive_draft_invariants(
    draft: &SpireRecursiveRoutingBuildDraft,
) -> Result<SpireRecursiveDraftInvariants, String> {
    validate_recursive_routing_build_draft(draft)?;
    Ok(SpireRecursiveDraftInvariants {
        leaf_parent_pids: recursive_routing_leaf_parent_pids(draft)?,
    })
}

pub(super) fn build_local_recursive_routing_epoch_draft(
    input: SpireRecursiveRoutingEpochInput,
    object_store: &mut SpireLocalObjectStore,
) -> Result<SpireRecursiveRoutingEpochDraft, String> {
    build_recursive_routing_epoch_draft_with_store(input, object_store)
}

pub(super) unsafe fn build_relation_recursive_routing_epoch_draft(
    input: SpireRecursiveRoutingEpochInput,
    object_store: &mut SpireRelationObjectStore,
) -> Result<SpireRecursiveRoutingEpochDraft, String> {
    build_recursive_routing_epoch_draft_with_store(input, object_store)
}

pub(super) fn build_local_recursive_routing_epoch_from_leaf_inputs(
    input: SpireRecursiveRoutingEpochObjectInput,
    object_store: &mut SpireLocalObjectStore,
) -> Result<SpireRecursiveRoutingEpochDraft, String> {
    build_recursive_routing_epoch_from_leaf_inputs_with_store(input, object_store)
}

pub(super) unsafe fn build_relation_recursive_routing_epoch_from_leaf_inputs(
    input: SpireRecursiveRoutingEpochObjectInput,
    object_store: &mut SpireRelationObjectStore,
) -> Result<SpireRecursiveRoutingEpochDraft, String> {
    build_recursive_routing_epoch_from_leaf_inputs_with_store(input, object_store)
}

fn build_recursive_routing_epoch_from_leaf_inputs_with_store(
    input: SpireRecursiveRoutingEpochObjectInput,
    object_store: &mut impl SpireRecursiveRoutingEpochObjectStore,
) -> Result<SpireRecursiveRoutingEpochDraft, String> {
    let invariants = assert_recursive_draft_invariants(&input.routing_draft)?;
    let expected_leaf_parents = invariants.leaf_parent_pids;
    let mut seen_leaf_pids = HashSet::with_capacity(input.leaf_inputs.len());
    let mut leaf_placements = Vec::with_capacity(input.leaf_inputs.len());
    for leaf_input in input.leaf_inputs {
        if !seen_leaf_pids.insert(leaf_input.pid) {
            return Err(format!(
                "ec_spire recursive routing epoch duplicate leaf object input pid {}",
                leaf_input.pid
            ));
        }
        let expected_parent_pid = expected_leaf_parents.get(&leaf_input.pid).ok_or_else(|| {
            format!(
                "ec_spire recursive routing epoch unexpected leaf object input pid {}",
                leaf_input.pid
            )
        })?;
        if leaf_input.parent_pid != *expected_parent_pid {
            return Err(format!(
                "ec_spire recursive routing epoch leaf object input pid {} parent {} does not match routing parent {}",
                leaf_input.pid, leaf_input.parent_pid, expected_parent_pid
            ));
        }
        leaf_placements.push(object_store.write_recursive_leaf_object_v2_from_rows(
            input.epoch,
            leaf_input.pid,
            leaf_input.object_version,
            leaf_input.parent_pid,
            &leaf_input.rows,
        )?);
    }
    let expected_leaf_pids = expected_leaf_parents
        .keys()
        .copied()
        .collect::<HashSet<_>>();
    if seen_leaf_pids != expected_leaf_pids {
        let missing = expected_leaf_pids
            .difference(&seen_leaf_pids)
            .copied()
            .collect::<Vec<_>>();
        let extra = seen_leaf_pids
            .difference(&expected_leaf_pids)
            .copied()
            .collect::<Vec<_>>();
        return Err(format!(
            "ec_spire recursive routing epoch leaf object input mismatch: missing {missing:?}, extra {extra:?}"
        ));
    }

    build_recursive_routing_epoch_draft_with_store(
        SpireRecursiveRoutingEpochInput {
            epoch: input.epoch,
            published_at_micros: input.published_at_micros,
            retain_until_micros: input.retain_until_micros,
            consistency_mode: input.consistency_mode,
            routing_draft: input.routing_draft,
            leaf_placements,
        },
        object_store,
    )
}

fn build_recursive_routing_epoch_draft_with_store(
    input: SpireRecursiveRoutingEpochInput,
    object_store: &mut impl SpireRecursiveRoutingEpochObjectStore,
) -> Result<SpireRecursiveRoutingEpochDraft, String> {
    let invariants = assert_recursive_draft_invariants(&input.routing_draft)?;

    let epoch_manifest = SpireEpochManifest {
        epoch: input.epoch,
        state: SpireEpochState::Published,
        consistency_mode: input.consistency_mode,
        published_at_micros: input.published_at_micros,
        retain_until_micros: input.retain_until_micros,
        active_query_count: 0,
    };
    epoch_manifest.validate()?;

    validate_recursive_epoch_leaf_placements(&input, &invariants.leaf_parent_pids, object_store)?;

    let mut placements =
        Vec::with_capacity(input.routing_draft.routing_objects.len() + input.leaf_placements.len());
    for object in &input.routing_draft.routing_objects {
        placements.push(object_store.write_recursive_routing_object(input.epoch, object)?);
    }
    placements.extend(input.leaf_placements);

    let object_manifest = SpireObjectManifest::from_entries(
        input.epoch,
        placements
            .iter()
            .map(|placement| SpireManifestEntry {
                epoch: placement.epoch,
                pid: placement.pid,
                object_version: placement.object_version,
                placement_tid: placement.object_tid,
            })
            .collect(),
    )?;
    let placement_directory = SpirePlacementDirectory::from_entries(input.epoch, placements)?;

    let next_pid =
        next_recursive_epoch_pid(input.routing_draft.next_pid, &placement_directory.entries)?;
    let draft = SpireRecursiveRoutingEpochDraft {
        epoch_manifest,
        object_manifest,
        placement_directory,
        root_pid: input.routing_draft.root_pid,
        centroid_records: input.routing_draft.centroid_records.clone(),
        routing_objects: input.routing_draft.routing_objects,
        next_pid,
    };
    SpireValidatedEpochSnapshot::new(
        &draft.epoch_manifest,
        &draft.object_manifest,
        &draft.placement_directory,
    )?;
    Ok(draft)
}

fn validate_recursive_epoch_leaf_placements(
    input: &SpireRecursiveRoutingEpochInput,
    expected_leaf_parents: &HashMap<u64, u64>,
    object_store: &impl SpireObjectReader,
) -> Result<(), String> {
    let expected_leaf_pids = expected_leaf_parents
        .keys()
        .copied()
        .collect::<HashSet<_>>();
    let mut actual_leaf_pids = HashSet::with_capacity(input.leaf_placements.len());
    for placement in &input.leaf_placements {
        placement.encode()?;
        if placement.epoch != input.epoch {
            return Err(format!(
                "ec_spire recursive routing epoch leaf placement pid {} epoch {} does not match epoch {}",
                placement.pid, placement.epoch, input.epoch
            ));
        }
        if !actual_leaf_pids.insert(placement.pid) {
            return Err(format!(
                "ec_spire recursive routing epoch duplicate leaf placement pid {}",
                placement.pid
            ));
        }
        let header = object_store.read_object_header(placement)?;
        if header.kind != SpirePartitionObjectKind::Leaf {
            return Err(format!(
                "ec_spire recursive routing epoch placement pid {} is not a leaf object",
                placement.pid
            ));
        }
        let expected_parent_pid = expected_leaf_parents.get(&placement.pid).ok_or_else(|| {
            format!(
                "ec_spire recursive routing epoch unexpected leaf placement pid {}",
                placement.pid
            )
        })?;
        if header.parent_pid != *expected_parent_pid {
            return Err(format!(
                "ec_spire recursive routing epoch leaf pid {} parent {} does not match routing parent {}",
                placement.pid, header.parent_pid, expected_parent_pid
            ));
        }
    }
    if actual_leaf_pids != expected_leaf_pids {
        let missing = expected_leaf_pids
            .difference(&actual_leaf_pids)
            .copied()
            .collect::<Vec<_>>();
        let extra = actual_leaf_pids
            .difference(&expected_leaf_pids)
            .copied()
            .collect::<Vec<_>>();
        return Err(format!(
            "ec_spire recursive routing epoch leaf placement mismatch: missing {missing:?}, extra {extra:?}"
        ));
    }
    Ok(())
}

fn recursive_routing_leaf_parent_pids(
    draft: &SpireRecursiveRoutingBuildDraft,
) -> Result<HashMap<u64, u64>, String> {
    let mut leaf_parents = HashMap::new();
    for object in &draft.routing_objects {
        if object.header.level != 1 {
            continue;
        }
        for child in object.children() {
            if leaf_parents
                .insert(child.child_pid, object.header.pid)
                .is_some()
            {
                return Err(format!(
                    "ec_spire recursive routing epoch duplicate leaf child pid {}",
                    child.child_pid
                ));
            }
        }
    }
    if leaf_parents.is_empty() {
        return Err("ec_spire recursive routing epoch requires leaf child pids".to_owned());
    }
    Ok(leaf_parents)
}

fn next_recursive_epoch_pid(
    routing_next_pid: u64,
    placements: &[SpirePlacementEntry],
) -> Result<u64, String> {
    placements
        .iter()
        .try_fold(routing_next_pid, |next_pid, placement| {
            let after_placement = placement
                .pid
                .checked_add(1)
                .ok_or_else(|| "ec_spire recursive routing epoch pid overflow".to_owned())?;
            Ok(next_pid.max(after_placement))
        })
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

impl SpireRecursiveRoutingEpochDraft {
    fn publish_input(&self, next_local_vec_seq: u64) -> SpirePublishCoordinatorInput<'_> {
        SpirePublishCoordinatorInput {
            epoch_manifest: &self.epoch_manifest,
            object_manifest: &self.object_manifest,
            placement_directory: &self.placement_directory,
            next_pid: self.next_pid,
            next_local_vec_seq,
        }
    }

    pub(super) fn encode_manifest_bundle(
        &self,
        next_local_vec_seq: u64,
    ) -> Result<SpireEncodedManifestBundle, String> {
        encode_manifest_bundle_for_publish(self.publish_input(next_local_vec_seq))
    }

    pub(super) fn root_control_state(
        &self,
        next_local_vec_seq: u64,
        locators: SpirePublishedManifestLocators,
    ) -> Result<SpireRootControlState, String> {
        root_control_state_for_publish(self.publish_input(next_local_vec_seq), locators)
    }

    pub(super) fn encode_publish_bundle(
        &self,
        next_local_vec_seq: u64,
        locators: SpirePublishedManifestLocators,
    ) -> Result<SpireEncodedPublishBundle, String> {
        encode_publish_bundle_for_publish(self.publish_input(next_local_vec_seq), locators)
    }

    fn relation_publish_input<'a>(
        &'a self,
        object_manifest: &'a SpireObjectManifest,
        next_local_vec_seq: u64,
    ) -> SpirePublishCoordinatorInput<'a> {
        SpirePublishCoordinatorInput {
            epoch_manifest: &self.epoch_manifest,
            object_manifest,
            placement_directory: &self.placement_directory,
            next_pid: self.next_pid,
            next_local_vec_seq,
        }
    }
}

pub(super) unsafe fn publish_relation_recursive_routing_epoch_draft(
    index_relation: pg_sys::Relation,
    draft: &SpireRecursiveRoutingEpochDraft,
    next_local_vec_seq: u64,
) -> Result<(), String> {
    let placement_evidence =
        unsafe { write_placement_entries_to_relation(index_relation, &draft.placement_directory)? };
    let object_manifest = object_manifest_from_placement_writes(
        draft.epoch_manifest.epoch,
        &draft.placement_directory,
        &placement_evidence,
    )?;
    let input = draft.relation_publish_input(&object_manifest, next_local_vec_seq);
    let manifests = encode_manifest_bundle_for_publish(input)?;
    let locators = unsafe { write_manifest_bundle_to_relation(index_relation, &manifests)? };
    let root_control = root_control_state_for_publish(input, locators)?;
    unsafe { page::initialize_root_control_page(index_relation, root_control) };
    Ok(())
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

    let assignments_by_centroid = group_assignments_by_centroid(
        input.assignments,
        &input.centroid_plan.assignment_indexes,
        centroid_count,
    )?;

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

fn group_assignments_by_centroid(
    assignments: Vec<SpireLeafAssignmentInput>,
    assignment_indexes: &[u32],
    centroid_count: usize,
) -> Result<Vec<Vec<SpireLeafAssignmentInput>>, String> {
    if assignments.len() != assignment_indexes.len() {
        return Err(format!(
            "ec_spire centroid assignment count {} does not match assignment index count {}",
            assignments.len(),
            assignment_indexes.len()
        ));
    }
    let mut assignments_by_centroid = vec![Vec::new(); centroid_count];
    for (assignment, assignment_index) in assignments.into_iter().zip(assignment_indexes.iter()) {
        let centroid_index = usize::try_from(*assignment_index)
            .map_err(|_| "ec_spire centroid assignment index exceeds usize".to_owned())?;
        let assignments = assignments_by_centroid.get_mut(centroid_index).ok_or_else(|| {
            format!(
                "ec_spire centroid assignment index {centroid_index} exceeds centroid count {centroid_count}"
            )
        })?;
        assignments.push(assignment);
    }
    Ok(assignments_by_centroid)
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

    let assignments_by_centroid = group_assignments_by_centroid(
        assignments,
        &centroid_plan.assignment_indexes,
        centroid_count,
    )?;

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

unsafe fn publish_relation_recursive_routing_build(
    index_relation: pg_sys::Relation,
    state: &SpireBuildState,
    target_fanout: u32,
) -> Result<usize, String> {
    if state.scanned_tuples == 0 {
        return Ok(0);
    }

    let (published_at_micros, retain_until_micros) = unsafe { current_epoch_publish_times()? };
    let centroid_plan = state.train_centroid_plan()?;
    let mut pid_allocator = SpirePidAllocator::default();
    let mut local_vec_id_allocator = SpireLocalVecIdAllocator::default();
    let coordinator = build_recursive_epoch_input_from_centroid_plan(
        SpireRecursiveBuildCoordinatorInput {
            epoch: SPIRE_INITIAL_EPOCH,
            object_version: SPIRE_INITIAL_OBJECT_VERSION,
            published_at_micros,
            retain_until_micros,
            consistency_mode: SpireConsistencyMode::Strict,
            target_fanout,
            seed: state.options.seed as u64,
            assignments: state.assignment_inputs(),
            centroid_plan,
        },
        &mut pid_allocator,
        &mut local_vec_id_allocator,
    )?;
    let store = unsafe { SpireRelationObjectStore::for_index_relation(index_relation)? };
    let mut store = store;
    let draft = unsafe {
        build_relation_recursive_routing_epoch_from_leaf_inputs(
            coordinator.epoch_input,
            &mut store,
        )?
    };
    if draft.next_pid != coordinator.next_pid {
        return Err(format!(
            "ec_spire recursive relation build next_pid {} does not match coordinator next_pid {}",
            draft.next_pid, coordinator.next_pid
        ));
    }
    unsafe {
        publish_relation_recursive_routing_epoch_draft(
            index_relation,
            &draft,
            coordinator.next_local_vec_seq,
        )?
    };
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
            if options.local_store_count != 1 {
                pgrx::error!(
                    "ec_spire local_store_count > 1 is parsed but store relation creation is not implemented yet"
                );
            }
            let recursive_fanout = options.recursive_fanout();
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
            } else if let Some(recursive_fanout) = recursive_fanout {
                publish_relation_recursive_routing_build(index_relation, &state, recursive_fanout)
                    .unwrap_or_else(|e| {
                        pgrx::error!("ec_spire recursive populated ambuild failed: {e}")
                    }) as f64
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
    use super::retired_epoch_manifest_from;
    use super::{
        build_local_recursive_routing_epoch_draft, build_partitioned_single_level_leaf_epoch_draft,
        build_single_level_leaf_epoch_draft, encode_publish_bundle_for_publish,
        object_manifest_from_placement_writes, object_write_evidence_from_placement_directory,
        placement_write_evidence_from_object_manifest, resolve_training_sample_count,
        train_single_level_centroid_plan, SpireBuildState, SpireBuildTuple, SpireIndexedVectorKind,
        SpirePartitionedSingleLevelBuildInput, SpirePublishPlacementWriteEvidence,
        SpirePublishStage, SpirePublishWritingObjects, SpireRecursiveBuildCoordinatorInput,
        SpireRecursiveLeafObjectInput, SpireRecursiveRoutingBuildInput,
        SpireRecursiveRoutingChildInput, SpireRecursiveRoutingEpochInput,
        SpireRecursiveRoutingEpochObjectInput, SpireSingleLevelBuildInput,
        SpireSingleLevelCentroidPlan, SpireSingleLevelRouteMap,
    };
    use super::{SpirePublishedManifestLocators, SpireSingleLevelBuildDraft};
    use crate::am::ec_spire::assign::{
        SpireLeafAssignmentInput, SpireLocalVecIdAllocator, SpirePidAllocator,
        SPIRE_FIRST_LOCAL_VEC_SEQ, SPIRE_FIRST_PID,
    };
    use crate::am::ec_spire::meta::{
        SpireConsistencyMode, SpireEpochState, SpirePublishedEpochSnapshot,
    };
    use crate::am::ec_spire::meta::{
        SpireEpochManifest, SpireObjectManifest, SpirePlacementDirectory, SpireRootControlState,
    };
    use crate::am::ec_spire::quantizer::{self, SpireAssignmentPayloadFormat};
    use crate::am::ec_spire::storage::{
        SpireLeafAssignmentRow, SpireLocalObjectStore, SpirePartitionObjectKind, SpireVecId,
        SPIRE_ASSIGNMENT_FLAG_PRIMARY,
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

    fn options(training_sample_rows: i32) -> super::options::EcSpireOptions {
        super::options::EcSpireOptions {
            nlists: 2,
            recursive_fanout: 0,
            local_store_count: 1,
            nprobe: 0,
            rerank_width: 0,
            training_sample_rows,
            seed: 7,
            pq_group_size: 0,
            storage_format: super::options::SpireStorageFormat::TurboQuant,
            local_store_tablespaces: None,
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

    fn recursive_child(pid: u64, centroid: Vec<f32>) -> SpireRecursiveRoutingChildInput {
        SpireRecursiveRoutingChildInput {
            child_pid: pid,
            child_level: 0,
            centroid,
            source_count: 1,
        }
    }

    fn primary_row(vec_seq: u64, block_number: u32, offset_number: u16) -> SpireLeafAssignmentRow {
        SpireLeafAssignmentRow {
            flags: SPIRE_ASSIGNMENT_FLAG_PRIMARY,
            vec_id: SpireVecId::local(vec_seq),
            heap_tid: tid(block_number, offset_number),
            payload_format: 1,
            gamma: 0.5,
            encoded_payload: vec![1, 2, 3],
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
    fn recursive_routing_build_keeps_single_level_shape_when_under_fanout() {
        let mut pid_allocator = SpirePidAllocator::default();

        let draft = super::build_recursive_routing_hierarchy_draft(
            SpireRecursiveRoutingBuildInput {
                object_version: 3,
                dimensions: 2,
                target_fanout: 4,
                seed: 42,
                children: vec![
                    recursive_child(11, vec![1.0, 0.0]),
                    recursive_child(12, vec![-1.0, 0.0]),
                ],
            },
            &mut pid_allocator,
        )
        .expect("recursive routing draft should build");

        assert_eq!(draft.root_pid, SPIRE_FIRST_PID);
        assert_eq!(draft.root_level, 1);
        assert_eq!(draft.next_pid, SPIRE_FIRST_PID + 1);
        assert_eq!(pid_allocator.next_pid(), draft.next_pid);
        assert_eq!(draft.routing_objects.len(), 1);
        assert_eq!(draft.centroid_records.len(), 2);
        let root = &draft.routing_objects[0];
        assert_eq!(root.header.pid, draft.root_pid);
        assert_eq!(root.header.level, 1);
        assert_eq!(
            root.children()
                .map(|child| child.child_pid)
                .collect::<Vec<_>>(),
            vec![11, 12]
        );
        assert_eq!(
            draft
                .centroid_records
                .iter()
                .map(|record| (
                    record.parent_pid,
                    record.child_pid,
                    record.child_level,
                    record.centroid_ordinal
                ))
                .collect::<Vec<_>>(),
            vec![(draft.root_pid, 11, 0, 0), (draft.root_pid, 12, 0, 1)]
        );
    }

    #[test]
    fn recursive_routing_build_validation_rejects_sparse_centroid_ordinals() {
        let mut pid_allocator = SpirePidAllocator::default();
        let mut draft = super::build_recursive_routing_hierarchy_draft(
            SpireRecursiveRoutingBuildInput {
                object_version: 3,
                dimensions: 2,
                target_fanout: 4,
                seed: 42,
                children: vec![
                    recursive_child(11, vec![1.0, 0.0]),
                    recursive_child(12, vec![-1.0, 0.0]),
                ],
            },
            &mut pid_allocator,
        )
        .expect("recursive routing draft should build");
        draft.centroid_records[1].centroid_ordinal = 7;

        let error = super::validate_recursive_routing_build_draft(&draft).unwrap_err();

        assert!(error.contains("centroid ordinals"));
    }

    #[test]
    fn recursive_routing_build_materializes_internal_level() {
        let mut pid_allocator = SpirePidAllocator::default();

        let draft = super::build_recursive_routing_hierarchy_draft(
            SpireRecursiveRoutingBuildInput {
                object_version: 3,
                dimensions: 2,
                target_fanout: 2,
                seed: 42,
                children: vec![
                    recursive_child(11, vec![1.0, 0.0]),
                    recursive_child(12, vec![0.9, 0.1]),
                    recursive_child(13, vec![-1.0, 0.0]),
                    recursive_child(14, vec![-0.9, 0.1]),
                ],
            },
            &mut pid_allocator,
        )
        .expect("recursive routing draft should build");

        assert_eq!(draft.root_level, 2);
        assert_eq!(draft.routing_objects.len(), 3);
        assert_eq!(draft.centroid_records.len(), 6);
        let root = &draft.routing_objects[0];
        assert_eq!(root.header.pid, draft.root_pid);
        assert_eq!(root.header.level, 2);
        assert_eq!(root.header.parent_pid, 0);
        assert_eq!(root.child_count(), 2);
        let root_child_pids = root
            .children()
            .map(|child| child.child_pid)
            .collect::<Vec<_>>();
        let internal_objects = draft.routing_objects.iter().skip(1).collect::<Vec<_>>();
        assert_eq!(
            internal_objects
                .iter()
                .map(|object| object.header.pid)
                .collect::<Vec<_>>(),
            root_child_pids
        );
        for object in internal_objects {
            assert_eq!(object.header.kind, SpirePartitionObjectKind::Internal);
            assert_eq!(object.header.level, 1);
            assert_eq!(object.header.parent_pid, draft.root_pid);
            assert!(object.child_count() >= 1);
            assert!(object
                .children()
                .all(|child| [11, 12, 13, 14].contains(&child.child_pid)));
        }
        let root_centroid_records = draft
            .centroid_records
            .iter()
            .filter(|record| record.parent_pid == draft.root_pid)
            .collect::<Vec<_>>();
        assert_eq!(root_centroid_records.len(), 2);
        assert!(root_centroid_records
            .iter()
            .all(|record| record.child_level == 1 && record.source_count >= 1));
        assert!(draft
            .centroid_records
            .iter()
            .filter(|record| record.child_level == 0)
            .all(|record| [11, 12, 13, 14].contains(&record.child_pid)));
    }

    #[test]
    fn local_recursive_routing_epoch_draft_combines_routing_and_leaf_placements() {
        let mut pid_allocator = SpirePidAllocator::default();
        let routing_draft = super::build_recursive_routing_hierarchy_draft(
            SpireRecursiveRoutingBuildInput {
                object_version: 3,
                dimensions: 2,
                target_fanout: 2,
                seed: 42,
                children: vec![
                    recursive_child(11, vec![1.0, 0.0]),
                    recursive_child(12, vec![0.9, 0.1]),
                    recursive_child(13, vec![-1.0, 0.0]),
                    recursive_child(14, vec![-0.9, 0.1]),
                ],
            },
            &mut pid_allocator,
        )
        .expect("recursive routing draft should build");
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let mut leaf_placements = Vec::new();
        for object in routing_draft
            .routing_objects
            .iter()
            .filter(|object| object.header.level == 1)
        {
            for child in object.children() {
                leaf_placements.push(
                    object_store
                        .insert_leaf_object_v2_from_rows(
                            7,
                            child.child_pid,
                            routing_draft.routing_objects[0].header.object_version,
                            object.header.pid,
                            &[],
                        )
                        .unwrap(),
                );
            }
        }

        let draft = build_local_recursive_routing_epoch_draft(
            SpireRecursiveRoutingEpochInput {
                epoch: 7,
                published_at_micros: 1000,
                retain_until_micros: 2000,
                consistency_mode: SpireConsistencyMode::Strict,
                routing_draft,
                leaf_placements,
            },
            &mut object_store,
        )
        .unwrap();

        assert_eq!(draft.root_pid, SPIRE_FIRST_PID + 2);
        assert_eq!(draft.routing_objects.len(), 3);
        assert_eq!(draft.centroid_records.len(), 6);
        assert_eq!(draft.object_manifest.entries.len(), 7);
        assert_eq!(draft.placement_directory.entries.len(), 7);
        assert!(draft.next_pid >= 15);
        let root_placement = draft.placement_directory.get(draft.root_pid).unwrap();
        let stored_root = object_store.read_routing_object(root_placement).unwrap();
        assert_eq!(stored_root.header.kind, SpirePartitionObjectKind::Root);
        assert_eq!(stored_root.header.level, 2);
        SpirePublishedEpochSnapshot::new(
            &draft.epoch_manifest,
            &draft.object_manifest,
            &draft.placement_directory,
        )
        .unwrap();
    }

    #[test]
    fn local_recursive_routing_epoch_draft_rejects_missing_leaf_placement() {
        let mut pid_allocator = SpirePidAllocator::default();
        let routing_draft = super::build_recursive_routing_hierarchy_draft(
            SpireRecursiveRoutingBuildInput {
                object_version: 3,
                dimensions: 2,
                target_fanout: 4,
                seed: 42,
                children: vec![
                    recursive_child(11, vec![1.0, 0.0]),
                    recursive_child(12, vec![-1.0, 0.0]),
                ],
            },
            &mut pid_allocator,
        )
        .expect("recursive routing draft should build");
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let leaf_placement = object_store
            .insert_leaf_object_v2_from_rows(7, 11, 3, routing_draft.root_pid, &[])
            .unwrap();

        let error = build_local_recursive_routing_epoch_draft(
            SpireRecursiveRoutingEpochInput {
                epoch: 7,
                published_at_micros: 1000,
                retain_until_micros: 2000,
                consistency_mode: SpireConsistencyMode::Strict,
                routing_draft,
                leaf_placements: vec![leaf_placement],
            },
            &mut object_store,
        )
        .unwrap_err();

        assert!(error.contains("leaf placement mismatch"));
    }

    #[test]
    fn local_recursive_routing_epoch_draft_rejects_leaf_parent_drift() {
        let mut pid_allocator = SpirePidAllocator::default();
        let routing_draft = super::build_recursive_routing_hierarchy_draft(
            SpireRecursiveRoutingBuildInput {
                object_version: 3,
                dimensions: 2,
                target_fanout: 4,
                seed: 42,
                children: vec![
                    recursive_child(11, vec![1.0, 0.0]),
                    recursive_child(12, vec![-1.0, 0.0]),
                ],
            },
            &mut pid_allocator,
        )
        .expect("recursive routing draft should build");
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let first_leaf = object_store
            .insert_leaf_object_v2_from_rows(7, 11, 3, routing_draft.root_pid + 99, &[])
            .unwrap();
        let second_leaf = object_store
            .insert_leaf_object_v2_from_rows(7, 12, 3, routing_draft.root_pid, &[])
            .unwrap();

        let error = build_local_recursive_routing_epoch_draft(
            SpireRecursiveRoutingEpochInput {
                epoch: 7,
                published_at_micros: 1000,
                retain_until_micros: 2000,
                consistency_mode: SpireConsistencyMode::Strict,
                routing_draft,
                leaf_placements: vec![first_leaf, second_leaf],
            },
            &mut object_store,
        )
        .unwrap_err();

        assert!(error.contains("parent"));
        assert!(error.contains("does not match routing parent"));
    }

    #[test]
    fn local_recursive_routing_epoch_from_leaf_inputs_writes_leaf_objects() {
        let mut pid_allocator = SpirePidAllocator::default();
        let routing_draft = super::build_recursive_routing_hierarchy_draft(
            SpireRecursiveRoutingBuildInput {
                object_version: 3,
                dimensions: 2,
                target_fanout: 4,
                seed: 42,
                children: vec![
                    recursive_child(11, vec![1.0, 0.0]),
                    recursive_child(12, vec![-1.0, 0.0]),
                ],
            },
            &mut pid_allocator,
        )
        .expect("recursive routing draft should build");
        let root_pid = routing_draft.root_pid;
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();

        let draft = super::build_local_recursive_routing_epoch_from_leaf_inputs(
            SpireRecursiveRoutingEpochObjectInput {
                epoch: 7,
                published_at_micros: 1000,
                retain_until_micros: 2000,
                consistency_mode: SpireConsistencyMode::Strict,
                routing_draft,
                leaf_inputs: vec![
                    SpireRecursiveLeafObjectInput {
                        pid: 11,
                        object_version: 3,
                        parent_pid: root_pid,
                        rows: vec![primary_row(1, 10, 1)],
                    },
                    SpireRecursiveLeafObjectInput {
                        pid: 12,
                        object_version: 3,
                        parent_pid: root_pid,
                        rows: vec![primary_row(2, 10, 2)],
                    },
                ],
            },
            &mut object_store,
        )
        .unwrap();

        assert_eq!(draft.object_manifest.entries.len(), 3);
        assert_eq!(draft.centroid_records.len(), 2);
        let first_leaf_placement = draft.placement_directory.get(11).unwrap();
        let first_leaf = object_store
            .read_leaf_object_v2(first_leaf_placement)
            .unwrap();
        assert_eq!(first_leaf.meta.header.parent_pid, root_pid);
        assert_eq!(
            first_leaf.assignment_rows().unwrap()[0].heap_tid,
            tid(10, 1)
        );
        SpirePublishedEpochSnapshot::new(
            &draft.epoch_manifest,
            &draft.object_manifest,
            &draft.placement_directory,
        )
        .unwrap();
    }

    #[test]
    fn local_recursive_routing_epoch_from_leaf_inputs_rejects_parent_drift() {
        let mut pid_allocator = SpirePidAllocator::default();
        let routing_draft = super::build_recursive_routing_hierarchy_draft(
            SpireRecursiveRoutingBuildInput {
                object_version: 3,
                dimensions: 2,
                target_fanout: 4,
                seed: 42,
                children: vec![
                    recursive_child(11, vec![1.0, 0.0]),
                    recursive_child(12, vec![-1.0, 0.0]),
                ],
            },
            &mut pid_allocator,
        )
        .expect("recursive routing draft should build");
        let root_pid = routing_draft.root_pid;
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();

        let error = super::build_local_recursive_routing_epoch_from_leaf_inputs(
            SpireRecursiveRoutingEpochObjectInput {
                epoch: 7,
                published_at_micros: 1000,
                retain_until_micros: 2000,
                consistency_mode: SpireConsistencyMode::Strict,
                routing_draft,
                leaf_inputs: vec![
                    SpireRecursiveLeafObjectInput {
                        pid: 11,
                        object_version: 3,
                        parent_pid: root_pid + 99,
                        rows: Vec::new(),
                    },
                    SpireRecursiveLeafObjectInput {
                        pid: 12,
                        object_version: 3,
                        parent_pid: root_pid,
                        rows: Vec::new(),
                    },
                ],
            },
            &mut object_store,
        )
        .unwrap_err();

        assert!(error.contains("does not match routing parent"));
    }

    #[test]
    fn recursive_build_coordinator_assembles_epoch_input_from_centroid_plan() {
        let mut pid_allocator = SpirePidAllocator::default();
        let mut local_vec_id_allocator = SpireLocalVecIdAllocator::default();
        let centroid_plan = SpireSingleLevelCentroidPlan {
            dimensions: 2,
            centroids: vec![
                vec![1.0, 0.0],
                vec![0.9, 0.1],
                vec![-1.0, 0.0],
                vec![-0.9, 0.1],
            ],
            assignment_indexes: vec![0, 1, 2, 3],
        };

        let draft = super::build_recursive_epoch_input_from_centroid_plan(
            SpireRecursiveBuildCoordinatorInput {
                epoch: 7,
                object_version: 3,
                published_at_micros: 1000,
                retain_until_micros: 2000,
                consistency_mode: SpireConsistencyMode::Strict,
                target_fanout: 2,
                seed: 42,
                assignments: vec![
                    assignment_input(10, 1),
                    assignment_input(10, 2),
                    assignment_input(10, 3),
                    assignment_input(10, 4),
                ],
                centroid_plan,
            },
            &mut pid_allocator,
            &mut local_vec_id_allocator,
        )
        .unwrap();

        assert_eq!(
            draft.leaf_pids,
            vec![
                SPIRE_FIRST_PID,
                SPIRE_FIRST_PID + 1,
                SPIRE_FIRST_PID + 2,
                SPIRE_FIRST_PID + 3
            ]
        );
        assert_eq!(draft.epoch_input.routing_draft.root_level, 2);
        assert_eq!(draft.epoch_input.leaf_inputs.len(), 4);
        assert!(draft
            .epoch_input
            .leaf_inputs
            .iter()
            .all(|leaf_input| leaf_input.parent_pid != 0 && leaf_input.rows.len() == 1));
        assert_eq!(
            draft
                .epoch_input
                .leaf_inputs
                .iter()
                .map(|leaf_input| leaf_input.rows[0].vec_id.clone())
                .collect::<Vec<_>>(),
            vec![
                SpireVecId::local(SPIRE_FIRST_LOCAL_VEC_SEQ),
                SpireVecId::local(SPIRE_FIRST_LOCAL_VEC_SEQ + 1),
                SpireVecId::local(SPIRE_FIRST_LOCAL_VEC_SEQ + 2),
                SpireVecId::local(SPIRE_FIRST_LOCAL_VEC_SEQ + 3),
            ]
        );

        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let next_pid = draft.next_pid;
        let next_local_vec_seq = draft.next_local_vec_seq;
        let epoch_draft = super::build_local_recursive_routing_epoch_from_leaf_inputs(
            draft.epoch_input,
            &mut object_store,
        )
        .unwrap();
        assert_eq!(epoch_draft.root_pid, SPIRE_FIRST_PID + 6);
        assert_eq!(epoch_draft.object_manifest.entries.len(), 7);
        assert_eq!(epoch_draft.centroid_records.len(), 6);
        assert_eq!(pid_allocator.next_pid(), next_pid);
        assert_eq!(
            local_vec_id_allocator.next_local_vec_seq(),
            next_local_vec_seq
        );
    }

    #[test]
    fn recursive_build_coordinator_rejects_assignment_count_mismatch() {
        let mut pid_allocator = SpirePidAllocator::default();
        let mut local_vec_id_allocator = SpireLocalVecIdAllocator::default();
        let centroid_plan = SpireSingleLevelCentroidPlan {
            dimensions: 2,
            centroids: vec![vec![1.0, 0.0], vec![-1.0, 0.0]],
            assignment_indexes: vec![0, 1],
        };

        let error = super::build_recursive_epoch_input_from_centroid_plan(
            SpireRecursiveBuildCoordinatorInput {
                epoch: 7,
                object_version: 3,
                published_at_micros: 1000,
                retain_until_micros: 2000,
                consistency_mode: SpireConsistencyMode::Strict,
                target_fanout: 2,
                seed: 42,
                assignments: vec![assignment_input(10, 1)],
                centroid_plan,
            },
            &mut pid_allocator,
            &mut local_vec_id_allocator,
        )
        .unwrap_err();

        assert!(error.contains("assignment count"));
        assert_eq!(pid_allocator.next_pid(), SPIRE_FIRST_PID);
        assert_eq!(
            local_vec_id_allocator.next_local_vec_seq(),
            SPIRE_FIRST_LOCAL_VEC_SEQ
        );
    }

    #[test]
    fn recursive_epoch_draft_encodes_publish_bundle_with_allocator_cursor() {
        let mut pid_allocator = SpirePidAllocator::default();
        let mut local_vec_id_allocator = SpireLocalVecIdAllocator::default();
        let centroid_plan = SpireSingleLevelCentroidPlan {
            dimensions: 2,
            centroids: vec![vec![1.0, 0.0], vec![-1.0, 0.0]],
            assignment_indexes: vec![0, 1],
        };
        let coordinator = super::build_recursive_epoch_input_from_centroid_plan(
            SpireRecursiveBuildCoordinatorInput {
                epoch: 7,
                object_version: 3,
                published_at_micros: 1000,
                retain_until_micros: 2000,
                consistency_mode: SpireConsistencyMode::Strict,
                target_fanout: 2,
                seed: 42,
                assignments: vec![assignment_input(10, 1), assignment_input(10, 2)],
                centroid_plan,
            },
            &mut pid_allocator,
            &mut local_vec_id_allocator,
        )
        .unwrap();
        let next_local_vec_seq = coordinator.next_local_vec_seq;
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let draft = super::build_local_recursive_routing_epoch_from_leaf_inputs(
            coordinator.epoch_input,
            &mut object_store,
        )
        .unwrap();

        let encoded = draft
            .encode_publish_bundle(next_local_vec_seq, manifest_locators())
            .unwrap();
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
        assert_eq!(root_control.next_local_vec_seq, next_local_vec_seq);
        assert_eq!(root_control.epoch_manifest_tid, tid(70, 1));
        assert_eq!(root_control.object_manifest_tid, tid(70, 2));
        assert_eq!(root_control.placement_directory_tid, tid(70, 3));
    }

    #[test]
    fn recursive_epoch_relation_publish_input_uses_durable_placement_manifest() {
        let mut pid_allocator = SpirePidAllocator::default();
        let mut local_vec_id_allocator = SpireLocalVecIdAllocator::default();
        let centroid_plan = SpireSingleLevelCentroidPlan {
            dimensions: 2,
            centroids: vec![vec![1.0, 0.0], vec![-1.0, 0.0]],
            assignment_indexes: vec![0, 1],
        };
        let coordinator = super::build_recursive_epoch_input_from_centroid_plan(
            SpireRecursiveBuildCoordinatorInput {
                epoch: 7,
                object_version: 3,
                published_at_micros: 1000,
                retain_until_micros: 2000,
                consistency_mode: SpireConsistencyMode::Strict,
                target_fanout: 2,
                seed: 42,
                assignments: vec![assignment_input(10, 1), assignment_input(10, 2)],
                centroid_plan,
            },
            &mut pid_allocator,
            &mut local_vec_id_allocator,
        )
        .unwrap();
        let next_local_vec_seq = coordinator.next_local_vec_seq;
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let draft = super::build_local_recursive_routing_epoch_from_leaf_inputs(
            coordinator.epoch_input,
            &mut object_store,
        )
        .unwrap();
        let placement_evidence = draft
            .placement_directory
            .entries
            .iter()
            .enumerate()
            .map(|(index, placement)| SpirePublishPlacementWriteEvidence {
                pid: placement.pid,
                placement_tid: tid(90, (index + 1) as u16),
            })
            .collect::<Vec<_>>();
        let durable_manifest = object_manifest_from_placement_writes(
            draft.epoch_manifest.epoch,
            &draft.placement_directory,
            &placement_evidence,
        )
        .unwrap();

        let encoded = encode_publish_bundle_for_publish(
            draft.relation_publish_input(&durable_manifest, next_local_vec_seq),
            manifest_locators(),
        )
        .unwrap();

        assert_eq!(
            SpireObjectManifest::decode(&encoded.manifests.object_manifest).unwrap(),
            durable_manifest
        );
        assert!(durable_manifest
            .entries
            .iter()
            .all(|entry| entry.placement_tid.block_number == 90));
        let root_control = SpireRootControlState::decode(&encoded.root_control_state).unwrap();
        assert_eq!(root_control.next_pid, draft.next_pid);
        assert_eq!(root_control.next_local_vec_seq, next_local_vec_seq);
    }

    #[test]
    fn recursive_routing_build_rejects_mixed_child_levels() {
        let mut pid_allocator = SpirePidAllocator::default();
        let mut internal_child = recursive_child(12, vec![-1.0, 0.0]);
        internal_child.child_level = 1;

        let error = super::build_recursive_routing_hierarchy_draft(
            SpireRecursiveRoutingBuildInput {
                object_version: 3,
                dimensions: 2,
                target_fanout: 2,
                seed: 42,
                children: vec![recursive_child(11, vec![1.0, 0.0]), internal_child],
            },
            &mut pid_allocator,
        )
        .unwrap_err();

        assert!(error.contains("does not match expected level"));
        assert_eq!(pid_allocator.next_pid(), SPIRE_FIRST_PID);
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
    fn retired_epoch_manifest_requires_published_input() {
        let retired_manifest = SpireEpochManifest {
            epoch: 7,
            state: SpireEpochState::Retired,
            consistency_mode: SpireConsistencyMode::Strict,
            published_at_micros: 1000,
            retain_until_micros: 2000,
            active_query_count: 0,
        };

        let error = retired_epoch_manifest_from(retired_manifest)
            .expect_err("retiring an already-retired manifest should fail");

        assert_eq!(
            error,
            "ec_spire can only retire a previously published epoch manifest"
        );
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
