//! Epoch-published insert/delete, split, merge, and cleanup mechanics live here.

use std::collections::{HashMap, HashSet};

use super::assign::{
    build_delete_delta_assignments, build_insert_delta_assignments, SpireDeleteDeltaInput,
    SpireLeafAssignmentInput, SpireLocalVecIdAllocator, SpirePidAllocator,
};
use super::build::{
    encode_manifest_bundle_for_publish, encode_publish_bundle_for_publish,
    object_manifest_from_placement_writes, publish_replacement_epoch_to_relation,
    root_control_state_for_publish, write_placement_entries_to_relation,
    SpireEncodedManifestBundle, SpireEncodedPublishBundle, SpirePublishCoordinatorInput,
    SpirePublishPlacementWriteEvidence, SpirePublishedManifestLocators,
};
use super::meta::{
    SpireConsistencyMode, SpireEpochManifest, SpireEpochState, SpireManifestEntry,
    SpireObjectManifest, SpirePlacementDirectory, SpirePlacementEntry, SpirePlacementState,
    SpirePublishedEpochSnapshot, SpireRootControlState, SpireValidatedEpochSnapshot,
};
use super::scan::{collect_validated_snapshot_visible_primary_rows, SpireLeafScanRow};
use super::storage::{
    is_delete_delta_assignment, is_visible_primary_assignment, SpireDeltaPartitionObject,
    SpireLeafAssignmentRow, SpireLocalObjectStore, SpireObjectReader, SpirePartitionObjectKind,
    SpireRelationObjectStore, SpireRoutingChildEntry, SpireRoutingPartitionObject, SpireVecId,
    SPIRE_ASSIGNMENT_FLAG_DELTA_INSERT,
};
use super::SpireIndexLeafSnapshotRow;
use crate::am::common::training as common_training;
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

pub(super) fn plan_leaf_replacement_pids(
    mode: SpireLeafReplacementMode,
    affected_leaf_pids: &[u64],
    replacement_leaf_count: usize,
    pid_allocator: &mut SpirePidAllocator,
) -> Result<SpireLeafReplacementPidPlan, String> {
    validate_affected_leaf_pids(affected_leaf_pids)?;
    if replacement_leaf_count == 0 {
        return Err("ec_spire leaf replacement requires at least one replacement leaf".to_owned());
    }

    let mut cursor = *pid_allocator;
    for pid in affected_leaf_pids {
        cursor.observe(*pid)?;
    }

    let (replacement_pids, reuses_existing_pid) = match mode {
        SpireLeafReplacementMode::Split => {
            if affected_leaf_pids.len() != 1 {
                return Err(
                    "ec_spire split replacement requires exactly one affected leaf pid".to_owned(),
                );
            }
            if replacement_leaf_count < 2 {
                return Err(
                    "ec_spire split replacement requires at least two replacement leaves"
                        .to_owned(),
                );
            }
            (
                allocate_replacement_pids(&mut cursor, replacement_leaf_count)?,
                false,
            )
        }
        SpireLeafReplacementMode::Merge => {
            if replacement_leaf_count != 1 {
                return Err(
                    "ec_spire merge replacement publishes exactly one replacement leaf".to_owned(),
                );
            }
            (vec![cursor.allocate()?], false)
        }
        SpireLeafReplacementMode::Rebalance {
            parent_centroid_byte_equal,
        } => {
            if affected_leaf_pids.len() != 1 || replacement_leaf_count != 1 {
                return Err(
                    "ec_spire rebalance replacement requires exactly one affected leaf and one replacement leaf"
                        .to_owned(),
                );
            }
            if !parent_centroid_byte_equal {
                return Err(
                    "ec_spire rebalance may reuse a pid only when the parent routing centroid is byte-equal"
                        .to_owned(),
                );
            }
            (vec![affected_leaf_pids[0]], true)
        }
    };

    let plan = SpireLeafReplacementPidPlan {
        replacement_pids,
        reuses_existing_pid,
        next_pid: cursor.next_pid(),
    };
    *pid_allocator = cursor;
    Ok(plan)
}

pub(super) fn plan_scheduled_leaf_replacement_pids(
    decision: &SpireLeafReplacementScheduleDecision,
    pid_allocator: &mut SpirePidAllocator,
) -> Result<SpireLeafReplacementPidPlan, String> {
    validate_leaf_replacement_schedule_decision_shape(decision)?;
    let mode = match decision.mode {
        SpireLeafReplacementScheduleMode::Split => SpireLeafReplacementMode::Split,
        SpireLeafReplacementScheduleMode::Merge => SpireLeafReplacementMode::Merge,
    };
    plan_leaf_replacement_pids(
        mode,
        &decision.affected_leaf_pids,
        decision.replacement_leaf_count,
        pid_allocator,
    )
}

pub(super) fn choose_leaf_replacement_schedule(
    rows: &[SpireIndexLeafSnapshotRow],
) -> Result<Option<SpireLeafReplacementScheduleDecision>, String> {
    validate_leaf_replacement_schedule_rows(rows)?;
    if let Some(row) = rows
        .iter()
        .filter(|row| row.split_recommended)
        .max_by_key(|row| {
            (
                row.effective_assignment_count,
                std::cmp::Reverse(row.leaf_pid),
            )
        })
    {
        return Ok(Some(SpireLeafReplacementScheduleDecision {
            mode: SpireLeafReplacementScheduleMode::Split,
            active_epoch: row.active_epoch,
            replaced_parent_pid: row.parent_pid,
            affected_leaf_pids: vec![row.leaf_pid],
            replacement_leaf_count: 2,
            reason: "largest_split_candidate",
        }));
    }

    let mut merge_candidates_by_parent: HashMap<u64, Vec<&SpireIndexLeafSnapshotRow>> =
        HashMap::new();
    for row in rows.iter().filter(|row| row.merge_recommended) {
        merge_candidates_by_parent
            .entry(row.parent_pid)
            .or_default()
            .push(row);
    }
    let mut best_pair: Option<(&SpireIndexLeafSnapshotRow, &SpireIndexLeafSnapshotRow)> = None;
    for candidates in merge_candidates_by_parent.values_mut() {
        if candidates.len() < 2 {
            continue;
        }
        candidates.sort_by_key(|row| (row.effective_assignment_count, row.leaf_pid));
        let pair = (candidates[0], candidates[1]);
        let replace = match best_pair {
            Some(best) => merge_pair_sort_key(pair) < merge_pair_sort_key(best),
            None => true,
        };
        if replace {
            best_pair = Some(pair);
        }
    }
    if let Some((left, right)) = best_pair {
        let mut affected_leaf_pids = vec![left.leaf_pid, right.leaf_pid];
        affected_leaf_pids.sort_unstable();
        return Ok(Some(SpireLeafReplacementScheduleDecision {
            mode: SpireLeafReplacementScheduleMode::Merge,
            active_epoch: left.active_epoch,
            replaced_parent_pid: left.parent_pid,
            affected_leaf_pids,
            replacement_leaf_count: 1,
            reason: "sparsest_same_parent_merge_pair",
        }));
    }

    Ok(None)
}

pub(super) fn recheck_leaf_replacement_schedule_decision(
    rows: &[SpireIndexLeafSnapshotRow],
    expected: &SpireLeafReplacementScheduleDecision,
) -> Result<(), String> {
    validate_leaf_replacement_schedule_decision_shape(expected)?;
    // Keep this recheck in lockstep with the selector: scheduler execution
    // treats selector tie-breaks as part of the publish-lock consistency
    // contract, not just as advisory ranking.
    let Some(observed) = choose_leaf_replacement_schedule(rows)? else {
        return Err("ec_spire replacement scheduler decision is no longer recommended".to_owned());
    };
    if !leaf_replacement_schedule_decisions_match(&observed, expected) {
        return Err(format!(
            "ec_spire replacement scheduler decision changed under publish lock: expected {:?} for pids {:?}, observed {:?} for pids {:?}",
            expected.mode, expected.affected_leaf_pids, observed.mode, observed.affected_leaf_pids
        ));
    }
    Ok(())
}

pub(super) fn build_merge_replacement_leaf_object_input(
    decision: &SpireLeafReplacementScheduleDecision,
    pid_plan: &SpireLeafReplacementPidPlan,
    replacement_leaf_rows: Vec<SpireReplacementLeafRows>,
) -> Result<SpireReplacementLeafObjectInput, String> {
    validate_leaf_replacement_schedule_decision_shape(decision)?;
    if decision.mode != SpireLeafReplacementScheduleMode::Merge {
        return Err("ec_spire merge replacement leaf input requires a merge decision".to_owned());
    }
    if pid_plan.reuses_existing_pid {
        return Err(
            "ec_spire merge replacement leaf input requires fresh replacement pid".to_owned(),
        );
    }
    let [replacement_pid] = pid_plan.replacement_pids.as_slice() else {
        return Err(
            "ec_spire merge replacement leaf input requires exactly one replacement pid".to_owned(),
        );
    };
    if *replacement_pid >= pid_plan.next_pid {
        return Err(format!(
            "ec_spire merge replacement leaf input pid plan next_pid {} does not advance past replacement pid {replacement_pid}",
            pid_plan.next_pid
        ));
    }

    let affected: HashSet<u64> = decision.affected_leaf_pids.iter().copied().collect();
    let mut rows_by_base_pid = HashMap::new();
    for leaf_rows in replacement_leaf_rows {
        if !affected.contains(&leaf_rows.base_pid) {
            return Err(format!(
                "ec_spire merge replacement leaf input got rows for unselected base pid {}",
                leaf_rows.base_pid
            ));
        }
        if rows_by_base_pid
            .insert(leaf_rows.base_pid, leaf_rows.rows)
            .is_some()
        {
            return Err(format!(
                "ec_spire merge replacement leaf input got duplicate rows for base pid {}",
                leaf_rows.base_pid
            ));
        }
    }

    let mut rows = Vec::new();
    for base_pid in &decision.affected_leaf_pids {
        let Some(mut leaf_rows) = rows_by_base_pid.remove(base_pid) else {
            return Err(format!(
                "ec_spire merge replacement leaf input missing rows for base pid {base_pid}"
            ));
        };
        rows.append(&mut leaf_rows);
    }
    let input = SpireReplacementLeafObjectInput {
        pid: *replacement_pid,
        rows,
    };
    validate_replacement_leaf_object_inputs(
        &[SpireRoutingReplacementChild {
            child_pid: input.pid,
            centroid: Vec::new(),
        }],
        std::slice::from_ref(&input),
    )?;
    Ok(input)
}

pub(super) fn build_split_replacement_leaf_object_inputs(
    decision: &SpireLeafReplacementScheduleDecision,
    pid_plan: &SpireLeafReplacementPidPlan,
    routed_leaf_inputs: Vec<SpireReplacementLeafObjectInput>,
) -> Result<Vec<SpireReplacementLeafObjectInput>, String> {
    validate_leaf_replacement_schedule_decision_shape(decision)?;
    if decision.mode != SpireLeafReplacementScheduleMode::Split {
        return Err("ec_spire split replacement leaf inputs require a split decision".to_owned());
    }
    if pid_plan.reuses_existing_pid {
        return Err(
            "ec_spire split replacement leaf inputs require fresh replacement pids".to_owned(),
        );
    }
    if pid_plan.replacement_pids.len() != decision.replacement_leaf_count {
        return Err(format!(
            "ec_spire split replacement leaf input pid count {} does not match decision replacement count {}",
            pid_plan.replacement_pids.len(),
            decision.replacement_leaf_count
        ));
    }
    if let Some(unadvanced_pid) = pid_plan
        .replacement_pids
        .iter()
        .copied()
        .find(|pid| *pid >= pid_plan.next_pid)
    {
        return Err(format!(
            "ec_spire split replacement leaf input pid plan next_pid {} does not advance past replacement pid {unadvanced_pid}",
            pid_plan.next_pid
        ));
    }
    if routed_leaf_inputs.len() != pid_plan.replacement_pids.len() {
        return Err(format!(
            "ec_spire split replacement leaf input count {} does not match replacement pid count {}",
            routed_leaf_inputs.len(),
            pid_plan.replacement_pids.len()
        ));
    }

    let children = pid_plan
        .replacement_pids
        .iter()
        .map(|pid| SpireRoutingReplacementChild {
            child_pid: *pid,
            centroid: Vec::new(),
        })
        .collect::<Vec<_>>();
    validate_replacement_leaf_object_inputs(&children, &routed_leaf_inputs)?;

    let mut inputs_by_pid = routed_leaf_inputs
        .into_iter()
        .map(|input| (input.pid, input))
        .collect::<HashMap<_, _>>();
    let mut ordered = Vec::with_capacity(pid_plan.replacement_pids.len());
    for pid in &pid_plan.replacement_pids {
        let input = inputs_by_pid.remove(pid).ok_or_else(|| {
            format!("ec_spire split replacement leaf input missing replacement pid {pid}")
        })?;
        ordered.push(input);
    }
    Ok(ordered)
}

pub(super) fn build_scheduled_routing_replacement_children(
    decision: &SpireLeafReplacementScheduleDecision,
    pid_plan: &SpireLeafReplacementPidPlan,
    centroids: Vec<Vec<f32>>,
) -> Result<Vec<SpireRoutingReplacementChild>, String> {
    validate_leaf_replacement_schedule_decision_shape(decision)?;
    if pid_plan.reuses_existing_pid {
        return Err(
            "ec_spire scheduled routing replacement requires fresh replacement pids".to_owned(),
        );
    }
    if pid_plan.replacement_pids.len() != decision.replacement_leaf_count {
        return Err(format!(
            "ec_spire scheduled routing replacement pid count {} does not match decision replacement count {}",
            pid_plan.replacement_pids.len(),
            decision.replacement_leaf_count
        ));
    }
    if centroids.len() != pid_plan.replacement_pids.len() {
        return Err(format!(
            "ec_spire scheduled routing replacement centroid count {} does not match replacement pid count {}",
            centroids.len(),
            pid_plan.replacement_pids.len()
        ));
    }

    let mut seen_pids = HashSet::new();
    let mut children = Vec::with_capacity(pid_plan.replacement_pids.len());
    for (pid, centroid) in pid_plan.replacement_pids.iter().zip(centroids) {
        if *pid == 0 {
            return Err("ec_spire scheduled routing replacement pid 0 is invalid".to_owned());
        }
        if *pid >= pid_plan.next_pid {
            return Err(format!(
                "ec_spire scheduled routing replacement pid plan next_pid {} does not advance past replacement pid {pid}",
                pid_plan.next_pid
            ));
        }
        if !seen_pids.insert(*pid) {
            return Err("ec_spire scheduled routing replacement pids must be unique".to_owned());
        }
        if centroid.is_empty() {
            return Err(format!(
                "ec_spire scheduled routing replacement child pid {pid} centroid is empty"
            ));
        }
        if centroid.iter().any(|component| !component.is_finite()) {
            return Err(format!(
                "ec_spire scheduled routing replacement child pid {pid} centroid must be finite"
            ));
        }
        children.push(SpireRoutingReplacementChild {
            child_pid: *pid,
            centroid,
        });
    }

    Ok(children)
}

pub(super) fn build_scheduled_merge_replacement_centroids(
    decision: &SpireLeafReplacementScheduleDecision,
    parent: &SpireRoutingPartitionObject,
    rows: &[SpireIndexLeafSnapshotRow],
) -> Result<Vec<Vec<f32>>, String> {
    validate_leaf_replacement_schedule_decision_shape(decision)?;
    validate_leaf_replacement_schedule_rows(rows)?;
    if decision.mode != SpireLeafReplacementScheduleMode::Merge {
        return Err("ec_spire scheduled merge centroid requires a merge decision".to_owned());
    }
    if parent.header.pid != decision.replaced_parent_pid {
        return Err(format!(
            "ec_spire scheduled merge centroid parent pid {} does not match decision parent pid {}",
            parent.header.pid, decision.replaced_parent_pid
        ));
    }

    let dimensions = usize::from(parent.dimensions);
    if dimensions == 0 {
        return Err("ec_spire scheduled merge centroid parent dimensions 0 is invalid".to_owned());
    }
    let children_by_pid = parent
        .children()
        .map(|child| (child.child_pid, child))
        .collect::<HashMap<_, _>>();
    let rows_by_leaf_pid = rows
        .iter()
        .map(|row| (row.leaf_pid, row))
        .collect::<HashMap<_, _>>();
    let mut selected = Vec::with_capacity(decision.affected_leaf_pids.len());
    let mut total_weight = 0_u64;
    for leaf_pid in &decision.affected_leaf_pids {
        let row = rows_by_leaf_pid.get(leaf_pid).ok_or_else(|| {
            format!(
                "ec_spire scheduled merge centroid missing snapshot row for leaf pid {leaf_pid}"
            )
        })?;
        if row.active_epoch != decision.active_epoch {
            return Err(format!(
                "ec_spire scheduled merge centroid row epoch {} does not match decision epoch {}",
                row.active_epoch, decision.active_epoch
            ));
        }
        if row.parent_pid != decision.replaced_parent_pid {
            return Err(format!(
                "ec_spire scheduled merge centroid row parent pid {} does not match decision parent pid {}",
                row.parent_pid, decision.replaced_parent_pid
            ));
        }
        if !row.merge_recommended {
            return Err(format!(
                "ec_spire scheduled merge centroid affected leaf pid {leaf_pid} is no longer merge recommended"
            ));
        }
        let child = children_by_pid.get(leaf_pid).ok_or_else(|| {
            format!(
                "ec_spire scheduled merge centroid parent is missing affected leaf pid {leaf_pid}"
            )
        })?;
        if child.centroid.len() != dimensions {
            return Err(format!(
                "ec_spire scheduled merge centroid child pid {leaf_pid} dimensions {} do not match parent dimensions {dimensions}",
                child.centroid.len()
            ));
        }
        if child
            .centroid
            .iter()
            .any(|component| !component.is_finite())
        {
            return Err(format!(
                "ec_spire scheduled merge centroid child pid {leaf_pid} centroid must be finite"
            ));
        }
        total_weight = total_weight
            .checked_add(row.effective_assignment_count)
            .ok_or_else(|| "ec_spire scheduled merge centroid row weight overflow".to_owned())?;
        selected.push((*child, row.effective_assignment_count));
    }

    let mut centroid = vec![0.0_f64; dimensions];
    if total_weight == 0 {
        let weight = 1.0 / selected.len() as f64;
        for (child, _) in &selected {
            for (sum, component) in centroid.iter_mut().zip(child.centroid.iter()) {
                *sum += f64::from(*component) * weight;
            }
        }
    } else {
        let total_weight = total_weight as f64;
        for (child, row_weight) in &selected {
            let weight = *row_weight as f64 / total_weight;
            for (sum, component) in centroid.iter_mut().zip(child.centroid.iter()) {
                *sum += f64::from(*component) * weight;
            }
        }
    }

    let centroid = centroid
        .into_iter()
        .map(|component| component as f32)
        .collect::<Vec<_>>();
    let centroid = common_training::normalize_vector(
        "ec_spire scheduled merge centroid",
        &centroid,
        dimensions,
    )?;
    Ok(vec![centroid])
}

pub(super) fn load_scheduled_replacement_parent_routing(
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    object_store: &impl SpireObjectReader,
    decision: &SpireLeafReplacementScheduleDecision,
) -> Result<SpireRoutingPartitionObject, String> {
    validate_leaf_replacement_schedule_decision_shape(decision)?;
    let snapshot = SpireValidatedEpochSnapshot::from_snapshot(*snapshot)?;
    if snapshot.epoch_manifest().epoch != decision.active_epoch {
        return Err(format!(
            "ec_spire scheduled replacement parent snapshot epoch {} does not match decision active epoch {}",
            snapshot.epoch_manifest().epoch, decision.active_epoch
        ));
    }
    let lookup = snapshot.require_lookup(
        decision.replaced_parent_pid,
        "scheduled replacement parent routing",
    )?;
    if lookup.placement.state != SpirePlacementState::Available {
        return Err(format!(
            "ec_spire scheduled replacement parent pid {} placement is not available",
            decision.replaced_parent_pid
        ));
    }
    let parent = object_store.read_routing_object(lookup.placement)?;
    if parent.header.pid != decision.replaced_parent_pid {
        return Err(format!(
            "ec_spire scheduled replacement loaded parent pid {} does not match decision parent pid {}",
            parent.header.pid, decision.replaced_parent_pid
        ));
    }
    let parent_child_pids = parent
        .children()
        .map(|child| child.child_pid)
        .collect::<HashSet<_>>();
    for affected_pid in &decision.affected_leaf_pids {
        if !parent_child_pids.contains(affected_pid) {
            return Err(format!(
                "ec_spire scheduled replacement parent pid {} is missing affected leaf pid {affected_pid}",
                decision.replaced_parent_pid
            ));
        }
    }
    Ok(parent)
}

pub(super) fn load_selected_scheduled_replacement_parent_routing(
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    object_store: &impl SpireObjectReader,
    selected: &SpireSelectedScheduledReplacementPublishLockPlan,
) -> Result<SpireRoutingPartitionObject, String> {
    validate_selected_scheduled_replacement_execution_snapshot(snapshot, selected)?;
    load_scheduled_replacement_parent_routing(snapshot, object_store, &selected.decision)
}

pub(super) fn build_scheduled_merge_replacement_routing_parts(
    decision: &SpireLeafReplacementScheduleDecision,
    pid_plan: &SpireLeafReplacementPidPlan,
    parent: &SpireRoutingPartitionObject,
    rows: &[SpireIndexLeafSnapshotRow],
    parent_object_version: u64,
) -> Result<SpireScheduledReplacementRoutingParts, String> {
    let centroids = build_scheduled_merge_replacement_centroids(decision, parent, rows)?;
    let replacement_children =
        build_scheduled_routing_replacement_children(decision, pid_plan, centroids)?;
    let replacement_parent = rewrite_scheduled_replacement_parent_routing(
        parent,
        decision,
        replacement_children.clone(),
        parent_object_version,
    )?;
    Ok(SpireScheduledReplacementRoutingParts {
        replacement_parent,
        replacement_children,
    })
}

pub(super) fn build_scheduled_split_replacement_routing_parts(
    decision: &SpireLeafReplacementScheduleDecision,
    pid_plan: &SpireLeafReplacementPidPlan,
    parent: &SpireRoutingPartitionObject,
    centroids: Vec<Vec<f32>>,
    parent_object_version: u64,
) -> Result<SpireScheduledReplacementRoutingParts, String> {
    validate_leaf_replacement_schedule_decision_shape(decision)?;
    if decision.mode != SpireLeafReplacementScheduleMode::Split {
        return Err("ec_spire scheduled split routing parts require a split decision".to_owned());
    }
    if parent.header.pid != decision.replaced_parent_pid {
        return Err(format!(
            "ec_spire scheduled split routing parent pid {} does not match decision parent pid {}",
            parent.header.pid, decision.replaced_parent_pid
        ));
    }
    let replacement_children =
        build_scheduled_routing_replacement_children(decision, pid_plan, centroids)?;
    let replacement_parent = rewrite_scheduled_replacement_parent_routing(
        parent,
        decision,
        replacement_children.clone(),
        parent_object_version,
    )?;
    Ok(SpireScheduledReplacementRoutingParts {
        replacement_parent,
        replacement_children,
    })
}

pub(super) fn build_relation_scheduled_merge_replacement_execution_parts(
    decision: &SpireLeafReplacementScheduleDecision,
    pid_plan: &SpireLeafReplacementPidPlan,
    parent: &SpireRoutingPartitionObject,
    rows: &[SpireIndexLeafSnapshotRow],
    replacement_leaf_rows: Vec<SpireReplacementLeafRows>,
    parent_object_version: u64,
    leaf_object_version: u64,
    published_at_micros: i64,
    retain_until_micros: i64,
) -> Result<SpireRelationScheduledReplacementExecutionParts, String> {
    let routing_parts = build_scheduled_merge_replacement_routing_parts(
        decision,
        pid_plan,
        parent,
        rows,
        parent_object_version,
    )?;
    let leaf_input =
        build_merge_replacement_leaf_object_input(decision, pid_plan, replacement_leaf_rows)?;
    let leaf_inputs = vec![leaf_input];
    validate_replacement_leaf_object_inputs(&routing_parts.replacement_children, &leaf_inputs)?;
    Ok(SpireRelationScheduledReplacementExecutionParts {
        published_at_micros,
        retain_until_micros,
        replacement_parent: routing_parts.replacement_parent,
        replacement_children: routing_parts.replacement_children,
        leaf_object_version,
        leaf_inputs,
    })
}

pub(super) fn build_relation_scheduled_split_replacement_execution_parts(
    decision: &SpireLeafReplacementScheduleDecision,
    pid_plan: &SpireLeafReplacementPidPlan,
    parent: &SpireRoutingPartitionObject,
    centroids: Vec<Vec<f32>>,
    routed_leaf_inputs: Vec<SpireReplacementLeafObjectInput>,
    parent_object_version: u64,
    leaf_object_version: u64,
    published_at_micros: i64,
    retain_until_micros: i64,
) -> Result<SpireRelationScheduledReplacementExecutionParts, String> {
    let routing_parts = build_scheduled_split_replacement_routing_parts(
        decision,
        pid_plan,
        parent,
        centroids,
        parent_object_version,
    )?;
    let leaf_inputs =
        build_split_replacement_leaf_object_inputs(decision, pid_plan, routed_leaf_inputs)?;
    validate_replacement_leaf_object_inputs(&routing_parts.replacement_children, &leaf_inputs)?;
    Ok(SpireRelationScheduledReplacementExecutionParts {
        published_at_micros,
        retain_until_micros,
        replacement_parent: routing_parts.replacement_parent,
        replacement_children: routing_parts.replacement_children,
        leaf_object_version,
        leaf_inputs,
    })
}

fn local_scheduled_replacement_execution_parts_from_relation_parts(
    parts: SpireRelationScheduledReplacementExecutionParts,
    placement_write_evidence: Vec<SpirePublishPlacementWriteEvidence>,
) -> SpireLocalScheduledReplacementExecutionParts {
    SpireLocalScheduledReplacementExecutionParts {
        published_at_micros: parts.published_at_micros,
        retain_until_micros: parts.retain_until_micros,
        replacement_parent: parts.replacement_parent,
        replacement_children: parts.replacement_children,
        leaf_object_version: parts.leaf_object_version,
        leaf_inputs: parts.leaf_inputs,
        placement_write_evidence,
    }
}

pub(super) fn build_local_scheduled_merge_replacement_execution_parts(
    decision: &SpireLeafReplacementScheduleDecision,
    pid_plan: &SpireLeafReplacementPidPlan,
    parent: &SpireRoutingPartitionObject,
    rows: &[SpireIndexLeafSnapshotRow],
    replacement_leaf_rows: Vec<SpireReplacementLeafRows>,
    parent_object_version: u64,
    leaf_object_version: u64,
    published_at_micros: i64,
    retain_until_micros: i64,
    placement_write_evidence: Vec<SpirePublishPlacementWriteEvidence>,
) -> Result<SpireLocalScheduledReplacementExecutionParts, String> {
    let parts = build_relation_scheduled_merge_replacement_execution_parts(
        decision,
        pid_plan,
        parent,
        rows,
        replacement_leaf_rows,
        parent_object_version,
        leaf_object_version,
        published_at_micros,
        retain_until_micros,
    )?;
    Ok(
        local_scheduled_replacement_execution_parts_from_relation_parts(
            parts,
            placement_write_evidence,
        ),
    )
}

pub(super) fn build_local_scheduled_split_replacement_execution_parts(
    decision: &SpireLeafReplacementScheduleDecision,
    pid_plan: &SpireLeafReplacementPidPlan,
    parent: &SpireRoutingPartitionObject,
    centroids: Vec<Vec<f32>>,
    routed_leaf_inputs: Vec<SpireReplacementLeafObjectInput>,
    parent_object_version: u64,
    leaf_object_version: u64,
    published_at_micros: i64,
    retain_until_micros: i64,
    placement_write_evidence: Vec<SpirePublishPlacementWriteEvidence>,
) -> Result<SpireLocalScheduledReplacementExecutionParts, String> {
    let parts = build_relation_scheduled_split_replacement_execution_parts(
        decision,
        pid_plan,
        parent,
        centroids,
        routed_leaf_inputs,
        parent_object_version,
        leaf_object_version,
        published_at_micros,
        retain_until_micros,
    )?;
    Ok(
        local_scheduled_replacement_execution_parts_from_relation_parts(
            parts,
            placement_write_evidence,
        ),
    )
}

pub(super) fn build_relation_scheduled_split_replacement_execution_input(
    publish_plan: &SpireScheduledReplacementPublishPlan,
    pid_plan: &SpireLeafReplacementPidPlan,
    decision: &SpireLeafReplacementScheduleDecision,
    parent: &SpireRoutingPartitionObject,
    centroids: Vec<Vec<f32>>,
    routed_leaf_inputs: Vec<SpireReplacementLeafObjectInput>,
    parent_object_version: u64,
    leaf_object_version: u64,
    published_at_micros: i64,
    retain_until_micros: i64,
) -> Result<SpireRelationScheduledReplacementExecutionInput, String> {
    let parts = build_relation_scheduled_split_replacement_execution_parts(
        decision,
        pid_plan,
        parent,
        centroids,
        routed_leaf_inputs,
        parent_object_version,
        leaf_object_version,
        published_at_micros,
        retain_until_micros,
    )?;
    build_relation_scheduled_replacement_execution_input_from_publish_plan(
        publish_plan,
        pid_plan,
        decision,
        parts,
    )
}

pub(super) fn build_relation_selected_scheduled_split_replacement_execution_input(
    selected: &SpireSelectedScheduledReplacementPublishLockPlan,
    parent: &SpireRoutingPartitionObject,
    centroids: Vec<Vec<f32>>,
    routed_leaf_inputs: Vec<SpireReplacementLeafObjectInput>,
    parent_object_version: u64,
    leaf_object_version: u64,
    published_at_micros: i64,
    retain_until_micros: i64,
) -> Result<SpireRelationScheduledReplacementExecutionInput, String> {
    build_relation_scheduled_split_replacement_execution_input(
        &selected.lock_plan.publish_plan,
        &selected.lock_plan.pid_plan,
        &selected.decision,
        parent,
        centroids,
        routed_leaf_inputs,
        parent_object_version,
        leaf_object_version,
        published_at_micros,
        retain_until_micros,
    )
}

pub(super) fn build_local_selected_scheduled_split_replacement_execution_input(
    selected: &SpireSelectedScheduledReplacementPublishLockPlan,
    parent: &SpireRoutingPartitionObject,
    centroids: Vec<Vec<f32>>,
    routed_leaf_inputs: Vec<SpireReplacementLeafObjectInput>,
    parent_object_version: u64,
    leaf_object_version: u64,
    published_at_micros: i64,
    retain_until_micros: i64,
    placement_write_evidence: Vec<SpirePublishPlacementWriteEvidence>,
) -> Result<SpireLocalScheduledReplacementExecutionInput, String> {
    build_local_scheduled_split_replacement_execution_input(
        &selected.lock_plan.publish_plan,
        &selected.lock_plan.pid_plan,
        &selected.decision,
        parent,
        centroids,
        routed_leaf_inputs,
        parent_object_version,
        leaf_object_version,
        published_at_micros,
        retain_until_micros,
        placement_write_evidence,
    )
}

pub(super) fn build_local_scheduled_split_replacement_execution_input(
    publish_plan: &SpireScheduledReplacementPublishPlan,
    pid_plan: &SpireLeafReplacementPidPlan,
    decision: &SpireLeafReplacementScheduleDecision,
    parent: &SpireRoutingPartitionObject,
    centroids: Vec<Vec<f32>>,
    routed_leaf_inputs: Vec<SpireReplacementLeafObjectInput>,
    parent_object_version: u64,
    leaf_object_version: u64,
    published_at_micros: i64,
    retain_until_micros: i64,
    placement_write_evidence: Vec<SpirePublishPlacementWriteEvidence>,
) -> Result<SpireLocalScheduledReplacementExecutionInput, String> {
    let parts = build_local_scheduled_split_replacement_execution_parts(
        decision,
        pid_plan,
        parent,
        centroids,
        routed_leaf_inputs,
        parent_object_version,
        leaf_object_version,
        published_at_micros,
        retain_until_micros,
        placement_write_evidence,
    )?;
    build_local_scheduled_replacement_execution_input_from_publish_plan(
        publish_plan,
        pid_plan,
        decision,
        parts,
    )
}

pub(super) fn build_local_scheduled_merge_replacement_execution_input(
    publish_plan: &SpireScheduledReplacementPublishPlan,
    pid_plan: &SpireLeafReplacementPidPlan,
    decision: &SpireLeafReplacementScheduleDecision,
    parent: &SpireRoutingPartitionObject,
    rows: &[SpireIndexLeafSnapshotRow],
    replacement_leaf_rows: Vec<SpireReplacementLeafRows>,
    parent_object_version: u64,
    leaf_object_version: u64,
    published_at_micros: i64,
    retain_until_micros: i64,
    placement_write_evidence: Vec<SpirePublishPlacementWriteEvidence>,
) -> Result<SpireLocalScheduledReplacementExecutionInput, String> {
    let parts = build_local_scheduled_merge_replacement_execution_parts(
        decision,
        pid_plan,
        parent,
        rows,
        replacement_leaf_rows,
        parent_object_version,
        leaf_object_version,
        published_at_micros,
        retain_until_micros,
        placement_write_evidence,
    )?;
    build_local_scheduled_replacement_execution_input_from_publish_plan(
        publish_plan,
        pid_plan,
        decision,
        parts,
    )
}

pub(super) fn build_relation_scheduled_merge_replacement_execution_input(
    publish_plan: &SpireScheduledReplacementPublishPlan,
    pid_plan: &SpireLeafReplacementPidPlan,
    decision: &SpireLeafReplacementScheduleDecision,
    parent: &SpireRoutingPartitionObject,
    rows: &[SpireIndexLeafSnapshotRow],
    replacement_leaf_rows: Vec<SpireReplacementLeafRows>,
    parent_object_version: u64,
    leaf_object_version: u64,
    published_at_micros: i64,
    retain_until_micros: i64,
) -> Result<SpireRelationScheduledReplacementExecutionInput, String> {
    let parts = build_relation_scheduled_merge_replacement_execution_parts(
        decision,
        pid_plan,
        parent,
        rows,
        replacement_leaf_rows,
        parent_object_version,
        leaf_object_version,
        published_at_micros,
        retain_until_micros,
    )?;
    build_relation_scheduled_replacement_execution_input_from_publish_plan(
        publish_plan,
        pid_plan,
        decision,
        parts,
    )
}

pub(super) fn build_relation_selected_scheduled_merge_replacement_execution_input(
    selected: &SpireSelectedScheduledReplacementPublishLockPlan,
    parent: &SpireRoutingPartitionObject,
    rows: &[SpireIndexLeafSnapshotRow],
    replacement_leaf_rows: Vec<SpireReplacementLeafRows>,
    parent_object_version: u64,
    leaf_object_version: u64,
    published_at_micros: i64,
    retain_until_micros: i64,
) -> Result<SpireRelationScheduledReplacementExecutionInput, String> {
    build_relation_scheduled_merge_replacement_execution_input(
        &selected.lock_plan.publish_plan,
        &selected.lock_plan.pid_plan,
        &selected.decision,
        parent,
        rows,
        replacement_leaf_rows,
        parent_object_version,
        leaf_object_version,
        published_at_micros,
        retain_until_micros,
    )
}

pub(super) fn build_local_selected_scheduled_merge_replacement_execution_input(
    selected: &SpireSelectedScheduledReplacementPublishLockPlan,
    parent: &SpireRoutingPartitionObject,
    rows: &[SpireIndexLeafSnapshotRow],
    replacement_leaf_rows: Vec<SpireReplacementLeafRows>,
    parent_object_version: u64,
    leaf_object_version: u64,
    published_at_micros: i64,
    retain_until_micros: i64,
    placement_write_evidence: Vec<SpirePublishPlacementWriteEvidence>,
) -> Result<SpireLocalScheduledReplacementExecutionInput, String> {
    build_local_scheduled_merge_replacement_execution_input(
        &selected.lock_plan.publish_plan,
        &selected.lock_plan.pid_plan,
        &selected.decision,
        parent,
        rows,
        replacement_leaf_rows,
        parent_object_version,
        leaf_object_version,
        published_at_micros,
        retain_until_micros,
        placement_write_evidence,
    )
}

