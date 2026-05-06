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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SpireLeafReplacementMode {
    Split,
    Merge,
    Rebalance { parent_centroid_byte_equal: bool },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct SpireLeafReplacementPidPlan {
    pub(super) replacement_pids: Vec<u64>,
    pub(super) reuses_existing_pid: bool,
    pub(super) next_pid: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SpireReplacementLeafRows {
    pub(super) base_pid: u64,
    pub(super) rows: Vec<SpireLeafAssignmentRow>,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SpireSplitReplacementSourceRow {
    pub(super) base_pid: u64,
    pub(super) assignment: SpireLeafAssignmentRow,
    pub(super) source_vector: Vec<f32>,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SpireSplitReplacementFetchedSourceVector {
    pub(super) heap_tid: ItemPointer,
    pub(super) source_vector: Vec<f32>,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SpireSplitReplacementMaterialization {
    pub(super) centroids: Vec<Vec<f32>>,
    pub(super) leaf_inputs: Vec<SpireReplacementLeafObjectInput>,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SpireReplacementEpochInput {
    pub(super) epoch: u64,
    pub(super) published_at_micros: i64,
    pub(super) retain_until_micros: i64,
    pub(super) consistency_mode: SpireConsistencyMode,
    pub(super) placement_directory: SpirePlacementDirectory,
    pub(super) placement_write_evidence: Vec<SpirePublishPlacementWriteEvidence>,
    pub(super) next_pid: u64,
    pub(super) next_local_vec_seq: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SpireReplacementEpochDraft {
    pub(super) epoch_manifest: SpireEpochManifest,
    pub(super) object_manifest: SpireObjectManifest,
    pub(super) placement_directory: SpirePlacementDirectory,
    pub(super) next_pid: u64,
    pub(super) next_local_vec_seq: u64,
}

impl SpireReplacementEpochDraft {
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

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SpireReplacementLeafObjectInput {
    pub(super) pid: u64,
    pub(super) rows: Vec<SpireLeafAssignmentRow>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct SpireReplacementObjectPlacements {
    pub(super) parent_placement: SpirePlacementEntry,
    pub(super) leaf_placements: Vec<SpirePlacementEntry>,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SpireReplacementEpochObjectPlacementInput {
    pub(super) epoch: u64,
    pub(super) published_at_micros: i64,
    pub(super) retain_until_micros: i64,
    pub(super) consistency_mode: SpireConsistencyMode,
    pub(super) replaced_parent_pid: u64,
    pub(super) affected_leaf_pids: Vec<u64>,
    pub(super) replacement_object_placements: SpireReplacementObjectPlacements,
    pub(super) placement_write_evidence: Vec<SpirePublishPlacementWriteEvidence>,
    pub(super) next_pid: u64,
    pub(super) next_local_vec_seq: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SpireRelationReplacementEpochObjectPlacementInput {
    pub(super) epoch: u64,
    pub(super) published_at_micros: i64,
    pub(super) retain_until_micros: i64,
    pub(super) consistency_mode: SpireConsistencyMode,
    pub(super) replaced_parent_pid: u64,
    pub(super) affected_leaf_pids: Vec<u64>,
    pub(super) replacement_object_placements: SpireReplacementObjectPlacements,
    pub(super) next_pid: u64,
    pub(super) next_local_vec_seq: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SpireScheduledReplacementEpochObjectPlacementInput {
    pub(super) epoch: u64,
    pub(super) published_at_micros: i64,
    pub(super) retain_until_micros: i64,
    pub(super) consistency_mode: SpireConsistencyMode,
    pub(super) replacement_object_placements: SpireReplacementObjectPlacements,
    pub(super) placement_write_evidence: Vec<SpirePublishPlacementWriteEvidence>,
    pub(super) next_pid: u64,
    pub(super) next_local_vec_seq: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SpireLocalScheduledReplacementExecutionInput {
    pub(super) epoch: u64,
    pub(super) published_at_micros: i64,
    pub(super) retain_until_micros: i64,
    pub(super) consistency_mode: SpireConsistencyMode,
    pub(super) replacement_parent: SpireRoutingPartitionObject,
    pub(super) replacement_children: Vec<SpireRoutingReplacementChild>,
    pub(super) leaf_object_version: u64,
    pub(super) leaf_inputs: Vec<SpireReplacementLeafObjectInput>,
    pub(super) placement_write_evidence: Vec<SpirePublishPlacementWriteEvidence>,
    pub(super) next_local_vec_seq: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SpireLocalScheduledReplacementExecutionParts {
    pub(super) published_at_micros: i64,
    pub(super) retain_until_micros: i64,
    pub(super) replacement_parent: SpireRoutingPartitionObject,
    pub(super) replacement_children: Vec<SpireRoutingReplacementChild>,
    pub(super) leaf_object_version: u64,
    pub(super) leaf_inputs: Vec<SpireReplacementLeafObjectInput>,
    pub(super) placement_write_evidence: Vec<SpirePublishPlacementWriteEvidence>,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SpireRelationScheduledReplacementExecutionInput {
    pub(super) epoch: u64,
    pub(super) published_at_micros: i64,
    pub(super) retain_until_micros: i64,
    pub(super) consistency_mode: SpireConsistencyMode,
    pub(super) replacement_parent: SpireRoutingPartitionObject,
    pub(super) replacement_children: Vec<SpireRoutingReplacementChild>,
    pub(super) leaf_object_version: u64,
    pub(super) leaf_inputs: Vec<SpireReplacementLeafObjectInput>,
    pub(super) next_local_vec_seq: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SpireRelationScheduledReplacementExecutionParts {
    pub(super) published_at_micros: i64,
    pub(super) retain_until_micros: i64,
    pub(super) replacement_parent: SpireRoutingPartitionObject,
    pub(super) replacement_children: Vec<SpireRoutingReplacementChild>,
    pub(super) leaf_object_version: u64,
    pub(super) leaf_inputs: Vec<SpireReplacementLeafObjectInput>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct SpireScheduledReplacementPublishPlan {
    pub(super) epoch: u64,
    pub(super) consistency_mode: SpireConsistencyMode,
    pub(super) next_pid: u64,
    pub(super) next_local_vec_seq: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct SpireScheduledReplacementPublishLockPlan {
    pub(super) pid_plan: SpireLeafReplacementPidPlan,
    pub(super) publish_plan: SpireScheduledReplacementPublishPlan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct SpireSelectedScheduledReplacementPublishLockPlan {
    pub(super) decision: SpireLeafReplacementScheduleDecision,
    pub(super) lock_plan: SpireScheduledReplacementPublishLockPlan,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SpireRoutingReplacementChild {
    pub(super) child_pid: u64,
    pub(super) centroid: Vec<f32>,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SpireScheduledReplacementRoutingParts {
    pub(super) replacement_parent: SpireRoutingPartitionObject,
    pub(super) replacement_children: Vec<SpireRoutingReplacementChild>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SpireLeafReplacementScheduleMode {
    Split,
    Merge,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct SpireLeafReplacementScheduleDecision {
    pub(super) mode: SpireLeafReplacementScheduleMode,
    pub(super) active_epoch: u64,
    pub(super) replaced_parent_pid: u64,
    pub(super) affected_leaf_pids: Vec<u64>,
    pub(super) replacement_leaf_count: usize,
    pub(super) reason: &'static str,
}

trait SpireReplacementObjectWriter {
    fn write_replacement_parent_object(
        &mut self,
        epoch: u64,
        object: &SpireRoutingPartitionObject,
    ) -> Result<SpirePlacementEntry, String>;

    fn write_replacement_leaf_object_v2_from_rows(
        &mut self,
        epoch: u64,
        pid: u64,
        object_version: u64,
        parent_pid: u64,
        rows: &[SpireLeafAssignmentRow],
    ) -> Result<SpirePlacementEntry, String>;
}

impl SpireReplacementObjectWriter for SpireLocalObjectStore {
    fn write_replacement_parent_object(
        &mut self,
        epoch: u64,
        object: &SpireRoutingPartitionObject,
    ) -> Result<SpirePlacementEntry, String> {
        self.insert_routing_object(epoch, object)
    }

    fn write_replacement_leaf_object_v2_from_rows(
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

impl SpireReplacementObjectWriter for SpireRelationObjectStore {
    fn write_replacement_parent_object(
        &mut self,
        epoch: u64,
        object: &SpireRoutingPartitionObject,
    ) -> Result<SpirePlacementEntry, String> {
        unsafe { self.insert_routing_object(epoch, object) }
    }

    fn write_replacement_leaf_object_v2_from_rows(
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
