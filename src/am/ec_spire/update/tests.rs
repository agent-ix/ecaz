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
        build_local_selected_scheduled_merge_replacement_execution_input_from_snapshot,
        build_local_selected_scheduled_replacement_epoch_draft,
        build_local_selected_scheduled_split_replacement_epoch_draft,
        build_local_selected_scheduled_split_replacement_epoch_draft_from_snapshot,
        build_local_selected_scheduled_split_replacement_execution_input,
        build_local_selected_scheduled_split_replacement_execution_input_from_snapshot,
        build_merge_replacement_leaf_object_input,
        build_relation_scheduled_merge_replacement_execution_input,
        build_relation_scheduled_merge_replacement_execution_parts,
        build_relation_scheduled_replacement_execution_input_from_publish_plan,
        build_relation_scheduled_split_replacement_execution_input,
        build_relation_scheduled_split_replacement_execution_parts,
        build_relation_selected_scheduled_merge_replacement_execution_input,
        build_relation_selected_scheduled_merge_replacement_execution_input_from_snapshot,
        build_relation_selected_scheduled_split_replacement_execution_input,
        build_relation_selected_scheduled_split_replacement_execution_input_from_snapshot,
        build_relation_selected_scheduled_split_replacement_execution_input_from_snapshot_sources,
        build_replacement_epoch_draft, build_replacement_epoch_draft_from_object_placements,
        build_scheduled_merge_replacement_centroids,
        build_scheduled_merge_replacement_routing_parts,
        build_scheduled_replacement_epoch_draft_from_object_placements,
        build_scheduled_routing_replacement_children,
        build_scheduled_split_replacement_routing_parts,
        build_split_replacement_leaf_materialization,
        build_split_replacement_leaf_materialization_from_rows,
        build_split_replacement_leaf_object_inputs, build_split_replacement_source_rows,
        choose_leaf_replacement_schedule, choose_scheduled_replacement_publish_lock_plan,
        collect_replacement_leaf_rows, collect_selected_scheduled_replacement_leaf_rows,
        filter_split_replacement_rows_to_fetched_sources,
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
        SpireSelectedScheduledReplacementPublishLockPlan, SpireSplitReplacementFetchedSourceVector,
        SpireSplitReplacementSourceRow,
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
        SPIRE_ASSIGNMENT_FLAG_BOUNDARY_REPLICA, SPIRE_ASSIGNMENT_FLAG_DELTA_DELETE,
        SPIRE_ASSIGNMENT_FLAG_DELTA_INSERT, SPIRE_ASSIGNMENT_FLAG_PRIMARY,
        SPIRE_ASSIGNMENT_FLAG_STALE_LOCATOR, SPIRE_ASSIGNMENT_FLAG_TOMBSTONE,
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
            local_store_config_tid: tid(90, 4),
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
            base_primary_assignment_count: effective_assignment_count,
            base_boundary_replica_assignment_count: 0,
            delta_object_count: 0,
            delta_insert_assignment_count: 0,
            delta_boundary_replica_insert_assignment_count: 0,
            delta_delete_assignment_count: 0,
            effective_assignment_count,
            effective_boundary_replica_assignment_count: 0,
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


    include!("tests/delta_basic.rs");
    include!("tests/scheduler.rs");
    include!("tests/scheduled_parent.rs");
    include!("tests/split_execution.rs");
    include!("tests/local_split_execution.rs");
    include!("tests/merge_execution.rs");
    include!("tests/selected_validators.rs");
    include!("tests/materialization.rs");
    include!("tests/replacement_epoch.rs");
    include!("tests/local_drafts.rs");
    include!("tests/local_execution_publish.rs");
    include!("tests/publish_lock.rs");
    include!("tests/relation_execution_publish.rs");
    include!("tests/delta_snapshot.rs");
}