pub(super) fn rewrite_scheduled_replacement_parent_routing(
    parent: &SpireRoutingPartitionObject,
    decision: &SpireLeafReplacementScheduleDecision,
    replacement_children: Vec<SpireRoutingReplacementChild>,
    object_version: u64,
) -> Result<SpireRoutingPartitionObject, String> {
    validate_leaf_replacement_schedule_decision_shape(decision)?;
    if parent.header.pid != decision.replaced_parent_pid {
        return Err(format!(
            "ec_spire scheduled routing replacement parent pid {} does not match decision parent pid {}",
            parent.header.pid, decision.replaced_parent_pid
        ));
    }
    if replacement_children.len() != decision.replacement_leaf_count {
        return Err(format!(
            "ec_spire scheduled routing replacement child count {} does not match decision replacement count {}",
            replacement_children.len(),
            decision.replacement_leaf_count
        ));
    }
    if object_version == 0 {
        return Err(
            "ec_spire scheduled routing replacement object_version 0 is invalid".to_owned(),
        );
    }
    rewrite_routing_partition_for_leaf_replacement(
        parent,
        &decision.affected_leaf_pids,
        replacement_children,
        object_version,
    )
}

fn leaf_replacement_schedule_decisions_match(
    observed: &SpireLeafReplacementScheduleDecision,
    expected: &SpireLeafReplacementScheduleDecision,
) -> bool {
    observed.mode == expected.mode
        && observed.active_epoch == expected.active_epoch
        && observed.replaced_parent_pid == expected.replaced_parent_pid
        && observed.affected_leaf_pids == expected.affected_leaf_pids
        && observed.replacement_leaf_count == expected.replacement_leaf_count
}

fn validate_leaf_replacement_schedule_decision_shape(
    decision: &SpireLeafReplacementScheduleDecision,
) -> Result<(), String> {
    if decision.active_epoch == 0 {
        return Err("ec_spire replacement scheduler decision active_epoch 0 is invalid".to_owned());
    }
    if decision.replaced_parent_pid == 0 {
        return Err("ec_spire replacement scheduler decision parent pid 0 is invalid".to_owned());
    }
    validate_affected_leaf_pids(&decision.affected_leaf_pids)?;
    match decision.mode {
        SpireLeafReplacementScheduleMode::Split => {
            if decision.affected_leaf_pids.len() != 1 || decision.replacement_leaf_count < 2 {
                return Err(
                    "ec_spire replacement scheduler split decision requires one affected leaf and at least two replacement leaves"
                        .to_owned(),
                );
            }
        }
        SpireLeafReplacementScheduleMode::Merge => {
            if decision.affected_leaf_pids.len() < 2 || decision.replacement_leaf_count != 1 {
                return Err(
                    "ec_spire replacement scheduler merge decision requires at least two affected leaves and one replacement leaf"
                        .to_owned(),
                );
            }
        }
    }
    Ok(())
}

fn validate_leaf_replacement_schedule_rows(
    rows: &[SpireIndexLeafSnapshotRow],
) -> Result<(), String> {
    let mut active_epoch = None;
    let mut seen_leaf_pids = HashSet::new();
    for row in rows {
        if row.active_epoch == 0 {
            return Err("ec_spire replacement scheduler row active_epoch 0 is invalid".to_owned());
        }
        if row.leaf_pid == 0 {
            return Err("ec_spire replacement scheduler row leaf_pid 0 is invalid".to_owned());
        }
        if !seen_leaf_pids.insert(row.leaf_pid) {
            return Err(format!(
                "ec_spire replacement scheduler duplicate row for leaf pid {}",
                row.leaf_pid
            ));
        }
        if row.parent_pid == 0 && (row.split_recommended || row.merge_recommended) {
            return Err(format!(
                "ec_spire replacement scheduler candidate leaf pid {} has parent_pid 0",
                row.leaf_pid
            ));
        }
        if row.split_recommended && row.merge_recommended {
            return Err(format!(
                "ec_spire replacement scheduler leaf pid {} cannot be both split and merge recommended",
                row.leaf_pid
            ));
        }
        match active_epoch {
            Some(epoch) if epoch != row.active_epoch => {
                return Err(format!(
                    "ec_spire replacement scheduler rows span multiple active epochs: {epoch} and {}",
                    row.active_epoch
                ));
            }
            Some(_) => {}
            None => active_epoch = Some(row.active_epoch),
        }
    }
    Ok(())
}

fn merge_pair_sort_key(
    pair: (&SpireIndexLeafSnapshotRow, &SpireIndexLeafSnapshotRow),
) -> (u64, u64, u64, u64) {
    (
        pair.0
            .effective_assignment_count
            .saturating_add(pair.1.effective_assignment_count),
        pair.0.effective_assignment_count,
        pair.0.leaf_pid,
        pair.1.leaf_pid,
    )
}

pub(super) fn collect_replacement_leaf_rows(
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    object_store: &impl SpireObjectReader,
    affected_leaf_pids: &[u64],
) -> Result<Vec<SpireReplacementLeafRows>, String> {
    validate_affected_leaf_pids(affected_leaf_pids)?;
    let affected: HashSet<u64> = affected_leaf_pids.iter().copied().collect();
    let snapshot = SpireValidatedEpochSnapshot::from_snapshot(*snapshot)?;
    validate_delta_base_snapshot_placements_available(&snapshot)?;
    let deleted_by_base_pid = collect_delete_vec_ids_by_base_pid(&snapshot, object_store)?;
    let mut rows_by_base_pid: HashMap<u64, Vec<SpireLeafAssignmentRow>> = HashMap::new();
    let mut active_leaf_pids = HashSet::new();
    let mut visible_vec_ids = HashSet::new();

    for manifest_entry in &snapshot.object_manifest().entries {
        let lookup = snapshot.require_lookup(manifest_entry.pid, "replacement leaf row")?;
        let placement = lookup.placement;
        let header = object_store.read_object_header(placement)?;
        match header.kind {
            SpirePartitionObjectKind::Leaf if affected.contains(&manifest_entry.pid) => {
                active_leaf_pids.insert(manifest_entry.pid);
                let deleted = deleted_by_base_pid.get(&manifest_entry.pid);
                for assignment in read_leaf_assignments_for_replacement(object_store, placement)? {
                    push_visible_replacement_row(
                        &mut rows_by_base_pid,
                        &mut visible_vec_ids,
                        manifest_entry.pid,
                        assignment,
                        deleted,
                    )?;
                }
            }
            SpirePartitionObjectKind::Delta if affected.contains(&header.parent_pid) => {
                let deleted = deleted_by_base_pid.get(&header.parent_pid);
                let delta = object_store.read_delta_object(placement)?;
                for assignment in delta.assignments {
                    push_visible_replacement_row(
                        &mut rows_by_base_pid,
                        &mut visible_vec_ids,
                        header.parent_pid,
                        assignment,
                        deleted,
                    )?;
                }
            }
            SpirePartitionObjectKind::Root
            | SpirePartitionObjectKind::Internal
            | SpirePartitionObjectKind::Leaf
            | SpirePartitionObjectKind::Delta => {}
        }
    }

    if active_leaf_pids.len() != affected.len() {
        let mut missing = affected
            .difference(&active_leaf_pids)
            .copied()
            .collect::<Vec<_>>();
        missing.sort_unstable();
        return Err(format!(
            "ec_spire replacement leaf rows require active leaf pids for all affected pids: missing {missing:?}"
        ));
    }

    let mut folded = rows_by_base_pid
        .into_iter()
        .map(|(base_pid, rows)| SpireReplacementLeafRows { base_pid, rows })
        .collect::<Vec<_>>();
    folded.sort_by_key(|entry| entry.base_pid);
    Ok(folded)
}

pub(super) fn collect_selected_scheduled_replacement_leaf_rows(
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    object_store: &impl SpireObjectReader,
    selected: &SpireSelectedScheduledReplacementPublishLockPlan,
) -> Result<Vec<SpireReplacementLeafRows>, String> {
    validate_selected_scheduled_replacement_execution_snapshot(snapshot, selected)?;
    collect_replacement_leaf_rows(
        snapshot,
        object_store,
        &selected.decision.affected_leaf_pids,
    )
}

pub(super) fn rewrite_routing_partition_for_leaf_replacement(
    parent: &SpireRoutingPartitionObject,
    affected_child_pids: &[u64],
    replacement_children: Vec<SpireRoutingReplacementChild>,
    object_version: u64,
) -> Result<SpireRoutingPartitionObject, String> {
    validate_affected_leaf_pids(affected_child_pids)?;
    validate_replacement_routing_children(parent, affected_child_pids, &replacement_children)?;

    let affected: HashSet<u64> = affected_child_pids.iter().copied().collect();
    let mut inserted_replacements = false;
    let mut children = Vec::with_capacity(
        parent
            .child_count()
            .saturating_sub(affected.len())
            .saturating_add(replacement_children.len()),
    );
    for child in parent.children() {
        if affected.contains(&child.child_pid) {
            if !inserted_replacements {
                append_replacement_routing_children(&mut children, &replacement_children)?;
                inserted_replacements = true;
            }
            continue;
        }
        let centroid_index = u32::try_from(children.len())
            .map_err(|_| "ec_spire routing replacement child count exceeds u32".to_owned())?;
        children.push(SpireRoutingChildEntry {
            centroid_index,
            child_pid: child.child_pid,
            centroid: child.centroid.to_vec(),
        });
    }

    if !inserted_replacements {
        return Err("ec_spire routing replacement did not find any affected child pid".to_owned());
    }

    match parent.header.kind {
        SpirePartitionObjectKind::Root => SpireRoutingPartitionObject::root(
            parent.header.pid,
            object_version,
            parent.dimensions,
            children,
        ),
        SpirePartitionObjectKind::Internal => SpireRoutingPartitionObject::internal(
            parent.header.pid,
            object_version,
            parent.header.level,
            parent.header.parent_pid,
            parent.dimensions,
            children,
        ),
        other => Err(format!(
            "ec_spire routing replacement parent must be Root or Internal, got {other:?}"
        )),
    }
}

pub(super) fn plan_replacement_epoch_placement_directory(
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    object_store: &impl SpireObjectReader,
    new_epoch: u64,
    replaced_parent_pid: u64,
    replacement_parent_placement: SpirePlacementEntry,
    affected_leaf_pids: &[u64],
    replacement_leaf_placements: Vec<SpirePlacementEntry>,
) -> Result<SpirePlacementDirectory, String> {
    if new_epoch == 0 {
        return Err("ec_spire replacement placement epoch 0 is invalid".to_owned());
    }
    if replaced_parent_pid == 0 {
        return Err("ec_spire replacement parent pid 0 is invalid".to_owned());
    }
    validate_affected_leaf_pids(affected_leaf_pids)?;
    let snapshot = SpireValidatedEpochSnapshot::from_snapshot(*snapshot)?;
    if new_epoch <= snapshot.epoch_manifest().epoch {
        return Err(format!(
            "ec_spire replacement placement epoch {new_epoch} must be newer than base epoch {}",
            snapshot.epoch_manifest().epoch
        ));
    }
    validate_delta_base_snapshot_placements_available(&snapshot)?;
    validate_replacement_placement(
        new_epoch,
        replaced_parent_pid,
        "replacement parent",
        &replacement_parent_placement,
    )?;
    let affected: HashSet<u64> = affected_leaf_pids.iter().copied().collect();
    let mut replacement_leaf_pids = HashSet::new();
    for placement in &replacement_leaf_placements {
        validate_replacement_placement(new_epoch, placement.pid, "replacement leaf", placement)?;
        if placement.pid == replaced_parent_pid {
            return Err(
                "ec_spire replacement leaf placement cannot use the parent routing pid".to_owned(),
            );
        }
        if !replacement_leaf_pids.insert(placement.pid) {
            return Err("ec_spire replacement leaf placements must have unique pids".to_owned());
        }
    }

    let mut entries = Vec::with_capacity(
        snapshot
            .placement_directory()
            .entries
            .len()
            .saturating_add(1)
            .saturating_add(replacement_leaf_placements.len()),
    );
    let mut active_affected_leaves = HashSet::new();
    for manifest_entry in &snapshot.object_manifest().entries {
        let lookup = snapshot.require_lookup(manifest_entry.pid, "replacement placement")?;
        let placement = lookup.placement;
        let header = object_store.read_object_header(placement)?;
        match header.kind {
            SpirePartitionObjectKind::Root | SpirePartitionObjectKind::Internal
                if manifest_entry.pid == replaced_parent_pid =>
            {
                continue;
            }
            SpirePartitionObjectKind::Leaf if affected.contains(&manifest_entry.pid) => {
                active_affected_leaves.insert(manifest_entry.pid);
                continue;
            }
            SpirePartitionObjectKind::Delta if affected.contains(&header.parent_pid) => {
                continue;
            }
            SpirePartitionObjectKind::Root
            | SpirePartitionObjectKind::Internal
            | SpirePartitionObjectKind::Leaf
            | SpirePartitionObjectKind::Delta => {
                let mut carried = *placement;
                carried.epoch = new_epoch;
                entries.push(carried);
            }
        }
    }

    if active_affected_leaves.len() != affected.len() {
        let mut missing = affected
            .difference(&active_affected_leaves)
            .copied()
            .collect::<Vec<_>>();
        missing.sort_unstable();
        return Err(format!(
            "ec_spire replacement placement requires active leaf pids for all affected pids: missing {missing:?}"
        ));
    }

    entries.push(replacement_parent_placement);
    entries.extend(replacement_leaf_placements);
    SpirePlacementDirectory::from_entries(new_epoch, entries)
}

pub(super) fn build_replacement_epoch_draft(
    input: SpireReplacementEpochInput,
) -> Result<SpireReplacementEpochDraft, String> {
    let epoch_manifest = SpireEpochManifest {
        epoch: input.epoch,
        state: SpireEpochState::Published,
        consistency_mode: input.consistency_mode,
        published_at_micros: input.published_at_micros,
        retain_until_micros: input.retain_until_micros,
        active_query_count: 0,
    };
    epoch_manifest.validate()?;
    let object_manifest = object_manifest_from_placement_writes(
        input.epoch,
        &input.placement_directory,
        &input.placement_write_evidence,
    )?;
    let draft = SpireReplacementEpochDraft {
        epoch_manifest,
        object_manifest,
        placement_directory: input.placement_directory,
        next_pid: input.next_pid,
        next_local_vec_seq: input.next_local_vec_seq,
    };
    SpireValidatedEpochSnapshot::new(
        &draft.epoch_manifest,
        &draft.object_manifest,
        &draft.placement_directory,
    )?;
    root_control_state_for_publish(draft.publish_input(), manifest_locators_for_validation())?;
    Ok(draft)
}

pub(super) fn build_replacement_epoch_draft_from_object_placements(
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    object_store: &impl SpireObjectReader,
    input: SpireReplacementEpochObjectPlacementInput,
) -> Result<SpireReplacementEpochDraft, String> {
    let placement_directory = replacement_placement_directory_from_object_placements(
        snapshot,
        object_store,
        input.epoch,
        input.replaced_parent_pid,
        input.affected_leaf_pids,
        input.replacement_object_placements,
    )?;

    build_replacement_epoch_draft(SpireReplacementEpochInput {
        epoch: input.epoch,
        published_at_micros: input.published_at_micros,
        retain_until_micros: input.retain_until_micros,
        consistency_mode: input.consistency_mode,
        placement_directory,
        placement_write_evidence: input.placement_write_evidence,
        next_pid: input.next_pid,
        next_local_vec_seq: input.next_local_vec_seq,
    })
}

pub(super) fn build_scheduled_replacement_epoch_draft_from_object_placements(
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    object_store: &impl SpireObjectReader,
    decision: &SpireLeafReplacementScheduleDecision,
    input: SpireScheduledReplacementEpochObjectPlacementInput,
) -> Result<SpireReplacementEpochDraft, String> {
    validate_leaf_replacement_schedule_decision_shape(decision)?;
    if snapshot.epoch_manifest.epoch != decision.active_epoch {
        return Err(format!(
            "ec_spire scheduled replacement publish snapshot epoch {} does not match decision active epoch {}",
            snapshot.epoch_manifest.epoch, decision.active_epoch
        ));
    }
    let expected_epoch = decision
        .active_epoch
        .checked_add(1)
        .ok_or_else(|| "ec_spire scheduled replacement publish epoch overflow".to_owned())?;
    if input.epoch != expected_epoch {
        return Err(format!(
            "ec_spire scheduled replacement publish epoch {} must be the immediate successor of active epoch {}",
            input.epoch, decision.active_epoch
        ));
    }
    if input.consistency_mode != snapshot.epoch_manifest.consistency_mode {
        return Err(format!(
            "ec_spire scheduled replacement consistency mode {:?} does not match active epoch consistency mode {:?}",
            input.consistency_mode, snapshot.epoch_manifest.consistency_mode
        ));
    }
    if input.replacement_object_placements.leaf_placements.len() != decision.replacement_leaf_count
    {
        return Err(format!(
            "ec_spire scheduled replacement publish leaf placement count {} does not match decision replacement count {}",
            input.replacement_object_placements.leaf_placements.len(),
            decision.replacement_leaf_count
        ));
    }
    build_replacement_epoch_draft_from_object_placements(
        snapshot,
        object_store,
        SpireReplacementEpochObjectPlacementInput {
            epoch: input.epoch,
            published_at_micros: input.published_at_micros,
            retain_until_micros: input.retain_until_micros,
            consistency_mode: input.consistency_mode,
            replaced_parent_pid: decision.replaced_parent_pid,
            affected_leaf_pids: decision.affected_leaf_pids.clone(),
            replacement_object_placements: input.replacement_object_placements,
            placement_write_evidence: input.placement_write_evidence,
            next_pid: input.next_pid,
            next_local_vec_seq: input.next_local_vec_seq,
        },
    )
}

pub(super) fn validate_scheduled_replacement_pid_plan_output(
    decision: &SpireLeafReplacementScheduleDecision,
    pid_plan: &SpireLeafReplacementPidPlan,
    replacement_object_placements: &SpireReplacementObjectPlacements,
    next_pid: u64,
) -> Result<(), String> {
    validate_leaf_replacement_schedule_decision_shape(decision)?;
    if pid_plan.reuses_existing_pid {
        return Err(
            "ec_spire scheduled replacement publish requires fresh replacement pids".to_owned(),
        );
    }
    if pid_plan.replacement_pids.len() != decision.replacement_leaf_count {
        return Err(format!(
            "ec_spire scheduled replacement pid plan count {} does not match decision replacement count {}",
            pid_plan.replacement_pids.len(),
            decision.replacement_leaf_count
        ));
    }
    if replacement_object_placements.parent_placement.pid != decision.replaced_parent_pid {
        return Err(format!(
            "ec_spire scheduled replacement parent placement pid {} does not match decision parent pid {}",
            replacement_object_placements.parent_placement.pid,
            decision.replaced_parent_pid
        ));
    }
    let placement_pids = replacement_object_placements
        .leaf_placements
        .iter()
        .map(|placement| placement.pid)
        .collect::<Vec<_>>();
    if placement_pids != pid_plan.replacement_pids {
        return Err(format!(
            "ec_spire scheduled replacement leaf placement pids {:?} do not match pid plan {:?}",
            placement_pids, pid_plan.replacement_pids
        ));
    }
    if next_pid != pid_plan.next_pid {
        return Err(format!(
            "ec_spire scheduled replacement next_pid {next_pid} does not match pid plan next_pid {}",
            pid_plan.next_pid
        ));
    }
    Ok(())
}

pub(super) fn plan_scheduled_replacement_publish_epoch(
    root_control: &SpireRootControlState,
    active_epoch_manifest: &SpireEpochManifest,
    decision: &SpireLeafReplacementScheduleDecision,
    pid_plan: &SpireLeafReplacementPidPlan,
) -> Result<SpireScheduledReplacementPublishPlan, String> {
    validate_leaf_replacement_schedule_decision_shape(decision)?;
    if root_control.active_epoch != decision.active_epoch {
        return Err(format!(
            "ec_spire scheduled replacement root/control active epoch {} does not match decision active epoch {}",
            root_control.active_epoch, decision.active_epoch
        ));
    }
    if active_epoch_manifest.epoch != decision.active_epoch {
        return Err(format!(
            "ec_spire scheduled replacement manifest epoch {} does not match decision active epoch {}",
            active_epoch_manifest.epoch, decision.active_epoch
        ));
    }
    if active_epoch_manifest.state != SpireEpochState::Published {
        return Err(format!(
            "ec_spire scheduled replacement active epoch must be published, got {:?}",
            active_epoch_manifest.state
        ));
    }
    if pid_plan.reuses_existing_pid {
        return Err(
            "ec_spire scheduled replacement publish plan requires fresh replacement pids"
                .to_owned(),
        );
    }
    if pid_plan.replacement_pids.len() != decision.replacement_leaf_count {
        return Err(format!(
            "ec_spire scheduled replacement publish plan pid count {} does not match decision replacement count {}",
            pid_plan.replacement_pids.len(),
            decision.replacement_leaf_count
        ));
    }
    let mut seen_replacement_pids = HashSet::new();
    if let Some(duplicate_pid) = pid_plan
        .replacement_pids
        .iter()
        .copied()
        .find(|pid| !seen_replacement_pids.insert(*pid))
    {
        return Err(format!(
            "ec_spire scheduled replacement publish plan replacement pid {duplicate_pid} appears more than once"
        ));
    }
    if let Some(stale_pid) = pid_plan
        .replacement_pids
        .iter()
        .copied()
        .find(|pid| *pid < root_control.next_pid)
    {
        return Err(format!(
            "ec_spire scheduled replacement pid {stale_pid} is behind root/control next_pid {}",
            root_control.next_pid
        ));
    }
    if let Some(unadvanced_pid) = pid_plan
        .replacement_pids
        .iter()
        .copied()
        .find(|pid| *pid >= pid_plan.next_pid)
    {
        return Err(format!(
            "ec_spire scheduled replacement pid plan next_pid {} does not advance past replacement pid {unadvanced_pid}",
            pid_plan.next_pid
        ));
    }
    if pid_plan.next_pid < root_control.next_pid {
        return Err(format!(
            "ec_spire scheduled replacement pid plan next_pid {} is behind root/control next_pid {}",
            pid_plan.next_pid, root_control.next_pid
        ));
    }
    let epoch = decision
        .active_epoch
        .checked_add(1)
        .ok_or_else(|| "ec_spire scheduled replacement publish epoch overflow".to_owned())?;
    Ok(SpireScheduledReplacementPublishPlan {
        epoch,
        consistency_mode: active_epoch_manifest.consistency_mode,
        next_pid: pid_plan.next_pid,
        next_local_vec_seq: root_control.next_local_vec_seq,
    })
}

pub(super) fn plan_scheduled_replacement_publish_lock(
    root_control: &SpireRootControlState,
    active_epoch_manifest: &SpireEpochManifest,
    decision: &SpireLeafReplacementScheduleDecision,
    pid_allocator: &mut SpirePidAllocator,
) -> Result<SpireScheduledReplacementPublishLockPlan, String> {
    let mut planned_pid_allocator = *pid_allocator;
    let pid_plan = plan_scheduled_leaf_replacement_pids(decision, &mut planned_pid_allocator)?;
    let publish_plan = plan_scheduled_replacement_publish_epoch(
        root_control,
        active_epoch_manifest,
        decision,
        &pid_plan,
    )?;
    *pid_allocator = planned_pid_allocator;
    Ok(SpireScheduledReplacementPublishLockPlan {
        pid_plan,
        publish_plan,
    })
}

pub(super) fn plan_rechecked_scheduled_replacement_publish_lock(
    rows: &[SpireIndexLeafSnapshotRow],
    root_control: &SpireRootControlState,
    active_epoch_manifest: &SpireEpochManifest,
    decision: &SpireLeafReplacementScheduleDecision,
    pid_allocator: &mut SpirePidAllocator,
) -> Result<SpireScheduledReplacementPublishLockPlan, String> {
    recheck_leaf_replacement_schedule_decision(rows, decision)?;
    plan_scheduled_replacement_publish_lock(
        root_control,
        active_epoch_manifest,
        decision,
        pid_allocator,
    )
}

pub(super) fn choose_scheduled_replacement_publish_lock_plan(
    rows: &[SpireIndexLeafSnapshotRow],
    root_control: &SpireRootControlState,
    active_epoch_manifest: &SpireEpochManifest,
    pid_allocator: &mut SpirePidAllocator,
) -> Result<Option<SpireSelectedScheduledReplacementPublishLockPlan>, String> {
    let Some(decision) = choose_leaf_replacement_schedule(rows)? else {
        return Ok(None);
    };
    let lock_plan = plan_rechecked_scheduled_replacement_publish_lock(
        rows,
        root_control,
        active_epoch_manifest,
        &decision,
        pid_allocator,
    )?;
    Ok(Some(SpireSelectedScheduledReplacementPublishLockPlan {
        decision,
        lock_plan,
    }))
}

pub(super) fn build_local_scheduled_replacement_execution_input_from_publish_plan(
    publish_plan: &SpireScheduledReplacementPublishPlan,
    pid_plan: &SpireLeafReplacementPidPlan,
    decision: &SpireLeafReplacementScheduleDecision,
    parts: SpireLocalScheduledReplacementExecutionParts,
) -> Result<SpireLocalScheduledReplacementExecutionInput, String> {
    validate_scheduled_replacement_execution_publish_plan_parts(
        publish_plan,
        pid_plan,
        decision,
        &parts.replacement_parent,
        &parts.replacement_children,
        parts.leaf_object_version,
        parts.published_at_micros,
        &parts.leaf_inputs,
    )?;

    Ok(SpireLocalScheduledReplacementExecutionInput {
        epoch: publish_plan.epoch,
        published_at_micros: parts.published_at_micros,
        retain_until_micros: parts.retain_until_micros,
        consistency_mode: publish_plan.consistency_mode,
        replacement_parent: parts.replacement_parent,
        replacement_children: parts.replacement_children,
        leaf_object_version: parts.leaf_object_version,
        leaf_inputs: parts.leaf_inputs,
        placement_write_evidence: parts.placement_write_evidence,
        next_local_vec_seq: publish_plan.next_local_vec_seq,
    })
}

pub(super) fn build_relation_scheduled_replacement_execution_input_from_publish_plan(
    publish_plan: &SpireScheduledReplacementPublishPlan,
    pid_plan: &SpireLeafReplacementPidPlan,
    decision: &SpireLeafReplacementScheduleDecision,
    parts: SpireRelationScheduledReplacementExecutionParts,
) -> Result<SpireRelationScheduledReplacementExecutionInput, String> {
    validate_scheduled_replacement_execution_publish_plan_parts(
        publish_plan,
        pid_plan,
        decision,
        &parts.replacement_parent,
        &parts.replacement_children,
        parts.leaf_object_version,
        parts.published_at_micros,
        &parts.leaf_inputs,
    )?;

    Ok(SpireRelationScheduledReplacementExecutionInput {
        epoch: publish_plan.epoch,
        published_at_micros: parts.published_at_micros,
        retain_until_micros: parts.retain_until_micros,
        consistency_mode: publish_plan.consistency_mode,
        replacement_parent: parts.replacement_parent,
        replacement_children: parts.replacement_children,
        leaf_object_version: parts.leaf_object_version,
        leaf_inputs: parts.leaf_inputs,
        next_local_vec_seq: publish_plan.next_local_vec_seq,
    })
}

pub(super) fn validate_relation_scheduled_replacement_execution_publish_plan(
    publish_plan: &SpireScheduledReplacementPublishPlan,
    pid_plan: &SpireLeafReplacementPidPlan,
    decision: &SpireLeafReplacementScheduleDecision,
    input: &SpireRelationScheduledReplacementExecutionInput,
) -> Result<(), String> {
    if input.epoch != publish_plan.epoch {
        return Err(format!(
            "ec_spire relation scheduled replacement execution epoch {} does not match publish plan epoch {}",
            input.epoch, publish_plan.epoch
        ));
    }
    if input.consistency_mode != publish_plan.consistency_mode {
        return Err(format!(
            "ec_spire relation scheduled replacement execution consistency mode {:?} does not match publish plan consistency mode {:?}",
            input.consistency_mode, publish_plan.consistency_mode
        ));
    }
    if input.next_local_vec_seq != publish_plan.next_local_vec_seq {
        return Err(format!(
            "ec_spire relation scheduled replacement execution next_local_vec_seq {} does not match publish plan next_local_vec_seq {}",
            input.next_local_vec_seq, publish_plan.next_local_vec_seq
        ));
    }
    validate_scheduled_replacement_execution_publish_plan_parts(
        publish_plan,
        pid_plan,
        decision,
        &input.replacement_parent,
        &input.replacement_children,
        input.leaf_object_version,
        input.published_at_micros,
        &input.leaf_inputs,
    )
}

pub(super) fn validate_relation_selected_scheduled_replacement_execution_publish_plan(
    selected: &SpireSelectedScheduledReplacementPublishLockPlan,
    input: &SpireRelationScheduledReplacementExecutionInput,
) -> Result<(), String> {
    validate_relation_scheduled_replacement_execution_publish_plan(
        &selected.lock_plan.publish_plan,
        &selected.lock_plan.pid_plan,
        &selected.decision,
        input,
    )
}

pub(super) fn validate_local_scheduled_replacement_execution_publish_plan(
    publish_plan: &SpireScheduledReplacementPublishPlan,
    pid_plan: &SpireLeafReplacementPidPlan,
    decision: &SpireLeafReplacementScheduleDecision,
    input: &SpireLocalScheduledReplacementExecutionInput,
) -> Result<(), String> {
    if input.epoch != publish_plan.epoch {
        return Err(format!(
            "ec_spire local scheduled replacement execution epoch {} does not match publish plan epoch {}",
            input.epoch, publish_plan.epoch
        ));
    }
    if input.consistency_mode != publish_plan.consistency_mode {
        return Err(format!(
            "ec_spire local scheduled replacement execution consistency mode {:?} does not match publish plan consistency mode {:?}",
            input.consistency_mode, publish_plan.consistency_mode
        ));
    }
    if input.next_local_vec_seq != publish_plan.next_local_vec_seq {
        return Err(format!(
            "ec_spire local scheduled replacement execution next_local_vec_seq {} does not match publish plan next_local_vec_seq {}",
            input.next_local_vec_seq, publish_plan.next_local_vec_seq
        ));
    }
    validate_scheduled_replacement_execution_publish_plan_parts(
        publish_plan,
        pid_plan,
        decision,
        &input.replacement_parent,
        &input.replacement_children,
        input.leaf_object_version,
        input.published_at_micros,
        &input.leaf_inputs,
    )
}

pub(super) fn validate_local_selected_scheduled_replacement_execution_publish_plan(
    selected: &SpireSelectedScheduledReplacementPublishLockPlan,
    input: &SpireLocalScheduledReplacementExecutionInput,
) -> Result<(), String> {
    validate_local_scheduled_replacement_execution_publish_plan(
        &selected.lock_plan.publish_plan,
        &selected.lock_plan.pid_plan,
        &selected.decision,
        input,
    )
}

fn validate_scheduled_replacement_execution_publish_plan_parts(
    publish_plan: &SpireScheduledReplacementPublishPlan,
    pid_plan: &SpireLeafReplacementPidPlan,
    decision: &SpireLeafReplacementScheduleDecision,
    replacement_parent: &SpireRoutingPartitionObject,
    replacement_children: &[SpireRoutingReplacementChild],
    leaf_object_version: u64,
    published_at_micros: i64,
    leaf_inputs: &[SpireReplacementLeafObjectInput],
) -> Result<(), String> {
    validate_leaf_replacement_schedule_decision_shape(decision)?;
    let expected_epoch = decision.active_epoch.checked_add(1).ok_or_else(|| {
        "ec_spire scheduled replacement execution publish epoch overflow".to_owned()
    })?;
    if publish_plan.epoch != expected_epoch {
        return Err(format!(
            "ec_spire scheduled replacement execution publish plan epoch {} must be the immediate successor of active epoch {}",
            publish_plan.epoch, decision.active_epoch
        ));
    }
    if replacement_parent.header.pid != decision.replaced_parent_pid {
        return Err(format!(
            "ec_spire scheduled replacement execution parent pid {} does not match decision parent pid {}",
            replacement_parent.header.pid, decision.replaced_parent_pid
        ));
    }
    if replacement_children.len() != decision.replacement_leaf_count {
        return Err(format!(
            "ec_spire scheduled replacement execution child count {} does not match decision replacement count {}",
            replacement_children.len(),
            decision.replacement_leaf_count
        ));
    }
    validate_scheduled_replacement_parent_contents(
        replacement_parent,
        decision,
        replacement_children,
    )?;
    if leaf_object_version == 0 {
        return Err(
            "ec_spire scheduled replacement execution leaf object_version 0 is invalid".to_owned(),
        );
    }
    if published_at_micros <= 0 {
        return Err(
            "ec_spire scheduled replacement execution requires a publish timestamp".to_owned(),
        );
    }
    if pid_plan.reuses_existing_pid {
        return Err(
            "ec_spire scheduled replacement execution input requires fresh replacement pids"
                .to_owned(),
        );
    }
    if publish_plan.next_pid != pid_plan.next_pid {
        return Err(format!(
            "ec_spire scheduled replacement execution input next_pid {} does not match pid plan next_pid {}",
            publish_plan.next_pid, pid_plan.next_pid
        ));
    }
    let child_pids = replacement_children
        .iter()
        .map(|child| child.child_pid)
        .collect::<Vec<_>>();
    if child_pids != pid_plan.replacement_pids {
        return Err(format!(
            "ec_spire scheduled replacement execution child pids {:?} do not match pid plan {:?}",
            child_pids, pid_plan.replacement_pids
        ));
    }
    validate_replacement_leaf_object_inputs(replacement_children, leaf_inputs)
}

fn validate_scheduled_replacement_parent_contents(
    replacement_parent: &SpireRoutingPartitionObject,
    decision: &SpireLeafReplacementScheduleDecision,
    replacement_children: &[SpireRoutingReplacementChild],
) -> Result<(), String> {
    let parent_child_pids = replacement_parent
        .children()
        .map(|child| child.child_pid)
        .collect::<HashSet<_>>();
    for child in replacement_children {
        if !parent_child_pids.contains(&child.child_pid) {
            return Err(format!(
                "ec_spire scheduled replacement execution parent is missing replacement child pid {}",
                child.child_pid
            ));
        }
    }
    for affected_pid in &decision.affected_leaf_pids {
        if parent_child_pids.contains(affected_pid) {
            return Err(format!(
                "ec_spire scheduled replacement execution parent still contains affected leaf pid {affected_pid}"
            ));
        }
    }
    Ok(())
}

fn validate_scheduled_replacement_execution_snapshot(
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    decision: &SpireLeafReplacementScheduleDecision,
    publish_plan: &SpireScheduledReplacementPublishPlan,
) -> Result<(), String> {
    if snapshot.epoch_manifest.epoch != decision.active_epoch {
        return Err(format!(
            "ec_spire scheduled replacement execution snapshot epoch {} does not match decision active epoch {}",
            snapshot.epoch_manifest.epoch, decision.active_epoch
        ));
    }
    if publish_plan.consistency_mode != snapshot.epoch_manifest.consistency_mode {
        return Err(format!(
            "ec_spire scheduled replacement execution publish plan consistency mode {:?} does not match active snapshot consistency mode {:?}",
            publish_plan.consistency_mode, snapshot.epoch_manifest.consistency_mode
        ));
    }
    Ok(())
}

pub(super) fn validate_selected_scheduled_replacement_execution_snapshot(
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    selected: &SpireSelectedScheduledReplacementPublishLockPlan,
) -> Result<(), String> {
    validate_scheduled_replacement_execution_snapshot(
        snapshot,
        &selected.decision,
        &selected.lock_plan.publish_plan,
    )
}

pub(super) fn validate_relation_selected_scheduled_replacement_publish_inputs(
    previous_epoch_manifest: &SpireEpochManifest,
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    selected: &SpireSelectedScheduledReplacementPublishLockPlan,
    input: &SpireRelationScheduledReplacementExecutionInput,
) -> Result<(), String> {
    if previous_epoch_manifest != snapshot.epoch_manifest {
        return Err(format!(
            "ec_spire selected scheduled replacement publish previous epoch manifest mismatch: got {}, expected {}",
            previous_epoch_manifest.epoch, snapshot.epoch_manifest.epoch
        ));
    }
    validate_relation_selected_scheduled_replacement_execution_publish_plan(selected, input)?;
    validate_selected_scheduled_replacement_execution_snapshot(snapshot, selected)
}

