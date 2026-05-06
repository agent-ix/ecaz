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
    fn relation_selected_scheduled_split_replacement_execution_input_from_snapshot_loads_parent() {
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

        let input =
            build_relation_selected_scheduled_split_replacement_execution_input_from_snapshot(
                &snapshot,
                &object_store,
                &selected,
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
            )
            .unwrap();

        assert_eq!(input.epoch, 8);
        assert_eq!(input.next_local_vec_seq, 7);
        assert_eq!(
            input
                .replacement_parent
                .children()
                .map(|child| child.child_pid)
                .collect::<Vec<_>>(),
            vec![11, 21, 22, 13]
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
    fn relation_selected_scheduled_split_replacement_execution_input_from_snapshot_sources_materializes(
    ) {
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

        let input =
            build_relation_selected_scheduled_split_replacement_execution_input_from_snapshot_sources(
                &snapshot,
                &object_store,
                &selected,
                vec![SpireSplitReplacementFetchedSourceVector {
                    heap_tid: tid(10, 2),
                    source_vector: vec![1.0, 0.0],
                }],
                2,
                42,
                8,
                4,
                2,
                3000,
                4000,
            )
            .unwrap();

        assert_eq!(input.epoch, 8);
        assert_eq!(
            input
                .replacement_parent
                .children()
                .map(|child| child.child_pid)
                .collect::<Vec<_>>(),
            vec![11, 21, 22, 13]
        );
        assert_eq!(
            input
                .replacement_children
                .iter()
                .map(|child| (child.child_pid, child.centroid.clone()))
                .collect::<Vec<_>>(),
            vec![(21, vec![1.0, 0.0]), (22, vec![1.0, 0.0])]
        );
        assert_eq!(
            input
                .leaf_inputs
                .iter()
                .map(|leaf_input| (leaf_input.pid, leaf_input.rows.len()))
                .collect::<Vec<_>>(),
            vec![(21, 1), (22, 0)]
        );
    }

    #[test]
    fn relation_selected_scheduled_split_replacement_execution_input_from_snapshot_rejects_merge_plan(
    ) {
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
            build_relation_selected_scheduled_split_replacement_execution_input_from_snapshot(
                &snapshot,
                &object_store,
                &selected,
                vec![vec![0.5, 0.5]],
                vec![SpireReplacementLeafObjectInput {
                    pid: 21,
                    rows: vec![primary_row(5, 30, 1)],
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
    fn local_selected_scheduled_split_replacement_execution_input_from_snapshot_loads_parent() {
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

        let input = build_local_selected_scheduled_split_replacement_execution_input_from_snapshot(
            &snapshot,
            &object_store,
            &selected,
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

        assert_eq!(input.epoch, 8);
        assert_eq!(
            input
                .placement_write_evidence
                .iter()
                .map(|evidence| evidence.pid)
                .collect::<Vec<_>>(),
            vec![1, 11, 13, 21, 22]
        );
        assert_eq!(
            input
                .replacement_parent
                .children()
                .map(|child| child.child_pid)
                .collect::<Vec<_>>(),
            vec![11, 21, 22, 13]
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
    fn local_selected_scheduled_split_replacement_execution_input_from_snapshot_rejects_merge_plan()
    {
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
            build_local_selected_scheduled_split_replacement_execution_input_from_snapshot(
                &snapshot,
                &object_store,
                &selected,
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
    fn relation_selected_scheduled_merge_replacement_execution_input_from_snapshot_loads_inputs() {
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

        let input =
            build_relation_selected_scheduled_merge_replacement_execution_input_from_snapshot(
                &snapshot,
                &object_store,
                &selected,
                &rows,
                4,
                2,
                3000,
                4000,
            )
            .unwrap();

        assert_eq!(input.epoch, 8);
        assert_eq!(input.next_local_vec_seq, 7);
        assert_eq!(
            input
                .replacement_parent
                .children()
                .map(|child| child.child_pid)
                .collect::<Vec<_>>(),
            vec![21, 13]
        );
        assert_eq!(input.leaf_inputs[0].pid, 21);
        assert_eq!(
            input.leaf_inputs[0]
                .rows
                .iter()
                .map(|row| row.heap_tid)
                .collect::<Vec<_>>(),
            vec![tid(10, 1), tid(10, 2)]
        );
    }

    #[test]
    fn relation_selected_scheduled_merge_replacement_execution_input_from_snapshot_rejects_split_plan(
    ) {
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
            build_relation_selected_scheduled_merge_replacement_execution_input_from_snapshot(
                &snapshot,
                &object_store,
                &selected,
                &rows,
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
    fn local_selected_scheduled_merge_replacement_execution_input_from_snapshot_loads_inputs() {
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

        let input = build_local_selected_scheduled_merge_replacement_execution_input_from_snapshot(
            &snapshot,
            &object_store,
            &selected,
            &rows,
            4,
            2,
            3000,
            4000,
            placement_write_evidence_for_pids(&[1, 13, 21]),
        )
        .unwrap();

        assert_eq!(input.epoch, 8);
        assert_eq!(input.next_local_vec_seq, 7);
        assert_eq!(
            input
                .placement_write_evidence
                .iter()
                .map(|evidence| evidence.pid)
                .collect::<Vec<_>>(),
            vec![1, 13, 21]
        );
        assert_eq!(
            input
                .replacement_parent
                .children()
                .map(|child| child.child_pid)
                .collect::<Vec<_>>(),
            vec![21, 13]
        );
        assert_eq!(
            input.leaf_inputs[0]
                .rows
                .iter()
                .map(|row| row.heap_tid)
                .collect::<Vec<_>>(),
            vec![tid(10, 1), tid(10, 2)]
        );
    }

    #[test]
    fn local_selected_scheduled_merge_replacement_execution_input_from_snapshot_rejects_split_plan()
    {
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
            build_local_selected_scheduled_merge_replacement_execution_input_from_snapshot(
                &snapshot,
                &object_store,
                &selected,
                &rows,
                4,
                2,
                3000,
                4000,
                placement_write_evidence_for_pids(&[1, 11, 13, 21, 22]),
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
    fn selected_scheduled_replacement_leaf_rows_keeps_empty_affected_leaf() {
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let root = root_routing_object();
        let root_placement = object_store.insert_routing_object(7, &root).unwrap();
        let leaf_11 = object_store
            .insert_leaf_object_v2_from_rows(7, 11, 1, root.header.pid, &[primary_row(1, 10, 1)])
            .unwrap();
        let leaf_12 = object_store
            .insert_leaf_object_v2_from_rows(7, 12, 1, root.header.pid, &[])
            .unwrap();
        let leaf_13 = object_store
            .insert_leaf_object_v2_from_rows(7, 13, 1, root.header.pid, &[primary_row(3, 10, 3)])
            .unwrap();
        let placements = vec![root_placement, leaf_11, leaf_12, leaf_13];
        let epoch_manifest = SpireEpochManifest {
            epoch: 7,
            state: SpireEpochState::Published,
            consistency_mode: SpireConsistencyMode::Strict,
            published_at_micros: 1000,
            retain_until_micros: 2000,
            active_query_count: 0,
        };
        let object_manifest = SpireObjectManifest::from_entries(
            7,
            placements.iter().map(manifest_entry_for).collect(),
        )
        .unwrap();
        let placement_directory = SpirePlacementDirectory::from_entries(7, placements).unwrap();
        let snapshot = SpirePublishedEpochSnapshot::new(
            &epoch_manifest,
            &object_manifest,
            &placement_directory,
        )
        .unwrap();
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
        assert_eq!(rows[0].rows.len(), 1);
        assert!(rows[1].rows.is_empty());
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
    fn split_replacement_source_rows_hydrate_fetched_vectors_in_row_order() {
        let decision = scheduled_split_decision(7);

        let source_rows = build_split_replacement_source_rows(
            &decision,
            vec![SpireReplacementLeafRows {
                base_pid: 12,
                rows: vec![primary_row(1, 20, 1), primary_row(2, 20, 2)],
            }],
            vec![
                SpireSplitReplacementFetchedSourceVector {
                    heap_tid: tid(20, 2),
                    source_vector: vec![-1.0, 0.0],
                },
                SpireSplitReplacementFetchedSourceVector {
                    heap_tid: tid(20, 1),
                    source_vector: vec![1.0, 0.0],
                },
            ],
        )
        .unwrap();

        assert_eq!(
            source_rows
                .iter()
                .map(|row| row.assignment.vec_id.clone())
                .collect::<Vec<_>>(),
            vec![SpireVecId::local(1), SpireVecId::local(2)]
        );
        assert_eq!(
            source_rows
                .iter()
                .map(|row| row.source_vector.clone())
                .collect::<Vec<_>>(),
            vec![vec![1.0, 0.0], vec![-1.0, 0.0]]
        );
    }

    #[test]
    fn split_replacement_source_rows_reject_missing_or_stale_vectors() {
        let decision = scheduled_split_decision(7);

        assert!(build_split_replacement_source_rows(
            &decision,
            vec![SpireReplacementLeafRows {
                base_pid: 13,
                rows: vec![primary_row(1, 20, 1)],
            }],
            vec![SpireSplitReplacementFetchedSourceVector {
                heap_tid: tid(20, 1),
                source_vector: vec![1.0, 0.0],
            }],
        )
        .unwrap_err()
        .contains("unselected base pid"));

        assert!(build_split_replacement_source_rows(
            &decision,
            vec![SpireReplacementLeafRows {
                base_pid: 12,
                rows: vec![primary_row(1, 20, 1)],
            }],
            Vec::new(),
        )
        .unwrap_err()
        .contains("missing source vector"));

        assert!(build_split_replacement_source_rows(
            &decision,
            vec![SpireReplacementLeafRows {
                base_pid: 12,
                rows: vec![primary_row(1, 20, 1)],
            }],
            vec![
                SpireSplitReplacementFetchedSourceVector {
                    heap_tid: tid(20, 1),
                    source_vector: vec![1.0, 0.0],
                },
                SpireSplitReplacementFetchedSourceVector {
                    heap_tid: tid(20, 2),
                    source_vector: vec![-1.0, 0.0],
                },
            ],
        )
        .unwrap_err()
        .contains("unused source vector"));
    }

    #[test]
    fn split_replacement_rows_filter_to_fetched_heap_sources() {
        let filtered = filter_split_replacement_rows_to_fetched_sources(
            vec![SpireReplacementLeafRows {
                base_pid: 12,
                rows: vec![
                    primary_row(1, 20, 1),
                    primary_row(2, 20, 2),
                    primary_row(3, 20, 3),
                ],
            }],
            &[
                SpireSplitReplacementFetchedSourceVector {
                    heap_tid: tid(20, 3),
                    source_vector: vec![-1.0, 0.0],
                },
                SpireSplitReplacementFetchedSourceVector {
                    heap_tid: tid(20, 1),
                    source_vector: vec![1.0, 0.0],
                },
            ],
        )
        .unwrap();

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].base_pid, 12);
        assert_eq!(
            filtered[0]
                .rows
                .iter()
                .map(|row| row.heap_tid)
                .collect::<Vec<_>>(),
            vec![tid(20, 1), tid(20, 3)]
        );

        assert!(filter_split_replacement_rows_to_fetched_sources(
            vec![SpireReplacementLeafRows {
                base_pid: 12,
                rows: vec![primary_row(1, 20, 1)],
            }],
            &[
                SpireSplitReplacementFetchedSourceVector {
                    heap_tid: tid(20, 1),
                    source_vector: vec![1.0, 0.0],
                },
                SpireSplitReplacementFetchedSourceVector {
                    heap_tid: tid(20, 1),
                    source_vector: vec![1.0, 0.0],
                },
            ],
        )
        .unwrap_err()
        .contains("duplicate heap tid"));
    }

    #[test]
    fn split_replacement_materialization_trains_and_routes_source_rows() {
        let decision = scheduled_split_decision(7);
        let pid_plan = SpireLeafReplacementPidPlan {
            replacement_pids: vec![30, 31],
            reuses_existing_pid: false,
            next_pid: 32,
        };

        let materialized = build_split_replacement_leaf_materialization(
            &decision,
            &pid_plan,
            vec![
                SpireSplitReplacementSourceRow {
                    base_pid: 12,
                    assignment: primary_row(1, 20, 1),
                    source_vector: vec![1.0, 0.0],
                },
                SpireSplitReplacementSourceRow {
                    base_pid: 12,
                    assignment: primary_row(2, 20, 2),
                    source_vector: vec![-1.0, 0.0],
                },
            ],
            2,
            42,
            8,
        )
        .unwrap();

        assert_eq!(
            materialized.centroids,
            vec![vec![1.0, 0.0], vec![-1.0, 0.0]]
        );
        assert_eq!(
            materialized
                .leaf_inputs
                .iter()
                .map(|input| input.pid)
                .collect::<Vec<_>>(),
            vec![30, 31]
        );
        assert_eq!(
            materialized.leaf_inputs[0].rows[0].vec_id,
            SpireVecId::local(1)
        );
        assert_eq!(
            materialized.leaf_inputs[1].rows[0].vec_id,
            SpireVecId::local(2)
        );
    }

    #[test]
    fn split_replacement_materialization_from_rows_hydrates_trains_and_routes() {
        let decision = scheduled_split_decision(7);
        let pid_plan = SpireLeafReplacementPidPlan {
            replacement_pids: vec![30, 31],
            reuses_existing_pid: false,
            next_pid: 32,
        };

        let materialized = build_split_replacement_leaf_materialization_from_rows(
            &decision,
            &pid_plan,
            vec![SpireReplacementLeafRows {
                base_pid: 12,
                rows: vec![primary_row(1, 20, 1), primary_row(2, 20, 2)],
            }],
            vec![
                SpireSplitReplacementFetchedSourceVector {
                    heap_tid: tid(20, 2),
                    source_vector: vec![-1.0, 0.0],
                },
                SpireSplitReplacementFetchedSourceVector {
                    heap_tid: tid(20, 1),
                    source_vector: vec![1.0, 0.0],
                },
            ],
            2,
            42,
            8,
        )
        .unwrap();

        assert_eq!(
            materialized.centroids,
            vec![vec![1.0, 0.0], vec![-1.0, 0.0]]
        );
        assert_eq!(materialized.leaf_inputs[0].rows[0].heap_tid, tid(20, 1));
        assert_eq!(materialized.leaf_inputs[1].rows[0].heap_tid, tid(20, 2));
    }

    #[test]
    fn split_replacement_materialization_rejects_stale_or_bad_source_rows() {
        let decision = scheduled_split_decision(7);
        let pid_plan = SpireLeafReplacementPidPlan {
            replacement_pids: vec![30, 31],
            reuses_existing_pid: false,
            next_pid: 32,
        };

        assert!(build_split_replacement_leaf_materialization(
            &decision,
            &pid_plan,
            vec![SpireSplitReplacementSourceRow {
                base_pid: 13,
                assignment: primary_row(1, 20, 1),
                source_vector: vec![1.0, 0.0],
            }],
            2,
            42,
            8,
        )
        .unwrap_err()
        .contains("unselected base pid"));

        assert!(build_split_replacement_leaf_materialization(
            &decision,
            &pid_plan,
            vec![SpireSplitReplacementSourceRow {
                base_pid: 12,
                assignment: delta_insert_row(1, 20, 1),
                source_vector: vec![1.0, 0.0],
            }],
            2,
            42,
            8,
        )
        .unwrap_err()
        .contains("normalized base rows"));

        assert!(build_split_replacement_leaf_materialization(
            &decision,
            &pid_plan,
            vec![SpireSplitReplacementSourceRow {
                base_pid: 12,
                assignment: primary_row(1, 20, 1),
                source_vector: vec![0.0, 0.0],
            }],
            2,
            42,
            8,
        )
        .unwrap_err()
        .contains("non-zero vectors"));
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
    fn local_selected_scheduled_split_replacement_epoch_draft_from_snapshot_loads_parent() {
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

        let draft = build_local_selected_scheduled_split_replacement_epoch_draft_from_snapshot(
            &snapshot,
            &selected,
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
    fn local_selected_scheduled_split_replacement_epoch_draft_from_snapshot_rejects_merge_plan() {
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
            build_local_selected_scheduled_split_replacement_epoch_draft_from_snapshot(
                &snapshot,
                &selected,
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