pub(super) fn validate_local_selected_scheduled_replacement_draft_inputs(
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    selected: &SpireSelectedScheduledReplacementPublishLockPlan,
    input: &SpireLocalScheduledReplacementExecutionInput,
) -> Result<(), String> {
    validate_local_selected_scheduled_replacement_execution_publish_plan(selected, input)?;
    validate_selected_scheduled_replacement_execution_snapshot(snapshot, selected)
}

pub(super) fn build_local_scheduled_replacement_epoch_draft(
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    decision: &SpireLeafReplacementScheduleDecision,
    pid_plan: &SpireLeafReplacementPidPlan,
    publish_plan: &SpireScheduledReplacementPublishPlan,
    input: SpireLocalScheduledReplacementExecutionInput,
    object_store: &mut SpireLocalObjectStore,
) -> Result<SpireReplacementEpochDraft, String> {
    validate_local_scheduled_replacement_execution_publish_plan(
        publish_plan,
        pid_plan,
        decision,
        &input,
    )?;
    validate_scheduled_replacement_execution_snapshot(snapshot, decision, publish_plan)?;
    let replacement_object_placements = write_local_scheduled_replacement_objects(
        input.epoch,
        &input.replacement_parent,
        decision,
        &input.replacement_children,
        input.leaf_object_version,
        input.leaf_inputs,
        object_store,
    )?;
    validate_scheduled_replacement_pid_plan_output(
        decision,
        pid_plan,
        &replacement_object_placements,
        pid_plan.next_pid,
    )?;
    build_scheduled_replacement_epoch_draft_from_object_placements(
        snapshot,
        object_store,
        decision,
        SpireScheduledReplacementEpochObjectPlacementInput {
            epoch: input.epoch,
            published_at_micros: input.published_at_micros,
            retain_until_micros: input.retain_until_micros,
            consistency_mode: input.consistency_mode,
            replacement_object_placements,
            placement_write_evidence: input.placement_write_evidence,
            next_pid: pid_plan.next_pid,
            next_local_vec_seq: input.next_local_vec_seq,
        },
    )
}

pub(super) fn build_local_selected_scheduled_replacement_epoch_draft(
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    selected: &SpireSelectedScheduledReplacementPublishLockPlan,
    input: SpireLocalScheduledReplacementExecutionInput,
    object_store: &mut SpireLocalObjectStore,
) -> Result<SpireReplacementEpochDraft, String> {
    validate_local_selected_scheduled_replacement_draft_inputs(snapshot, selected, &input)?;
    build_local_scheduled_replacement_epoch_draft(
        snapshot,
        &selected.decision,
        &selected.lock_plan.pid_plan,
        &selected.lock_plan.publish_plan,
        input,
        object_store,
    )
}

pub(super) fn build_local_selected_scheduled_split_replacement_epoch_draft(
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    selected: &SpireSelectedScheduledReplacementPublishLockPlan,
    parent: &SpireRoutingPartitionObject,
    centroids: Vec<Vec<f32>>,
    routed_leaf_inputs: Vec<SpireReplacementLeafObjectInput>,
    parent_object_version: u64,
    leaf_object_version: u64,
    published_at_micros: i64,
    retain_until_micros: i64,
    placement_write_evidence: Vec<SpirePublishPlacementWriteEvidence>,
    object_store: &mut SpireLocalObjectStore,
) -> Result<SpireReplacementEpochDraft, String> {
    let input = build_local_selected_scheduled_split_replacement_execution_input(
        selected,
        parent,
        centroids,
        routed_leaf_inputs,
        parent_object_version,
        leaf_object_version,
        published_at_micros,
        retain_until_micros,
        placement_write_evidence,
    )?;
    build_local_selected_scheduled_replacement_epoch_draft(snapshot, selected, input, object_store)
}

pub(super) fn build_local_selected_scheduled_merge_replacement_epoch_draft(
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    selected: &SpireSelectedScheduledReplacementPublishLockPlan,
    parent: &SpireRoutingPartitionObject,
    rows: &[SpireIndexLeafSnapshotRow],
    replacement_leaf_rows: Vec<SpireReplacementLeafRows>,
    parent_object_version: u64,
    leaf_object_version: u64,
    published_at_micros: i64,
    retain_until_micros: i64,
    placement_write_evidence: Vec<SpirePublishPlacementWriteEvidence>,
    object_store: &mut SpireLocalObjectStore,
) -> Result<SpireReplacementEpochDraft, String> {
    let input = build_local_selected_scheduled_merge_replacement_execution_input(
        selected,
        parent,
        rows,
        replacement_leaf_rows,
        parent_object_version,
        leaf_object_version,
        published_at_micros,
        retain_until_micros,
        placement_write_evidence,
    )?;
    build_local_selected_scheduled_replacement_epoch_draft(snapshot, selected, input, object_store)
}

pub(super) fn build_local_selected_scheduled_merge_replacement_epoch_draft_from_snapshot(
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    selected: &SpireSelectedScheduledReplacementPublishLockPlan,
    rows: &[SpireIndexLeafSnapshotRow],
    parent_object_version: u64,
    leaf_object_version: u64,
    published_at_micros: i64,
    retain_until_micros: i64,
    placement_write_evidence: Vec<SpirePublishPlacementWriteEvidence>,
    object_store: &mut SpireLocalObjectStore,
) -> Result<SpireReplacementEpochDraft, String> {
    let parent =
        load_selected_scheduled_replacement_parent_routing(snapshot, object_store, selected)?;
    let replacement_leaf_rows =
        collect_selected_scheduled_replacement_leaf_rows(snapshot, object_store, selected)?;
    build_local_selected_scheduled_merge_replacement_epoch_draft(
        snapshot,
        selected,
        &parent,
        rows,
        replacement_leaf_rows,
        parent_object_version,
        leaf_object_version,
        published_at_micros,
        retain_until_micros,
        placement_write_evidence,
        object_store,
    )
}

pub(super) unsafe fn publish_relation_scheduled_replacement_epoch(
    index_relation: pgrx::pg_sys::Relation,
    previous_epoch_manifest: SpireEpochManifest,
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    decision: &SpireLeafReplacementScheduleDecision,
    pid_plan: &SpireLeafReplacementPidPlan,
    publish_plan: &SpireScheduledReplacementPublishPlan,
    input: SpireRelationScheduledReplacementExecutionInput,
    object_store: &mut SpireRelationObjectStore,
) -> Result<SpireReplacementEpochDraft, String> {
    if &previous_epoch_manifest != snapshot.epoch_manifest {
        return Err(format!(
            "ec_spire scheduled replacement publish previous epoch manifest mismatch: got {}, expected {}",
            previous_epoch_manifest.epoch, snapshot.epoch_manifest.epoch
        ));
    }
    validate_relation_scheduled_replacement_execution_publish_plan(
        publish_plan,
        pid_plan,
        decision,
        &input,
    )?;
    validate_scheduled_replacement_execution_snapshot(snapshot, decision, publish_plan)?;
    let replacement_object_placements = unsafe {
        write_relation_scheduled_replacement_objects(
            input.epoch,
            &input.replacement_parent,
            decision,
            &input.replacement_children,
            input.leaf_object_version,
            input.leaf_inputs,
            object_store,
        )?
    };
    validate_scheduled_replacement_pid_plan_output(
        decision,
        pid_plan,
        &replacement_object_placements,
        pid_plan.next_pid,
    )?;
    let placement_directory = replacement_placement_directory_from_object_placements(
        snapshot,
        object_store,
        input.epoch,
        decision.replaced_parent_pid,
        decision.affected_leaf_pids.clone(),
        replacement_object_placements.clone(),
    )?;
    let placement_write_evidence =
        unsafe { write_placement_entries_to_relation(index_relation, &placement_directory)? };
    let draft = build_scheduled_replacement_epoch_draft_from_object_placements(
        snapshot,
        object_store,
        decision,
        SpireScheduledReplacementEpochObjectPlacementInput {
            epoch: input.epoch,
            published_at_micros: input.published_at_micros,
            retain_until_micros: input.retain_until_micros,
            consistency_mode: input.consistency_mode,
            replacement_object_placements,
            placement_write_evidence,
            next_pid: pid_plan.next_pid,
            next_local_vec_seq: input.next_local_vec_seq,
        },
    )?;
    unsafe {
        publish_replacement_epoch_to_relation(
            index_relation,
            previous_epoch_manifest,
            draft.publish_input(),
        )?;
    }
    Ok(draft)
}

pub(super) unsafe fn publish_relation_selected_scheduled_replacement_epoch(
    index_relation: pgrx::pg_sys::Relation,
    previous_epoch_manifest: SpireEpochManifest,
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    selected: &SpireSelectedScheduledReplacementPublishLockPlan,
    input: SpireRelationScheduledReplacementExecutionInput,
    object_store: &mut SpireRelationObjectStore,
) -> Result<SpireReplacementEpochDraft, String> {
    validate_relation_selected_scheduled_replacement_publish_inputs(
        &previous_epoch_manifest,
        snapshot,
        selected,
        &input,
    )?;
    unsafe {
        publish_relation_scheduled_replacement_epoch(
            index_relation,
            previous_epoch_manifest,
            snapshot,
            &selected.decision,
            &selected.lock_plan.pid_plan,
            &selected.lock_plan.publish_plan,
            input,
            object_store,
        )
    }
}

pub(super) unsafe fn publish_relation_replacement_epoch_from_object_placements(
    index_relation: pgrx::pg_sys::Relation,
    previous_epoch_manifest: SpireEpochManifest,
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    object_store: &impl SpireObjectReader,
    input: SpireRelationReplacementEpochObjectPlacementInput,
) -> Result<SpireReplacementEpochDraft, String> {
    if &previous_epoch_manifest != snapshot.epoch_manifest {
        return Err(format!(
            "ec_spire replacement publish previous epoch manifest mismatch: got {}, expected {}",
            previous_epoch_manifest.epoch, snapshot.epoch_manifest.epoch
        ));
    }
    let placement_directory = replacement_placement_directory_from_object_placements(
        snapshot,
        object_store,
        input.epoch,
        input.replaced_parent_pid,
        input.affected_leaf_pids,
        input.replacement_object_placements,
    )?;
    let placement_write_evidence =
        unsafe { write_placement_entries_to_relation(index_relation, &placement_directory)? };
    let draft = build_replacement_epoch_draft(SpireReplacementEpochInput {
        epoch: input.epoch,
        published_at_micros: input.published_at_micros,
        retain_until_micros: input.retain_until_micros,
        consistency_mode: input.consistency_mode,
        placement_directory,
        placement_write_evidence,
        next_pid: input.next_pid,
        next_local_vec_seq: input.next_local_vec_seq,
    })?;
    unsafe {
        publish_replacement_epoch_to_relation(
            index_relation,
            previous_epoch_manifest,
            draft.publish_input(),
        )?;
    }
    Ok(draft)
}

fn replacement_placement_directory_from_object_placements(
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    object_store: &impl SpireObjectReader,
    epoch: u64,
    replaced_parent_pid: u64,
    affected_leaf_pids: Vec<u64>,
    replacement_object_placements: SpireReplacementObjectPlacements,
) -> Result<SpirePlacementDirectory, String> {
    plan_replacement_epoch_placement_directory(
        snapshot,
        object_store,
        epoch,
        replaced_parent_pid,
        replacement_object_placements.parent_placement,
        &affected_leaf_pids,
        replacement_object_placements.leaf_placements,
    )
}

pub(super) fn validate_replacement_leaf_object_inputs(
    replacement_children: &[SpireRoutingReplacementChild],
    leaf_inputs: &[SpireReplacementLeafObjectInput],
) -> Result<(), String> {
    if replacement_children.is_empty() {
        return Err("ec_spire replacement leaf object inputs require children".to_owned());
    }
    if replacement_children.len() != leaf_inputs.len() {
        return Err(format!(
            "ec_spire replacement leaf object input count {} does not match replacement child count {}",
            leaf_inputs.len(),
            replacement_children.len()
        ));
    }

    let mut child_pids = HashSet::new();
    for child in replacement_children {
        if child.child_pid == 0 {
            return Err("ec_spire replacement child pid 0 is invalid".to_owned());
        }
        if !child_pids.insert(child.child_pid) {
            return Err("ec_spire replacement child pids must be unique".to_owned());
        }
    }

    let mut input_pids = HashSet::new();
    let mut vec_ids = HashSet::new();
    for input in leaf_inputs {
        if input.pid == 0 {
            return Err("ec_spire replacement leaf object input pid 0 is invalid".to_owned());
        }
        if !input_pids.insert(input.pid) {
            return Err("ec_spire replacement leaf object input pids must be unique".to_owned());
        }
        if !child_pids.contains(&input.pid) {
            return Err(format!(
                "ec_spire replacement leaf object input pid {} has no replacement routing child",
                input.pid
            ));
        }
        for row in &input.rows {
            if !is_visible_primary_assignment(row) {
                return Err(format!(
                    "ec_spire replacement leaf object input pid {} contains a non-visible-primary row",
                    input.pid
                ));
            }
            if row.flags & SPIRE_ASSIGNMENT_FLAG_DELTA_INSERT != 0 {
                return Err(format!(
                    "ec_spire replacement leaf object input pid {} must not contain delta-insert rows",
                    input.pid
                ));
            }
            if !vec_ids.insert(row.vec_id.clone()) {
                return Err(
                    "ec_spire replacement leaf object inputs contain duplicate vec_id rows"
                        .to_owned(),
                );
            }
        }
    }

    for child_pid in child_pids {
        if !input_pids.contains(&child_pid) {
            return Err(format!(
                "ec_spire replacement routing child pid {child_pid} has no leaf object input"
            ));
        }
    }
    Ok(())
}

pub(super) fn write_local_replacement_objects(
    epoch: u64,
    replacement_parent: &SpireRoutingPartitionObject,
    replacement_children: &[SpireRoutingReplacementChild],
    leaf_object_version: u64,
    leaf_inputs: Vec<SpireReplacementLeafObjectInput>,
    object_store: &mut SpireLocalObjectStore,
) -> Result<SpireReplacementObjectPlacements, String> {
    write_replacement_objects_with_writer(
        epoch,
        replacement_parent,
        replacement_children,
        leaf_object_version,
        leaf_inputs,
        object_store,
    )
}

pub(super) unsafe fn write_relation_replacement_objects(
    epoch: u64,
    replacement_parent: &SpireRoutingPartitionObject,
    replacement_children: &[SpireRoutingReplacementChild],
    leaf_object_version: u64,
    leaf_inputs: Vec<SpireReplacementLeafObjectInput>,
    object_store: &mut SpireRelationObjectStore,
) -> Result<SpireReplacementObjectPlacements, String> {
    write_replacement_objects_with_writer(
        epoch,
        replacement_parent,
        replacement_children,
        leaf_object_version,
        leaf_inputs,
        object_store,
    )
}

pub(super) fn write_local_scheduled_replacement_objects(
    epoch: u64,
    replacement_parent: &SpireRoutingPartitionObject,
    decision: &SpireLeafReplacementScheduleDecision,
    replacement_children: &[SpireRoutingReplacementChild],
    leaf_object_version: u64,
    leaf_inputs: Vec<SpireReplacementLeafObjectInput>,
    object_store: &mut SpireLocalObjectStore,
) -> Result<SpireReplacementObjectPlacements, String> {
    write_scheduled_replacement_objects_with_writer(
        epoch,
        replacement_parent,
        decision,
        replacement_children,
        leaf_object_version,
        leaf_inputs,
        object_store,
    )
}

pub(super) unsafe fn write_relation_scheduled_replacement_objects(
    epoch: u64,
    replacement_parent: &SpireRoutingPartitionObject,
    decision: &SpireLeafReplacementScheduleDecision,
    replacement_children: &[SpireRoutingReplacementChild],
    leaf_object_version: u64,
    leaf_inputs: Vec<SpireReplacementLeafObjectInput>,
    object_store: &mut SpireRelationObjectStore,
) -> Result<SpireReplacementObjectPlacements, String> {
    write_scheduled_replacement_objects_with_writer(
        epoch,
        replacement_parent,
        decision,
        replacement_children,
        leaf_object_version,
        leaf_inputs,
        object_store,
    )
}

fn write_scheduled_replacement_objects_with_writer(
    epoch: u64,
    replacement_parent: &SpireRoutingPartitionObject,
    decision: &SpireLeafReplacementScheduleDecision,
    replacement_children: &[SpireRoutingReplacementChild],
    leaf_object_version: u64,
    leaf_inputs: Vec<SpireReplacementLeafObjectInput>,
    object_store: &mut impl SpireReplacementObjectWriter,
) -> Result<SpireReplacementObjectPlacements, String> {
    validate_leaf_replacement_schedule_decision_shape(decision)?;
    let expected_epoch = decision
        .active_epoch
        .checked_add(1)
        .ok_or_else(|| "ec_spire scheduled replacement object writer epoch overflow".to_owned())?;
    if epoch != expected_epoch {
        return Err(format!(
            "ec_spire scheduled replacement object writer epoch {epoch} must be the immediate successor of active epoch {}",
            decision.active_epoch
        ));
    }
    if replacement_parent.header.pid != decision.replaced_parent_pid {
        return Err(format!(
            "ec_spire scheduled replacement object writer parent pid {} does not match decision parent pid {}",
            replacement_parent.header.pid, decision.replaced_parent_pid
        ));
    }
    if replacement_children.len() != decision.replacement_leaf_count {
        return Err(format!(
            "ec_spire scheduled replacement object writer child count {} does not match decision replacement count {}",
            replacement_children.len(),
            decision.replacement_leaf_count
        ));
    }
    write_replacement_objects_with_writer(
        epoch,
        replacement_parent,
        replacement_children,
        leaf_object_version,
        leaf_inputs,
        object_store,
    )
}

fn write_replacement_objects_with_writer(
    epoch: u64,
    replacement_parent: &SpireRoutingPartitionObject,
    replacement_children: &[SpireRoutingReplacementChild],
    leaf_object_version: u64,
    leaf_inputs: Vec<SpireReplacementLeafObjectInput>,
    object_store: &mut impl SpireReplacementObjectWriter,
) -> Result<SpireReplacementObjectPlacements, String> {
    if epoch == 0 {
        return Err("ec_spire replacement object epoch 0 is invalid".to_owned());
    }
    if leaf_object_version == 0 {
        return Err("ec_spire replacement leaf object_version 0 is invalid".to_owned());
    }
    match replacement_parent.header.kind {
        SpirePartitionObjectKind::Root | SpirePartitionObjectKind::Internal => {}
        other => {
            return Err(format!(
                "ec_spire replacement parent must be Root or Internal, got {other:?}"
            ));
        }
    }
    validate_replacement_leaf_object_inputs(replacement_children, &leaf_inputs)?;

    let parent_placement =
        object_store.write_replacement_parent_object(epoch, replacement_parent)?;
    let inputs_by_pid = leaf_inputs
        .into_iter()
        .map(|input| (input.pid, input))
        .collect::<HashMap<_, _>>();
    let mut leaf_placements = Vec::with_capacity(replacement_children.len());
    for child in replacement_children {
        let input = inputs_by_pid.get(&child.child_pid).ok_or_else(|| {
            format!(
                "ec_spire replacement child pid {} has no leaf input",
                child.child_pid
            )
        })?;
        leaf_placements.push(object_store.write_replacement_leaf_object_v2_from_rows(
            epoch,
            input.pid,
            leaf_object_version,
            replacement_parent.header.pid,
            &input.rows,
        )?);
    }

    Ok(SpireReplacementObjectPlacements {
        parent_placement,
        leaf_placements,
    })
}

fn manifest_locators_for_validation() -> SpirePublishedManifestLocators {
    SpirePublishedManifestLocators {
        epoch_manifest_tid: ItemPointer {
            block_number: 1,
            offset_number: 1,
        },
        object_manifest_tid: ItemPointer {
            block_number: 1,
            offset_number: 2,
        },
        placement_directory_tid: ItemPointer {
            block_number: 1,
            offset_number: 3,
        },
    }
}

fn validate_affected_leaf_pids(affected_leaf_pids: &[u64]) -> Result<(), String> {
    if affected_leaf_pids.is_empty() {
        return Err("ec_spire leaf replacement requires at least one affected leaf pid".to_owned());
    }
    let mut seen = HashSet::new();
    for pid in affected_leaf_pids {
        if *pid == 0 {
            return Err("ec_spire leaf replacement affected pid 0 is invalid".to_owned());
        }
        if !seen.insert(*pid) {
            return Err("ec_spire leaf replacement affected leaf pids must be unique".to_owned());
        }
    }
    Ok(())
}

fn validate_replacement_placement(
    epoch: u64,
    expected_pid: u64,
    label: &str,
    placement: &SpirePlacementEntry,
) -> Result<(), String> {
    if placement.epoch != epoch {
        return Err(format!(
            "ec_spire {label} placement epoch {} does not match replacement epoch {epoch}",
            placement.epoch
        ));
    }
    if placement.pid != expected_pid {
        return Err(format!(
            "ec_spire {label} placement pid {} does not match expected pid {expected_pid}",
            placement.pid
        ));
    }
    if placement.state != SpirePlacementState::Available {
        return Err(format!(
            "ec_spire {label} placement must be available, got {:?}",
            placement.state
        ));
    }
    placement.encode()?;
    Ok(())
}

fn validate_replacement_routing_children(
    parent: &SpireRoutingPartitionObject,
    affected_child_pids: &[u64],
    replacement_children: &[SpireRoutingReplacementChild],
) -> Result<(), String> {
    if replacement_children.is_empty() {
        return Err(
            "ec_spire routing replacement requires at least one replacement child".to_owned(),
        );
    }
    let affected: HashSet<u64> = affected_child_pids.iter().copied().collect();
    let mut parent_child_pids = HashSet::new();
    let mut found_affected = HashSet::new();
    for child in parent.children() {
        if !parent_child_pids.insert(child.child_pid) {
            return Err(
                "ec_spire routing replacement parent contains duplicate child pids".to_owned(),
            );
        }
        if affected.contains(&child.child_pid) {
            found_affected.insert(child.child_pid);
        }
    }
    if found_affected.len() != affected.len() {
        let mut missing = affected
            .difference(&found_affected)
            .copied()
            .collect::<Vec<_>>();
        missing.sort_unstable();
        return Err(format!(
            "ec_spire routing replacement affected child pids are missing from parent: {missing:?}"
        ));
    }

    let mut replacement_pids = HashSet::new();
    let dimensions = usize::from(parent.dimensions);
    for replacement in replacement_children {
        if replacement.child_pid == 0 {
            return Err("ec_spire routing replacement child pid 0 is invalid".to_owned());
        }
        if !replacement_pids.insert(replacement.child_pid) {
            return Err("ec_spire routing replacement child pids must be unique".to_owned());
        }
        if !affected.contains(&replacement.child_pid)
            && parent_child_pids.contains(&replacement.child_pid)
        {
            return Err(format!(
                "ec_spire routing replacement child pid {} already exists outside the affected set",
                replacement.child_pid
            ));
        }
        if replacement.centroid.len() != dimensions {
            return Err(format!(
                "ec_spire routing replacement child pid {} centroid dimensions mismatch: got {}, expected {dimensions}",
                replacement.child_pid,
                replacement.centroid.len()
            ));
        }
        if replacement
            .centroid
            .iter()
            .any(|component| !component.is_finite())
        {
            return Err(format!(
                "ec_spire routing replacement child pid {} centroid must be finite",
                replacement.child_pid
            ));
        }
    }
    Ok(())
}

fn append_replacement_routing_children(
    children: &mut Vec<SpireRoutingChildEntry>,
    replacement_children: &[SpireRoutingReplacementChild],
) -> Result<(), String> {
    for replacement in replacement_children {
        let centroid_index = u32::try_from(children.len())
            .map_err(|_| "ec_spire routing replacement child count exceeds u32".to_owned())?;
        children.push(SpireRoutingChildEntry {
            centroid_index,
            child_pid: replacement.child_pid,
            centroid: replacement.centroid.clone(),
        });
    }
    Ok(())
}

fn allocate_replacement_pids(
    pid_allocator: &mut SpirePidAllocator,
    count: usize,
) -> Result<Vec<u64>, String> {
    let mut pids = Vec::with_capacity(count);
    for _ in 0..count {
        pids.push(pid_allocator.allocate()?);
    }
    Ok(pids)
}

fn push_visible_replacement_row(
    rows_by_base_pid: &mut HashMap<u64, Vec<SpireLeafAssignmentRow>>,
    visible_vec_ids: &mut HashSet<SpireVecId>,
    base_pid: u64,
    mut assignment: SpireLeafAssignmentRow,
    deleted: Option<&HashSet<SpireVecId>>,
) -> Result<(), String> {
    if !is_visible_primary_assignment(&assignment) {
        return Ok(());
    }
    if deleted.is_some_and(|deleted| deleted.contains(&assignment.vec_id)) {
        return Ok(());
    }
    assignment.flags &= !SPIRE_ASSIGNMENT_FLAG_DELTA_INSERT;
    if !visible_vec_ids.insert(assignment.vec_id.clone()) {
        return Err(
            "ec_spire replacement leaf rows contain duplicate visible vec_id assignments"
                .to_owned(),
        );
    }
    rows_by_base_pid
        .entry(base_pid)
        .or_default()
        .push(assignment);
    Ok(())
}

fn collect_delete_vec_ids_by_base_pid(
    snapshot: &SpireValidatedEpochSnapshot<'_>,
    object_store: &impl SpireObjectReader,
) -> Result<HashMap<u64, HashSet<SpireVecId>>, String> {
    let mut deleted_by_base_pid: HashMap<u64, HashSet<SpireVecId>> = HashMap::new();
    for manifest_entry in &snapshot.object_manifest().entries {
        let lookup =
            snapshot.require_lookup(manifest_entry.pid, "replacement delete assignment")?;
        let placement = lookup.placement;
        let header = object_store.read_object_header(placement)?;
        if header.kind != SpirePartitionObjectKind::Delta {
            continue;
        }
        let delta = object_store.read_delta_object(placement)?;
        for assignment in delta.assignments {
            if is_delete_delta_assignment(&assignment) {
                deleted_by_base_pid
                    .entry(header.parent_pid)
                    .or_default()
                    .insert(assignment.vec_id);
            }
        }
    }
    Ok(deleted_by_base_pid)
}

fn read_leaf_assignments_for_replacement(
    object_store: &impl SpireObjectReader,
    placement: &SpirePlacementEntry,
) -> Result<Vec<SpireLeafAssignmentRow>, String> {
    match object_store.read_leaf_object(placement) {
        Ok(object) => Ok(object.assignments),
        Err(v1_error) => object_store
            .read_leaf_object_v2(placement)
            .map_err(|v2_error| {
                format!(
                    "ec_spire replacement leaf rows could not read leaf pid {} as V1 or V2: V1 error: {v1_error}; V2 error: {v2_error}",
                    placement.pid
                )
            })?
            .assignment_rows(),
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
    let base_snapshot = SpireValidatedEpochSnapshot::from_snapshot(*base_snapshot)?;
    validate_delta_base_snapshot_placements_available(&base_snapshot)?;
    if input.epoch <= base_snapshot.epoch_manifest().epoch {
        return Err(format!(
            "ec_spire delta epoch {} must be newer than base epoch {}",
            input.epoch,
            base_snapshot.epoch_manifest().epoch
        ));
    }
    let base_lookup = base_snapshot.require_lookup(input.base_pid, "delta epoch base pid")?;
    let base_header = object_store.read_object_header(base_lookup.placement)?;
    if base_header.kind != SpirePartitionObjectKind::Leaf {
        return Err(format!(
            "ec_spire delta epoch base_pid {} must reference a leaf partition object",
            input.base_pid
        ));
    }
    let epoch = input.epoch;
    let carried_manifest_entries = base_snapshot
        .object_manifest()
        .entries
        .iter()
        .cloned()
        .map(|mut entry| {
            entry.epoch = epoch;
            entry
        })
        .collect();
    let carried_placement_entries = base_snapshot
        .placement_directory()
        .entries
        .iter()
        .cloned()
        .map(|mut entry| {
            entry.epoch = epoch;
            entry
        })
        .collect();
    let observed_vec_ids = collect_snapshot_assignment_vec_ids(&base_snapshot, object_store)?;
    let visible_rows =
        collect_validated_snapshot_visible_primary_rows(&base_snapshot, object_store)?;
    validate_delete_delta_targets(&input.delete_assignments, &visible_rows)?;

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
    SpireValidatedEpochSnapshot::new(
        &draft.epoch_manifest,
        &draft.object_manifest,
        &draft.placement_directory,
    )?;

    *pid_allocator = pid_cursor;
    *local_vec_id_allocator = local_vec_id_cursor;
    Ok(draft)
}

fn validate_delta_base_snapshot_placements_available(
    snapshot: &SpireValidatedEpochSnapshot<'_>,
) -> Result<(), String> {
    for placement in &snapshot.placement_directory().entries {
        if placement.state != SpirePlacementState::Available {
            return Err(format!(
                "ec_spire delta epoch base snapshot requires available placement for pid {}: got {:?}",
                placement.pid, placement.state
            ));
        }
    }
    Ok(())
}

fn collect_snapshot_assignment_vec_ids(
    snapshot: &SpireValidatedEpochSnapshot<'_>,
    object_store: &impl SpireObjectReader,
) -> Result<Vec<SpireVecId>, String> {
    let mut vec_ids = Vec::new();
    for manifest_entry in &snapshot.object_manifest().entries {
        let placement = snapshot
            .require_lookup(
                manifest_entry.pid,
                "delta draft assignment vec_id collection",
            )?
            .placement;
        let header = object_store.read_object_header(placement)?;
        match header.kind {
            SpirePartitionObjectKind::Leaf => {
                let assignments = match object_store.read_leaf_object(placement) {
                    Ok(object) => object.assignments,
                    Err(v1_error) => object_store
                        .read_leaf_object_v2(placement)
                        .map_err(|v2_error| {
                            format!(
                                "ec_spire delta draft could not read leaf pid {} as V1 or V2: V1 error: {v1_error}; V2 error: {v2_error}",
                                placement.pid
                            )
                        })?
                        .assignment_rows()?,
                };
                vec_ids.extend(assignments.into_iter().map(|assignment| assignment.vec_id));
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
    visible_rows: &[SpireLeafScanRow],
) -> Result<(), String> {
    let mut visible_targets = HashMap::new();
    for row in visible_rows {
        if visible_targets
            .insert(row.assignment.vec_id.clone(), row.assignment.heap_tid)
            .is_some()
        {
            return Err(
                "ec_spire visible snapshot contains duplicate vec_id assignments".to_owned(),
            );
        }
    }

    let mut seen_deletes = HashSet::new();
    for assignment in delete_assignments {
        if !seen_deletes.insert(assignment.vec_id.clone()) {
            return Err(
                "ec_spire delete delta vec_id appears more than once in the draft".to_owned(),
            );
        }
        let Some(heap_tid) = visible_targets.get(&assignment.vec_id) else {
            return Err(
                "ec_spire delete delta vec_id is not present in the base snapshot".to_owned(),
            );
        };
        if *heap_tid != assignment.heap_tid {
            return Err(
                "ec_spire delete delta heap_tid does not match the visible base row".to_owned(),
            );
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::SpireIndexLeafSnapshotRow;
    use super::{
        build_delta_epoch_draft, build_delta_epoch_draft_from_snapshot,
        build_local_scheduled_merge_replacement_execution_input,
        build_local_scheduled_merge_replacement_execution_parts,
        build_local_scheduled_replacement_epoch_draft,
        build_local_scheduled_replacement_execution_input_from_publish_plan,
        build_local_scheduled_split_replacement_execution_input,
        build_local_scheduled_split_replacement_execution_parts,
        build_local_selected_scheduled_merge_replacement_epoch_draft,
        build_local_selected_scheduled_merge_replacement_epoch_draft_from_snapshot,
        build_local_selected_scheduled_merge_replacement_execution_input,
        build_local_selected_scheduled_replacement_epoch_draft,
        build_local_selected_scheduled_split_replacement_epoch_draft,
        build_local_selected_scheduled_split_replacement_execution_input,
        build_merge_replacement_leaf_object_input,
        build_relation_scheduled_merge_replacement_execution_input,
        build_relation_scheduled_merge_replacement_execution_parts,
        build_relation_scheduled_replacement_execution_input_from_publish_plan,
        build_relation_scheduled_split_replacement_execution_input,
        build_relation_scheduled_split_replacement_execution_parts,
        build_relation_selected_scheduled_merge_replacement_execution_input,
        build_relation_selected_scheduled_split_replacement_execution_input,
        build_replacement_epoch_draft, build_replacement_epoch_draft_from_object_placements,
        build_scheduled_merge_replacement_centroids,
        build_scheduled_merge_replacement_routing_parts,
        build_scheduled_replacement_epoch_draft_from_object_placements,
        build_scheduled_routing_replacement_children,
        build_scheduled_split_replacement_routing_parts,
        build_split_replacement_leaf_object_inputs, choose_leaf_replacement_schedule,
        choose_scheduled_replacement_publish_lock_plan, collect_replacement_leaf_rows,
        collect_selected_scheduled_replacement_leaf_rows,
        load_scheduled_replacement_parent_routing,
        load_selected_scheduled_replacement_parent_routing, plan_leaf_replacement_pids,
        plan_rechecked_scheduled_replacement_publish_lock,
        plan_replacement_epoch_placement_directory, plan_scheduled_leaf_replacement_pids,
        plan_scheduled_replacement_publish_epoch, plan_scheduled_replacement_publish_lock,
        recheck_leaf_replacement_schedule_decision, rewrite_routing_partition_for_leaf_replacement,
        rewrite_scheduled_replacement_parent_routing,
        validate_local_scheduled_replacement_execution_publish_plan,
        validate_local_selected_scheduled_replacement_draft_inputs,
        validate_local_selected_scheduled_replacement_execution_publish_plan,
        validate_relation_scheduled_replacement_execution_publish_plan,
        validate_relation_selected_scheduled_replacement_execution_publish_plan,
        validate_relation_selected_scheduled_replacement_publish_inputs,
        validate_replacement_leaf_object_inputs, validate_scheduled_replacement_pid_plan_output,
        validate_selected_scheduled_replacement_execution_snapshot,
        write_local_replacement_objects, write_local_scheduled_replacement_objects,
        SpireDeltaEpochInput, SpireLeafReplacementMode, SpireLeafReplacementPidPlan,
        SpireLeafReplacementScheduleDecision, SpireLeafReplacementScheduleMode,
        SpireLocalScheduledReplacementExecutionInput, SpireLocalScheduledReplacementExecutionParts,
        SpireRelationScheduledReplacementExecutionInput,
        SpireRelationScheduledReplacementExecutionParts, SpireReplacementEpochInput,
        SpireReplacementEpochObjectPlacementInput, SpireReplacementLeafObjectInput,
        SpireReplacementLeafRows, SpireReplacementObjectPlacements, SpireRoutingReplacementChild,
        SpireScheduledReplacementEpochObjectPlacementInput,
        SpireScheduledReplacementPublishLockPlan, SpireScheduledReplacementPublishPlan,
        SpireSelectedScheduledReplacementPublishLockPlan,
    };
    use crate::am::ec_spire::assign::{
        SpireDeleteDeltaInput, SpireLeafAssignmentInput, SpireLocalVecIdAllocator,
        SpirePidAllocator,
    };
    use crate::am::ec_spire::build::{
        build_single_level_leaf_epoch_draft, SpirePublishPlacementWriteEvidence,
        SpirePublishedManifestLocators, SpireSingleLevelBuildInput,
    };
    use crate::am::ec_spire::meta::{
        SpireConsistencyMode, SpireEpochManifest, SpireEpochState, SpireManifestEntry,
        SpireObjectManifest, SpirePlacementDirectory, SpirePlacementEntry, SpirePlacementState,
        SpirePublishedEpochSnapshot, SpireRootControlState,
    };
    use crate::am::ec_spire::storage::{
        SpireDeltaPartitionObject, SpireLeafAssignmentRow, SpireLeafPartitionObject,
        SpireLocalObjectStore, SpireRoutingChildEntry, SpireRoutingPartitionObject, SpireVecId,
        SPIRE_ASSIGNMENT_FLAG_DELTA_DELETE, SPIRE_ASSIGNMENT_FLAG_DELTA_INSERT,
        SPIRE_ASSIGNMENT_FLAG_PRIMARY, SPIRE_ASSIGNMENT_FLAG_STALE_LOCATOR,
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

    fn routing_child(
        centroid_index: u32,
        child_pid: u64,
        centroid: Vec<f32>,
    ) -> SpireRoutingChildEntry {
        SpireRoutingChildEntry {
            centroid_index,
            child_pid,
            centroid,
        }
    }

    fn root_routing_object() -> SpireRoutingPartitionObject {
        SpireRoutingPartitionObject::root(
            1,
            3,
            2,
            vec![
                routing_child(0, 11, vec![1.0, 0.0]),
                routing_child(1, 12, vec![0.0, 1.0]),
                routing_child(2, 13, vec![-1.0, 0.0]),
            ],
        )
        .unwrap()
    }

    fn replacement_child(child_pid: u64, centroid: Vec<f32>) -> SpireRoutingReplacementChild {
        SpireRoutingReplacementChild {
            child_pid,
            centroid,
        }
    }

    fn scheduled_split_decision(active_epoch: u64) -> SpireLeafReplacementScheduleDecision {
        SpireLeafReplacementScheduleDecision {
            mode: SpireLeafReplacementScheduleMode::Split,
            active_epoch,
            replaced_parent_pid: 1,
            affected_leaf_pids: vec![12],
            replacement_leaf_count: 2,
            reason: "test_split",
        }
    }

    fn scheduled_split_replacement_children() -> Vec<SpireRoutingReplacementChild> {
        vec![
            replacement_child(21, vec![0.5, 0.5]),
            replacement_child(22, vec![-0.5, 0.5]),
        ]
    }

    fn scheduled_rewritten_parent_for_decision(
        decision: &SpireLeafReplacementScheduleDecision,
        replacement_children: Vec<SpireRoutingReplacementChild>,
    ) -> SpireRoutingPartitionObject {
        rewrite_scheduled_replacement_parent_routing(
            &root_routing_object(),
            decision,
            replacement_children,
            4,
        )
        .unwrap()
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

    fn delta_insert_row(
        vec_seq: u64,
        block_number: u32,
        offset_number: u16,
    ) -> SpireLeafAssignmentRow {
        let mut row = primary_row(vec_seq, block_number, offset_number);
        row.flags |= SPIRE_ASSIGNMENT_FLAG_DELTA_INSERT;
        row
    }

    fn manifest_entry_for(
        placement: &crate::am::ec_spire::meta::SpirePlacementEntry,
    ) -> SpireManifestEntry {
        SpireManifestEntry {
            epoch: placement.epoch,
            pid: placement.pid,
            object_version: placement.object_version,
            placement_tid: placement.object_tid,
        }
    }

    fn manifest_locators() -> SpirePublishedManifestLocators {
        SpirePublishedManifestLocators {
            epoch_manifest_tid: tid(90, 1),
            object_manifest_tid: tid(90, 2),
            placement_directory_tid: tid(90, 3),
        }
    }

    fn placement_write_evidence_for_pids(pids: &[u64]) -> Vec<SpirePublishPlacementWriteEvidence> {
        pids.iter()
            .enumerate()
            .map(|(index, pid)| SpirePublishPlacementWriteEvidence {
                pid: *pid,
                placement_tid: tid(90, u16::try_from(index + 1).unwrap()),
            })
            .collect()
    }

    fn leaf_snapshot_row(
        leaf_pid: u64,
        parent_pid: u64,
        effective_assignment_count: u64,
        split_recommended: bool,
        merge_recommended: bool,
    ) -> SpireIndexLeafSnapshotRow {
        SpireIndexLeafSnapshotRow {
            active_epoch: 7,
            leaf_pid,
            parent_pid,
            object_version: 1,
            node_id: 0,
            local_store_id: 12345,
            placement_state: "available",
            base_assignment_count: effective_assignment_count,
            delta_object_count: 0,
            delta_insert_assignment_count: 0,
            delta_delete_assignment_count: 0,
            effective_assignment_count,
            split_assignment_threshold: 32,
            merge_assignment_threshold: 2,
            split_recommended,
            merge_recommended,
            maintenance_action: "none",
            maintenance_reason: "test",
            leaf_object_bytes: 128,
            delta_object_bytes: 0,
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
        let mut expected_delta = draft.delta_object.clone();
        expected_delta.header.published_epoch_backref = draft.epoch_manifest.epoch;

        assert_eq!(stored_delta, expected_delta);
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
    fn replacement_pid_plan_allocates_split_children_from_pid_cursor() {
        let mut pid_allocator = SpirePidAllocator::new(3).unwrap();

        let plan = plan_leaf_replacement_pids(
            SpireLeafReplacementMode::Split,
            &[10],
            2,
            &mut pid_allocator,
        )
        .unwrap();

        assert_eq!(plan.replacement_pids, vec![11, 12]);
        assert!(!plan.reuses_existing_pid);
        assert_eq!(plan.next_pid, 13);
        assert_eq!(pid_allocator.next_pid(), 13);
    }

    #[test]
    fn replacement_pid_plan_allocates_merge_survivor_from_pid_cursor() {
        let mut pid_allocator = SpirePidAllocator::new(20).unwrap();

        let plan = plan_leaf_replacement_pids(
            SpireLeafReplacementMode::Merge,
            &[4, 5],
            1,
            &mut pid_allocator,
        )
        .unwrap();

        assert_eq!(plan.replacement_pids, vec![20]);
        assert!(!plan.reuses_existing_pid);
        assert_eq!(plan.next_pid, 21);
        assert_eq!(pid_allocator.next_pid(), 21);
    }

    #[test]
    fn replacement_pid_plan_rebalance_reuses_pid_only_for_byte_equal_centroid() {
        let mut pid_allocator = SpirePidAllocator::new(20).unwrap();

        let plan = plan_leaf_replacement_pids(
            SpireLeafReplacementMode::Rebalance {
                parent_centroid_byte_equal: true,
            },
            &[4],
            1,
            &mut pid_allocator,
        )
        .unwrap();

        assert_eq!(plan.replacement_pids, vec![4]);
        assert!(plan.reuses_existing_pid);
        assert_eq!(plan.next_pid, 20);
        assert_eq!(pid_allocator.next_pid(), 20);

        let err = plan_leaf_replacement_pids(
            SpireLeafReplacementMode::Rebalance {
                parent_centroid_byte_equal: false,
            },
            &[4],
            1,
            &mut pid_allocator,
        )
        .unwrap_err();
        assert!(err.contains("parent routing centroid is byte-equal"));
        assert_eq!(pid_allocator.next_pid(), 20);
    }

    #[test]
    fn replacement_scheduler_prefers_largest_split_candidate() {
        let rows = vec![
            leaf_snapshot_row(11, 1, 1, false, true),
            leaf_snapshot_row(12, 1, 80, true, false),
            leaf_snapshot_row(13, 1, 40, true, false),
        ];

        let decision = choose_leaf_replacement_schedule(&rows).unwrap().unwrap();

        assert_eq!(decision.mode, SpireLeafReplacementScheduleMode::Split);
        assert_eq!(decision.active_epoch, 7);
        assert_eq!(decision.replaced_parent_pid, 1);
        assert_eq!(decision.affected_leaf_pids, vec![12]);
        assert_eq!(decision.replacement_leaf_count, 2);
        assert_eq!(decision.reason, "largest_split_candidate");
    }

    #[test]
    fn replacement_scheduler_selects_sparsest_same_parent_merge_pair() {
        let rows = vec![
            leaf_snapshot_row(11, 1, 2, false, true),
            leaf_snapshot_row(12, 1, 1, false, true),
            leaf_snapshot_row(13, 2, 0, false, true),
            leaf_snapshot_row(14, 2, 10, false, true),
        ];

        let decision = choose_leaf_replacement_schedule(&rows).unwrap().unwrap();

        assert_eq!(decision.mode, SpireLeafReplacementScheduleMode::Merge);
        assert_eq!(decision.replaced_parent_pid, 1);
        assert_eq!(decision.affected_leaf_pids, vec![11, 12]);
        assert_eq!(decision.replacement_leaf_count, 1);
        assert_eq!(decision.reason, "sparsest_same_parent_merge_pair");
    }

    #[test]
    fn replacement_scheduler_rejects_ambiguous_or_cross_epoch_rows() {
        let mut ambiguous = leaf_snapshot_row(11, 1, 40, true, true);
        assert!(choose_leaf_replacement_schedule(&[ambiguous.clone()])
            .unwrap_err()
            .contains("cannot be both split and merge"));

        ambiguous.split_recommended = false;
        ambiguous.merge_recommended = true;
        ambiguous.parent_pid = 0;
        assert!(choose_leaf_replacement_schedule(&[ambiguous.clone()])
            .unwrap_err()
            .contains("parent_pid 0"));

        let mut newer = leaf_snapshot_row(12, 1, 1, false, true);
        newer.active_epoch = 8;
        assert!(choose_leaf_replacement_schedule(&[
            leaf_snapshot_row(11, 1, 1, false, true),
            newer
        ])
        .unwrap_err()
        .contains("multiple active epochs"));

        assert!(choose_leaf_replacement_schedule(&[
            leaf_snapshot_row(11, 1, 1, false, true),
            leaf_snapshot_row(11, 1, 2, false, true),
        ])
        .unwrap_err()
        .contains("duplicate row"));
    }

    #[test]
    fn scheduled_replacement_pid_plan_allocates_from_decision() {
        let mut pid_allocator = SpirePidAllocator::new(20).unwrap();
        let split_decision = SpireLeafReplacementScheduleDecision {
            mode: SpireLeafReplacementScheduleMode::Split,
            active_epoch: 7,
            replaced_parent_pid: 1,
            affected_leaf_pids: vec![12],
            replacement_leaf_count: 2,
            reason: "test_split",
        };

        let split_plan =
            plan_scheduled_leaf_replacement_pids(&split_decision, &mut pid_allocator).unwrap();

        assert_eq!(split_plan.replacement_pids, vec![20, 21]);
        assert_eq!(split_plan.next_pid, 22);
        assert_eq!(pid_allocator.next_pid(), 22);

        let merge_decision = SpireLeafReplacementScheduleDecision {
            mode: SpireLeafReplacementScheduleMode::Merge,
            active_epoch: 7,
            replaced_parent_pid: 1,
            affected_leaf_pids: vec![11, 13],
            replacement_leaf_count: 1,
            reason: "test_merge",
        };

        let merge_plan =
            plan_scheduled_leaf_replacement_pids(&merge_decision, &mut pid_allocator).unwrap();

        assert_eq!(merge_plan.replacement_pids, vec![22]);
        assert_eq!(merge_plan.next_pid, 23);
        assert_eq!(pid_allocator.next_pid(), 23);
    }

    #[test]
    fn scheduled_replacement_pid_plan_rejects_malformed_decision_without_advancing_cursor() {
        let mut pid_allocator = SpirePidAllocator::new(20).unwrap();
        let malformed = SpireLeafReplacementScheduleDecision {
            mode: SpireLeafReplacementScheduleMode::Merge,
            active_epoch: 7,
            replaced_parent_pid: 1,
            affected_leaf_pids: vec![11],
            replacement_leaf_count: 1,
            reason: "bad_merge",
        };

        assert!(
            plan_scheduled_leaf_replacement_pids(&malformed, &mut pid_allocator)
                .unwrap_err()
                .contains("merge decision requires")
        );
        assert_eq!(pid_allocator.next_pid(), 20);
    }

    #[test]
    fn replacement_scheduler_recheck_accepts_stable_decision() {
        let rows = vec![
            leaf_snapshot_row(11, 1, 2, false, true),
            leaf_snapshot_row(12, 1, 1, false, true),
        ];
        let decision = choose_leaf_replacement_schedule(&rows).unwrap().unwrap();

        recheck_leaf_replacement_schedule_decision(&rows, &decision).unwrap();
    }

    #[test]
    fn replacement_scheduler_recheck_rejects_changed_decision() {
        let rows = vec![
            leaf_snapshot_row(11, 1, 2, false, true),
            leaf_snapshot_row(12, 1, 1, false, true),
        ];
        let decision = choose_leaf_replacement_schedule(&rows).unwrap().unwrap();
        let changed = vec![leaf_snapshot_row(13, 1, 80, true, false)];

        assert!(
            recheck_leaf_replacement_schedule_decision(&changed, &decision)
                .unwrap_err()
                .contains("decision changed under publish lock")
        );
    }

    #[test]
    fn replacement_scheduler_recheck_rejects_no_longer_recommended_decision() {
        let rows = vec![
            leaf_snapshot_row(11, 1, 2, false, true),
            leaf_snapshot_row(12, 1, 1, false, true),
        ];
        let decision = choose_leaf_replacement_schedule(&rows).unwrap().unwrap();
        let quiet = vec![leaf_snapshot_row(11, 1, 10, false, false)];

        assert!(
            recheck_leaf_replacement_schedule_decision(&quiet, &decision)
                .unwrap_err()
                .contains("no longer recommended")
        );
    }

    #[test]
    fn scheduled_merge_replacement_centroid_weights_parent_child_centroids() {
        let decision = SpireLeafReplacementScheduleDecision {
            mode: SpireLeafReplacementScheduleMode::Merge,
            active_epoch: 7,
            replaced_parent_pid: 1,
            affected_leaf_pids: vec![11, 12],
            replacement_leaf_count: 1,
            reason: "test_merge",
        };
        let rows = vec![
            leaf_snapshot_row(11, 1, 3, false, true),
            leaf_snapshot_row(12, 1, 1, false, true),
        ];

        let centroids =
            build_scheduled_merge_replacement_centroids(&decision, &root_routing_object(), &rows)
                .unwrap();

        assert_eq!(centroids.len(), 1);
        assert!((centroids[0][0] - 0.9486833).abs() < 0.0001);
        assert!((centroids[0][1] - 0.31622776).abs() < 0.0001);
    }

    #[test]
    fn scheduled_merge_replacement_centroid_uses_equal_weight_for_empty_leaves() {
        let decision = SpireLeafReplacementScheduleDecision {
            mode: SpireLeafReplacementScheduleMode::Merge,
            active_epoch: 7,
            replaced_parent_pid: 1,
            affected_leaf_pids: vec![11, 12],
            replacement_leaf_count: 1,
            reason: "test_merge",
        };
        let rows = vec![
            leaf_snapshot_row(11, 1, 0, false, true),
            leaf_snapshot_row(12, 1, 0, false, true),
        ];

        let centroids =
            build_scheduled_merge_replacement_centroids(&decision, &root_routing_object(), &rows)
                .unwrap();

        assert_eq!(centroids.len(), 1);
        assert!((centroids[0][0] - std::f32::consts::FRAC_1_SQRT_2).abs() < 0.0001);
        assert!((centroids[0][1] - std::f32::consts::FRAC_1_SQRT_2).abs() < 0.0001);
    }

    #[test]
    fn scheduled_merge_replacement_centroid_rejects_missing_or_stale_inputs() {
        let decision = SpireLeafReplacementScheduleDecision {
            mode: SpireLeafReplacementScheduleMode::Merge,
            active_epoch: 7,
            replaced_parent_pid: 1,
            affected_leaf_pids: vec![11, 12],
            replacement_leaf_count: 1,
            reason: "test_merge",
        };

        assert!(build_scheduled_merge_replacement_centroids(
            &decision,
            &root_routing_object(),
            &[leaf_snapshot_row(11, 1, 1, false, true)],
        )
        .unwrap_err()
        .contains("missing snapshot row"));

        let stale_parent_rows = vec![
            leaf_snapshot_row(11, 1, 1, false, true),
            leaf_snapshot_row(12, 2, 1, false, true),
        ];
        assert!(build_scheduled_merge_replacement_centroids(
            &decision,
            &root_routing_object(),
            &stale_parent_rows,
        )
        .unwrap_err()
        .contains("row parent pid"));

        let duplicate_rows = vec![
            leaf_snapshot_row(11, 1, 1, false, true),
            leaf_snapshot_row(11, 1, 2, false, true),
            leaf_snapshot_row(12, 1, 1, false, true),
        ];
        assert!(build_scheduled_merge_replacement_centroids(
            &decision,
            &root_routing_object(),
            &duplicate_rows,
        )
        .unwrap_err()
        .contains("duplicate row"));

        let quiet_rows = vec![
            leaf_snapshot_row(11, 1, 1, false, true),
            leaf_snapshot_row(12, 1, 1, false, false),
        ];
        assert!(build_scheduled_merge_replacement_centroids(
            &decision,
            &root_routing_object(),
            &quiet_rows,
        )
        .unwrap_err()
        .contains("no longer merge recommended"));
    }

    struct ScheduledReplacementSnapshotFixture {
        epoch_manifest: SpireEpochManifest,
        object_manifest: SpireObjectManifest,
        placement_directory: SpirePlacementDirectory,
    }

    impl ScheduledReplacementSnapshotFixture {
        fn snapshot(&self) -> SpirePublishedEpochSnapshot<'_> {
            SpirePublishedEpochSnapshot::new(
                &self.epoch_manifest,
                &self.object_manifest,
                &self.placement_directory,
            )
            .unwrap()
        }
    }

    fn scheduled_replacement_snapshot_fixture(
        object_store: &mut SpireLocalObjectStore,
        active_epoch: u64,
        root: &SpireRoutingPartitionObject,
    ) -> ScheduledReplacementSnapshotFixture {
        let root_placement = object_store
            .insert_routing_object(active_epoch, root)
            .unwrap();
        let leaf_11 = object_store
            .insert_leaf_object_v2_from_rows(
                active_epoch,
                11,
                1,
                root.header.pid,
                &[primary_row(1, 10, 1)],
            )
            .unwrap();
        let leaf_12 = object_store
            .insert_leaf_object_v2_from_rows(
                active_epoch,
                12,
                1,
                root.header.pid,
                &[primary_row(2, 10, 2)],
            )
            .unwrap();
        let leaf_13 = object_store
            .insert_leaf_object_v2_from_rows(
                active_epoch,
                13,
                1,
                root.header.pid,
                &[primary_row(3, 10, 3)],
            )
            .unwrap();
        let placements = vec![root_placement, leaf_11, leaf_12, leaf_13];
        let epoch_manifest = SpireEpochManifest {
            epoch: active_epoch,
            state: SpireEpochState::Published,
            consistency_mode: SpireConsistencyMode::Strict,
            published_at_micros: 1000,
            retain_until_micros: 2000,
            active_query_count: 0,
        };
        let object_manifest = SpireObjectManifest::from_entries(
            active_epoch,
            placements
                .iter()
                .map(manifest_entry_for)
                .collect::<Vec<_>>(),
        )
        .unwrap();
        let placement_directory =
            SpirePlacementDirectory::from_entries(active_epoch, placements).unwrap();
        ScheduledReplacementSnapshotFixture {
            epoch_manifest,
            object_manifest,
            placement_directory,
        }
    }

    #[test]
    fn scheduled_replacement_parent_loader_reads_decision_parent() {
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let root = root_routing_object();
        let fixture = scheduled_replacement_snapshot_fixture(&mut object_store, 7, &root);
        let snapshot = fixture.snapshot();
        let decision = scheduled_split_decision(7);

        let parent =
            load_scheduled_replacement_parent_routing(&snapshot, &object_store, &decision).unwrap();
        let mut expected = root;
        expected.header.published_epoch_backref = 7;

        assert_eq!(parent, expected);
    }

    #[test]
    fn scheduled_replacement_parent_loader_rejects_stale_inputs() {
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let root = root_routing_object();
        let fixture = scheduled_replacement_snapshot_fixture(&mut object_store, 7, &root);
        let snapshot = fixture.snapshot();
        let mut stale_decision = scheduled_split_decision(8);
        assert!(load_scheduled_replacement_parent_routing(
            &snapshot,
            &object_store,
            &stale_decision,
        )
        .unwrap_err()
        .contains("snapshot epoch"));

        stale_decision.active_epoch = 7;
        stale_decision.replaced_parent_pid = 12;
        assert!(load_scheduled_replacement_parent_routing(
            &snapshot,
            &object_store,
            &stale_decision,
        )
        .is_err());

        let missing_child_root = SpireRoutingPartitionObject::root(
            1,
            2,
            2,
            vec![
                routing_child(0, 11, vec![1.0, 0.0]),
                routing_child(1, 13, vec![-1.0, 0.0]),
            ],
        )
        .unwrap();
        let mut missing_child_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let missing_child_fixture = scheduled_replacement_snapshot_fixture(
            &mut missing_child_store,
            7,
            &missing_child_root,
        );
        let missing_child_snapshot = missing_child_fixture.snapshot();
        let decision = scheduled_split_decision(7);
        assert!(load_scheduled_replacement_parent_routing(
            &missing_child_snapshot,
            &missing_child_store,
            &decision,
        )
        .unwrap_err()
        .contains("missing affected leaf"));
    }

    #[test]
    fn selected_scheduled_replacement_parent_loader_uses_lock_plan() {
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let root = root_routing_object();
        let fixture = scheduled_replacement_snapshot_fixture(&mut object_store, 7, &root);
        let snapshot = fixture.snapshot();
        let selected = SpireSelectedScheduledReplacementPublishLockPlan {
            decision: scheduled_split_decision(7),
            lock_plan: SpireScheduledReplacementPublishLockPlan {
                pid_plan: SpireLeafReplacementPidPlan {
                    replacement_pids: vec![21, 22],
                    reuses_existing_pid: false,
                    next_pid: 23,
                },
                publish_plan: SpireScheduledReplacementPublishPlan {
                    epoch: 8,
                    consistency_mode: SpireConsistencyMode::Strict,
                    next_pid: 23,
                    next_local_vec_seq: 7,
                },
            },
        };

        let parent =
            load_selected_scheduled_replacement_parent_routing(&snapshot, &object_store, &selected)
                .unwrap();

        assert_eq!(parent.header.pid, 1);
        assert_eq!(
            parent
                .children()
                .map(|child| child.child_pid)
                .collect::<Vec<_>>(),
            vec![11, 12, 13]
        );
    }

    #[test]
    fn selected_scheduled_replacement_parent_loader_rejects_snapshot_drift() {
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let root = root_routing_object();
        let fixture = scheduled_replacement_snapshot_fixture(&mut object_store, 7, &root);
        let snapshot = fixture.snapshot();
        let selected = SpireSelectedScheduledReplacementPublishLockPlan {
            decision: SpireLeafReplacementScheduleDecision {
                active_epoch: 6,
                ..scheduled_split_decision(7)
            },
            lock_plan: SpireScheduledReplacementPublishLockPlan {
                pid_plan: SpireLeafReplacementPidPlan {
                    replacement_pids: vec![21, 22],
                    reuses_existing_pid: false,
                    next_pid: 23,
                },
                publish_plan: SpireScheduledReplacementPublishPlan {
                    epoch: 7,
                    consistency_mode: SpireConsistencyMode::Strict,
                    next_pid: 23,
                    next_local_vec_seq: 7,
                },
            },
        };

        assert!(load_selected_scheduled_replacement_parent_routing(
            &snapshot,
            &object_store,
            &selected
        )
        .unwrap_err()
        .contains("snapshot epoch"));
    }

    #[test]
    fn scheduled_split_replacement_routing_parts_rewrite_parent() {
        let decision = scheduled_split_decision(7);
        let pid_plan = SpireLeafReplacementPidPlan {
            replacement_pids: vec![21, 22],
            reuses_existing_pid: false,
            next_pid: 23,
        };

        let parts = build_scheduled_split_replacement_routing_parts(
            &decision,
            &pid_plan,
            &root_routing_object(),
            vec![vec![0.5, 0.5], vec![-0.5, 0.5]],
            4,
        )
        .unwrap();

        assert_eq!(
            parts
                .replacement_children
                .iter()
                .map(|child| child.child_pid)
                .collect::<Vec<_>>(),
            vec![21, 22]
        );
        assert_eq!(parts.replacement_parent.header.object_version, 4);
        assert_eq!(
            parts
                .replacement_parent
                .children()
                .map(|child| child.child_pid)
                .collect::<Vec<_>>(),
            vec![11, 21, 22, 13]
        );
    }

    #[test]
    fn scheduled_split_replacement_routing_parts_rejects_invalid_inputs() {
        let decision = scheduled_split_decision(7);
        let pid_plan = SpireLeafReplacementPidPlan {
            replacement_pids: vec![21, 22],
            reuses_existing_pid: false,
            next_pid: 23,
        };

        assert!(build_scheduled_split_replacement_routing_parts(
            &decision,
            &pid_plan,
            &root_routing_object(),
            vec![vec![0.5, 0.5]],
            4,
        )
        .unwrap_err()
        .contains("centroid count"));

        let merge_decision = SpireLeafReplacementScheduleDecision {
            mode: SpireLeafReplacementScheduleMode::Merge,
            active_epoch: 7,
            replaced_parent_pid: 1,
            affected_leaf_pids: vec![11, 12],
            replacement_leaf_count: 1,
            reason: "test_merge",
        };
        assert!(build_scheduled_split_replacement_routing_parts(
            &merge_decision,
            &SpireLeafReplacementPidPlan {
                replacement_pids: vec![21],
                reuses_existing_pid: false,
                next_pid: 22,
            },
            &root_routing_object(),
            vec![vec![0.5, 0.5]],
            4,
        )
        .unwrap_err()
        .contains("split decision"));
    }

    #[test]
    fn relation_scheduled_split_replacement_execution_parts_compose_inputs() {
        let decision = scheduled_split_decision(7);
        let pid_plan = SpireLeafReplacementPidPlan {
            replacement_pids: vec![21, 22],
            reuses_existing_pid: false,
            next_pid: 23,
        };

        let parts = build_relation_scheduled_split_replacement_execution_parts(
            &decision,
            &pid_plan,
            &root_routing_object(),
            vec![vec![0.5, 0.5], vec![-0.5, 0.5]],
            vec![
                SpireReplacementLeafObjectInput {
                    pid: 22,
                    rows: vec![primary_row(2, 10, 2)],
                },
                SpireReplacementLeafObjectInput {
                    pid: 21,
                    rows: vec![primary_row(1, 10, 1)],
                },
            ],
            4,
            2,
            3000,
            4000,
        )
        .unwrap();

        assert_eq!(parts.published_at_micros, 3000);
        assert_eq!(parts.replacement_parent.header.object_version, 4);
        assert_eq!(
            parts
                .replacement_children
                .iter()
                .map(|child| child.child_pid)
                .collect::<Vec<_>>(),
            vec![21, 22]
        );
        assert_eq!(
            parts
                .leaf_inputs
                .iter()
                .map(|input| input.pid)
                .collect::<Vec<_>>(),
            vec![21, 22]
        );
    }

    #[test]
    fn relation_scheduled_split_replacement_execution_parts_rejects_drift() {
        let decision = scheduled_split_decision(7);
        let pid_plan = SpireLeafReplacementPidPlan {
            replacement_pids: vec![21, 22],
            reuses_existing_pid: false,
            next_pid: 23,
        };

        assert!(build_relation_scheduled_split_replacement_execution_parts(
            &decision,
            &pid_plan,
            &root_routing_object(),
            vec![vec![0.5, 0.5], vec![-0.5, 0.5]],
            vec![SpireReplacementLeafObjectInput {
                pid: 21,
                rows: vec![primary_row(1, 10, 1)],
            }],
            4,
            2,
            3000,
            4000,
        )
        .unwrap_err()
        .contains("input count"));
    }

    #[test]
    fn relation_scheduled_split_replacement_execution_input_uses_publish_plan() {
        let publish_plan = SpireScheduledReplacementPublishPlan {
            epoch: 8,
            consistency_mode: SpireConsistencyMode::Strict,
            next_pid: 23,
            next_local_vec_seq: 100,
        };
        let decision = scheduled_split_decision(7);
        let pid_plan = SpireLeafReplacementPidPlan {
            replacement_pids: vec![21, 22],
            reuses_existing_pid: false,
            next_pid: 23,
        };

        let input = build_relation_scheduled_split_replacement_execution_input(
            &publish_plan,
            &pid_plan,
            &decision,
            &root_routing_object(),
            vec![vec![0.5, 0.5], vec![-0.5, 0.5]],
            vec![
                SpireReplacementLeafObjectInput {
                    pid: 22,
                    rows: vec![primary_row(2, 10, 2)],
                },
                SpireReplacementLeafObjectInput {
                    pid: 21,
                    rows: vec![primary_row(1, 10, 1)],
                },
            ],
            4,
            2,
            3000,
            4000,
        )
        .unwrap();

        assert_eq!(input.epoch, 8);
        assert_eq!(input.next_local_vec_seq, 100);
        assert_eq!(
            input
                .replacement_children
                .iter()
                .map(|child| child.child_pid)
                .collect::<Vec<_>>(),
            vec![21, 22]
        );
        assert_eq!(
            input
                .leaf_inputs
                .iter()
                .map(|leaf_input| leaf_input.pid)
                .collect::<Vec<_>>(),
            vec![21, 22]
        );
    }

    #[test]
    fn relation_scheduled_split_replacement_execution_input_rejects_plan_drift() {
        let publish_plan = SpireScheduledReplacementPublishPlan {
            epoch: 8,
            consistency_mode: SpireConsistencyMode::Strict,
            next_pid: 24,
            next_local_vec_seq: 100,
        };
        let decision = scheduled_split_decision(7);
        let pid_plan = SpireLeafReplacementPidPlan {
            replacement_pids: vec![21, 22],
            reuses_existing_pid: false,
            next_pid: 23,
        };

        assert!(build_relation_scheduled_split_replacement_execution_input(
            &publish_plan,
            &pid_plan,
            &decision,
            &root_routing_object(),
            vec![vec![0.5, 0.5], vec![-0.5, 0.5]],
            vec![
                SpireReplacementLeafObjectInput {
                    pid: 21,
                    rows: vec![primary_row(1, 10, 1)],
                },
                SpireReplacementLeafObjectInput {
                    pid: 22,
                    rows: vec![primary_row(2, 10, 2)],
                },
            ],
            4,
            2,
            3000,
            4000,
        )
        .unwrap_err()
        .contains("next_pid"));

        let publish_plan = SpireScheduledReplacementPublishPlan {
            epoch: 8,
            consistency_mode: SpireConsistencyMode::Strict,
            next_pid: 23,
            next_local_vec_seq: 100,
        };
        assert!(build_relation_scheduled_split_replacement_execution_input(
            &publish_plan,
            &pid_plan,
            &decision,
            &root_routing_object(),
            vec![vec![0.5, 0.5], vec![-0.5, 0.5]],
            vec![SpireReplacementLeafObjectInput {
                pid: 21,
                rows: vec![primary_row(1, 10, 1)],
            }],
            4,
            2,
            3000,
            4000,
        )
        .unwrap_err()
        .contains("input count"));
    }

    #[test]
    fn relation_selected_scheduled_split_replacement_execution_input_uses_lock_plan() {
        let selected = SpireSelectedScheduledReplacementPublishLockPlan {
            decision: scheduled_split_decision(7),
            lock_plan: SpireScheduledReplacementPublishLockPlan {
                pid_plan: SpireLeafReplacementPidPlan {
                    replacement_pids: vec![21, 22],
                    reuses_existing_pid: false,
                    next_pid: 23,
                },
                publish_plan: SpireScheduledReplacementPublishPlan {
                    epoch: 8,
                    consistency_mode: SpireConsistencyMode::Strict,
                    next_pid: 23,
                    next_local_vec_seq: 100,
                },
            },
        };

        let input = build_relation_selected_scheduled_split_replacement_execution_input(
            &selected,
            &root_routing_object(),
            vec![vec![0.5, 0.5], vec![-0.5, 0.5]],
            vec![
                SpireReplacementLeafObjectInput {
                    pid: 22,
                    rows: vec![primary_row(2, 10, 2)],
                },
                SpireReplacementLeafObjectInput {
                    pid: 21,
                    rows: vec![primary_row(1, 10, 1)],
                },
            ],
            4,
            2,
            3000,
            4000,
        )
        .unwrap();

        assert_eq!(input.epoch, 8);
        assert_eq!(input.next_local_vec_seq, 100);
        assert_eq!(
            input
                .replacement_children
                .iter()
                .map(|child| child.child_pid)
                .collect::<Vec<_>>(),
            vec![21, 22]
        );
        assert_eq!(
            input
                .leaf_inputs
                .iter()
                .map(|leaf_input| leaf_input.pid)
                .collect::<Vec<_>>(),
            vec![21, 22]
        );
    }

    #[test]
    fn relation_selected_scheduled_split_replacement_execution_input_rejects_merge_plan() {
        let selected = SpireSelectedScheduledReplacementPublishLockPlan {
            decision: SpireLeafReplacementScheduleDecision {
                mode: SpireLeafReplacementScheduleMode::Merge,
                active_epoch: 7,
                replaced_parent_pid: 1,
                affected_leaf_pids: vec![11, 12],
                replacement_leaf_count: 1,
                reason: "test_merge",
            },
            lock_plan: SpireScheduledReplacementPublishLockPlan {
                pid_plan: SpireLeafReplacementPidPlan {
                    replacement_pids: vec![21],
                    reuses_existing_pid: false,
                    next_pid: 22,
                },
                publish_plan: SpireScheduledReplacementPublishPlan {
                    epoch: 8,
                    consistency_mode: SpireConsistencyMode::Strict,
                    next_pid: 22,
                    next_local_vec_seq: 100,
                },
            },
        };

        assert!(
            build_relation_selected_scheduled_split_replacement_execution_input(
                &selected,
                &root_routing_object(),
                vec![vec![0.5, 0.5]],
                vec![SpireReplacementLeafObjectInput {
                    pid: 21,
                    rows: vec![primary_row(1, 10, 1)],
                }],
                4,
                2,
                3000,
                4000,
            )
            .unwrap_err()
            .contains("split decision")
        );
    }

    #[test]
    fn local_selected_scheduled_split_replacement_execution_input_uses_lock_plan() {
        let selected = SpireSelectedScheduledReplacementPublishLockPlan {
            decision: scheduled_split_decision(7),
            lock_plan: SpireScheduledReplacementPublishLockPlan {
                pid_plan: SpireLeafReplacementPidPlan {
                    replacement_pids: vec![21, 22],
                    reuses_existing_pid: false,
                    next_pid: 23,
                },
                publish_plan: SpireScheduledReplacementPublishPlan {
                    epoch: 8,
                    consistency_mode: SpireConsistencyMode::Strict,
                    next_pid: 23,
                    next_local_vec_seq: 100,
                },
            },
        };

        let input = build_local_selected_scheduled_split_replacement_execution_input(
            &selected,
            &root_routing_object(),
            vec![vec![0.5, 0.5], vec![-0.5, 0.5]],
            vec![
                SpireReplacementLeafObjectInput {
                    pid: 22,
                    rows: vec![primary_row(2, 10, 2)],
                },
                SpireReplacementLeafObjectInput {
                    pid: 21,
                    rows: vec![primary_row(1, 10, 1)],
                },
            ],
            4,
            2,
            3000,
            4000,
            placement_write_evidence_for_pids(&[1, 21, 22]),
        )
        .unwrap();

        assert_eq!(input.epoch, 8);
        assert_eq!(input.next_local_vec_seq, 100);
        assert_eq!(
            input
                .leaf_inputs
                .iter()
                .map(|leaf_input| leaf_input.pid)
                .collect::<Vec<_>>(),
            vec![21, 22]
        );
        assert_eq!(
            input
                .placement_write_evidence
                .iter()
                .map(|evidence| evidence.pid)
                .collect::<Vec<_>>(),
            vec![1, 21, 22]
        );
    }

    #[test]
    fn local_selected_scheduled_split_replacement_execution_input_rejects_merge_plan() {
        let selected = SpireSelectedScheduledReplacementPublishLockPlan {
            decision: SpireLeafReplacementScheduleDecision {
                mode: SpireLeafReplacementScheduleMode::Merge,
                active_epoch: 7,
                replaced_parent_pid: 1,
                affected_leaf_pids: vec![11, 12],
                replacement_leaf_count: 1,
                reason: "test_merge",
            },
            lock_plan: SpireScheduledReplacementPublishLockPlan {
                pid_plan: SpireLeafReplacementPidPlan {
                    replacement_pids: vec![21],
                    reuses_existing_pid: false,
                    next_pid: 22,
                },
                publish_plan: SpireScheduledReplacementPublishPlan {
                    epoch: 8,
                    consistency_mode: SpireConsistencyMode::Strict,
                    next_pid: 22,
                    next_local_vec_seq: 100,
                },
            },
        };

        assert!(
            build_local_selected_scheduled_split_replacement_execution_input(
                &selected,
                &root_routing_object(),
                vec![vec![0.5, 0.5]],
                vec![SpireReplacementLeafObjectInput {
                    pid: 21,
                    rows: vec![primary_row(1, 10, 1)],
                }],
                4,
                2,
                3000,
                4000,
                placement_write_evidence_for_pids(&[1, 21]),
            )
            .unwrap_err()
            .contains("split decision")
        );
    }

    #[test]
    fn local_scheduled_split_replacement_execution_parts_preserve_write_evidence() {
        let decision = scheduled_split_decision(7);
        let pid_plan = SpireLeafReplacementPidPlan {
            replacement_pids: vec![21, 22],
            reuses_existing_pid: false,
            next_pid: 23,
        };

        let parts = build_local_scheduled_split_replacement_execution_parts(
            &decision,
            &pid_plan,
            &root_routing_object(),
            vec![vec![0.5, 0.5], vec![-0.5, 0.5]],
            vec![
                SpireReplacementLeafObjectInput {
                    pid: 22,
                    rows: vec![primary_row(2, 10, 2)],
                },
                SpireReplacementLeafObjectInput {
                    pid: 21,
                    rows: vec![primary_row(1, 10, 1)],
                },
            ],
            4,
            2,
            3000,
            4000,
            placement_write_evidence_for_pids(&[1, 21, 22]),
        )
        .unwrap();

        assert_eq!(
            parts
                .replacement_children
                .iter()
                .map(|child| child.child_pid)
                .collect::<Vec<_>>(),
            vec![21, 22]
        );
        assert_eq!(
            parts
                .leaf_inputs
                .iter()
                .map(|input| input.pid)
                .collect::<Vec<_>>(),
            vec![21, 22]
        );
        assert_eq!(
            parts
                .placement_write_evidence
                .iter()
                .map(|evidence| evidence.pid)
                .collect::<Vec<_>>(),
            vec![1, 21, 22]
        );
    }

    #[test]
    fn local_scheduled_split_replacement_execution_input_uses_publish_plan() {
        let publish_plan = SpireScheduledReplacementPublishPlan {
            epoch: 8,
            consistency_mode: SpireConsistencyMode::Strict,
            next_pid: 23,
            next_local_vec_seq: 100,
        };
        let decision = scheduled_split_decision(7);
        let pid_plan = SpireLeafReplacementPidPlan {
            replacement_pids: vec![21, 22],
            reuses_existing_pid: false,
            next_pid: 23,
        };

        let input = build_local_scheduled_split_replacement_execution_input(
            &publish_plan,
            &pid_plan,
            &decision,
            &root_routing_object(),
            vec![vec![0.5, 0.5], vec![-0.5, 0.5]],
            vec![
                SpireReplacementLeafObjectInput {
                    pid: 22,
                    rows: vec![primary_row(2, 10, 2)],
                },
                SpireReplacementLeafObjectInput {
                    pid: 21,
                    rows: vec![primary_row(1, 10, 1)],
                },
            ],
            4,
            2,
            3000,
            4000,
            placement_write_evidence_for_pids(&[1, 21, 22]),
        )
        .unwrap();

        assert_eq!(input.epoch, 8);
        assert_eq!(input.next_local_vec_seq, 100);
        assert_eq!(
            input
                .leaf_inputs
                .iter()
                .map(|leaf_input| leaf_input.pid)
                .collect::<Vec<_>>(),
            vec![21, 22]
        );
        assert_eq!(
            input
                .placement_write_evidence
                .iter()
                .map(|evidence| evidence.pid)
                .collect::<Vec<_>>(),
            vec![1, 21, 22]
        );
    }

    #[test]
    fn local_scheduled_split_replacement_execution_input_rejects_plan_drift() {
        let publish_plan = SpireScheduledReplacementPublishPlan {
            epoch: 8,
            consistency_mode: SpireConsistencyMode::Strict,
            next_pid: 24,
            next_local_vec_seq: 100,
        };
        let decision = scheduled_split_decision(7);
        let pid_plan = SpireLeafReplacementPidPlan {
            replacement_pids: vec![21, 22],
            reuses_existing_pid: false,
            next_pid: 23,
        };

        assert!(build_local_scheduled_split_replacement_execution_input(
            &publish_plan,
            &pid_plan,
            &decision,
            &root_routing_object(),
            vec![vec![0.5, 0.5], vec![-0.5, 0.5]],
            vec![
                SpireReplacementLeafObjectInput {
                    pid: 21,
                    rows: vec![primary_row(1, 10, 1)],
                },
                SpireReplacementLeafObjectInput {
                    pid: 22,
                    rows: vec![primary_row(2, 10, 2)],
                },
            ],
            4,
            2,
            3000,
            4000,
            placement_write_evidence_for_pids(&[1, 21, 22]),
        )
        .unwrap_err()
        .contains("next_pid"));
    }

    #[test]
    fn scheduled_merge_replacement_routing_parts_rewrite_parent() {
        let decision = SpireLeafReplacementScheduleDecision {
            mode: SpireLeafReplacementScheduleMode::Merge,
            active_epoch: 7,
            replaced_parent_pid: 1,
            affected_leaf_pids: vec![11, 12],
            replacement_leaf_count: 1,
            reason: "test_merge",
        };
        let pid_plan = SpireLeafReplacementPidPlan {
            replacement_pids: vec![21],
            reuses_existing_pid: false,
            next_pid: 22,
        };
        let rows = vec![
            leaf_snapshot_row(11, 1, 3, false, true),
            leaf_snapshot_row(12, 1, 1, false, true),
        ];

        let parts = build_scheduled_merge_replacement_routing_parts(
            &decision,
            &pid_plan,
            &root_routing_object(),
            &rows,
            4,
        )
        .unwrap();

        assert_eq!(
            parts
                .replacement_children
                .iter()
                .map(|child| child.child_pid)
                .collect::<Vec<_>>(),
            vec![21]
        );
        assert_eq!(parts.replacement_parent.header.object_version, 4);
        assert_eq!(
            parts
                .replacement_parent
                .children()
                .map(|child| child.child_pid)
                .collect::<Vec<_>>(),
            vec![21, 13]
        );
        assert!((parts.replacement_children[0].centroid[0] - 0.9486833).abs() < 0.0001);
    }

    #[test]
    fn scheduled_merge_replacement_routing_parts_rejects_invalid_plan() {
        let decision = SpireLeafReplacementScheduleDecision {
            mode: SpireLeafReplacementScheduleMode::Merge,
            active_epoch: 7,
            replaced_parent_pid: 1,
            affected_leaf_pids: vec![11, 12],
            replacement_leaf_count: 1,
            reason: "test_merge",
        };
        let rows = vec![
            leaf_snapshot_row(11, 1, 1, false, true),
            leaf_snapshot_row(12, 1, 1, false, true),
        ];
        let reused_pid_plan = SpireLeafReplacementPidPlan {
            replacement_pids: vec![21],
            reuses_existing_pid: true,
            next_pid: 22,
        };
        assert!(build_scheduled_merge_replacement_routing_parts(
            &decision,
            &reused_pid_plan,
            &root_routing_object(),
            &rows,
            4,
        )
        .unwrap_err()
        .contains("fresh replacement pids"));

        let pid_plan = SpireLeafReplacementPidPlan {
            replacement_pids: vec![21],
            reuses_existing_pid: false,
            next_pid: 22,
        };
        assert!(build_scheduled_merge_replacement_routing_parts(
            &decision,
            &pid_plan,
            &root_routing_object(),
            &rows,
            0,
        )
        .unwrap_err()
        .contains("object_version"));
    }

    #[test]
    fn relation_scheduled_merge_replacement_execution_parts_compose_inputs() {
        let decision = SpireLeafReplacementScheduleDecision {
            mode: SpireLeafReplacementScheduleMode::Merge,
            active_epoch: 7,
            replaced_parent_pid: 1,
            affected_leaf_pids: vec![11, 12],
            replacement_leaf_count: 1,
            reason: "test_merge",
        };
        let pid_plan = SpireLeafReplacementPidPlan {
            replacement_pids: vec![21],
            reuses_existing_pid: false,
            next_pid: 22,
        };
        let rows = vec![
            leaf_snapshot_row(11, 1, 3, false, true),
            leaf_snapshot_row(12, 1, 1, false, true),
        ];

        let parts = build_relation_scheduled_merge_replacement_execution_parts(
            &decision,
            &pid_plan,
            &root_routing_object(),
            &rows,
            vec![
                SpireReplacementLeafRows {
                    base_pid: 11,
                    rows: vec![primary_row(1, 10, 1)],
                },
                SpireReplacementLeafRows {
                    base_pid: 12,
                    rows: vec![primary_row(2, 10, 2)],
                },
            ],
            4,
            2,
            3000,
            4000,
        )
        .unwrap();

        assert_eq!(parts.published_at_micros, 3000);
        assert_eq!(parts.retain_until_micros, 4000);
        assert_eq!(parts.replacement_parent.header.object_version, 4);
        assert_eq!(parts.replacement_children[0].child_pid, 21);
        assert_eq!(parts.leaf_object_version, 2);
        assert_eq!(parts.leaf_inputs.len(), 1);
        assert_eq!(parts.leaf_inputs[0].pid, 21);
        assert_eq!(parts.leaf_inputs[0].rows.len(), 2);
    }

    #[test]
    fn relation_scheduled_merge_replacement_execution_parts_rejects_drift() {
        let decision = SpireLeafReplacementScheduleDecision {
            mode: SpireLeafReplacementScheduleMode::Merge,
            active_epoch: 7,
            replaced_parent_pid: 1,
            affected_leaf_pids: vec![11, 12],
            replacement_leaf_count: 1,
            reason: "test_merge",
        };
        let pid_plan = SpireLeafReplacementPidPlan {
            replacement_pids: vec![21],
            reuses_existing_pid: false,
            next_pid: 22,
        };
        let rows = vec![
            leaf_snapshot_row(11, 1, 1, false, true),
            leaf_snapshot_row(12, 1, 1, false, true),
        ];

        assert!(build_relation_scheduled_merge_replacement_execution_parts(
            &decision,
            &pid_plan,
            &root_routing_object(),
            &rows,
            vec![SpireReplacementLeafRows {
                base_pid: 11,
                rows: vec![primary_row(1, 10, 1)],
            }],
            4,
            2,
            3000,
            4000,
        )
        .unwrap_err()
        .contains("missing rows"));

        let reused_pid_plan = SpireLeafReplacementPidPlan {
            replacement_pids: vec![21],
            reuses_existing_pid: true,
            next_pid: 22,
        };
        assert!(build_relation_scheduled_merge_replacement_execution_parts(
            &decision,
            &reused_pid_plan,
            &root_routing_object(),
            &rows,
            vec![
                SpireReplacementLeafRows {
                    base_pid: 11,
                    rows: vec![primary_row(1, 10, 1)],
                },
                SpireReplacementLeafRows {
                    base_pid: 12,
                    rows: vec![primary_row(2, 10, 2)],
                },
            ],
            4,
            2,
            3000,
            4000,
        )
        .unwrap_err()
        .contains("fresh replacement pids"));
    }

    #[test]
    fn relation_scheduled_merge_replacement_execution_input_uses_publish_plan() {
        let publish_plan = SpireScheduledReplacementPublishPlan {
            epoch: 8,
            consistency_mode: SpireConsistencyMode::Strict,
            next_pid: 22,
            next_local_vec_seq: 100,
        };
        let decision = SpireLeafReplacementScheduleDecision {
            mode: SpireLeafReplacementScheduleMode::Merge,
            active_epoch: 7,
            replaced_parent_pid: 1,
            affected_leaf_pids: vec![11, 12],
            replacement_leaf_count: 1,
            reason: "test_merge",
        };
        let pid_plan = SpireLeafReplacementPidPlan {
            replacement_pids: vec![21],
            reuses_existing_pid: false,
            next_pid: 22,
        };
        let rows = vec![
            leaf_snapshot_row(11, 1, 3, false, true),
            leaf_snapshot_row(12, 1, 1, false, true),
        ];

        let input = build_relation_scheduled_merge_replacement_execution_input(
            &publish_plan,
            &pid_plan,
            &decision,
            &root_routing_object(),
            &rows,
            vec![
                SpireReplacementLeafRows {
                    base_pid: 11,
                    rows: vec![primary_row(1, 10, 1)],
                },
                SpireReplacementLeafRows {
                    base_pid: 12,
                    rows: vec![primary_row(2, 10, 2)],
                },
            ],
            4,
            2,
            3000,
            4000,
        )
        .unwrap();

        assert_eq!(input.epoch, 8);
        assert_eq!(input.consistency_mode, SpireConsistencyMode::Strict);
        assert_eq!(input.next_local_vec_seq, 100);
        assert_eq!(input.replacement_children[0].child_pid, 21);
        assert_eq!(
            input
                .replacement_parent
                .children()
                .map(|child| child.child_pid)
                .collect::<Vec<_>>(),
            vec![21, 13]
        );
    }

    #[test]
    fn relation_scheduled_merge_replacement_execution_input_rejects_plan_drift() {
        let publish_plan = SpireScheduledReplacementPublishPlan {
            epoch: 8,
            consistency_mode: SpireConsistencyMode::Strict,
            next_pid: 23,
            next_local_vec_seq: 100,
        };
        let decision = SpireLeafReplacementScheduleDecision {
            mode: SpireLeafReplacementScheduleMode::Merge,
            active_epoch: 7,
            replaced_parent_pid: 1,
            affected_leaf_pids: vec![11, 12],
            replacement_leaf_count: 1,
            reason: "test_merge",
        };
        let pid_plan = SpireLeafReplacementPidPlan {
            replacement_pids: vec![21],
            reuses_existing_pid: false,
            next_pid: 22,
        };
        let rows = vec![
            leaf_snapshot_row(11, 1, 1, false, true),
            leaf_snapshot_row(12, 1, 1, false, true),
        ];

        assert!(build_relation_scheduled_merge_replacement_execution_input(
            &publish_plan,
            &pid_plan,
            &decision,
            &root_routing_object(),
            &rows,
            vec![
                SpireReplacementLeafRows {
                    base_pid: 11,
                    rows: vec![primary_row(1, 10, 1)],
                },
                SpireReplacementLeafRows {
                    base_pid: 12,
                    rows: vec![primary_row(2, 10, 2)],
                },
            ],
            4,
            2,
            3000,
            4000,
        )
        .unwrap_err()
        .contains("next_pid"));

        let publish_plan = SpireScheduledReplacementPublishPlan {
            epoch: 8,
            consistency_mode: SpireConsistencyMode::Strict,
            next_pid: 22,
            next_local_vec_seq: 100,
        };
        assert!(build_relation_scheduled_merge_replacement_execution_input(
            &publish_plan,
            &pid_plan,
            &decision,
            &root_routing_object(),
            &rows,
            vec![
                SpireReplacementLeafRows {
                    base_pid: 11,
                    rows: vec![primary_row(1, 10, 1)],
                },
                SpireReplacementLeafRows {
                    base_pid: 12,
                    rows: vec![primary_row(2, 10, 2)],
                },
            ],
            4,
            0,
            3000,
            4000,
        )
        .unwrap_err()
        .contains("object_version"));
    }

    #[test]
    fn local_scheduled_merge_replacement_execution_parts_preserve_write_evidence() {
        let decision = SpireLeafReplacementScheduleDecision {
            mode: SpireLeafReplacementScheduleMode::Merge,
            active_epoch: 7,
            replaced_parent_pid: 1,
            affected_leaf_pids: vec![11, 12],
            replacement_leaf_count: 1,
            reason: "test_merge",
        };
        let pid_plan = SpireLeafReplacementPidPlan {
            replacement_pids: vec![21],
            reuses_existing_pid: false,
            next_pid: 22,
        };
        let rows = vec![
            leaf_snapshot_row(11, 1, 3, false, true),
            leaf_snapshot_row(12, 1, 1, false, true),
        ];

        let parts = build_local_scheduled_merge_replacement_execution_parts(
            &decision,
            &pid_plan,
            &root_routing_object(),
            &rows,
            vec![
                SpireReplacementLeafRows {
                    base_pid: 11,
                    rows: vec![primary_row(1, 10, 1)],
                },
                SpireReplacementLeafRows {
                    base_pid: 12,
                    rows: vec![primary_row(2, 10, 2)],
                },
            ],
            4,
            2,
            3000,
            4000,
            placement_write_evidence_for_pids(&[1, 21]),
        )
        .unwrap();

        assert_eq!(parts.published_at_micros, 3000);
        assert_eq!(parts.replacement_children[0].child_pid, 21);
        assert_eq!(parts.leaf_inputs[0].pid, 21);
        assert_eq!(
            parts
                .placement_write_evidence
                .iter()
                .map(|evidence| evidence.pid)
                .collect::<Vec<_>>(),
            vec![1, 21]
        );
    }

    #[test]
    fn local_scheduled_merge_replacement_execution_input_uses_publish_plan() {
        let publish_plan = SpireScheduledReplacementPublishPlan {
            epoch: 8,
            consistency_mode: SpireConsistencyMode::Strict,
            next_pid: 22,
            next_local_vec_seq: 100,
        };
        let decision = SpireLeafReplacementScheduleDecision {
            mode: SpireLeafReplacementScheduleMode::Merge,
            active_epoch: 7,
            replaced_parent_pid: 1,
            affected_leaf_pids: vec![11, 12],
            replacement_leaf_count: 1,
            reason: "test_merge",
        };
        let pid_plan = SpireLeafReplacementPidPlan {
            replacement_pids: vec![21],
            reuses_existing_pid: false,
            next_pid: 22,
        };
        let rows = vec![
            leaf_snapshot_row(11, 1, 3, false, true),
            leaf_snapshot_row(12, 1, 1, false, true),
        ];

        let input = build_local_scheduled_merge_replacement_execution_input(
            &publish_plan,
            &pid_plan,
            &decision,
            &root_routing_object(),
            &rows,
            vec![
                SpireReplacementLeafRows {
                    base_pid: 11,
                    rows: vec![primary_row(1, 10, 1)],
                },
                SpireReplacementLeafRows {
                    base_pid: 12,
                    rows: vec![primary_row(2, 10, 2)],
                },
            ],
            4,
            2,
            3000,
            4000,
            placement_write_evidence_for_pids(&[1, 21]),
        )
        .unwrap();

        assert_eq!(input.epoch, 8);
        assert_eq!(input.consistency_mode, SpireConsistencyMode::Strict);
        assert_eq!(input.next_local_vec_seq, 100);
        assert_eq!(
            input
                .placement_write_evidence
                .iter()
                .map(|evidence| evidence.pid)
                .collect::<Vec<_>>(),
            vec![1, 21]
        );
    }

    #[test]
    fn local_scheduled_merge_replacement_execution_input_rejects_plan_drift() {
        let publish_plan = SpireScheduledReplacementPublishPlan {
            epoch: 8,
            consistency_mode: SpireConsistencyMode::Strict,
            next_pid: 23,
            next_local_vec_seq: 100,
        };
        let decision = SpireLeafReplacementScheduleDecision {
            mode: SpireLeafReplacementScheduleMode::Merge,
            active_epoch: 7,
            replaced_parent_pid: 1,
            affected_leaf_pids: vec![11, 12],
            replacement_leaf_count: 1,
            reason: "test_merge",
        };
        let pid_plan = SpireLeafReplacementPidPlan {
            replacement_pids: vec![21],
            reuses_existing_pid: false,
            next_pid: 22,
        };
        let rows = vec![
            leaf_snapshot_row(11, 1, 1, false, true),
            leaf_snapshot_row(12, 1, 1, false, true),
        ];

        assert!(build_local_scheduled_merge_replacement_execution_input(
            &publish_plan,
            &pid_plan,
            &decision,
            &root_routing_object(),
            &rows,
            vec![
                SpireReplacementLeafRows {
                    base_pid: 11,
                    rows: vec![primary_row(1, 10, 1)],
                },
                SpireReplacementLeafRows {
                    base_pid: 12,
                    rows: vec![primary_row(2, 10, 2)],
                },
            ],
            4,
            2,
            3000,
            4000,
            placement_write_evidence_for_pids(&[1, 21]),
        )
        .unwrap_err()
        .contains("next_pid"));
    }

    #[test]
    fn relation_selected_scheduled_merge_replacement_execution_input_uses_lock_plan() {
        let selected = SpireSelectedScheduledReplacementPublishLockPlan {
            decision: SpireLeafReplacementScheduleDecision {
                mode: SpireLeafReplacementScheduleMode::Merge,
                active_epoch: 7,
                replaced_parent_pid: 1,
                affected_leaf_pids: vec![11, 12],
                replacement_leaf_count: 1,
                reason: "test_merge",
            },
            lock_plan: SpireScheduledReplacementPublishLockPlan {
                pid_plan: SpireLeafReplacementPidPlan {
                    replacement_pids: vec![21],
                    reuses_existing_pid: false,
                    next_pid: 22,
                },
                publish_plan: SpireScheduledReplacementPublishPlan {
                    epoch: 8,
                    consistency_mode: SpireConsistencyMode::Strict,
                    next_pid: 22,
                    next_local_vec_seq: 100,
                },
            },
        };
        let rows = vec![
            leaf_snapshot_row(11, 1, 3, false, true),
            leaf_snapshot_row(12, 1, 1, false, true),
        ];

        let input = build_relation_selected_scheduled_merge_replacement_execution_input(
            &selected,
            &root_routing_object(),
            &rows,
            vec![
                SpireReplacementLeafRows {
                    base_pid: 11,
                    rows: vec![primary_row(1, 10, 1)],
                },
                SpireReplacementLeafRows {
                    base_pid: 12,
                    rows: vec![primary_row(2, 10, 2)],
                },
            ],
            4,
            2,
            3000,
            4000,
        )
        .unwrap();

        assert_eq!(input.epoch, 8);
        assert_eq!(input.next_local_vec_seq, 100);
        assert_eq!(input.replacement_children[0].child_pid, 21);
        assert_eq!(
            input
                .replacement_parent
                .children()
                .map(|child| child.child_pid)
                .collect::<Vec<_>>(),
            vec![21, 13]
        );
    }

    #[test]
    fn relation_selected_scheduled_merge_replacement_execution_input_rejects_split_plan() {
        let selected = SpireSelectedScheduledReplacementPublishLockPlan {
            decision: scheduled_split_decision(7),
            lock_plan: SpireScheduledReplacementPublishLockPlan {
                pid_plan: SpireLeafReplacementPidPlan {
                    replacement_pids: vec![21, 22],
                    reuses_existing_pid: false,
                    next_pid: 23,
                },
                publish_plan: SpireScheduledReplacementPublishPlan {
                    epoch: 8,
                    consistency_mode: SpireConsistencyMode::Strict,
                    next_pid: 23,
                    next_local_vec_seq: 100,
                },
            },
        };
        let rows = vec![leaf_snapshot_row(12, 1, 100, true, false)];

        assert!(
            build_relation_selected_scheduled_merge_replacement_execution_input(
                &selected,
                &root_routing_object(),
                &rows,
                vec![SpireReplacementLeafRows {
                    base_pid: 12,
                    rows: vec![primary_row(1, 10, 1)],
                }],
                4,
                2,
                3000,
                4000,
            )
            .unwrap_err()
            .contains("merge decision")
        );
    }

    #[test]
    fn local_selected_scheduled_merge_replacement_execution_input_uses_lock_plan() {
        let selected = SpireSelectedScheduledReplacementPublishLockPlan {
            decision: SpireLeafReplacementScheduleDecision {
                mode: SpireLeafReplacementScheduleMode::Merge,
                active_epoch: 7,
                replaced_parent_pid: 1,
                affected_leaf_pids: vec![11, 12],
                replacement_leaf_count: 1,
                reason: "test_merge",
            },
            lock_plan: SpireScheduledReplacementPublishLockPlan {
                pid_plan: SpireLeafReplacementPidPlan {
                    replacement_pids: vec![21],
                    reuses_existing_pid: false,
                    next_pid: 22,
                },
                publish_plan: SpireScheduledReplacementPublishPlan {
                    epoch: 8,
                    consistency_mode: SpireConsistencyMode::Strict,
                    next_pid: 22,
                    next_local_vec_seq: 100,
                },
            },
        };
        let rows = vec![
            leaf_snapshot_row(11, 1, 3, false, true),
            leaf_snapshot_row(12, 1, 1, false, true),
        ];

        let input = build_local_selected_scheduled_merge_replacement_execution_input(
            &selected,
            &root_routing_object(),
            &rows,
            vec![
                SpireReplacementLeafRows {
                    base_pid: 11,
                    rows: vec![primary_row(1, 10, 1)],
                },
                SpireReplacementLeafRows {
                    base_pid: 12,
                    rows: vec![primary_row(2, 10, 2)],
                },
            ],
            4,
            2,
            3000,
            4000,
            placement_write_evidence_for_pids(&[1, 21]),
        )
        .unwrap();

        assert_eq!(input.epoch, 8);
        assert_eq!(input.next_local_vec_seq, 100);
        assert_eq!(
            input
                .placement_write_evidence
                .iter()
                .map(|evidence| evidence.pid)
                .collect::<Vec<_>>(),
            vec![1, 21]
        );
        assert_eq!(input.replacement_children[0].child_pid, 21);
    }

    #[test]
    fn local_selected_scheduled_merge_replacement_execution_input_rejects_split_plan() {
        let selected = SpireSelectedScheduledReplacementPublishLockPlan {
            decision: scheduled_split_decision(7),
            lock_plan: SpireScheduledReplacementPublishLockPlan {
                pid_plan: SpireLeafReplacementPidPlan {
                    replacement_pids: vec![21, 22],
                    reuses_existing_pid: false,
                    next_pid: 23,
                },
                publish_plan: SpireScheduledReplacementPublishPlan {
                    epoch: 8,
                    consistency_mode: SpireConsistencyMode::Strict,
                    next_pid: 23,
                    next_local_vec_seq: 100,
                },
            },
        };
        let rows = vec![leaf_snapshot_row(12, 1, 100, true, false)];

        assert!(
            build_local_selected_scheduled_merge_replacement_execution_input(
                &selected,
                &root_routing_object(),
                &rows,
                vec![SpireReplacementLeafRows {
                    base_pid: 12,
                    rows: vec![primary_row(1, 10, 1)],
                }],
                4,
                2,
                3000,
                4000,
                placement_write_evidence_for_pids(&[1, 21, 22]),
            )
            .unwrap_err()
            .contains("merge decision")
        );
    }

    #[test]
    fn selected_scheduled_execution_publish_plan_validators_use_lock_plan() {
        let selected = SpireSelectedScheduledReplacementPublishLockPlan {
            decision: SpireLeafReplacementScheduleDecision {
                mode: SpireLeafReplacementScheduleMode::Merge,
                active_epoch: 7,
                replaced_parent_pid: 1,
                affected_leaf_pids: vec![11, 12],
                replacement_leaf_count: 1,
                reason: "test_merge",
            },
            lock_plan: SpireScheduledReplacementPublishLockPlan {
                pid_plan: SpireLeafReplacementPidPlan {
                    replacement_pids: vec![21],
                    reuses_existing_pid: false,
                    next_pid: 22,
                },
                publish_plan: SpireScheduledReplacementPublishPlan {
                    epoch: 8,
                    consistency_mode: SpireConsistencyMode::Strict,
                    next_pid: 22,
                    next_local_vec_seq: 100,
                },
            },
        };
        let rows = vec![
            leaf_snapshot_row(11, 1, 3, false, true),
            leaf_snapshot_row(12, 1, 1, false, true),
        ];
        let relation_input = build_relation_selected_scheduled_merge_replacement_execution_input(
            &selected,
            &root_routing_object(),
            &rows,
            vec![
                SpireReplacementLeafRows {
                    base_pid: 11,
                    rows: vec![primary_row(1, 10, 1)],
                },
                SpireReplacementLeafRows {
                    base_pid: 12,
                    rows: vec![primary_row(2, 10, 2)],
                },
            ],
            4,
            2,
            3000,
            4000,
        )
        .unwrap();
        let local_input = build_local_selected_scheduled_merge_replacement_execution_input(
            &selected,
            &root_routing_object(),
            &rows,
            vec![
                SpireReplacementLeafRows {
                    base_pid: 11,
                    rows: vec![primary_row(1, 10, 1)],
                },
                SpireReplacementLeafRows {
                    base_pid: 12,
                    rows: vec![primary_row(2, 10, 2)],
                },
            ],
            4,
            2,
            3000,
            4000,
            placement_write_evidence_for_pids(&[1, 21]),
        )
        .unwrap();

        validate_relation_selected_scheduled_replacement_execution_publish_plan(
            &selected,
            &relation_input,
        )
        .unwrap();
        validate_local_selected_scheduled_replacement_execution_publish_plan(
            &selected,
            &local_input,
        )
        .unwrap();
    }

    #[test]
    fn selected_scheduled_execution_publish_plan_validators_reject_drift() {
        let selected = SpireSelectedScheduledReplacementPublishLockPlan {
            decision: SpireLeafReplacementScheduleDecision {
                mode: SpireLeafReplacementScheduleMode::Merge,
                active_epoch: 7,
                replaced_parent_pid: 1,
                affected_leaf_pids: vec![11, 12],
                replacement_leaf_count: 1,
                reason: "test_merge",
            },
            lock_plan: SpireScheduledReplacementPublishLockPlan {
                pid_plan: SpireLeafReplacementPidPlan {
                    replacement_pids: vec![21],
                    reuses_existing_pid: false,
                    next_pid: 22,
                },
                publish_plan: SpireScheduledReplacementPublishPlan {
                    epoch: 8,
                    consistency_mode: SpireConsistencyMode::Strict,
                    next_pid: 22,
                    next_local_vec_seq: 100,
                },
            },
        };
        let rows = vec![
            leaf_snapshot_row(11, 1, 3, false, true),
            leaf_snapshot_row(12, 1, 1, false, true),
        ];
        let relation_input = build_relation_selected_scheduled_merge_replacement_execution_input(
            &selected,
            &root_routing_object(),
            &rows,
            vec![
                SpireReplacementLeafRows {
                    base_pid: 11,
                    rows: vec![primary_row(1, 10, 1)],
                },
                SpireReplacementLeafRows {
                    base_pid: 12,
                    rows: vec![primary_row(2, 10, 2)],
                },
            ],
            4,
            2,
            3000,
            4000,
        )
        .unwrap();
        let mut stale_relation_input = relation_input.clone();
        stale_relation_input.next_local_vec_seq = 101;
        assert!(
            validate_relation_selected_scheduled_replacement_execution_publish_plan(
                &selected,
                &stale_relation_input,
            )
            .unwrap_err()
            .contains("next_local_vec_seq")
        );

        let local_input = build_local_selected_scheduled_merge_replacement_execution_input(
            &selected,
            &root_routing_object(),
            &rows,
            vec![
                SpireReplacementLeafRows {
                    base_pid: 11,
                    rows: vec![primary_row(1, 10, 1)],
                },
                SpireReplacementLeafRows {
                    base_pid: 12,
                    rows: vec![primary_row(2, 10, 2)],
                },
            ],
            4,
            2,
            3000,
            4000,
            placement_write_evidence_for_pids(&[1, 21]),
        )
        .unwrap();
        let mut stale_local_input = local_input.clone();
        stale_local_input.next_local_vec_seq = 101;
        assert!(
            validate_local_selected_scheduled_replacement_execution_publish_plan(
                &selected,
                &stale_local_input,
            )
            .unwrap_err()
            .contains("next_local_vec_seq")
        );
    }

    #[test]
    fn selected_scheduled_replacement_execution_snapshot_validator_uses_lock_plan() {
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let root = root_routing_object();
        let fixture = scheduled_replacement_snapshot_fixture(&mut object_store, 7, &root);
        let snapshot = fixture.snapshot();
        let selected = SpireSelectedScheduledReplacementPublishLockPlan {
            decision: scheduled_split_decision(7),
            lock_plan: SpireScheduledReplacementPublishLockPlan {
                pid_plan: SpireLeafReplacementPidPlan {
                    replacement_pids: vec![21, 22],
                    reuses_existing_pid: false,
                    next_pid: 23,
                },
                publish_plan: SpireScheduledReplacementPublishPlan {
                    epoch: 8,
                    consistency_mode: SpireConsistencyMode::Strict,
                    next_pid: 23,
                    next_local_vec_seq: 7,
                },
            },
        };

        validate_selected_scheduled_replacement_execution_snapshot(&snapshot, &selected).unwrap();
    }

    #[test]
    fn selected_scheduled_replacement_execution_snapshot_validator_rejects_drift() {
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let root = root_routing_object();
        let fixture = scheduled_replacement_snapshot_fixture(&mut object_store, 7, &root);
        let snapshot = fixture.snapshot();
        let stale_decision = SpireSelectedScheduledReplacementPublishLockPlan {
            decision: scheduled_split_decision(8),
            lock_plan: SpireScheduledReplacementPublishLockPlan {
                pid_plan: SpireLeafReplacementPidPlan {
                    replacement_pids: vec![21, 22],
                    reuses_existing_pid: false,
                    next_pid: 23,
                },
                publish_plan: SpireScheduledReplacementPublishPlan {
                    epoch: 9,
                    consistency_mode: SpireConsistencyMode::Strict,
                    next_pid: 23,
                    next_local_vec_seq: 7,
                },
            },
        };
        assert!(validate_selected_scheduled_replacement_execution_snapshot(
            &snapshot,
            &stale_decision
        )
        .unwrap_err()
        .contains("snapshot epoch"));

        let stale_consistency = SpireSelectedScheduledReplacementPublishLockPlan {
            decision: scheduled_split_decision(7),
            lock_plan: SpireScheduledReplacementPublishLockPlan {
                pid_plan: SpireLeafReplacementPidPlan {
                    replacement_pids: vec![21, 22],
                    reuses_existing_pid: false,
                    next_pid: 23,
                },
                publish_plan: SpireScheduledReplacementPublishPlan {
                    epoch: 8,
                    consistency_mode: SpireConsistencyMode::Degraded,
                    next_pid: 23,
                    next_local_vec_seq: 7,
                },
            },
        };
        assert!(validate_selected_scheduled_replacement_execution_snapshot(
            &snapshot,
            &stale_consistency
        )
        .unwrap_err()
        .contains("consistency mode"));
    }

    #[test]
    fn relation_selected_scheduled_replacement_publish_inputs_validate_bundle() {
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let root = root_routing_object();
        let fixture = scheduled_replacement_snapshot_fixture(&mut object_store, 7, &root);
        let snapshot = fixture.snapshot();
        let selected = SpireSelectedScheduledReplacementPublishLockPlan {
            decision: scheduled_split_decision(7),
            lock_plan: SpireScheduledReplacementPublishLockPlan {
                pid_plan: SpireLeafReplacementPidPlan {
                    replacement_pids: vec![21, 22],
                    reuses_existing_pid: false,
                    next_pid: 23,
                },
                publish_plan: SpireScheduledReplacementPublishPlan {
                    epoch: 8,
                    consistency_mode: SpireConsistencyMode::Strict,
                    next_pid: 23,
                    next_local_vec_seq: 7,
                },
            },
        };
        let input = build_relation_selected_scheduled_split_replacement_execution_input(
            &selected,
            &root,
            vec![vec![0.5, 0.5], vec![-0.5, 0.5]],
            vec![
                SpireReplacementLeafObjectInput {
                    pid: 21,
                    rows: vec![primary_row(5, 30, 1)],
                },
                SpireReplacementLeafObjectInput {
                    pid: 22,
                    rows: vec![primary_row(6, 30, 2)],
                },
            ],
            4,
            2,
            3000,
            4000,
        )
        .unwrap();

        validate_relation_selected_scheduled_replacement_publish_inputs(
            &fixture.epoch_manifest,
            &snapshot,
            &selected,
            &input,
        )
        .unwrap();
    }

    #[test]
    fn relation_selected_scheduled_replacement_publish_inputs_reject_drift() {
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let root = root_routing_object();
        let fixture = scheduled_replacement_snapshot_fixture(&mut object_store, 7, &root);
        let snapshot = fixture.snapshot();
        let selected = SpireSelectedScheduledReplacementPublishLockPlan {
            decision: scheduled_split_decision(7),
            lock_plan: SpireScheduledReplacementPublishLockPlan {
                pid_plan: SpireLeafReplacementPidPlan {
                    replacement_pids: vec![21, 22],
                    reuses_existing_pid: false,
                    next_pid: 23,
                },
                publish_plan: SpireScheduledReplacementPublishPlan {
                    epoch: 8,
                    consistency_mode: SpireConsistencyMode::Strict,
                    next_pid: 23,
                    next_local_vec_seq: 7,
                },
            },
        };
        let input = build_relation_selected_scheduled_split_replacement_execution_input(
            &selected,
            &root,
            vec![vec![0.5, 0.5], vec![-0.5, 0.5]],
            vec![
                SpireReplacementLeafObjectInput {
                    pid: 21,
                    rows: vec![primary_row(5, 30, 1)],
                },
                SpireReplacementLeafObjectInput {
                    pid: 22,
                    rows: vec![primary_row(6, 30, 2)],
                },
            ],
            4,
            2,
            3000,
            4000,
        )
        .unwrap();

        let stale_previous_manifest = SpireEpochManifest {
            epoch: 6,
            ..fixture.epoch_manifest.clone()
        };
        assert!(
            validate_relation_selected_scheduled_replacement_publish_inputs(
                &stale_previous_manifest,
                &snapshot,
                &selected,
                &input,
            )
            .unwrap_err()
            .contains("previous epoch manifest")
        );

        let mut stale_input = input.clone();
        stale_input.next_local_vec_seq = 8;
        assert!(
            validate_relation_selected_scheduled_replacement_publish_inputs(
                &fixture.epoch_manifest,
                &snapshot,
                &selected,
                &stale_input,
            )
            .unwrap_err()
            .contains("next_local_vec_seq")
        );
    }

    #[test]
    fn selected_scheduled_replacement_leaf_rows_collects_affected_rows() {
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let root = root_routing_object();
        let fixture = scheduled_replacement_snapshot_fixture(&mut object_store, 7, &root);
        let snapshot = fixture.snapshot();
        let selected = SpireSelectedScheduledReplacementPublishLockPlan {
            decision: SpireLeafReplacementScheduleDecision {
                mode: SpireLeafReplacementScheduleMode::Merge,
                active_epoch: 7,
                replaced_parent_pid: 1,
                affected_leaf_pids: vec![11, 12],
                replacement_leaf_count: 1,
                reason: "test_merge",
            },
            lock_plan: SpireScheduledReplacementPublishLockPlan {
                pid_plan: SpireLeafReplacementPidPlan {
                    replacement_pids: vec![21],
                    reuses_existing_pid: false,
                    next_pid: 22,
                },
                publish_plan: SpireScheduledReplacementPublishPlan {
                    epoch: 8,
                    consistency_mode: SpireConsistencyMode::Strict,
                    next_pid: 22,
                    next_local_vec_seq: 7,
                },
            },
        };

        let rows =
            collect_selected_scheduled_replacement_leaf_rows(&snapshot, &object_store, &selected)
                .unwrap();

        assert_eq!(
            rows.iter().map(|row| row.base_pid).collect::<Vec<_>>(),
            vec![11, 12]
        );
        assert_eq!(rows[0].rows[0].heap_tid, tid(10, 1));
        assert_eq!(rows[1].rows[0].heap_tid, tid(10, 2));
    }

    #[test]
    fn selected_scheduled_replacement_leaf_rows_rejects_snapshot_drift() {
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let root = root_routing_object();
        let fixture = scheduled_replacement_snapshot_fixture(&mut object_store, 7, &root);
        let snapshot = fixture.snapshot();
        let selected = SpireSelectedScheduledReplacementPublishLockPlan {
            decision: scheduled_split_decision(7),
            lock_plan: SpireScheduledReplacementPublishLockPlan {
                pid_plan: SpireLeafReplacementPidPlan {
                    replacement_pids: vec![21, 22],
                    reuses_existing_pid: false,
                    next_pid: 23,
                },
                publish_plan: SpireScheduledReplacementPublishPlan {
                    epoch: 8,
                    consistency_mode: SpireConsistencyMode::Degraded,
                    next_pid: 23,
                    next_local_vec_seq: 7,
                },
            },
        };

        assert!(collect_selected_scheduled_replacement_leaf_rows(
            &snapshot,
            &object_store,
            &selected
        )
        .unwrap_err()
        .contains("consistency mode"));
    }

    #[test]
    fn merge_replacement_leaf_input_combines_folded_rows_in_decision_order() {
        let decision = SpireLeafReplacementScheduleDecision {
            mode: SpireLeafReplacementScheduleMode::Merge,
            active_epoch: 7,
            replaced_parent_pid: 1,
            affected_leaf_pids: vec![11, 12],
            replacement_leaf_count: 1,
            reason: "test_merge",
        };
        let pid_plan = SpireLeafReplacementPidPlan {
            replacement_pids: vec![30],
            reuses_existing_pid: false,
            next_pid: 31,
        };

        let input = build_merge_replacement_leaf_object_input(
            &decision,
            &pid_plan,
            vec![
                SpireReplacementLeafRows {
                    base_pid: 12,
                    rows: vec![primary_row(2, 20, 2)],
                },
                SpireReplacementLeafRows {
                    base_pid: 11,
                    rows: vec![primary_row(1, 20, 1)],
                },
            ],
        )
        .unwrap();

        assert_eq!(input.pid, 30);
        assert_eq!(
            input
                .rows
                .iter()
                .map(|row| row.heap_tid)
                .collect::<Vec<_>>(),
            vec![tid(20, 1), tid(20, 2)]
        );
    }

    #[test]
    fn merge_replacement_leaf_input_rejects_wrong_mode_or_row_set() {
        let split_decision = SpireLeafReplacementScheduleDecision {
            mode: SpireLeafReplacementScheduleMode::Split,
            active_epoch: 7,
            replaced_parent_pid: 1,
            affected_leaf_pids: vec![11],
            replacement_leaf_count: 2,
            reason: "test_split",
        };
        let pid_plan = SpireLeafReplacementPidPlan {
            replacement_pids: vec![30],
            reuses_existing_pid: false,
            next_pid: 31,
        };
        assert!(
            build_merge_replacement_leaf_object_input(&split_decision, &pid_plan, Vec::new())
                .unwrap_err()
                .contains("requires a merge decision")
        );
        assert!(build_merge_replacement_leaf_object_input(
            &SpireLeafReplacementScheduleDecision {
                mode: SpireLeafReplacementScheduleMode::Merge,
                active_epoch: 7,
                replaced_parent_pid: 1,
                affected_leaf_pids: vec![11, 12],
                replacement_leaf_count: 1,
                reason: "test_merge",
            },
            &SpireLeafReplacementPidPlan {
                replacement_pids: vec![30],
                reuses_existing_pid: false,
                next_pid: 30,
            },
            Vec::new(),
        )
        .unwrap_err()
        .contains("does not advance"));

        let merge_decision = SpireLeafReplacementScheduleDecision {
            mode: SpireLeafReplacementScheduleMode::Merge,
            active_epoch: 7,
            replaced_parent_pid: 1,
            affected_leaf_pids: vec![11, 12],
            replacement_leaf_count: 1,
            reason: "test_merge",
        };
        assert!(build_merge_replacement_leaf_object_input(
            &merge_decision,
            &pid_plan,
            vec![SpireReplacementLeafRows {
                base_pid: 11,
                rows: vec![primary_row(1, 20, 1)],
            }],
        )
        .unwrap_err()
        .contains("missing rows"));
        assert!(build_merge_replacement_leaf_object_input(
            &merge_decision,
            &pid_plan,
            vec![
                SpireReplacementLeafRows {
                    base_pid: 11,
                    rows: vec![primary_row(1, 20, 1)],
                },
                SpireReplacementLeafRows {
                    base_pid: 13,
                    rows: Vec::new(),
                },
            ],
        )
        .unwrap_err()
        .contains("unselected base pid"));
    }

    #[test]
    fn split_replacement_leaf_inputs_validate_and_follow_pid_plan_order() {
        let decision = SpireLeafReplacementScheduleDecision {
            mode: SpireLeafReplacementScheduleMode::Split,
            active_epoch: 7,
            replaced_parent_pid: 1,
            affected_leaf_pids: vec![12],
            replacement_leaf_count: 2,
            reason: "test_split",
        };
        let pid_plan = SpireLeafReplacementPidPlan {
            replacement_pids: vec![30, 31],
            reuses_existing_pid: false,
            next_pid: 32,
        };

        let inputs = build_split_replacement_leaf_object_inputs(
            &decision,
            &pid_plan,
            vec![
                SpireReplacementLeafObjectInput {
                    pid: 31,
                    rows: vec![primary_row(2, 20, 2)],
                },
                SpireReplacementLeafObjectInput {
                    pid: 30,
                    rows: vec![primary_row(1, 20, 1)],
                },
            ],
        )
        .unwrap();

        assert_eq!(
            inputs.iter().map(|input| input.pid).collect::<Vec<_>>(),
            vec![30, 31]
        );
        assert_eq!(inputs[0].rows[0].heap_tid, tid(20, 1));
        assert_eq!(inputs[1].rows[0].heap_tid, tid(20, 2));
    }

    #[test]
    fn split_replacement_leaf_inputs_reject_wrong_shape_or_duplicate_vec_id() {
        let decision = SpireLeafReplacementScheduleDecision {
            mode: SpireLeafReplacementScheduleMode::Split,
            active_epoch: 7,
            replaced_parent_pid: 1,
            affected_leaf_pids: vec![12],
            replacement_leaf_count: 2,
            reason: "test_split",
        };
        let pid_plan = SpireLeafReplacementPidPlan {
            replacement_pids: vec![30, 31],
            reuses_existing_pid: false,
            next_pid: 32,
        };
        assert!(build_split_replacement_leaf_object_inputs(
            &decision,
            &pid_plan,
            vec![SpireReplacementLeafObjectInput {
                pid: 30,
                rows: Vec::new(),
            }],
        )
        .unwrap_err()
        .contains("input count"));

        assert!(build_split_replacement_leaf_object_inputs(
            &decision,
            &SpireLeafReplacementPidPlan {
                replacement_pids: vec![30, 31],
                reuses_existing_pid: false,
                next_pid: 31,
            },
            vec![
                SpireReplacementLeafObjectInput {
                    pid: 30,
                    rows: vec![primary_row(1, 20, 1)],
                },
                SpireReplacementLeafObjectInput {
                    pid: 31,
                    rows: vec![primary_row(2, 20, 2)],
                },
            ],
        )
        .unwrap_err()
        .contains("does not advance"));

        assert!(build_split_replacement_leaf_object_inputs(
            &decision,
            &pid_plan,
            vec![
                SpireReplacementLeafObjectInput {
                    pid: 30,
                    rows: vec![primary_row(1, 20, 1)],
                },
                SpireReplacementLeafObjectInput {
                    pid: 31,
                    rows: vec![primary_row(1, 20, 1)],
                },
            ],
        )
        .unwrap_err()
        .contains("duplicate vec_id"));
    }

    #[test]
    fn scheduled_routing_replacement_children_pair_pids_and_centroids_in_plan_order() {
        let decision = SpireLeafReplacementScheduleDecision {
            mode: SpireLeafReplacementScheduleMode::Split,
            active_epoch: 7,
            replaced_parent_pid: 1,
            affected_leaf_pids: vec![12],
            replacement_leaf_count: 2,
            reason: "test_split",
        };
        let pid_plan = SpireLeafReplacementPidPlan {
            replacement_pids: vec![30, 31],
            reuses_existing_pid: false,
            next_pid: 32,
        };

        let children = build_scheduled_routing_replacement_children(
            &decision,
            &pid_plan,
            vec![vec![1.0, 0.0], vec![0.0, 1.0]],
        )
        .unwrap();

        assert_eq!(
            children,
            vec![
                replacement_child(30, vec![1.0, 0.0]),
                replacement_child(31, vec![0.0, 1.0])
            ]
        );
    }

    #[test]
    fn scheduled_routing_replacement_children_accept_merge_survivor() {
        let decision = SpireLeafReplacementScheduleDecision {
            mode: SpireLeafReplacementScheduleMode::Merge,
            active_epoch: 7,
            replaced_parent_pid: 1,
            affected_leaf_pids: vec![11, 12],
            replacement_leaf_count: 1,
            reason: "test_merge",
        };
        let pid_plan = SpireLeafReplacementPidPlan {
            replacement_pids: vec![30],
            reuses_existing_pid: false,
            next_pid: 31,
        };

        let children = build_scheduled_routing_replacement_children(
            &decision,
            &pid_plan,
            vec![vec![0.5, 0.5]],
        )
        .unwrap();

        assert_eq!(children, vec![replacement_child(30, vec![0.5, 0.5])]);
    }

    #[test]
    fn scheduled_routing_replacement_children_reject_count_and_centroid_shape() {
        let decision = SpireLeafReplacementScheduleDecision {
            mode: SpireLeafReplacementScheduleMode::Split,
            active_epoch: 7,
            replaced_parent_pid: 1,
            affected_leaf_pids: vec![12],
            replacement_leaf_count: 2,
            reason: "test_split",
        };
        let pid_plan = SpireLeafReplacementPidPlan {
            replacement_pids: vec![30, 31],
            reuses_existing_pid: false,
            next_pid: 32,
        };

        assert!(build_scheduled_routing_replacement_children(
            &decision,
            &pid_plan,
            vec![vec![1.0, 0.0]],
        )
        .unwrap_err()
        .contains("centroid count"));

        assert!(build_scheduled_routing_replacement_children(
            &decision,
            &pid_plan,
            vec![vec![1.0, 0.0], Vec::new()],
        )
        .unwrap_err()
        .contains("centroid is empty"));

        assert!(build_scheduled_routing_replacement_children(
            &decision,
            &pid_plan,
            vec![vec![1.0, 0.0], vec![f32::NAN, 1.0]],
        )
        .unwrap_err()
        .contains("centroid must be finite"));
    }

    #[test]
    fn scheduled_routing_replacement_children_reject_reused_or_mismatched_pids() {
        let decision = SpireLeafReplacementScheduleDecision {
            mode: SpireLeafReplacementScheduleMode::Split,
            active_epoch: 7,
            replaced_parent_pid: 1,
            affected_leaf_pids: vec![12],
            replacement_leaf_count: 2,
            reason: "test_split",
        };

        assert!(build_scheduled_routing_replacement_children(
            &decision,
            &SpireLeafReplacementPidPlan {
                replacement_pids: vec![12],
                reuses_existing_pid: true,
                next_pid: 30,
            },
            vec![vec![1.0, 0.0]],
        )
        .unwrap_err()
        .contains("fresh replacement pids"));

        assert!(build_scheduled_routing_replacement_children(
            &decision,
            &SpireLeafReplacementPidPlan {
                replacement_pids: vec![30],
                reuses_existing_pid: false,
                next_pid: 31,
            },
            vec![vec![1.0, 0.0]],
        )
        .unwrap_err()
        .contains("pid count"));

        assert!(build_scheduled_routing_replacement_children(
            &decision,
            &SpireLeafReplacementPidPlan {
                replacement_pids: vec![30, 31],
                reuses_existing_pid: false,
                next_pid: 31,
            },
            vec![vec![1.0, 0.0], vec![0.0, 1.0]],
        )
        .unwrap_err()
        .contains("does not advance"));
    }

    #[test]
    fn scheduled_routing_rewrite_applies_split_decision_to_parent() {
        let root = root_routing_object();
        let decision = SpireLeafReplacementScheduleDecision {
            mode: SpireLeafReplacementScheduleMode::Split,
            active_epoch: 7,
            replaced_parent_pid: 1,
            affected_leaf_pids: vec![12],
            replacement_leaf_count: 2,
            reason: "test_split",
        };

        let rewritten = rewrite_scheduled_replacement_parent_routing(
            &root,
            &decision,
            vec![
                replacement_child(30, vec![0.5, 0.5]),
                replacement_child(31, vec![0.25, 0.75]),
            ],
            4,
        )
        .unwrap();

        assert_eq!(rewritten.header.pid, 1);
        assert_eq!(rewritten.header.object_version, 4);
        assert_eq!(
            rewritten
                .children()
                .map(|child| child.child_pid)
                .collect::<Vec<_>>(),
            vec![11, 30, 31, 13]
        );
    }

    #[test]
    fn scheduled_routing_rewrite_applies_merge_decision_to_parent() {
        let root = root_routing_object();
        let decision = SpireLeafReplacementScheduleDecision {
            mode: SpireLeafReplacementScheduleMode::Merge,
            active_epoch: 7,
            replaced_parent_pid: 1,
            affected_leaf_pids: vec![11, 12],
            replacement_leaf_count: 1,
            reason: "test_merge",
        };

        let rewritten = rewrite_scheduled_replacement_parent_routing(
            &root,
            &decision,
            vec![replacement_child(30, vec![0.5, 0.5])],
            4,
        )
        .unwrap();

        assert_eq!(
            rewritten
                .children()
                .map(|child| child.child_pid)
                .collect::<Vec<_>>(),
            vec![30, 13]
        );
    }

    #[test]
    fn scheduled_routing_rewrite_rejects_wrong_parent_or_child_count() {
        let root = root_routing_object();
        let decision = SpireLeafReplacementScheduleDecision {
            mode: SpireLeafReplacementScheduleMode::Split,
            active_epoch: 7,
            replaced_parent_pid: 1,
            affected_leaf_pids: vec![12],
            replacement_leaf_count: 2,
            reason: "test_split",
        };
        let wrong_parent = SpireLeafReplacementScheduleDecision {
            replaced_parent_pid: 2,
            ..decision.clone()
        };

        assert!(rewrite_scheduled_replacement_parent_routing(
            &root,
            &wrong_parent,
            vec![
                replacement_child(30, vec![0.5, 0.5]),
                replacement_child(31, vec![0.25, 0.75]),
            ],
            4,
        )
        .unwrap_err()
        .contains("does not match decision parent pid"));

        assert!(rewrite_scheduled_replacement_parent_routing(
            &root,
            &decision,
            vec![replacement_child(30, vec![0.5, 0.5])],
            4,
        )
        .unwrap_err()
        .contains("child count"));

        assert!(rewrite_scheduled_replacement_parent_routing(
            &root,
            &decision,
            vec![
                replacement_child(30, vec![0.5, 0.5]),
                replacement_child(31, vec![0.25, 0.75]),
            ],
            0,
        )
        .unwrap_err()
        .contains("object_version"));
    }

    #[test]
    fn routing_rewrite_replaces_split_child_in_parent_order() {
        let root = root_routing_object();

        let rewritten = rewrite_routing_partition_for_leaf_replacement(
            &root,
            &[12],
            vec![
                replacement_child(21, vec![0.5, 0.5]),
                replacement_child(22, vec![-0.5, 0.5]),
            ],
            4,
        )
        .unwrap();
        let children = rewritten.children().collect::<Vec<_>>();

        assert_eq!(rewritten.header.pid, root.header.pid);
        assert_eq!(rewritten.header.object_version, 4);
        assert_eq!(
            children
                .iter()
                .map(|child| child.child_pid)
                .collect::<Vec<_>>(),
            vec![11, 21, 22, 13]
        );
        assert_eq!(
            children
                .iter()
                .map(|child| child.centroid_index)
                .collect::<Vec<_>>(),
            vec![0, 1, 2, 3]
        );
        assert_eq!(children[1].centroid, &[0.5, 0.5]);
        assert_eq!(children[2].centroid, &[-0.5, 0.5]);
    }

    #[test]
    fn routing_rewrite_merges_children_at_first_affected_position() {
        let root = root_routing_object();

        let rewritten = rewrite_routing_partition_for_leaf_replacement(
            &root,
            &[11, 12],
            vec![replacement_child(30, vec![0.5, 0.5])],
            4,
        )
        .unwrap();
        let children = rewritten.children().collect::<Vec<_>>();

        assert_eq!(
            children
                .iter()
                .map(|child| child.child_pid)
                .collect::<Vec<_>>(),
            vec![30, 13]
        );
        assert_eq!(children[0].centroid_index, 0);
        assert_eq!(children[1].centroid_index, 1);
    }

    #[test]
    fn routing_rewrite_allows_rebalance_to_replace_same_child_pid() {
        let root = root_routing_object();

        let rewritten = rewrite_routing_partition_for_leaf_replacement(
            &root,
            &[12],
            vec![replacement_child(12, vec![0.0, 1.0])],
            4,
        )
        .unwrap();
        let children = rewritten.children().collect::<Vec<_>>();

        assert_eq!(
            children
                .iter()
                .map(|child| child.child_pid)
                .collect::<Vec<_>>(),
            vec![11, 12, 13]
        );
        assert_eq!(children[1].centroid, &[0.0, 1.0]);
    }

    #[test]
    fn routing_rewrite_rejects_replacement_pid_colliding_with_unaffected_child() {
        let root = root_routing_object();

        let err = rewrite_routing_partition_for_leaf_replacement(
            &root,
            &[12],
            vec![replacement_child(13, vec![0.0, 1.0])],
            4,
        )
        .unwrap_err();

        assert!(err.contains("already exists outside the affected set"));
    }

    #[test]
    fn replacement_placement_directory_carries_unaffected_and_drops_old_leaf_and_delta() {
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let active_epoch = 7;
        let new_epoch = 8;
        let root = root_routing_object();
        let root_placement = object_store
            .insert_routing_object(active_epoch, &root)
            .unwrap();
        let leaf_11 = object_store
            .insert_leaf_object_v2_from_rows(
                active_epoch,
                11,
                1,
                root.header.pid,
                &[primary_row(1, 10, 1)],
            )
            .unwrap();
        let leaf_12 = object_store
            .insert_leaf_object_v2_from_rows(
                active_epoch,
                12,
                1,
                root.header.pid,
                &[primary_row(2, 10, 2)],
            )
            .unwrap();
        let leaf_13 = object_store
            .insert_leaf_object_v2_from_rows(
                active_epoch,
                13,
                1,
                root.header.pid,
                &[primary_row(3, 10, 3)],
            )
            .unwrap();
        let delta =
            SpireDeltaPartitionObject::new(40, 1, 12, vec![delta_insert_row(4, 20, 1)]).unwrap();
        let delta_placement = object_store
            .insert_delta_object(active_epoch, &delta)
            .unwrap();
        let active_placements = vec![root_placement, leaf_11, leaf_12, leaf_13, delta_placement];
        let epoch_manifest = SpireEpochManifest {
            epoch: active_epoch,
            state: SpireEpochState::Published,
            consistency_mode: SpireConsistencyMode::Strict,
            published_at_micros: 1000,
            retain_until_micros: 2000,
            active_query_count: 0,
        };
        let object_manifest = SpireObjectManifest::from_entries(
            active_epoch,
            active_placements
                .iter()
                .map(manifest_entry_for)
                .collect::<Vec<_>>(),
        )
        .unwrap();
        let placement_directory =
            SpirePlacementDirectory::from_entries(active_epoch, active_placements).unwrap();
        let snapshot = SpirePublishedEpochSnapshot::new(
            &epoch_manifest,
            &object_manifest,
            &placement_directory,
        )
        .unwrap();
        let replacement_root = rewrite_routing_partition_for_leaf_replacement(
            &root,
            &[12],
            vec![
                replacement_child(21, vec![0.5, 0.5]),
                replacement_child(22, vec![-0.5, 0.5]),
            ],
            4,
        )
        .unwrap();
        let replacement_root_placement = object_store
            .insert_routing_object(new_epoch, &replacement_root)
            .unwrap();
        let replacement_leaf_21 = object_store
            .insert_leaf_object_v2_from_rows(new_epoch, 21, 1, root.header.pid, &[])
            .unwrap();
        let replacement_leaf_22 = object_store
            .insert_leaf_object_v2_from_rows(new_epoch, 22, 1, root.header.pid, &[])
            .unwrap();

        let replacement_directory = plan_replacement_epoch_placement_directory(
            &snapshot,
            &object_store,
            new_epoch,
            root.header.pid,
            replacement_root_placement,
            &[12],
            vec![replacement_leaf_21, replacement_leaf_22],
        )
        .unwrap();

        let pids = replacement_directory
            .entries
            .iter()
            .map(|entry| entry.pid)
            .collect::<Vec<_>>();
        assert_eq!(pids, vec![1, 11, 13, 21, 22]);
        assert!(replacement_directory
            .entries
            .iter()
            .all(|entry| entry.epoch == new_epoch));
        assert!(replacement_directory.get(12).is_none());
        assert!(replacement_directory.get(40).is_none());
        assert_eq!(
            object_store
                .read_object_header(placement_directory.get(12).unwrap())
                .unwrap()
                .pid,
            12
        );
    }

    #[test]
    fn replacement_epoch_draft_builds_manifest_and_publish_bundle() {
        let placement_directory = SpirePlacementDirectory::from_entries(
            8,
            vec![
                SpirePlacementEntry::local_single_store_available(8, 1, 12345, 4, tid(30, 1), 128),
                SpirePlacementEntry::local_single_store_available(8, 21, 12345, 1, tid(31, 1), 256),
            ],
        )
        .unwrap();
        let draft = build_replacement_epoch_draft(SpireReplacementEpochInput {
            epoch: 8,
            published_at_micros: 1000,
            retain_until_micros: 2000,
            consistency_mode: SpireConsistencyMode::Strict,
            placement_directory,
            placement_write_evidence: vec![
                SpirePublishPlacementWriteEvidence {
                    pid: 21,
                    placement_tid: tid(90, 2),
                },
                SpirePublishPlacementWriteEvidence {
                    pid: 1,
                    placement_tid: tid(90, 1),
                },
            ],
            next_pid: 30,
            next_local_vec_seq: 5,
        })
        .unwrap();

        assert_eq!(draft.epoch_manifest.epoch, 8);
        assert_eq!(
            draft.object_manifest.get(1).unwrap().placement_tid,
            tid(90, 1)
        );
        assert_eq!(
            draft.object_manifest.get(21).unwrap().placement_tid,
            tid(90, 2)
        );
        let root_control = draft.root_control_state(manifest_locators()).unwrap();
        assert_eq!(root_control.active_epoch, 8);
        assert_eq!(root_control.next_pid, 30);
        assert_eq!(root_control.next_local_vec_seq, 5);
        let encoded = draft.encode_publish_bundle(manifest_locators()).unwrap();
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
    }

    #[test]
    fn replacement_leaf_object_inputs_match_replacement_children() {
        let children = vec![
            replacement_child(21, vec![0.5, 0.5]),
            replacement_child(22, vec![-0.5, 0.5]),
        ];
        let inputs = vec![
            SpireReplacementLeafObjectInput {
                pid: 21,
                rows: vec![primary_row(1, 10, 1)],
            },
            SpireReplacementLeafObjectInput {
                pid: 22,
                rows: vec![primary_row(2, 10, 2)],
            },
        ];

        validate_replacement_leaf_object_inputs(&children, &inputs).unwrap();
    }

    #[test]
    fn replacement_leaf_object_inputs_reject_delta_flags_and_pid_mismatch() {
        let children = vec![replacement_child(21, vec![0.5, 0.5])];
        let with_delta = vec![SpireReplacementLeafObjectInput {
            pid: 21,
            rows: vec![delta_insert_row(1, 10, 1)],
        }];
        assert!(
            validate_replacement_leaf_object_inputs(&children, &with_delta)
                .unwrap_err()
                .contains("delta-insert")
        );

        let wrong_pid = vec![SpireReplacementLeafObjectInput {
            pid: 22,
            rows: vec![primary_row(1, 10, 1)],
        }];
        assert!(
            validate_replacement_leaf_object_inputs(&children, &wrong_pid)
                .unwrap_err()
                .contains("no replacement routing child")
        );
    }

    #[test]
    fn local_replacement_object_writer_persists_routing_and_leaf_objects() {
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let root = root_routing_object();
        let children = vec![
            replacement_child(21, vec![0.5, 0.5]),
            replacement_child(22, vec![-0.5, 0.5]),
        ];
        let replacement_root =
            rewrite_routing_partition_for_leaf_replacement(&root, &[12], children.clone(), 4)
                .unwrap();

        let placements = write_local_replacement_objects(
            8,
            &replacement_root,
            &children,
            1,
            vec![
                SpireReplacementLeafObjectInput {
                    pid: 22,
                    rows: vec![primary_row(2, 20, 2)],
                },
                SpireReplacementLeafObjectInput {
                    pid: 21,
                    rows: vec![primary_row(1, 20, 1)],
                },
            ],
            &mut object_store,
        )
        .unwrap();

        let stored_root = object_store
            .read_routing_object(&placements.parent_placement)
            .unwrap();
        let stored_root_children = stored_root.children().collect::<Vec<_>>();
        assert_eq!(placements.parent_placement.epoch, 8);
        assert_eq!(placements.parent_placement.pid, replacement_root.header.pid);
        assert_eq!(stored_root.header.published_epoch_backref, 8);
        assert_eq!(
            stored_root_children
                .iter()
                .map(|child| child.child_pid)
                .collect::<Vec<_>>(),
            vec![11, 21, 22, 13]
        );

        assert_eq!(
            placements
                .leaf_placements
                .iter()
                .map(|placement| placement.pid)
                .collect::<Vec<_>>(),
            vec![21, 22]
        );
        let first_leaf = object_store
            .read_leaf_object_v2(&placements.leaf_placements[0])
            .unwrap();
        let second_leaf = object_store
            .read_leaf_object_v2(&placements.leaf_placements[1])
            .unwrap();
        assert_eq!(
            first_leaf.meta.header.parent_pid,
            replacement_root.header.pid
        );
        assert_eq!(
            second_leaf.meta.header.parent_pid,
            replacement_root.header.pid
        );
        assert_eq!(
            first_leaf.assignment_rows().unwrap()[0].heap_tid,
            tid(20, 1)
        );
        assert_eq!(
            second_leaf.assignment_rows().unwrap()[0].heap_tid,
            tid(20, 2)
        );
    }

    #[test]
    fn local_scheduled_replacement_object_writer_persists_decision_bound_objects() {
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let root = root_routing_object();
        let decision = SpireLeafReplacementScheduleDecision {
            mode: SpireLeafReplacementScheduleMode::Split,
            active_epoch: 7,
            replaced_parent_pid: root.header.pid,
            affected_leaf_pids: vec![12],
            replacement_leaf_count: 2,
            reason: "test_split",
        };
        let children = vec![
            replacement_child(21, vec![0.5, 0.5]),
            replacement_child(22, vec![-0.5, 0.5]),
        ];
        let replacement_root =
            rewrite_scheduled_replacement_parent_routing(&root, &decision, children.clone(), 4)
                .unwrap();

        let placements = write_local_scheduled_replacement_objects(
            8,
            &replacement_root,
            &decision,
            &children,
            1,
            vec![
                SpireReplacementLeafObjectInput {
                    pid: 22,
                    rows: vec![primary_row(2, 20, 2)],
                },
                SpireReplacementLeafObjectInput {
                    pid: 21,
                    rows: vec![primary_row(1, 20, 1)],
                },
            ],
            &mut object_store,
        )
        .unwrap();

        assert_eq!(placements.parent_placement.pid, root.header.pid);
        assert_eq!(
            placements
                .leaf_placements
                .iter()
                .map(|placement| placement.pid)
                .collect::<Vec<_>>(),
            vec![21, 22]
        );
        let stored_root = object_store
            .read_routing_object(&placements.parent_placement)
            .unwrap();
        assert_eq!(
            stored_root
                .children()
                .map(|child| child.child_pid)
                .collect::<Vec<_>>(),
            vec![11, 21, 22, 13]
        );
    }

    #[test]
    fn local_scheduled_replacement_object_writer_rejects_parent_or_child_count_mismatch() {
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let root = root_routing_object();
        let decision = SpireLeafReplacementScheduleDecision {
            mode: SpireLeafReplacementScheduleMode::Split,
            active_epoch: 7,
            replaced_parent_pid: root.header.pid,
            affected_leaf_pids: vec![12],
            replacement_leaf_count: 2,
            reason: "test_split",
        };
        let wrong_parent_decision = SpireLeafReplacementScheduleDecision {
            replaced_parent_pid: 2,
            ..decision.clone()
        };
        let children = vec![
            replacement_child(21, vec![0.5, 0.5]),
            replacement_child(22, vec![-0.5, 0.5]),
        ];
        let replacement_root =
            rewrite_scheduled_replacement_parent_routing(&root, &decision, children.clone(), 4)
                .unwrap();

        assert!(write_local_scheduled_replacement_objects(
            9,
            &replacement_root,
            &decision,
            &children,
            1,
            Vec::new(),
            &mut object_store,
        )
        .unwrap_err()
        .contains("immediate successor"));

        assert!(write_local_scheduled_replacement_objects(
            8,
            &replacement_root,
            &wrong_parent_decision,
            &children,
            1,
            Vec::new(),
            &mut object_store,
        )
        .unwrap_err()
        .contains("does not match decision parent pid"));

        assert!(write_local_scheduled_replacement_objects(
            8,
            &replacement_root,
            &decision,
            &children[..1],
            1,
            Vec::new(),
            &mut object_store,
        )
        .unwrap_err()
        .contains("child count"));
    }

    #[test]
    fn replacement_epoch_draft_from_object_placements_builds_directory_and_manifest() {
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let active_epoch = 7;
        let new_epoch = 8;
        let root = root_routing_object();
        let root_placement = object_store
            .insert_routing_object(active_epoch, &root)
            .unwrap();
        let leaf_11 = object_store
            .insert_leaf_object_v2_from_rows(
                active_epoch,
                11,
                1,
                root.header.pid,
                &[primary_row(1, 10, 1)],
            )
            .unwrap();
        let leaf_12 = object_store
            .insert_leaf_object_v2_from_rows(
                active_epoch,
                12,
                1,
                root.header.pid,
                &[primary_row(2, 10, 2)],
            )
            .unwrap();
        let leaf_13 = object_store
            .insert_leaf_object_v2_from_rows(
                active_epoch,
                13,
                1,
                root.header.pid,
                &[primary_row(3, 10, 3)],
            )
            .unwrap();
        let delta =
            SpireDeltaPartitionObject::new(40, 1, 12, vec![delta_insert_row(4, 20, 1)]).unwrap();
        let delta_placement = object_store
            .insert_delta_object(active_epoch, &delta)
            .unwrap();
        let active_placements = vec![root_placement, leaf_11, leaf_12, leaf_13, delta_placement];
        let epoch_manifest = SpireEpochManifest {
            epoch: active_epoch,
            state: SpireEpochState::Published,
            consistency_mode: SpireConsistencyMode::Strict,
            published_at_micros: 1000,
            retain_until_micros: 2000,
            active_query_count: 0,
        };
        let object_manifest = SpireObjectManifest::from_entries(
            active_epoch,
            active_placements
                .iter()
                .map(manifest_entry_for)
                .collect::<Vec<_>>(),
        )
        .unwrap();
        let placement_directory =
            SpirePlacementDirectory::from_entries(active_epoch, active_placements).unwrap();
        let snapshot = SpirePublishedEpochSnapshot::new(
            &epoch_manifest,
            &object_manifest,
            &placement_directory,
        )
        .unwrap();
        let children = vec![
            replacement_child(21, vec![0.5, 0.5]),
            replacement_child(22, vec![-0.5, 0.5]),
        ];
        let replacement_root =
            rewrite_routing_partition_for_leaf_replacement(&root, &[12], children.clone(), 4)
                .unwrap();
        let replacement_object_placements = write_local_replacement_objects(
            new_epoch,
            &replacement_root,
            &children,
            2,
            vec![
                SpireReplacementLeafObjectInput {
                    pid: 21,
                    rows: vec![primary_row(5, 30, 1)],
                },
                SpireReplacementLeafObjectInput {
                    pid: 22,
                    rows: vec![primary_row(6, 30, 2)],
                },
            ],
            &mut object_store,
        )
        .unwrap();

        let draft = build_replacement_epoch_draft_from_object_placements(
            &snapshot,
            &object_store,
            SpireReplacementEpochObjectPlacementInput {
                epoch: new_epoch,
                published_at_micros: 3000,
                retain_until_micros: 4000,
                consistency_mode: SpireConsistencyMode::Strict,
                replaced_parent_pid: root.header.pid,
                affected_leaf_pids: vec![12],
                replacement_object_placements,
                placement_write_evidence: placement_write_evidence_for_pids(&[1, 11, 13, 21, 22]),
                next_pid: 30,
                next_local_vec_seq: 7,
            },
        )
        .unwrap();

        let active_pids = draft
            .placement_directory
            .entries
            .iter()
            .map(|entry| entry.pid)
            .collect::<Vec<_>>();
        assert_eq!(active_pids, vec![1, 11, 13, 21, 22]);
        assert!(draft.placement_directory.get(12).is_none());
        assert!(draft.placement_directory.get(40).is_none());
        assert_eq!(
            draft.object_manifest.get(21).unwrap().placement_tid,
            tid(90, 4)
        );
        assert_eq!(
            draft.object_manifest.get(22).unwrap().placement_tid,
            tid(90, 5)
        );
        let root_control = draft.root_control_state(manifest_locators()).unwrap();
        assert_eq!(root_control.active_epoch, new_epoch);
        assert_eq!(root_control.next_pid, 30);
        assert_eq!(root_control.next_local_vec_seq, 7);
        let stored_root = object_store
            .read_routing_object(draft.placement_directory.get(1).unwrap())
            .unwrap();
        assert_eq!(
            stored_root
                .children()
                .map(|child| child.child_pid)
                .collect::<Vec<_>>(),
            vec![11, 21, 22, 13]
        );
    }

    #[test]
    fn scheduled_replacement_epoch_draft_uses_decision_shape_for_publish_assembly() {
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let active_epoch = 7;
        let new_epoch = 8;
        let root = root_routing_object();
        let root_placement = object_store
            .insert_routing_object(active_epoch, &root)
            .unwrap();
        let leaf_11 = object_store
            .insert_leaf_object_v2_from_rows(
                active_epoch,
                11,
                1,
                root.header.pid,
                &[primary_row(1, 10, 1)],
            )
            .unwrap();
        let leaf_12 = object_store
            .insert_leaf_object_v2_from_rows(
                active_epoch,
                12,
                1,
                root.header.pid,
                &[primary_row(2, 10, 2)],
            )
            .unwrap();
        let leaf_13 = object_store
            .insert_leaf_object_v2_from_rows(
                active_epoch,
                13,
                1,
                root.header.pid,
                &[primary_row(3, 10, 3)],
            )
            .unwrap();
        let active_placements = vec![root_placement, leaf_11, leaf_12, leaf_13];
        let epoch_manifest = SpireEpochManifest {
            epoch: active_epoch,
            state: SpireEpochState::Published,
            consistency_mode: SpireConsistencyMode::Strict,
            published_at_micros: 1000,
            retain_until_micros: 2000,
            active_query_count: 0,
        };
        let object_manifest = SpireObjectManifest::from_entries(
            active_epoch,
            active_placements
                .iter()
                .map(manifest_entry_for)
                .collect::<Vec<_>>(),
        )
        .unwrap();
        let placement_directory =
            SpirePlacementDirectory::from_entries(active_epoch, active_placements).unwrap();
        let snapshot = SpirePublishedEpochSnapshot::new(
            &epoch_manifest,
            &object_manifest,
            &placement_directory,
        )
        .unwrap();
        let decision = SpireLeafReplacementScheduleDecision {
            mode: SpireLeafReplacementScheduleMode::Split,
            active_epoch,
            replaced_parent_pid: root.header.pid,
            affected_leaf_pids: vec![12],
            replacement_leaf_count: 2,
            reason: "test_split",
        };
        let children = vec![
            replacement_child(21, vec![0.5, 0.5]),
            replacement_child(22, vec![-0.5, 0.5]),
        ];
        let replacement_root =
            rewrite_scheduled_replacement_parent_routing(&root, &decision, children.clone(), 4)
                .unwrap();
        let replacement_object_placements = write_local_replacement_objects(
            new_epoch,
            &replacement_root,
            &children,
            2,
            vec![
                SpireReplacementLeafObjectInput {
                    pid: 21,
                    rows: vec![primary_row(5, 30, 1)],
                },
                SpireReplacementLeafObjectInput {
                    pid: 22,
                    rows: vec![primary_row(6, 30, 2)],
                },
            ],
            &mut object_store,
        )
        .unwrap();

        let draft = build_scheduled_replacement_epoch_draft_from_object_placements(
            &snapshot,
            &object_store,
            &decision,
            SpireScheduledReplacementEpochObjectPlacementInput {
                epoch: new_epoch,
                published_at_micros: 3000,
                retain_until_micros: 4000,
                consistency_mode: SpireConsistencyMode::Strict,
                replacement_object_placements,
                placement_write_evidence: placement_write_evidence_for_pids(&[1, 11, 13, 21, 22]),
                next_pid: 30,
                next_local_vec_seq: 7,
            },
        )
        .unwrap();

        assert_eq!(
            draft
                .placement_directory
                .entries
                .iter()
                .map(|entry| entry.pid)
                .collect::<Vec<_>>(),
            vec![1, 11, 13, 21, 22]
        );
        assert!(draft.placement_directory.get(12).is_none());
        assert_eq!(
            draft.object_manifest.get(21).unwrap().placement_tid,
            tid(90, 4)
        );
        assert_eq!(
            draft.object_manifest.get(22).unwrap().placement_tid,
            tid(90, 5)
        );
    }

    #[test]
    fn scheduled_replacement_epoch_draft_rejects_epoch_or_placement_count_mismatch() {
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let active_epoch = 7;
        let root = root_routing_object();
        let root_placement = object_store
            .insert_routing_object(active_epoch, &root)
            .unwrap();
        let leaf_12 = object_store
            .insert_leaf_object_v2_from_rows(
                active_epoch,
                12,
                1,
                root.header.pid,
                &[primary_row(2, 10, 2)],
            )
            .unwrap();
        let active_placements = vec![root_placement, leaf_12];
        let epoch_manifest = SpireEpochManifest {
            epoch: active_epoch,
            state: SpireEpochState::Published,
            consistency_mode: SpireConsistencyMode::Strict,
            published_at_micros: 1000,
            retain_until_micros: 2000,
            active_query_count: 0,
        };
        let object_manifest = SpireObjectManifest::from_entries(
            active_epoch,
            active_placements
                .iter()
                .map(manifest_entry_for)
                .collect::<Vec<_>>(),
        )
        .unwrap();
        let placement_directory =
            SpirePlacementDirectory::from_entries(active_epoch, active_placements).unwrap();
        let snapshot = SpirePublishedEpochSnapshot::new(
            &epoch_manifest,
            &object_manifest,
            &placement_directory,
        )
        .unwrap();
        let decision = SpireLeafReplacementScheduleDecision {
            mode: SpireLeafReplacementScheduleMode::Split,
            active_epoch,
            replaced_parent_pid: root.header.pid,
            affected_leaf_pids: vec![12],
            replacement_leaf_count: 2,
            reason: "test_split",
        };
        let replacement_parent_placement =
            SpirePlacementEntry::local_single_store_available(8, 1, 12345, 4, tid(40, 1), 128);
        let replacement_leaf_21 =
            SpirePlacementEntry::local_single_store_available(8, 21, 12345, 2, tid(41, 1), 256);
        let replacement_leaf_22 =
            SpirePlacementEntry::local_single_store_available(8, 22, 12345, 2, tid(42, 1), 256);
        let placements = super::SpireReplacementObjectPlacements {
            parent_placement: replacement_parent_placement,
            leaf_placements: vec![replacement_leaf_21, replacement_leaf_22],
        };
        let wrong_epoch_decision = SpireLeafReplacementScheduleDecision {
            active_epoch: active_epoch + 1,
            ..decision.clone()
        };

        assert!(
            build_scheduled_replacement_epoch_draft_from_object_placements(
                &snapshot,
                &object_store,
                &wrong_epoch_decision,
                SpireScheduledReplacementEpochObjectPlacementInput {
                    epoch: 8,
                    published_at_micros: 3000,
                    retain_until_micros: 4000,
                    consistency_mode: SpireConsistencyMode::Strict,
                    replacement_object_placements: placements.clone(),
                    placement_write_evidence: placement_write_evidence_for_pids(&[1, 21, 22]),
                    next_pid: 30,
                    next_local_vec_seq: 7,
                },
            )
            .unwrap_err()
            .contains("snapshot epoch")
        );

        assert!(
            build_scheduled_replacement_epoch_draft_from_object_placements(
                &snapshot,
                &object_store,
                &decision,
                SpireScheduledReplacementEpochObjectPlacementInput {
                    epoch: 9,
                    published_at_micros: 3000,
                    retain_until_micros: 4000,
                    consistency_mode: SpireConsistencyMode::Strict,
                    replacement_object_placements: placements.clone(),
                    placement_write_evidence: placement_write_evidence_for_pids(&[1, 21, 22]),
                    next_pid: 30,
                    next_local_vec_seq: 7,
                },
            )
            .unwrap_err()
            .contains("immediate successor")
        );

        assert!(
            build_scheduled_replacement_epoch_draft_from_object_placements(
                &snapshot,
                &object_store,
                &decision,
                SpireScheduledReplacementEpochObjectPlacementInput {
                    epoch: 8,
                    published_at_micros: 3000,
                    retain_until_micros: 4000,
                    consistency_mode: SpireConsistencyMode::Degraded,
                    replacement_object_placements: placements.clone(),
                    placement_write_evidence: placement_write_evidence_for_pids(&[1, 21, 22]),
                    next_pid: 30,
                    next_local_vec_seq: 7,
                },
            )
            .unwrap_err()
            .contains("consistency mode")
        );

        let mut missing_leaf_placement = placements;
        missing_leaf_placement.leaf_placements.pop();
        assert!(
            build_scheduled_replacement_epoch_draft_from_object_placements(
                &snapshot,
                &object_store,
                &decision,
                SpireScheduledReplacementEpochObjectPlacementInput {
                    epoch: 8,
                    published_at_micros: 3000,
                    retain_until_micros: 4000,
                    consistency_mode: SpireConsistencyMode::Strict,
                    replacement_object_placements: missing_leaf_placement,
                    placement_write_evidence: placement_write_evidence_for_pids(&[1, 21]),
                    next_pid: 30,
                    next_local_vec_seq: 7,
                },
            )
            .unwrap_err()
            .contains("leaf placement count")
        );
    }

    #[test]
    fn scheduled_replacement_pid_plan_output_accepts_matching_placements_and_cursor() {
        let decision = SpireLeafReplacementScheduleDecision {
            mode: SpireLeafReplacementScheduleMode::Split,
            active_epoch: 7,
            replaced_parent_pid: 1,
            affected_leaf_pids: vec![12],
            replacement_leaf_count: 2,
            reason: "test_split",
        };
        let pid_plan = SpireLeafReplacementPidPlan {
            replacement_pids: vec![21, 22],
            reuses_existing_pid: false,
            next_pid: 23,
        };
        let placements = SpireReplacementObjectPlacements {
            parent_placement: SpirePlacementEntry::local_single_store_available(
                8,
                1,
                12345,
                4,
                tid(40, 1),
                128,
            ),
            leaf_placements: vec![
                SpirePlacementEntry::local_single_store_available(8, 21, 12345, 2, tid(41, 1), 256),
                SpirePlacementEntry::local_single_store_available(8, 22, 12345, 2, tid(42, 1), 256),
            ],
        };

        validate_scheduled_replacement_pid_plan_output(&decision, &pid_plan, &placements, 23)
            .unwrap();
    }

    #[test]
    fn scheduled_replacement_pid_plan_output_rejects_mismatched_outputs() {
        let decision = SpireLeafReplacementScheduleDecision {
            mode: SpireLeafReplacementScheduleMode::Split,
            active_epoch: 7,
            replaced_parent_pid: 1,
            affected_leaf_pids: vec![12],
            replacement_leaf_count: 2,
            reason: "test_split",
        };
        let pid_plan = SpireLeafReplacementPidPlan {
            replacement_pids: vec![21, 22],
            reuses_existing_pid: false,
            next_pid: 23,
        };
        let mut placements = SpireReplacementObjectPlacements {
            parent_placement: SpirePlacementEntry::local_single_store_available(
                8,
                1,
                12345,
                4,
                tid(40, 1),
                128,
            ),
            leaf_placements: vec![
                SpirePlacementEntry::local_single_store_available(8, 22, 12345, 2, tid(42, 1), 256),
                SpirePlacementEntry::local_single_store_available(8, 21, 12345, 2, tid(41, 1), 256),
            ],
        };

        assert!(validate_scheduled_replacement_pid_plan_output(
            &decision,
            &pid_plan,
            &placements,
            23
        )
        .unwrap_err()
        .contains("do not match pid plan"));

        placements.parent_placement.pid = 2;
        placements.leaf_placements.swap(0, 1);
        assert!(validate_scheduled_replacement_pid_plan_output(
            &decision,
            &pid_plan,
            &placements,
            23
        )
        .unwrap_err()
        .contains("parent placement pid"));

        placements.parent_placement.pid = 1;
        assert!(validate_scheduled_replacement_pid_plan_output(
            &decision,
            &pid_plan,
            &placements,
            24
        )
        .unwrap_err()
        .contains("next_pid"));
    }

    #[test]
    fn local_scheduled_replacement_execution_writes_objects_and_builds_draft() {
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let active_epoch = 7;
        let new_epoch = 8;
        let root = root_routing_object();
        let root_placement = object_store
            .insert_routing_object(active_epoch, &root)
            .unwrap();
        let leaf_11 = object_store
            .insert_leaf_object_v2_from_rows(
                active_epoch,
                11,
                1,
                root.header.pid,
                &[primary_row(1, 10, 1)],
            )
            .unwrap();
        let leaf_12 = object_store
            .insert_leaf_object_v2_from_rows(
                active_epoch,
                12,
                1,
                root.header.pid,
                &[primary_row(2, 10, 2)],
            )
            .unwrap();
        let leaf_13 = object_store
            .insert_leaf_object_v2_from_rows(
                active_epoch,
                13,
                1,
                root.header.pid,
                &[primary_row(3, 10, 3)],
            )
            .unwrap();
        let active_placements = vec![root_placement, leaf_11, leaf_12, leaf_13];
        let epoch_manifest = SpireEpochManifest {
            epoch: active_epoch,
            state: SpireEpochState::Published,
            consistency_mode: SpireConsistencyMode::Strict,
            published_at_micros: 1000,
            retain_until_micros: 2000,
            active_query_count: 0,
        };
        let object_manifest = SpireObjectManifest::from_entries(
            active_epoch,
            active_placements
                .iter()
                .map(manifest_entry_for)
                .collect::<Vec<_>>(),
        )
        .unwrap();
        let placement_directory =
            SpirePlacementDirectory::from_entries(active_epoch, active_placements).unwrap();
        let snapshot = SpirePublishedEpochSnapshot::new(
            &epoch_manifest,
            &object_manifest,
            &placement_directory,
        )
        .unwrap();
        let decision = SpireLeafReplacementScheduleDecision {
            mode: SpireLeafReplacementScheduleMode::Split,
            active_epoch,
            replaced_parent_pid: root.header.pid,
            affected_leaf_pids: vec![12],
            replacement_leaf_count: 2,
            reason: "test_split",
        };
        let pid_plan = SpireLeafReplacementPidPlan {
            replacement_pids: vec![21, 22],
            reuses_existing_pid: false,
            next_pid: 23,
        };
        let publish_plan = SpireScheduledReplacementPublishPlan {
            epoch: new_epoch,
            consistency_mode: SpireConsistencyMode::Strict,
            next_pid: 23,
            next_local_vec_seq: 7,
        };
        let replacement_children = vec![
            replacement_child(21, vec![0.5, 0.5]),
            replacement_child(22, vec![-0.5, 0.5]),
        ];
        let replacement_parent = rewrite_scheduled_replacement_parent_routing(
            &root,
            &decision,
            replacement_children.clone(),
            4,
        )
        .unwrap();

        assert!(build_local_scheduled_replacement_epoch_draft(
            &snapshot,
            &decision,
            &pid_plan,
            &SpireScheduledReplacementPublishPlan {
                consistency_mode: SpireConsistencyMode::Degraded,
                ..publish_plan.clone()
            },
            SpireLocalScheduledReplacementExecutionInput {
                epoch: new_epoch,
                published_at_micros: 3000,
                retain_until_micros: 4000,
                consistency_mode: SpireConsistencyMode::Degraded,
                replacement_parent: replacement_parent.clone(),
                replacement_children: replacement_children.clone(),
                leaf_object_version: 2,
                leaf_inputs: vec![
                    SpireReplacementLeafObjectInput {
                        pid: 21,
                        rows: vec![primary_row(5, 30, 1)],
                    },
                    SpireReplacementLeafObjectInput {
                        pid: 22,
                        rows: vec![primary_row(6, 30, 2)],
                    },
                ],
                placement_write_evidence: placement_write_evidence_for_pids(&[1, 11, 13, 21, 22]),
                next_local_vec_seq: 7,
            },
            &mut object_store,
        )
        .unwrap_err()
        .contains("active snapshot consistency mode"));

        let draft = build_local_scheduled_replacement_epoch_draft(
            &snapshot,
            &decision,
            &pid_plan,
            &publish_plan,
            SpireLocalScheduledReplacementExecutionInput {
                epoch: new_epoch,
                published_at_micros: 3000,
                retain_until_micros: 4000,
                consistency_mode: SpireConsistencyMode::Strict,
                replacement_parent,
                replacement_children,
                leaf_object_version: 2,
                leaf_inputs: vec![
                    SpireReplacementLeafObjectInput {
                        pid: 21,
                        rows: vec![primary_row(5, 30, 1)],
                    },
                    SpireReplacementLeafObjectInput {
                        pid: 22,
                        rows: vec![primary_row(6, 30, 2)],
                    },
                ],
                placement_write_evidence: placement_write_evidence_for_pids(&[1, 11, 13, 21, 22]),
                next_local_vec_seq: 7,
            },
            &mut object_store,
        )
        .unwrap();

        assert_eq!(draft.next_pid, 23);
        assert_eq!(draft.next_local_vec_seq, 7);
        assert_eq!(
            draft
                .placement_directory
                .entries
                .iter()
                .map(|entry| entry.pid)
                .collect::<Vec<_>>(),
            vec![1, 11, 13, 21, 22]
        );
    }

    #[test]
    fn local_selected_scheduled_replacement_epoch_draft_uses_lock_plan() {
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let root = root_routing_object();
        let fixture = scheduled_replacement_snapshot_fixture(&mut object_store, 7, &root);
        let snapshot = fixture.snapshot();
        let selected = SpireSelectedScheduledReplacementPublishLockPlan {
            decision: scheduled_split_decision(7),
            lock_plan: SpireScheduledReplacementPublishLockPlan {
                pid_plan: SpireLeafReplacementPidPlan {
                    replacement_pids: vec![21, 22],
                    reuses_existing_pid: false,
                    next_pid: 23,
                },
                publish_plan: SpireScheduledReplacementPublishPlan {
                    epoch: 8,
                    consistency_mode: SpireConsistencyMode::Strict,
                    next_pid: 23,
                    next_local_vec_seq: 7,
                },
            },
        };
        let input = build_local_selected_scheduled_split_replacement_execution_input(
            &selected,
            &root,
            vec![vec![0.5, 0.5], vec![-0.5, 0.5]],
            vec![
                SpireReplacementLeafObjectInput {
                    pid: 22,
                    rows: vec![primary_row(6, 30, 2)],
                },
                SpireReplacementLeafObjectInput {
                    pid: 21,
                    rows: vec![primary_row(5, 30, 1)],
                },
            ],
            4,
            2,
            3000,
            4000,
            placement_write_evidence_for_pids(&[1, 11, 13, 21, 22]),
        )
        .unwrap();

        let draft = build_local_selected_scheduled_replacement_epoch_draft(
            &snapshot,
            &selected,
            input,
            &mut object_store,
        )
        .unwrap();

        assert_eq!(draft.next_pid, 23);
        assert_eq!(draft.next_local_vec_seq, 7);
        assert_eq!(
            draft
                .placement_directory
                .entries
                .iter()
                .map(|entry| entry.pid)
                .collect::<Vec<_>>(),
            vec![1, 11, 13, 21, 22]
        );
    }

    #[test]
    fn local_selected_scheduled_replacement_epoch_draft_rejects_snapshot_drift() {
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let root = root_routing_object();
        let fixture = scheduled_replacement_snapshot_fixture(&mut object_store, 7, &root);
        let snapshot = fixture.snapshot();
        let selected = SpireSelectedScheduledReplacementPublishLockPlan {
            decision: scheduled_split_decision(8),
            lock_plan: SpireScheduledReplacementPublishLockPlan {
                pid_plan: SpireLeafReplacementPidPlan {
                    replacement_pids: vec![21, 22],
                    reuses_existing_pid: false,
                    next_pid: 23,
                },
                publish_plan: SpireScheduledReplacementPublishPlan {
                    epoch: 9,
                    consistency_mode: SpireConsistencyMode::Strict,
                    next_pid: 23,
                    next_local_vec_seq: 7,
                },
            },
        };
        let input = build_local_selected_scheduled_split_replacement_execution_input(
            &selected,
            &root,
            vec![vec![0.5, 0.5], vec![-0.5, 0.5]],
            vec![
                SpireReplacementLeafObjectInput {
                    pid: 21,
                    rows: vec![primary_row(5, 30, 1)],
                },
                SpireReplacementLeafObjectInput {
                    pid: 22,
                    rows: vec![primary_row(6, 30, 2)],
                },
            ],
            4,
            2,
            3000,
            4000,
            placement_write_evidence_for_pids(&[1, 11, 13, 21, 22]),
        )
        .unwrap();

        assert!(build_local_selected_scheduled_replacement_epoch_draft(
            &snapshot,
            &selected,
            input,
            &mut object_store,
        )
        .unwrap_err()
        .contains("snapshot epoch"));
    }

    #[test]
    fn local_selected_scheduled_replacement_draft_inputs_validate_bundle() {
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let root = root_routing_object();
        let fixture = scheduled_replacement_snapshot_fixture(&mut object_store, 7, &root);
        let snapshot = fixture.snapshot();
        let selected = SpireSelectedScheduledReplacementPublishLockPlan {
            decision: scheduled_split_decision(7),
            lock_plan: SpireScheduledReplacementPublishLockPlan {
                pid_plan: SpireLeafReplacementPidPlan {
                    replacement_pids: vec![21, 22],
                    reuses_existing_pid: false,
                    next_pid: 23,
                },
                publish_plan: SpireScheduledReplacementPublishPlan {
                    epoch: 8,
                    consistency_mode: SpireConsistencyMode::Strict,
                    next_pid: 23,
                    next_local_vec_seq: 7,
                },
            },
        };
        let input = build_local_selected_scheduled_split_replacement_execution_input(
            &selected,
            &root,
            vec![vec![0.5, 0.5], vec![-0.5, 0.5]],
            vec![
                SpireReplacementLeafObjectInput {
                    pid: 21,
                    rows: vec![primary_row(5, 30, 1)],
                },
                SpireReplacementLeafObjectInput {
                    pid: 22,
                    rows: vec![primary_row(6, 30, 2)],
                },
            ],
            4,
            2,
            3000,
            4000,
            placement_write_evidence_for_pids(&[1, 11, 13, 21, 22]),
        )
        .unwrap();

        validate_local_selected_scheduled_replacement_draft_inputs(&snapshot, &selected, &input)
            .unwrap();
    }

    #[test]
    fn local_selected_scheduled_replacement_draft_inputs_reject_drift() {
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let root = root_routing_object();
        let fixture = scheduled_replacement_snapshot_fixture(&mut object_store, 7, &root);
        let snapshot = fixture.snapshot();
        let selected = SpireSelectedScheduledReplacementPublishLockPlan {
            decision: scheduled_split_decision(7),
            lock_plan: SpireScheduledReplacementPublishLockPlan {
                pid_plan: SpireLeafReplacementPidPlan {
                    replacement_pids: vec![21, 22],
                    reuses_existing_pid: false,
                    next_pid: 23,
                },
                publish_plan: SpireScheduledReplacementPublishPlan {
                    epoch: 8,
                    consistency_mode: SpireConsistencyMode::Strict,
                    next_pid: 23,
                    next_local_vec_seq: 7,
                },
            },
        };
        let input = build_local_selected_scheduled_split_replacement_execution_input(
            &selected,
            &root,
            vec![vec![0.5, 0.5], vec![-0.5, 0.5]],
            vec![
                SpireReplacementLeafObjectInput {
                    pid: 21,
                    rows: vec![primary_row(5, 30, 1)],
                },
                SpireReplacementLeafObjectInput {
                    pid: 22,
                    rows: vec![primary_row(6, 30, 2)],
                },
            ],
            4,
            2,
            3000,
            4000,
            placement_write_evidence_for_pids(&[1, 11, 13, 21, 22]),
        )
        .unwrap();
        let mut stale_input = input.clone();
        stale_input.next_local_vec_seq = 8;
        assert!(validate_local_selected_scheduled_replacement_draft_inputs(
            &snapshot,
            &selected,
            &stale_input,
        )
        .unwrap_err()
        .contains("next_local_vec_seq"));

        let stale_selected = SpireSelectedScheduledReplacementPublishLockPlan {
            decision: scheduled_split_decision(8),
            lock_plan: SpireScheduledReplacementPublishLockPlan {
                publish_plan: SpireScheduledReplacementPublishPlan {
                    epoch: 9,
                    ..selected.lock_plan.publish_plan
                },
                ..selected.lock_plan
            },
        };
        let stale_selected_input =
            build_local_selected_scheduled_split_replacement_execution_input(
                &stale_selected,
                &root,
                vec![vec![0.5, 0.5], vec![-0.5, 0.5]],
                vec![
                    SpireReplacementLeafObjectInput {
                        pid: 21,
                        rows: vec![primary_row(5, 30, 1)],
                    },
                    SpireReplacementLeafObjectInput {
                        pid: 22,
                        rows: vec![primary_row(6, 30, 2)],
                    },
                ],
                4,
                2,
                3000,
                4000,
                placement_write_evidence_for_pids(&[1, 11, 13, 21, 22]),
            )
            .unwrap();
        assert!(validate_local_selected_scheduled_replacement_draft_inputs(
            &snapshot,
            &stale_selected,
            &stale_selected_input,
        )
        .unwrap_err()
        .contains("snapshot epoch"));
    }

    #[test]
    fn local_selected_scheduled_split_replacement_epoch_draft_builds_input_and_draft() {
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let root = root_routing_object();
        let fixture = scheduled_replacement_snapshot_fixture(&mut object_store, 7, &root);
        let snapshot = fixture.snapshot();
        let selected = SpireSelectedScheduledReplacementPublishLockPlan {
            decision: scheduled_split_decision(7),
            lock_plan: SpireScheduledReplacementPublishLockPlan {
                pid_plan: SpireLeafReplacementPidPlan {
                    replacement_pids: vec![21, 22],
                    reuses_existing_pid: false,
                    next_pid: 23,
                },
                publish_plan: SpireScheduledReplacementPublishPlan {
                    epoch: 8,
                    consistency_mode: SpireConsistencyMode::Strict,
                    next_pid: 23,
                    next_local_vec_seq: 7,
                },
            },
        };

        let draft = build_local_selected_scheduled_split_replacement_epoch_draft(
            &snapshot,
            &selected,
            &root,
            vec![vec![0.5, 0.5], vec![-0.5, 0.5]],
            vec![
                SpireReplacementLeafObjectInput {
                    pid: 22,
                    rows: vec![primary_row(6, 30, 2)],
                },
                SpireReplacementLeafObjectInput {
                    pid: 21,
                    rows: vec![primary_row(5, 30, 1)],
                },
            ],
            4,
            2,
            3000,
            4000,
            placement_write_evidence_for_pids(&[1, 11, 13, 21, 22]),
            &mut object_store,
        )
        .unwrap();

        assert_eq!(draft.next_pid, 23);
        assert_eq!(draft.next_local_vec_seq, 7);
        assert_eq!(
            draft
                .placement_directory
                .entries
                .iter()
                .map(|entry| entry.pid)
                .collect::<Vec<_>>(),
            vec![1, 11, 13, 21, 22]
        );
    }

    #[test]
    fn local_selected_scheduled_split_replacement_epoch_draft_rejects_merge_plan() {
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let root = root_routing_object();
        let fixture = scheduled_replacement_snapshot_fixture(&mut object_store, 7, &root);
        let snapshot = fixture.snapshot();
        let selected = SpireSelectedScheduledReplacementPublishLockPlan {
            decision: SpireLeafReplacementScheduleDecision {
                mode: SpireLeafReplacementScheduleMode::Merge,
                active_epoch: 7,
                replaced_parent_pid: 1,
                affected_leaf_pids: vec![11, 12],
                replacement_leaf_count: 1,
                reason: "test_merge",
            },
            lock_plan: SpireScheduledReplacementPublishLockPlan {
                pid_plan: SpireLeafReplacementPidPlan {
                    replacement_pids: vec![21],
                    reuses_existing_pid: false,
                    next_pid: 22,
                },
                publish_plan: SpireScheduledReplacementPublishPlan {
                    epoch: 8,
                    consistency_mode: SpireConsistencyMode::Strict,
                    next_pid: 22,
                    next_local_vec_seq: 7,
                },
            },
        };

        assert!(
            build_local_selected_scheduled_split_replacement_epoch_draft(
                &snapshot,
                &selected,
                &root,
                vec![vec![0.5, 0.5]],
                vec![SpireReplacementLeafObjectInput {
                    pid: 21,
                    rows: vec![primary_row(5, 30, 1)],
                }],
                4,
                2,
                3000,
                4000,
                placement_write_evidence_for_pids(&[1, 21]),
                &mut object_store,
            )
            .unwrap_err()
            .contains("split decision")
        );
    }

    #[test]
    fn local_selected_scheduled_merge_replacement_epoch_draft_builds_input_and_draft() {
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let root = root_routing_object();
        let fixture = scheduled_replacement_snapshot_fixture(&mut object_store, 7, &root);
        let snapshot = fixture.snapshot();
        let selected = SpireSelectedScheduledReplacementPublishLockPlan {
            decision: SpireLeafReplacementScheduleDecision {
                mode: SpireLeafReplacementScheduleMode::Merge,
                active_epoch: 7,
                replaced_parent_pid: 1,
                affected_leaf_pids: vec![11, 12],
                replacement_leaf_count: 1,
                reason: "test_merge",
            },
            lock_plan: SpireScheduledReplacementPublishLockPlan {
                pid_plan: SpireLeafReplacementPidPlan {
                    replacement_pids: vec![21],
                    reuses_existing_pid: false,
                    next_pid: 22,
                },
                publish_plan: SpireScheduledReplacementPublishPlan {
                    epoch: 8,
                    consistency_mode: SpireConsistencyMode::Strict,
                    next_pid: 22,
                    next_local_vec_seq: 7,
                },
            },
        };
        let rows = vec![
            leaf_snapshot_row(11, 1, 3, false, true),
            leaf_snapshot_row(12, 1, 1, false, true),
        ];

        let draft = build_local_selected_scheduled_merge_replacement_epoch_draft(
            &snapshot,
            &selected,
            &root,
            &rows,
            vec![
                SpireReplacementLeafRows {
                    base_pid: 11,
                    rows: vec![primary_row(1, 10, 1)],
                },
                SpireReplacementLeafRows {
                    base_pid: 12,
                    rows: vec![primary_row(2, 10, 2)],
                },
            ],
            4,
            2,
            3000,
            4000,
            placement_write_evidence_for_pids(&[1, 13, 21]),
            &mut object_store,
        )
        .unwrap();

        assert_eq!(draft.next_pid, 22);
        assert_eq!(draft.next_local_vec_seq, 7);
        assert_eq!(
            draft
                .placement_directory
                .entries
                .iter()
                .map(|entry| entry.pid)
                .collect::<Vec<_>>(),
            vec![1, 13, 21]
        );
    }

    #[test]
    fn local_selected_scheduled_merge_replacement_epoch_draft_rejects_split_plan() {
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let root = root_routing_object();
        let fixture = scheduled_replacement_snapshot_fixture(&mut object_store, 7, &root);
        let snapshot = fixture.snapshot();
        let selected = SpireSelectedScheduledReplacementPublishLockPlan {
            decision: scheduled_split_decision(7),
            lock_plan: SpireScheduledReplacementPublishLockPlan {
                pid_plan: SpireLeafReplacementPidPlan {
                    replacement_pids: vec![21, 22],
                    reuses_existing_pid: false,
                    next_pid: 23,
                },
                publish_plan: SpireScheduledReplacementPublishPlan {
                    epoch: 8,
                    consistency_mode: SpireConsistencyMode::Strict,
                    next_pid: 23,
                    next_local_vec_seq: 7,
                },
            },
        };
        let rows = vec![leaf_snapshot_row(12, 1, 100, true, false)];

        assert!(
            build_local_selected_scheduled_merge_replacement_epoch_draft(
                &snapshot,
                &selected,
                &root,
                &rows,
                vec![SpireReplacementLeafRows {
                    base_pid: 12,
                    rows: vec![primary_row(1, 10, 1)],
                }],
                4,
                2,
                3000,
                4000,
                placement_write_evidence_for_pids(&[1, 11, 13, 21, 22]),
                &mut object_store,
            )
            .unwrap_err()
            .contains("merge decision")
        );
    }

    #[test]
    fn local_selected_scheduled_merge_replacement_epoch_draft_from_snapshot_loads_inputs() {
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let root = root_routing_object();
        let fixture = scheduled_replacement_snapshot_fixture(&mut object_store, 7, &root);
        let snapshot = fixture.snapshot();
        let selected = SpireSelectedScheduledReplacementPublishLockPlan {
            decision: SpireLeafReplacementScheduleDecision {
                mode: SpireLeafReplacementScheduleMode::Merge,
                active_epoch: 7,
                replaced_parent_pid: 1,
                affected_leaf_pids: vec![11, 12],
                replacement_leaf_count: 1,
                reason: "test_merge",
            },
            lock_plan: SpireScheduledReplacementPublishLockPlan {
                pid_plan: SpireLeafReplacementPidPlan {
                    replacement_pids: vec![21],
                    reuses_existing_pid: false,
                    next_pid: 22,
                },
                publish_plan: SpireScheduledReplacementPublishPlan {
                    epoch: 8,
                    consistency_mode: SpireConsistencyMode::Strict,
                    next_pid: 22,
                    next_local_vec_seq: 7,
                },
            },
        };
        let rows = vec![
            leaf_snapshot_row(11, 1, 3, false, true),
            leaf_snapshot_row(12, 1, 1, false, true),
        ];

        let draft = build_local_selected_scheduled_merge_replacement_epoch_draft_from_snapshot(
            &snapshot,
            &selected,
            &rows,
            4,
            2,
            3000,
            4000,
            placement_write_evidence_for_pids(&[1, 13, 21]),
            &mut object_store,
        )
        .unwrap();

        assert_eq!(draft.next_pid, 22);
        assert_eq!(
            draft
                .placement_directory
                .entries
                .iter()
                .map(|entry| entry.pid)
                .collect::<Vec<_>>(),
            vec![1, 13, 21]
        );
    }

    #[test]
    fn local_selected_scheduled_merge_replacement_epoch_draft_from_snapshot_rejects_split_plan() {
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let root = root_routing_object();
        let fixture = scheduled_replacement_snapshot_fixture(&mut object_store, 7, &root);
        let snapshot = fixture.snapshot();
        let selected = SpireSelectedScheduledReplacementPublishLockPlan {
            decision: scheduled_split_decision(7),
            lock_plan: SpireScheduledReplacementPublishLockPlan {
                pid_plan: SpireLeafReplacementPidPlan {
                    replacement_pids: vec![21, 22],
                    reuses_existing_pid: false,
                    next_pid: 23,
                },
                publish_plan: SpireScheduledReplacementPublishPlan {
                    epoch: 8,
                    consistency_mode: SpireConsistencyMode::Strict,
                    next_pid: 23,
                    next_local_vec_seq: 7,
                },
            },
        };
        let rows = vec![leaf_snapshot_row(12, 1, 100, true, false)];

        assert!(
            build_local_selected_scheduled_merge_replacement_epoch_draft_from_snapshot(
                &snapshot,
                &selected,
                &rows,
                4,
                2,
                3000,
                4000,
                placement_write_evidence_for_pids(&[1, 11, 13, 21, 22]),
                &mut object_store,
            )
            .unwrap_err()
            .contains("merge decision")
        );
    }

    #[test]
    fn local_scheduled_replacement_execution_rejects_children_outside_pid_plan_order() {
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let active_epoch = 7;
        let root = root_routing_object();
        let root_placement = object_store
            .insert_routing_object(active_epoch, &root)
            .unwrap();
        let leaf_12 = object_store
            .insert_leaf_object_v2_from_rows(
                active_epoch,
                12,
                1,
                root.header.pid,
                &[primary_row(2, 10, 2)],
            )
            .unwrap();
        let active_placements = vec![root_placement, leaf_12];
        let epoch_manifest = SpireEpochManifest {
            epoch: active_epoch,
            state: SpireEpochState::Published,
            consistency_mode: SpireConsistencyMode::Strict,
            published_at_micros: 1000,
            retain_until_micros: 2000,
            active_query_count: 0,
        };
        let object_manifest = SpireObjectManifest::from_entries(
            active_epoch,
            active_placements
                .iter()
                .map(manifest_entry_for)
                .collect::<Vec<_>>(),
        )
        .unwrap();
        let placement_directory =
            SpirePlacementDirectory::from_entries(active_epoch, active_placements).unwrap();
        let snapshot = SpirePublishedEpochSnapshot::new(
            &epoch_manifest,
            &object_manifest,
            &placement_directory,
        )
        .unwrap();
        let decision = SpireLeafReplacementScheduleDecision {
            mode: SpireLeafReplacementScheduleMode::Split,
            active_epoch,
            replaced_parent_pid: root.header.pid,
            affected_leaf_pids: vec![12],
            replacement_leaf_count: 2,
            reason: "test_split",
        };
        let pid_plan = SpireLeafReplacementPidPlan {
            replacement_pids: vec![21, 22],
            reuses_existing_pid: false,
            next_pid: 23,
        };
        let publish_plan = SpireScheduledReplacementPublishPlan {
            epoch: 8,
            consistency_mode: SpireConsistencyMode::Strict,
            next_pid: 23,
            next_local_vec_seq: 7,
        };
        let replacement_children = vec![
            replacement_child(22, vec![-0.5, 0.5]),
            replacement_child(21, vec![0.5, 0.5]),
        ];
        let replacement_parent = rewrite_scheduled_replacement_parent_routing(
            &root,
            &decision,
            replacement_children.clone(),
            4,
        )
        .unwrap();

        assert!(build_local_scheduled_replacement_epoch_draft(
            &snapshot,
            &decision,
            &pid_plan,
            &publish_plan,
            SpireLocalScheduledReplacementExecutionInput {
                epoch: 8,
                published_at_micros: 3000,
                retain_until_micros: 4000,
                consistency_mode: SpireConsistencyMode::Strict,
                replacement_parent,
                replacement_children,
                leaf_object_version: 2,
                leaf_inputs: vec![
                    SpireReplacementLeafObjectInput {
                        pid: 21,
                        rows: vec![primary_row(5, 30, 1)],
                    },
                    SpireReplacementLeafObjectInput {
                        pid: 22,
                        rows: vec![primary_row(6, 30, 2)],
                    },
                ],
                placement_write_evidence: placement_write_evidence_for_pids(&[1, 21, 22]),
                next_local_vec_seq: 7,
            },
            &mut object_store,
        )
        .unwrap_err()
        .contains("do not match pid plan"));
    }

    #[test]
    fn local_scheduled_replacement_execution_input_uses_publish_plan() {
        let publish_plan = SpireScheduledReplacementPublishPlan {
            epoch: 8,
            consistency_mode: SpireConsistencyMode::Strict,
            next_pid: 23,
            next_local_vec_seq: 100,
        };
        let pid_plan = SpireLeafReplacementPidPlan {
            replacement_pids: vec![21, 22],
            reuses_existing_pid: false,
            next_pid: 23,
        };
        let decision = scheduled_split_decision(7);
        let replacement_children = scheduled_split_replacement_children();
        let replacement_parent =
            scheduled_rewritten_parent_for_decision(&decision, replacement_children.clone());

        let input = build_local_scheduled_replacement_execution_input_from_publish_plan(
            &publish_plan,
            &pid_plan,
            &decision,
            SpireLocalScheduledReplacementExecutionParts {
                published_at_micros: 3000,
                retain_until_micros: 4000,
                replacement_parent,
                replacement_children,
                leaf_object_version: 2,
                leaf_inputs: vec![
                    SpireReplacementLeafObjectInput {
                        pid: 21,
                        rows: vec![primary_row(5, 30, 1)],
                    },
                    SpireReplacementLeafObjectInput {
                        pid: 22,
                        rows: vec![primary_row(6, 30, 2)],
                    },
                ],
                placement_write_evidence: placement_write_evidence_for_pids(&[1, 21, 22]),
            },
        )
        .unwrap();

        assert_eq!(input.epoch, 8);
        assert_eq!(input.consistency_mode, SpireConsistencyMode::Strict);
        assert_eq!(input.next_local_vec_seq, 100);
        assert_eq!(
            input
                .placement_write_evidence
                .iter()
                .map(|evidence| evidence.pid)
                .collect::<Vec<_>>(),
            vec![1, 21, 22]
        );
    }

    #[test]
    fn local_scheduled_replacement_execution_input_rejects_plan_drift() {
        let publish_plan = SpireScheduledReplacementPublishPlan {
            epoch: 8,
            consistency_mode: SpireConsistencyMode::Strict,
            next_pid: 23,
            next_local_vec_seq: 100,
        };
        let pid_plan = SpireLeafReplacementPidPlan {
            replacement_pids: vec![21, 22],
            reuses_existing_pid: false,
            next_pid: 23,
        };
        let decision = scheduled_split_decision(7);
        let replacement_children = scheduled_split_replacement_children();
        let replacement_parent =
            scheduled_rewritten_parent_for_decision(&decision, replacement_children.clone());

        let err = build_local_scheduled_replacement_execution_input_from_publish_plan(
            &SpireScheduledReplacementPublishPlan {
                next_pid: 24,
                ..publish_plan
            },
            &pid_plan,
            &decision,
            SpireLocalScheduledReplacementExecutionParts {
                published_at_micros: 3000,
                retain_until_micros: 4000,
                replacement_parent,
                replacement_children,
                leaf_object_version: 2,
                leaf_inputs: vec![
                    SpireReplacementLeafObjectInput {
                        pid: 21,
                        rows: vec![primary_row(5, 30, 1)],
                    },
                    SpireReplacementLeafObjectInput {
                        pid: 22,
                        rows: vec![primary_row(6, 30, 2)],
                    },
                ],
                placement_write_evidence: placement_write_evidence_for_pids(&[1, 21, 22]),
            },
        )
        .unwrap_err();
        assert!(err.contains("next_pid"));
    }

    #[test]
    fn local_scheduled_replacement_execution_publish_plan_validator_rejects_input_drift() {
        let publish_plan = SpireScheduledReplacementPublishPlan {
            epoch: 8,
            consistency_mode: SpireConsistencyMode::Strict,
            next_pid: 23,
            next_local_vec_seq: 100,
        };
        let pid_plan = SpireLeafReplacementPidPlan {
            replacement_pids: vec![21, 22],
            reuses_existing_pid: false,
            next_pid: 23,
        };
        let decision = scheduled_split_decision(7);
        let replacement_children = scheduled_split_replacement_children();
        let replacement_parent =
            scheduled_rewritten_parent_for_decision(&decision, replacement_children.clone());
        let input = build_local_scheduled_replacement_execution_input_from_publish_plan(
            &publish_plan,
            &pid_plan,
            &decision,
            SpireLocalScheduledReplacementExecutionParts {
                published_at_micros: 3000,
                retain_until_micros: 4000,
                replacement_parent,
                replacement_children,
                leaf_object_version: 2,
                leaf_inputs: vec![
                    SpireReplacementLeafObjectInput {
                        pid: 21,
                        rows: vec![primary_row(5, 30, 1)],
                    },
                    SpireReplacementLeafObjectInput {
                        pid: 22,
                        rows: vec![primary_row(6, 30, 2)],
                    },
                ],
                placement_write_evidence: placement_write_evidence_for_pids(&[1, 21, 22]),
            },
        )
        .unwrap();

        validate_local_scheduled_replacement_execution_publish_plan(
            &publish_plan,
            &pid_plan,
            &decision,
            &input,
        )
        .unwrap();

        let stale_publish_plan = SpireScheduledReplacementPublishPlan {
            epoch: 9,
            ..publish_plan.clone()
        };
        assert!(validate_local_scheduled_replacement_execution_publish_plan(
            &stale_publish_plan,
            &pid_plan,
            &decision,
            &SpireLocalScheduledReplacementExecutionInput {
                epoch: 9,
                ..input.clone()
            },
        )
        .unwrap_err()
        .contains("immediate successor"));

        let stale_epoch_input = SpireLocalScheduledReplacementExecutionInput {
            epoch: 9,
            ..input.clone()
        };
        assert!(validate_local_scheduled_replacement_execution_publish_plan(
            &publish_plan,
            &pid_plan,
            &decision,
            &stale_epoch_input,
        )
        .unwrap_err()
        .contains("epoch"));

        let stale_child_count_input = SpireLocalScheduledReplacementExecutionInput {
            replacement_children: vec![replacement_child(21, vec![0.5, 0.5])],
            ..input.clone()
        };
        assert!(validate_local_scheduled_replacement_execution_publish_plan(
            &publish_plan,
            &pid_plan,
            &decision,
            &stale_child_count_input,
        )
        .unwrap_err()
        .contains("child count"));

        let stale_vec_cursor_input = SpireLocalScheduledReplacementExecutionInput {
            next_local_vec_seq: 101,
            ..input.clone()
        };
        assert!(validate_local_scheduled_replacement_execution_publish_plan(
            &publish_plan,
            &pid_plan,
            &decision,
            &stale_vec_cursor_input,
        )
        .unwrap_err()
        .contains("next_local_vec_seq"));

        let missing_publish_timestamp_input = SpireLocalScheduledReplacementExecutionInput {
            published_at_micros: 0,
            ..input
        };
        assert!(validate_local_scheduled_replacement_execution_publish_plan(
            &publish_plan,
            &pid_plan,
            &decision,
            &missing_publish_timestamp_input,
        )
        .unwrap_err()
        .contains("publish timestamp"));
    }

    #[test]
    fn scheduled_replacement_publish_plan_uses_root_control_and_active_manifest() {
        let root_control =
            SpireRootControlState::published(7, 20, 100, tid(90, 1), tid(90, 2), tid(90, 3))
                .unwrap();
        let active_epoch_manifest = SpireEpochManifest {
            epoch: 7,
            state: SpireEpochState::Published,
            consistency_mode: SpireConsistencyMode::Strict,
            published_at_micros: 1000,
            retain_until_micros: 2000,
            active_query_count: 0,
        };
        let decision = SpireLeafReplacementScheduleDecision {
            mode: SpireLeafReplacementScheduleMode::Split,
            active_epoch: 7,
            replaced_parent_pid: 1,
            affected_leaf_pids: vec![12],
            replacement_leaf_count: 2,
            reason: "test_split",
        };
        let pid_plan = SpireLeafReplacementPidPlan {
            replacement_pids: vec![20, 21],
            reuses_existing_pid: false,
            next_pid: 22,
        };

        let plan = plan_scheduled_replacement_publish_epoch(
            &root_control,
            &active_epoch_manifest,
            &decision,
            &pid_plan,
        )
        .unwrap();

        assert_eq!(
            plan,
            SpireScheduledReplacementPublishPlan {
                epoch: 8,
                consistency_mode: SpireConsistencyMode::Strict,
                next_pid: 22,
                next_local_vec_seq: 100,
            }
        );
    }

    #[test]
    fn scheduled_replacement_publish_lock_plans_pids_and_publish_epoch() {
        let root_control =
            SpireRootControlState::published(7, 20, 100, tid(90, 1), tid(90, 2), tid(90, 3))
                .unwrap();
        let active_epoch_manifest = SpireEpochManifest {
            epoch: 7,
            state: SpireEpochState::Published,
            consistency_mode: SpireConsistencyMode::Strict,
            published_at_micros: 1000,
            retain_until_micros: 2000,
            active_query_count: 0,
        };
        let decision = scheduled_split_decision(7);
        let mut pid_allocator = SpirePidAllocator::new(20).unwrap();

        let plan = plan_scheduled_replacement_publish_lock(
            &root_control,
            &active_epoch_manifest,
            &decision,
            &mut pid_allocator,
        )
        .unwrap();

        assert_eq!(
            plan,
            SpireScheduledReplacementPublishLockPlan {
                pid_plan: SpireLeafReplacementPidPlan {
                    replacement_pids: vec![20, 21],
                    reuses_existing_pid: false,
                    next_pid: 22,
                },
                publish_plan: SpireScheduledReplacementPublishPlan {
                    epoch: 8,
                    consistency_mode: SpireConsistencyMode::Strict,
                    next_pid: 22,
                    next_local_vec_seq: 100,
                },
            }
        );
        assert_eq!(pid_allocator.next_pid(), 22);
    }

    #[test]
    fn scheduled_replacement_publish_lock_does_not_advance_on_publish_plan_drift() {
        let root_control =
            SpireRootControlState::published(6, 20, 100, tid(90, 1), tid(90, 2), tid(90, 3))
                .unwrap();
        let active_epoch_manifest = SpireEpochManifest {
            epoch: 7,
            state: SpireEpochState::Published,
            consistency_mode: SpireConsistencyMode::Strict,
            published_at_micros: 1000,
            retain_until_micros: 2000,
            active_query_count: 0,
        };
        let decision = scheduled_split_decision(7);
        let mut pid_allocator = SpirePidAllocator::new(20).unwrap();

        assert!(plan_scheduled_replacement_publish_lock(
            &root_control,
            &active_epoch_manifest,
            &decision,
            &mut pid_allocator,
        )
        .unwrap_err()
        .contains("root/control active epoch"));
        assert_eq!(pid_allocator.next_pid(), 20);
    }

    #[test]
    fn rechecked_scheduled_replacement_publish_lock_plans_matching_decision() {
        let root_control =
            SpireRootControlState::published(7, 20, 100, tid(90, 1), tid(90, 2), tid(90, 3))
                .unwrap();
        let active_epoch_manifest = SpireEpochManifest {
            epoch: 7,
            state: SpireEpochState::Published,
            consistency_mode: SpireConsistencyMode::Strict,
            published_at_micros: 1000,
            retain_until_micros: 2000,
            active_query_count: 0,
        };
        let rows = vec![
            leaf_snapshot_row(11, 1, 3, false, false),
            leaf_snapshot_row(12, 1, 100, true, false),
            leaf_snapshot_row(13, 1, 2, false, false),
        ];
        let decision = choose_leaf_replacement_schedule(&rows).unwrap().unwrap();
        let mut pid_allocator = SpirePidAllocator::new(20).unwrap();

        let plan = plan_rechecked_scheduled_replacement_publish_lock(
            &rows,
            &root_control,
            &active_epoch_manifest,
            &decision,
            &mut pid_allocator,
        )
        .unwrap();

        assert_eq!(plan.pid_plan.replacement_pids, vec![20, 21]);
        assert_eq!(plan.publish_plan.epoch, 8);
        assert_eq!(pid_allocator.next_pid(), 22);
    }

    #[test]
    fn rechecked_scheduled_replacement_publish_lock_rejects_changed_decision() {
        let root_control =
            SpireRootControlState::published(7, 20, 100, tid(90, 1), tid(90, 2), tid(90, 3))
                .unwrap();
        let active_epoch_manifest = SpireEpochManifest {
            epoch: 7,
            state: SpireEpochState::Published,
            consistency_mode: SpireConsistencyMode::Strict,
            published_at_micros: 1000,
            retain_until_micros: 2000,
            active_query_count: 0,
        };
        let rows = vec![
            leaf_snapshot_row(11, 1, 3, false, false),
            leaf_snapshot_row(12, 1, 100, true, false),
            leaf_snapshot_row(13, 1, 2, false, false),
        ];
        let decision = choose_leaf_replacement_schedule(&rows).unwrap().unwrap();
        let changed_rows = vec![
            leaf_snapshot_row(11, 1, 3, false, false),
            leaf_snapshot_row(12, 1, 100, false, false),
            leaf_snapshot_row(13, 1, 2, false, false),
        ];
        let mut pid_allocator = SpirePidAllocator::new(20).unwrap();

        assert!(plan_rechecked_scheduled_replacement_publish_lock(
            &changed_rows,
            &root_control,
            &active_epoch_manifest,
            &decision,
            &mut pid_allocator,
        )
        .unwrap_err()
        .contains("no longer recommended"));
        assert_eq!(pid_allocator.next_pid(), 20);
    }

    #[test]
    fn selected_scheduled_replacement_publish_lock_returns_decision_and_plan() {
        let root_control =
            SpireRootControlState::published(7, 20, 100, tid(90, 1), tid(90, 2), tid(90, 3))
                .unwrap();
        let active_epoch_manifest = SpireEpochManifest {
            epoch: 7,
            state: SpireEpochState::Published,
            consistency_mode: SpireConsistencyMode::Strict,
            published_at_micros: 1000,
            retain_until_micros: 2000,
            active_query_count: 0,
        };
        let rows = vec![
            leaf_snapshot_row(11, 1, 3, false, false),
            leaf_snapshot_row(12, 1, 100, true, false),
            leaf_snapshot_row(13, 1, 2, false, false),
        ];
        let expected_decision = SpireLeafReplacementScheduleDecision {
            mode: SpireLeafReplacementScheduleMode::Split,
            active_epoch: 7,
            replaced_parent_pid: 1,
            affected_leaf_pids: vec![12],
            replacement_leaf_count: 2,
            reason: "largest_split_candidate",
        };
        let mut pid_allocator = SpirePidAllocator::new(20).unwrap();

        let selected = choose_scheduled_replacement_publish_lock_plan(
            &rows,
            &root_control,
            &active_epoch_manifest,
            &mut pid_allocator,
        )
        .unwrap();

        assert_eq!(
            selected,
            Some(SpireSelectedScheduledReplacementPublishLockPlan {
                decision: expected_decision,
                lock_plan: SpireScheduledReplacementPublishLockPlan {
                    pid_plan: SpireLeafReplacementPidPlan {
                        replacement_pids: vec![20, 21],
                        reuses_existing_pid: false,
                        next_pid: 22,
                    },
                    publish_plan: SpireScheduledReplacementPublishPlan {
                        epoch: 8,
                        consistency_mode: SpireConsistencyMode::Strict,
                        next_pid: 22,
                        next_local_vec_seq: 100,
                    },
                },
            })
        );
        assert_eq!(pid_allocator.next_pid(), 22);
    }

    #[test]
    fn selected_scheduled_replacement_publish_lock_returns_none_without_allocation() {
        let root_control =
            SpireRootControlState::published(7, 20, 100, tid(90, 1), tid(90, 2), tid(90, 3))
                .unwrap();
        let active_epoch_manifest = SpireEpochManifest {
            epoch: 7,
            state: SpireEpochState::Published,
            consistency_mode: SpireConsistencyMode::Strict,
            published_at_micros: 1000,
            retain_until_micros: 2000,
            active_query_count: 0,
        };
        let rows = vec![
            leaf_snapshot_row(11, 1, 3, false, false),
            leaf_snapshot_row(12, 1, 4, false, false),
            leaf_snapshot_row(13, 1, 2, false, false),
        ];
        let mut pid_allocator = SpirePidAllocator::new(20).unwrap();

        let selected = choose_scheduled_replacement_publish_lock_plan(
            &rows,
            &root_control,
            &active_epoch_manifest,
            &mut pid_allocator,
        )
        .unwrap();

        assert_eq!(selected, None);
        assert_eq!(pid_allocator.next_pid(), 20);
    }

    #[test]
    fn relation_scheduled_replacement_execution_input_uses_publish_plan() {
        let publish_plan = SpireScheduledReplacementPublishPlan {
            epoch: 8,
            consistency_mode: SpireConsistencyMode::Strict,
            next_pid: 23,
            next_local_vec_seq: 100,
        };
        let pid_plan = SpireLeafReplacementPidPlan {
            replacement_pids: vec![21, 22],
            reuses_existing_pid: false,
            next_pid: 23,
        };
        let decision = scheduled_split_decision(7);
        let replacement_children = scheduled_split_replacement_children();
        let replacement_parent =
            scheduled_rewritten_parent_for_decision(&decision, replacement_children.clone());
        let parts = SpireRelationScheduledReplacementExecutionParts {
            published_at_micros: 3000,
            retain_until_micros: 4000,
            replacement_parent,
            replacement_children,
            leaf_object_version: 2,
            leaf_inputs: vec![
                SpireReplacementLeafObjectInput {
                    pid: 21,
                    rows: vec![primary_row(5, 30, 1)],
                },
                SpireReplacementLeafObjectInput {
                    pid: 22,
                    rows: vec![primary_row(6, 30, 2)],
                },
            ],
        };

        let input = build_relation_scheduled_replacement_execution_input_from_publish_plan(
            &publish_plan,
            &pid_plan,
            &decision,
            parts,
        )
        .unwrap();

        assert_eq!(input.epoch, 8);
        assert_eq!(input.consistency_mode, SpireConsistencyMode::Strict);
        assert_eq!(input.next_local_vec_seq, 100);
        assert_eq!(
            input
                .replacement_children
                .iter()
                .map(|child| child.child_pid)
                .collect::<Vec<_>>(),
            vec![21, 22]
        );
    }

    #[test]
    fn relation_scheduled_replacement_execution_input_rejects_plan_drift() {
        let publish_plan = SpireScheduledReplacementPublishPlan {
            epoch: 8,
            consistency_mode: SpireConsistencyMode::Strict,
            next_pid: 23,
            next_local_vec_seq: 100,
        };
        let pid_plan = SpireLeafReplacementPidPlan {
            replacement_pids: vec![21, 22],
            reuses_existing_pid: false,
            next_pid: 23,
        };
        let decision = scheduled_split_decision(7);
        let replacement_children = scheduled_split_replacement_children();
        let replacement_parent =
            scheduled_rewritten_parent_for_decision(&decision, replacement_children.clone());
        let parts = SpireRelationScheduledReplacementExecutionParts {
            published_at_micros: 3000,
            retain_until_micros: 4000,
            replacement_parent,
            replacement_children,
            leaf_object_version: 2,
            leaf_inputs: vec![
                SpireReplacementLeafObjectInput {
                    pid: 21,
                    rows: vec![primary_row(5, 30, 1)],
                },
                SpireReplacementLeafObjectInput {
                    pid: 22,
                    rows: vec![primary_row(6, 30, 2)],
                },
            ],
        };

        let mismatched_plan = SpireScheduledReplacementPublishPlan {
            next_pid: 24,
            ..publish_plan.clone()
        };
        assert!(
            build_relation_scheduled_replacement_execution_input_from_publish_plan(
                &mismatched_plan,
                &pid_plan,
                &decision,
                parts.clone(),
            )
            .unwrap_err()
            .contains("next_pid")
        );

        let zero_version_parts = SpireRelationScheduledReplacementExecutionParts {
            leaf_object_version: 0,
            ..parts.clone()
        };
        assert!(
            build_relation_scheduled_replacement_execution_input_from_publish_plan(
                &publish_plan,
                &pid_plan,
                &decision,
                zero_version_parts,
            )
            .unwrap_err()
            .contains("object_version")
        );

        let unrewritten_parent_parts = SpireRelationScheduledReplacementExecutionParts {
            replacement_parent: root_routing_object(),
            ..parts.clone()
        };
        assert!(
            build_relation_scheduled_replacement_execution_input_from_publish_plan(
                &publish_plan,
                &pid_plan,
                &decision,
                unrewritten_parent_parts,
            )
            .unwrap_err()
            .contains("missing replacement child")
        );

        let stale_leaf_parent_parts = SpireRelationScheduledReplacementExecutionParts {
            replacement_parent: SpireRoutingPartitionObject::root(
                1,
                3,
                2,
                vec![
                    routing_child(0, 12, vec![0.0, 1.0]),
                    routing_child(1, 21, vec![0.5, 0.5]),
                    routing_child(2, 22, vec![-0.5, 0.5]),
                ],
            )
            .unwrap(),
            ..parts.clone()
        };
        assert!(
            build_relation_scheduled_replacement_execution_input_from_publish_plan(
                &publish_plan,
                &pid_plan,
                &decision,
                stale_leaf_parent_parts,
            )
            .unwrap_err()
            .contains("still contains affected leaf")
        );

        let swapped_parts = SpireRelationScheduledReplacementExecutionParts {
            replacement_children: vec![
                replacement_child(22, vec![-0.5, 0.5]),
                replacement_child(21, vec![0.5, 0.5]),
            ],
            ..parts
        };
        assert!(
            build_relation_scheduled_replacement_execution_input_from_publish_plan(
                &publish_plan,
                &pid_plan,
                &decision,
                swapped_parts,
            )
            .unwrap_err()
            .contains("do not match pid plan")
        );

        let wrong_parent_parts = SpireRelationScheduledReplacementExecutionParts {
            replacement_parent: SpireRoutingPartitionObject::root(
                99,
                3,
                2,
                vec![
                    routing_child(0, 11, vec![1.0, 0.0]),
                    routing_child(1, 12, vec![0.0, 1.0]),
                    routing_child(2, 13, vec![-1.0, 0.0]),
                ],
            )
            .unwrap(),
            replacement_children: vec![
                replacement_child(21, vec![0.5, 0.5]),
                replacement_child(22, vec![-0.5, 0.5]),
            ],
            leaf_inputs: vec![
                SpireReplacementLeafObjectInput {
                    pid: 21,
                    rows: vec![primary_row(5, 30, 1)],
                },
                SpireReplacementLeafObjectInput {
                    pid: 22,
                    rows: vec![primary_row(6, 30, 2)],
                },
            ],
            published_at_micros: 3000,
            retain_until_micros: 4000,
            leaf_object_version: 2,
        };
        assert!(
            build_relation_scheduled_replacement_execution_input_from_publish_plan(
                &publish_plan,
                &pid_plan,
                &decision,
                wrong_parent_parts,
            )
            .unwrap_err()
            .contains("parent pid")
        );
    }

    #[test]
    fn relation_scheduled_replacement_execution_publish_plan_validator_rejects_input_drift() {
        let publish_plan = SpireScheduledReplacementPublishPlan {
            epoch: 8,
            consistency_mode: SpireConsistencyMode::Strict,
            next_pid: 23,
            next_local_vec_seq: 100,
        };
        let pid_plan = SpireLeafReplacementPidPlan {
            replacement_pids: vec![21, 22],
            reuses_existing_pid: false,
            next_pid: 23,
        };
        let decision = scheduled_split_decision(7);
        let replacement_children = scheduled_split_replacement_children();
        let replacement_parent =
            scheduled_rewritten_parent_for_decision(&decision, replacement_children.clone());
        let input = build_relation_scheduled_replacement_execution_input_from_publish_plan(
            &publish_plan,
            &pid_plan,
            &decision,
            SpireRelationScheduledReplacementExecutionParts {
                published_at_micros: 3000,
                retain_until_micros: 4000,
                replacement_parent,
                replacement_children,
                leaf_object_version: 2,
                leaf_inputs: vec![
                    SpireReplacementLeafObjectInput {
                        pid: 21,
                        rows: vec![primary_row(5, 30, 1)],
                    },
                    SpireReplacementLeafObjectInput {
                        pid: 22,
                        rows: vec![primary_row(6, 30, 2)],
                    },
                ],
            },
        )
        .unwrap();

        validate_relation_scheduled_replacement_execution_publish_plan(
            &publish_plan,
            &pid_plan,
            &decision,
            &input,
        )
        .unwrap();

        let stale_epoch_input = SpireRelationScheduledReplacementExecutionInput {
            epoch: 9,
            ..input.clone()
        };
        assert!(
            validate_relation_scheduled_replacement_execution_publish_plan(
                &publish_plan,
                &pid_plan,
                &decision,
                &stale_epoch_input,
            )
            .unwrap_err()
            .contains("epoch")
        );

        let stale_vec_cursor_input = SpireRelationScheduledReplacementExecutionInput {
            next_local_vec_seq: 101,
            ..input
        };
        assert!(
            validate_relation_scheduled_replacement_execution_publish_plan(
                &publish_plan,
                &pid_plan,
                &decision,
                &stale_vec_cursor_input,
            )
            .unwrap_err()
            .contains("next_local_vec_seq")
        );
    }

    #[test]
    fn scheduled_replacement_publish_plan_rejects_stale_epoch_or_cursor() {
        let root_control =
            SpireRootControlState::published(7, 20, 100, tid(90, 1), tid(90, 2), tid(90, 3))
                .unwrap();
        let active_epoch_manifest = SpireEpochManifest {
            epoch: 7,
            state: SpireEpochState::Published,
            consistency_mode: SpireConsistencyMode::Strict,
            published_at_micros: 1000,
            retain_until_micros: 2000,
            active_query_count: 0,
        };
        let decision = SpireLeafReplacementScheduleDecision {
            mode: SpireLeafReplacementScheduleMode::Split,
            active_epoch: 7,
            replaced_parent_pid: 1,
            affected_leaf_pids: vec![12],
            replacement_leaf_count: 2,
            reason: "test_split",
        };
        let pid_plan = SpireLeafReplacementPidPlan {
            replacement_pids: vec![20, 21],
            reuses_existing_pid: false,
            next_pid: 22,
        };
        let stale_decision = SpireLeafReplacementScheduleDecision {
            active_epoch: 6,
            ..decision.clone()
        };
        assert!(plan_scheduled_replacement_publish_epoch(
            &root_control,
            &active_epoch_manifest,
            &stale_decision,
            &pid_plan,
        )
        .unwrap_err()
        .contains("root/control active epoch"));

        let wrong_count_pid_plan = SpireLeafReplacementPidPlan {
            replacement_pids: vec![20],
            reuses_existing_pid: false,
            next_pid: 21,
        };
        assert!(plan_scheduled_replacement_publish_epoch(
            &root_control,
            &active_epoch_manifest,
            &decision,
            &wrong_count_pid_plan,
        )
        .unwrap_err()
        .contains("pid count"));

        let duplicate_pid_plan = SpireLeafReplacementPidPlan {
            replacement_pids: vec![20, 20],
            reuses_existing_pid: false,
            next_pid: 21,
        };
        assert!(plan_scheduled_replacement_publish_epoch(
            &root_control,
            &active_epoch_manifest,
            &decision,
            &duplicate_pid_plan,
        )
        .unwrap_err()
        .contains("appears more than once"));

        let stale_pid_plan = SpireLeafReplacementPidPlan {
            replacement_pids: vec![18, 19],
            reuses_existing_pid: false,
            next_pid: 19,
        };
        assert!(plan_scheduled_replacement_publish_epoch(
            &root_control,
            &active_epoch_manifest,
            &decision,
            &stale_pid_plan,
        )
        .unwrap_err()
        .contains("behind root/control next_pid"));

        let stale_replacement_pid_plan = SpireLeafReplacementPidPlan {
            replacement_pids: vec![19, 20],
            reuses_existing_pid: false,
            next_pid: 22,
        };
        assert!(plan_scheduled_replacement_publish_epoch(
            &root_control,
            &active_epoch_manifest,
            &decision,
            &stale_replacement_pid_plan,
        )
        .unwrap_err()
        .contains("behind root/control next_pid"));

        let unadvanced_pid_plan = SpireLeafReplacementPidPlan {
            replacement_pids: vec![20, 21],
            reuses_existing_pid: false,
            next_pid: 21,
        };
        assert!(plan_scheduled_replacement_publish_epoch(
            &root_control,
            &active_epoch_manifest,
            &decision,
            &unadvanced_pid_plan,
        )
        .unwrap_err()
        .contains("does not advance"));
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
                .read_leaf_object_v2(base_placement)
                .unwrap()
                .meta
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
    fn replacement_leaf_rows_fold_active_deltas_into_base_leaf_rows() {
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
        let delta_draft = build_delta_epoch_draft_from_snapshot(
            input,
            &base_snapshot,
            &mut pid_allocator,
            &mut local_vec_id_allocator,
            &mut object_store,
        )
        .unwrap();
        let delta_snapshot = SpirePublishedEpochSnapshot::new(
            &delta_draft.epoch_manifest,
            &delta_draft.object_manifest,
            &delta_draft.placement_directory,
        )
        .unwrap();

        let folded = collect_replacement_leaf_rows(&delta_snapshot, &object_store, &[1]).unwrap();

        assert_eq!(folded.len(), 1);
        assert_eq!(folded[0].base_pid, 1);
        assert_eq!(folded[0].rows.len(), 1);
        assert_eq!(folded[0].rows[0].heap_tid, tid(20, 1));
        assert_eq!(folded[0].rows[0].flags, SPIRE_ASSIGNMENT_FLAG_PRIMARY);
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
    fn delta_epoch_draft_from_snapshot_rejects_mismatched_delete_heap_tid() {
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
        let mut input = delta_input(Vec::new(), vec![delete_assignment(1, 10, 2)]);
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
    fn delta_epoch_draft_from_snapshot_rejects_stale_delete_target() {
        let mut pid_allocator = SpirePidAllocator::new(2).unwrap();
        let mut local_vec_id_allocator = SpireLocalVecIdAllocator::new(2).unwrap();
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let stale_assignment = SpireLeafAssignmentRow {
            flags: SPIRE_ASSIGNMENT_FLAG_PRIMARY | SPIRE_ASSIGNMENT_FLAG_STALE_LOCATOR,
            vec_id: SpireVecId::local(1),
            heap_tid: tid(10, 1),
            payload_format: 1,
            gamma: 0.5,
            encoded_payload: vec![1, 2, 3],
        };
        let leaf_object = SpireLeafPartitionObject::new(1, 1, 0, vec![stale_assignment]).unwrap();
        let placement = object_store.insert_leaf_object(7, &leaf_object).unwrap();
        let epoch_manifest = SpireEpochManifest {
            epoch: 7,
            state: SpireEpochState::Published,
            consistency_mode: SpireConsistencyMode::Strict,
            published_at_micros: 900,
            retain_until_micros: 1900,
            active_query_count: 0,
        };
        let object_manifest = SpireObjectManifest::from_entries(
            7,
            vec![SpireManifestEntry {
                epoch: 7,
                pid: 1,
                object_version: 1,
                placement_tid: placement.object_tid,
            }],
        )
        .unwrap();
        let placement_directory =
            SpirePlacementDirectory::from_entries(7, vec![placement]).unwrap();
        let base_snapshot = SpirePublishedEpochSnapshot::new(
            &epoch_manifest,
            &object_manifest,
            &placement_directory,
        )
        .unwrap();
        let initial_page_count = object_store.page_count();
        let mut input = delta_input(Vec::new(), vec![delete_assignment(1, 10, 1)]);
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
    fn delta_epoch_draft_from_snapshot_rejects_delta_base_pid() {
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
        let mut first_delta_input = delta_input(vec![insert_assignment(20, 1)], Vec::new());
        first_delta_input.base_pid = 1;
        let first_delta = build_delta_epoch_draft_from_snapshot(
            first_delta_input,
            &base_snapshot,
            &mut pid_allocator,
            &mut local_vec_id_allocator,
            &mut object_store,
        )
        .unwrap();
        let delta_snapshot = SpirePublishedEpochSnapshot::new(
            &first_delta.epoch_manifest,
            &first_delta.object_manifest,
            &first_delta.placement_directory,
        )
        .unwrap();
        let initial_page_count = object_store.page_count();
        let mut nested_delta_input = delta_input(vec![insert_assignment(30, 1)], Vec::new());
        nested_delta_input.epoch = 9;
        nested_delta_input.base_pid = first_delta.delta_object.header.pid;

        assert!(build_delta_epoch_draft_from_snapshot(
            nested_delta_input,
            &delta_snapshot,
            &mut pid_allocator,
            &mut local_vec_id_allocator,
            &mut object_store,
        )
        .is_err());
        assert_eq!(pid_allocator.next_pid(), 3);
        assert_eq!(local_vec_id_allocator.next_local_vec_seq(), 3);
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
    fn delta_epoch_draft_from_snapshot_rejects_degraded_base_placements() {
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
        let mut epoch_manifest = base_draft.epoch_manifest;
        epoch_manifest.consistency_mode = SpireConsistencyMode::Degraded;
        let mut placement = *base_draft.placement_directory.get(1).unwrap();
        placement.state = SpirePlacementState::Skipped;
        let placement_directory =
            SpirePlacementDirectory::from_entries(base_draft.epoch_manifest.epoch, vec![placement])
                .unwrap();
        let base_snapshot = SpirePublishedEpochSnapshot::new(
            &epoch_manifest,
            &base_draft.object_manifest,
            &placement_directory,
        )
        .unwrap();
        let initial_page_count = object_store.page_count();
        let mut input = delta_input(vec![insert_assignment(20, 1)], Vec::new());
        input.base_pid = 1;

        let error = build_delta_epoch_draft_from_snapshot(
            input,
            &base_snapshot,
            &mut pid_allocator,
            &mut local_vec_id_allocator,
            &mut object_store,
        )
        .unwrap_err();

        assert!(error.contains("requires available placement"));
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
