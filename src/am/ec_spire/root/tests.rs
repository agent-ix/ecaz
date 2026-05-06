#[cfg(test)]
mod tests {
    use super::*;

    fn tid(block_number: u32, offset_number: u16) -> crate::storage::page::ItemPointer {
        crate::storage::page::ItemPointer {
            block_number,
            offset_number,
        }
    }

    fn published_epoch_manifest(epoch: u64) -> meta::SpireEpochManifest {
        meta::SpireEpochManifest {
            epoch,
            state: meta::SpireEpochState::Published,
            consistency_mode: meta::SpireConsistencyMode::Strict,
            published_at_micros: 1,
            retain_until_micros: 1,
            active_query_count: 0,
        }
    }

    fn retired_epoch_manifest(epoch: u64) -> meta::SpireEpochManifest {
        meta::SpireEpochManifest {
            epoch,
            state: meta::SpireEpochState::Retired,
            consistency_mode: meta::SpireConsistencyMode::Strict,
            published_at_micros: 1,
            retain_until_micros: 1,
            active_query_count: 0,
        }
    }

    fn manifest_entry_for(placement: &meta::SpirePlacementEntry) -> meta::SpireManifestEntry {
        meta::SpireManifestEntry {
            epoch: placement.epoch,
            pid: placement.pid,
            object_version: placement.object_version,
            placement_tid: placement.object_tid,
        }
    }

    fn empty_leaf_row(
        store: &mut storage::SpireLocalObjectStore,
        pid: u64,
        parent_pid: u64,
    ) -> meta::SpirePlacementEntry {
        store
            .insert_leaf_object_v2_from_rows(1, pid, 1, parent_pid, &[])
            .expect("empty leaf object should store")
    }

    #[test]
    fn scan_sanity_status_reports_empty_approximate_and_full_scan() {
        assert_eq!(
            scan_sanity_status(0, false, false),
            (
                "empty",
                "none",
                "build or insert rows to publish the first SPIRE epoch"
            )
        );
        assert_eq!(
            scan_sanity_status(1, false, false),
            (
                "approximate_leaf_coverage",
                "bounded_leaf_probe",
                "increase nprobe to active_leaf_count for exact leaf coverage sanity checks"
            )
        );
        assert_eq!(
            scan_sanity_status(1, true, false),
            (
                "exact_leaf_coverage_bounded_rerank",
                "bounded_rerank",
                "set rerank_width = 0 for full-frontier exact recall sanity checks"
            )
        );
        assert_eq!(
            scan_sanity_status(1, true, true),
            (
                "exact_leaf_and_frontier_coverage",
                "full_scan",
                "use this configuration only for recall sanity checks or small indexes"
            )
        );
    }

    #[test]
    fn epoch_snapshot_partial_retired_residue_keeps_root_manifest_authoritative() {
        let active_tid = tid(10, 1);
        let retired_residue_tid = tid(10, 2);
        let root_control =
            meta::SpireRootControlState::published(7, 40, 100, active_tid, tid(10, 3), tid(10, 4))
                .expect("root/control should build");

        let rows = epoch_snapshot_rows_from_manifests(
            root_control,
            vec![
                (active_tid, published_epoch_manifest(7)),
                (retired_residue_tid, retired_epoch_manifest(7)),
            ],
            2,
        )
        .expect("epoch snapshot rows should build");

        assert_eq!(rows.len(), 2);
        let active_row = rows
            .iter()
            .find(|row| row.manifest_offset == active_tid.offset_number)
            .expect("active root row should exist");
        let retired_residue_row = rows
            .iter()
            .find(|row| row.manifest_offset == retired_residue_tid.offset_number)
            .expect("retired residue row should exist");

        assert_eq!(active_row.state, "published");
        assert!(active_row.is_active_root_manifest);
        assert!(!active_row.cleanup_eligible_now);
        assert_eq!(active_row.cleanup_blocked_reason, "active_root_manifest");
        assert_eq!(retired_residue_row.state, "retired");
        assert!(!retired_residue_row.is_active_root_manifest);
        assert!(!retired_residue_row.cleanup_eligible_now);
        assert_eq!(
            retired_residue_row.cleanup_blocked_reason,
            "retained_retired_epoch"
        );
    }

    #[test]
    fn epoch_snapshot_bundle_residue_keeps_previous_root_manifest_authoritative() {
        let active_tid = tid(10, 1);
        let retired_residue_tid = tid(10, 2);
        let bundle_residue_tid = tid(10, 3);
        let root_control =
            meta::SpireRootControlState::published(7, 40, 100, active_tid, tid(10, 4), tid(10, 5))
                .expect("root/control should build");

        let rows = epoch_snapshot_rows_from_manifests(
            root_control,
            vec![
                (active_tid, published_epoch_manifest(7)),
                (retired_residue_tid, retired_epoch_manifest(7)),
                (bundle_residue_tid, published_epoch_manifest(8)),
            ],
            2,
        )
        .expect("epoch snapshot rows should build");

        assert_eq!(rows.len(), 3);
        let active_row = rows
            .iter()
            .find(|row| row.manifest_offset == active_tid.offset_number)
            .expect("active root row should exist");
        let bundle_residue_row = rows
            .iter()
            .find(|row| row.epoch == 8)
            .expect("bundle residue row should exist");

        assert_eq!(active_row.epoch, 7);
        assert_eq!(active_row.state, "published");
        assert!(active_row.is_active_root_manifest);
        assert!(!active_row.cleanup_eligible_now);
        assert_eq!(active_row.cleanup_blocked_reason, "active_root_manifest");
        assert_eq!(bundle_residue_row.state, "published");
        assert!(!bundle_residue_row.is_active_root_manifest);
        assert!(!bundle_residue_row.cleanup_eligible_now);
        assert_eq!(
            bundle_residue_row.cleanup_blocked_reason,
            "state_not_cleanup_eligible"
        );
    }

    #[test]
    fn leaf_maintenance_thresholds_use_named_split_merge_policy() {
        assert_eq!(leaf_maintenance_thresholds(0, 0), (0, 0));
        assert_eq!(leaf_maintenance_thresholds(2, 3), (32, 0));
        assert_eq!(leaf_maintenance_thresholds(120, 3), (160, 10));
    }

    fn root_for_child(pid: u64, child_pid: u64) -> storage::SpireRoutingPartitionObject {
        storage::SpireRoutingPartitionObject::root(
            pid,
            1,
            2,
            vec![storage::SpireRoutingChildEntry {
                centroid_index: 0,
                child_pid,
                centroid: vec![1.0, 0.0],
            }],
        )
        .expect("root routing object should build")
    }

    fn hierarchy_summary(
        pid: u64,
        kind: storage::SpirePartitionObjectKind,
        level: u16,
        parent_pid: u64,
        child_pids: Vec<u64>,
    ) -> SpireHierarchyObjectSummary {
        SpireHierarchyObjectSummary {
            pid,
            kind,
            level,
            parent_pid,
            child_pids,
        }
    }

    #[test]
    fn recursive_hierarchy_shape_accepts_single_level_root_to_leaves() {
        let objects = vec![
            hierarchy_summary(
                1,
                storage::SpirePartitionObjectKind::Root,
                1,
                0,
                vec![11, 12],
            ),
            hierarchy_summary(
                11,
                storage::SpirePartitionObjectKind::Leaf,
                0,
                1,
                Vec::new(),
            ),
            hierarchy_summary(
                12,
                storage::SpirePartitionObjectKind::Leaf,
                0,
                1,
                Vec::new(),
            ),
        ];

        let has_internal =
            validate_recursive_hierarchy_shape(&objects).expect("shape should validate");

        assert!(!has_internal);
    }

    #[test]
    fn recursive_hierarchy_shape_accepts_internal_level_between_root_and_leaves() {
        let objects = vec![
            hierarchy_summary(1, storage::SpirePartitionObjectKind::Root, 2, 0, vec![10]),
            hierarchy_summary(
                10,
                storage::SpirePartitionObjectKind::Internal,
                1,
                1,
                vec![11, 12],
            ),
            hierarchy_summary(
                11,
                storage::SpirePartitionObjectKind::Leaf,
                0,
                10,
                Vec::new(),
            ),
            hierarchy_summary(
                12,
                storage::SpirePartitionObjectKind::Leaf,
                0,
                10,
                Vec::new(),
            ),
        ];

        let has_internal =
            validate_recursive_hierarchy_shape(&objects).expect("shape should validate");

        assert!(has_internal);
    }

    #[test]
    fn recursive_hierarchy_shape_rejects_level_skip_to_leaf() {
        let objects = vec![
            hierarchy_summary(1, storage::SpirePartitionObjectKind::Root, 2, 0, vec![11]),
            hierarchy_summary(
                11,
                storage::SpirePartitionObjectKind::Leaf,
                0,
                1,
                Vec::new(),
            ),
        ];

        let err = validate_recursive_hierarchy_shape(&objects).unwrap_err();

        assert!(err.contains("child pid 11 has kind Leaf level 0"));
    }

    #[test]
    fn recursive_hierarchy_shape_rejects_orphan_leaf_parent_link() {
        let objects = vec![
            hierarchy_summary(1, storage::SpirePartitionObjectKind::Root, 1, 0, vec![11]),
            hierarchy_summary(
                11,
                storage::SpirePartitionObjectKind::Leaf,
                0,
                99,
                Vec::new(),
            ),
        ];

        let err = validate_recursive_hierarchy_shape(&objects).unwrap_err();

        assert!(err.contains("parent_pid 99 does not match routing pid 1"));
    }

    fn maintenance_leaf_row(
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
            node_id: meta::SPIRE_LOCAL_NODE_ID,
            local_store_id: meta::SPIRE_SINGLE_LOCAL_STORE_ID,
            placement_state: "available",
            base_assignment_count: effective_assignment_count,
            delta_object_count: 0,
            delta_insert_assignment_count: 0,
            delta_delete_assignment_count: 0,
            effective_assignment_count,
            split_assignment_threshold: 32,
            merge_assignment_threshold: 1,
            split_recommended,
            merge_recommended,
            maintenance_action: "none",
            maintenance_reason: "test",
            leaf_object_bytes: 1,
            delta_object_bytes: 0,
        }
    }

    #[test]
    fn maintenance_plan_snapshot_reports_selected_split_plan() {
        let root_control =
            meta::SpireRootControlState::published(7, 40, 100, tid(1, 1), tid(1, 2), tid(1, 3))
                .expect("root control should build");
        let rows = vec![
            maintenance_leaf_row(11, 1, 10, false, false),
            maintenance_leaf_row(12, 1, 100, true, false),
        ];

        let snapshot =
            maintenance_plan_snapshot_from_rows(root_control, &published_epoch_manifest(7), &rows)
                .expect("maintenance plan should build");

        assert_eq!(snapshot.active_epoch, 7);
        assert_eq!(snapshot.planner_status, "planned");
        assert_eq!(snapshot.planned_action, "split");
        assert_eq!(snapshot.planned_reason, "largest_split_candidate");
        assert_eq!(snapshot.replaced_parent_pid, 1);
        assert_eq!(snapshot.affected_leaf_pids, vec![12]);
        assert_eq!(snapshot.replacement_leaf_count, 2);
        assert_eq!(snapshot.replacement_leaf_pids, vec![40, 41]);
        assert_eq!(snapshot.publish_epoch, 8);
        assert_eq!(snapshot.next_pid, 42);
        assert_eq!(snapshot.next_local_vec_seq, 100);
    }

    #[test]
    fn maintenance_plan_snapshot_reports_no_action_without_candidate() {
        let root_control =
            meta::SpireRootControlState::published(7, 40, 100, tid(1, 1), tid(1, 2), tid(1, 3))
                .expect("root control should build");
        let rows = vec![
            maintenance_leaf_row(11, 1, 10, false, false),
            maintenance_leaf_row(12, 1, 11, false, false),
        ];

        let snapshot =
            maintenance_plan_snapshot_from_rows(root_control, &published_epoch_manifest(7), &rows)
                .expect("maintenance plan should build");

        assert_eq!(snapshot.active_epoch, 7);
        assert_eq!(snapshot.planner_status, "no_action");
        assert_eq!(snapshot.planned_action, "none");
        assert_eq!(snapshot.planned_reason, "no_candidate");
        assert_eq!(snapshot.replacement_leaf_pids, Vec::<u64>::new());
        assert_eq!(snapshot.next_pid, 40);
        assert_eq!(snapshot.next_local_vec_seq, 100);
    }

    #[test]
    fn maintenance_plan_snapshot_reports_selected_merge_plan() {
        let root_control =
            meta::SpireRootControlState::published(7, 40, 100, tid(1, 1), tid(1, 2), tid(1, 3))
                .expect("root control should build");
        let rows = vec![
            maintenance_leaf_row(11, 1, 3, false, true),
            maintenance_leaf_row(12, 1, 1, false, true),
            maintenance_leaf_row(13, 2, 20, false, false),
        ];

        let snapshot =
            maintenance_plan_snapshot_from_rows(root_control, &published_epoch_manifest(7), &rows)
                .expect("maintenance plan should build");

        assert_eq!(snapshot.active_epoch, 7);
        assert_eq!(snapshot.planner_status, "planned");
        assert_eq!(snapshot.planned_action, "merge");
        assert_eq!(snapshot.planned_reason, "sparsest_same_parent_merge_pair");
        assert_eq!(snapshot.replaced_parent_pid, 1);
        assert_eq!(snapshot.affected_leaf_pids, vec![11, 12]);
        assert_eq!(snapshot.replacement_leaf_count, 1);
        assert_eq!(snapshot.replacement_leaf_pids, vec![40]);
        assert_eq!(snapshot.publish_epoch, 8);
        assert_eq!(snapshot.next_pid, 41);
        assert_eq!(snapshot.next_local_vec_seq, 100);
    }

    fn selected_split_maintenance_plan() -> update::SpireSelectedScheduledReplacementPublishLockPlan
    {
        update::SpireSelectedScheduledReplacementPublishLockPlan {
            decision: update::SpireLeafReplacementScheduleDecision {
                mode: update::SpireLeafReplacementScheduleMode::Split,
                active_epoch: 7,
                replaced_parent_pid: 1,
                affected_leaf_pids: vec![12],
                replacement_leaf_count: 2,
                reason: "largest_split_candidate",
            },
            lock_plan: update::SpireScheduledReplacementPublishLockPlan {
                pid_plan: update::SpireLeafReplacementPidPlan {
                    replacement_pids: vec![40, 41],
                    reuses_existing_pid: false,
                    next_pid: 42,
                },
                publish_plan: update::SpireScheduledReplacementPublishPlan {
                    epoch: 8,
                    consistency_mode: meta::SpireConsistencyMode::Strict,
                    next_pid: 42,
                    next_local_vec_seq: 100,
                },
            },
        }
    }

    #[test]
    fn maintenance_run_result_reports_no_action() {
        let root_control =
            meta::SpireRootControlState::published(7, 40, 100, tid(1, 1), tid(1, 2), tid(1, 3))
                .expect("root control should build");

        let result = no_maintenance_run_result(
            root_control,
            7,
            "no_candidate",
            "active leaves are within split/merge thresholds",
        );

        assert_eq!(result.active_epoch_before, 7);
        assert_eq!(result.active_epoch_after, 7);
        assert_eq!(result.maintenance_status, "no_action");
        assert_eq!(result.planned_action, "none");
        assert_eq!(result.planned_reason, "no_candidate");
        assert_eq!(result.replacement_leaf_pids, Vec::<u64>::new());
        assert_eq!(result.publish_epoch, 0);
        assert_eq!(result.next_pid, 40);
        assert_eq!(result.next_local_vec_seq, 100);
        assert!(!result.published);
    }

    #[test]
    fn maintenance_run_result_reports_projected_selected_plan() {
        let result = selected_maintenance_run_result(
            selected_split_maintenance_plan(),
            "planned",
            false,
            "scheduled replacement selected; no epoch was published",
        )
        .expect("maintenance run result should build");

        assert_eq!(result.active_epoch_before, 7);
        assert_eq!(result.active_epoch_after, 7);
        assert_eq!(result.maintenance_status, "planned");
        assert_eq!(result.planned_action, "split");
        assert_eq!(result.planned_reason, "largest_split_candidate");
        assert_eq!(result.replaced_parent_pid, 1);
        assert_eq!(result.affected_leaf_pids, vec![12]);
        assert_eq!(result.replacement_leaf_count, 2);
        assert_eq!(result.replacement_leaf_pids, vec![40, 41]);
        assert_eq!(result.publish_epoch, 8);
        assert_eq!(result.next_pid, 42);
        assert_eq!(result.next_local_vec_seq, 100);
        assert!(!result.published);
    }

    #[test]
    fn maintenance_run_result_reports_published_selected_plan() {
        let result = selected_maintenance_run_result(
            selected_split_maintenance_plan(),
            "published",
            true,
            "scheduled replacement epoch was published",
        )
        .expect("maintenance run result should build");

        assert_eq!(result.active_epoch_before, 7);
        assert_eq!(result.active_epoch_after, 8);
        assert_eq!(result.maintenance_status, "published");
        assert_eq!(result.planned_action, "split");
        assert_eq!(result.publish_epoch, 8);
        assert!(result.published);
    }

    #[test]
    fn maintenance_run_plan_from_rows_reports_selected_split_without_publishing() {
        let root_control =
            meta::SpireRootControlState::published(7, 40, 100, tid(1, 1), tid(1, 2), tid(1, 3))
                .expect("root control should build");
        let rows = vec![
            maintenance_leaf_row(11, 1, 10, false, false),
            maintenance_leaf_row(12, 1, 100, true, false),
        ];

        let result =
            maintenance_run_result_from_rows(root_control, &published_epoch_manifest(7), &rows)
                .expect("maintenance run plan should build");

        assert_eq!(result.active_epoch_before, 7);
        assert_eq!(result.active_epoch_after, 7);
        assert_eq!(result.maintenance_status, "planned");
        assert_eq!(result.planned_action, "split");
        assert_eq!(result.planned_reason, "largest_split_candidate");
        assert_eq!(result.replacement_leaf_pids, vec![40, 41]);
        assert_eq!(result.publish_epoch, 8);
        assert_eq!(result.next_pid, 42);
        assert!(!result.published);
    }

    #[test]
    fn maintenance_run_plan_from_rows_reports_no_candidate() {
        let root_control =
            meta::SpireRootControlState::published(7, 40, 100, tid(1, 1), tid(1, 2), tid(1, 3))
                .expect("root control should build");
        let rows = vec![
            maintenance_leaf_row(11, 1, 10, false, false),
            maintenance_leaf_row(12, 1, 11, false, false),
        ];

        let result =
            maintenance_run_result_from_rows(root_control, &published_epoch_manifest(7), &rows)
                .expect("maintenance run plan should build");

        assert_eq!(result.active_epoch_before, 7);
        assert_eq!(result.active_epoch_after, 7);
        assert_eq!(result.maintenance_status, "no_action");
        assert_eq!(result.planned_action, "none");
        assert_eq!(result.planned_reason, "no_candidate");
        assert_eq!(result.replacement_leaf_pids, Vec::<u64>::new());
        assert_eq!(result.next_pid, 40);
        assert!(!result.published);
    }

    #[test]
    fn scheduled_replacement_object_version_plan_uses_successor_versions() {
        let selected = selected_split_maintenance_plan();
        let mut unaffected = maintenance_leaf_row(11, 1, 10, false, false);
        unaffected.object_version = 9;
        let mut affected = maintenance_leaf_row(12, 1, 100, true, false);
        affected.object_version = 3;

        let plan = scheduled_replacement_object_version_plan(&selected, 4, &[unaffected, affected])
            .expect("object version plan should build");

        assert_eq!(plan.parent_object_version, 5);
        assert_eq!(plan.leaf_object_version, 4);
    }

    #[test]
    fn scheduled_replacement_object_version_plan_uses_max_affected_leaf_successor() {
        let selected = update::SpireSelectedScheduledReplacementPublishLockPlan {
            decision: update::SpireLeafReplacementScheduleDecision {
                mode: update::SpireLeafReplacementScheduleMode::Merge,
                active_epoch: 7,
                replaced_parent_pid: 1,
                affected_leaf_pids: vec![11, 12],
                replacement_leaf_count: 1,
                reason: "sparsest_same_parent_merge_pair",
            },
            lock_plan: update::SpireScheduledReplacementPublishLockPlan {
                pid_plan: update::SpireLeafReplacementPidPlan {
                    replacement_pids: vec![40],
                    reuses_existing_pid: false,
                    next_pid: 41,
                },
                publish_plan: update::SpireScheduledReplacementPublishPlan {
                    epoch: 8,
                    consistency_mode: meta::SpireConsistencyMode::Strict,
                    next_pid: 41,
                    next_local_vec_seq: 100,
                },
            },
        };
        let mut first = maintenance_leaf_row(11, 1, 3, false, true);
        first.object_version = 2;
        let mut second = maintenance_leaf_row(12, 1, 1, false, true);
        second.object_version = 5;

        let plan = scheduled_replacement_object_version_plan(&selected, 4, &[first, second])
            .expect("object version plan should build");

        assert_eq!(plan.parent_object_version, 5);
        assert_eq!(plan.leaf_object_version, 6);
    }

    #[test]
    fn scheduled_replacement_object_version_plan_rejects_missing_affected_leaf() {
        let selected = selected_split_maintenance_plan();
        let rows = vec![maintenance_leaf_row(11, 1, 10, false, false)];

        let err = scheduled_replacement_object_version_plan(&selected, 4, &rows).unwrap_err();

        assert!(err.contains("missing affected leaf rows"));
    }

    #[test]
    fn leaf_snapshot_base_row_preserves_prior_delta_counts() {
        let mut rows_by_leaf_pid = HashMap::new();
        rows_by_leaf_pid.insert(
            20,
            SpireIndexLeafSnapshotRow {
                active_epoch: 7,
                leaf_pid: 20,
                parent_pid: 0,
                object_version: 0,
                node_id: meta::SPIRE_LOCAL_NODE_ID,
                local_store_id: meta::SPIRE_SINGLE_LOCAL_STORE_ID,
                placement_state: "missing_base_leaf",
                base_assignment_count: 0,
                delta_object_count: 2,
                delta_insert_assignment_count: 3,
                delta_delete_assignment_count: 1,
                effective_assignment_count: 0,
                split_assignment_threshold: 0,
                merge_assignment_threshold: 0,
                split_recommended: false,
                merge_recommended: false,
                maintenance_action: "none",
                maintenance_reason: "missing_base_leaf",
                leaf_object_bytes: 0,
                delta_object_bytes: 44,
            },
        );
        let header = storage::SpirePartitionObjectHeader {
            kind: storage::SpirePartitionObjectKind::Leaf,
            pid: 20,
            object_version: 9,
            published_epoch_backref: 7,
            level: 1,
            parent_pid: 10,
            child_count: 0,
            assignment_count: 5,
            flags: 0,
        };
        let placement = meta::SpirePlacementEntry::local_single_store_available(
            7,
            20,
            12345,
            9,
            crate::storage::page::ItemPointer {
                block_number: 30,
                offset_number: 4,
            },
            88,
        );

        apply_leaf_snapshot_base_row(&mut rows_by_leaf_pid, 7, &header, &placement);

        let row = rows_by_leaf_pid.get(&20).expect("leaf row should exist");
        assert_eq!(row.parent_pid, 10);
        assert_eq!(row.object_version, 9);
        assert_eq!(row.base_assignment_count, 5);
        assert_eq!(row.leaf_object_bytes, 88);
        assert_eq!(row.placement_state, "available");
        assert_eq!(row.maintenance_reason, "not_evaluated");
        assert_eq!(row.delta_object_count, 2);
        assert_eq!(row.delta_insert_assignment_count, 3);
        assert_eq!(row.delta_delete_assignment_count, 1);
        assert_eq!(row.delta_object_bytes, 44);
    }

    #[test]
    fn root_routing_snapshot_rejects_active_manifest_without_root() {
        let mut store = storage::SpireLocalObjectStore::with_default_page_size(12345)
            .expect("local store should build");
        let leaf = empty_leaf_row(&mut store, 20, 10);
        let epoch_manifest = published_epoch_manifest(1);
        let object_manifest =
            meta::SpireObjectManifest::from_entries(1, vec![manifest_entry_for(&leaf)])
                .expect("object manifest should build");
        let placement_directory = meta::SpirePlacementDirectory::from_entries(1, vec![leaf])
            .expect("placement directory should build");
        let snapshot = meta::SpireValidatedEpochSnapshot::new(
            &epoch_manifest,
            &object_manifest,
            &placement_directory,
        )
        .expect("snapshot should validate");

        let err = collect_root_routing_snapshot_rows(&snapshot, &store)
            .expect_err("rootless active snapshot should fail");

        assert_eq!(
            err,
            "ec_spire root routing snapshot found no active root object"
        );
    }

    #[test]
    fn root_routing_snapshot_rejects_active_manifest_with_multiple_roots() {
        let mut store = storage::SpireLocalObjectStore::with_default_page_size(12345)
            .expect("local store should build");
        let leaf = empty_leaf_row(&mut store, 20, 10);
        let first_root = store
            .insert_routing_object(1, &root_for_child(10, 20))
            .expect("first root should store");
        let second_root = store
            .insert_routing_object(1, &root_for_child(11, 20))
            .expect("second root should store");
        let epoch_manifest = published_epoch_manifest(1);
        let placements = vec![first_root, second_root, leaf];
        let object_manifest = meta::SpireObjectManifest::from_entries(
            1,
            placements.iter().map(manifest_entry_for).collect(),
        )
        .expect("object manifest should build");
        let placement_directory = meta::SpirePlacementDirectory::from_entries(1, placements)
            .expect("placement directory should build");
        let snapshot = meta::SpireValidatedEpochSnapshot::new(
            &epoch_manifest,
            &object_manifest,
            &placement_directory,
        )
        .expect("snapshot should validate");

        let err = collect_root_routing_snapshot_rows(&snapshot, &store)
            .expect_err("multi-root active snapshot should fail");

        assert_eq!(
            err,
            "ec_spire root routing snapshot found multiple root objects"
        );
    }

    #[test]
    fn recursive_maintenance_guard_rejects_recursive_hierarchy() {
        let mut store = storage::SpireLocalObjectStore::with_default_page_size(12345)
            .expect("local store should build");
        let leaf = empty_leaf_row(&mut store, 30, 20);
        let internal = store
            .insert_routing_object(
                1,
                &storage::SpireRoutingPartitionObject::internal(
                    20,
                    1,
                    1,
                    10,
                    2,
                    vec![storage::SpireRoutingChildEntry {
                        centroid_index: 0,
                        child_pid: 30,
                        centroid: vec![1.0, 0.0],
                    }],
                )
                .expect("internal routing object should build"),
            )
            .expect("internal should store");
        let root = store
            .insert_routing_object(
                1,
                &storage::SpireRoutingPartitionObject::root_at_level(
                    10,
                    1,
                    2,
                    2,
                    vec![storage::SpireRoutingChildEntry {
                        centroid_index: 0,
                        child_pid: 20,
                        centroid: vec![1.0, 0.0],
                    }],
                )
                .expect("root routing object should build"),
            )
            .expect("root should store");
        let epoch_manifest = published_epoch_manifest(1);
        let placements = vec![root, internal, leaf];
        let object_manifest = meta::SpireObjectManifest::from_entries(
            1,
            placements.iter().map(manifest_entry_for).collect(),
        )
        .expect("object manifest should build");
        let placement_directory = meta::SpirePlacementDirectory::from_entries(1, placements)
            .expect("placement directory should build");
        let snapshot = meta::SpireValidatedEpochSnapshot::new(
            &epoch_manifest,
            &object_manifest,
            &placement_directory,
        )
        .expect("snapshot should validate");

        let err = reject_recursive_maintenance_until_update_propagation(&snapshot, &store)
            .expect_err("recursive hierarchy should reject maintenance");

        assert_eq!(err, RECURSIVE_MAINTENANCE_DEFERRED_MESSAGE);
    }

    #[test]
    fn root_routing_snapshot_reports_child_rows_from_local_store() {
        let mut store = storage::SpireLocalObjectStore::with_default_page_size(12345)
            .expect("local store should build");
        let leaf = empty_leaf_row(&mut store, 20, 10);
        let root = store
            .insert_routing_object(1, &root_for_child(10, 20))
            .expect("root should store");
        let epoch_manifest = published_epoch_manifest(1);
        let placements = vec![root, leaf];
        let object_manifest = meta::SpireObjectManifest::from_entries(
            1,
            placements.iter().map(manifest_entry_for).collect(),
        )
        .expect("object manifest should build");
        let placement_directory = meta::SpirePlacementDirectory::from_entries(1, placements)
            .expect("placement directory should build");
        let snapshot = meta::SpireValidatedEpochSnapshot::new(
            &epoch_manifest,
            &object_manifest,
            &placement_directory,
        )
        .expect("snapshot should validate");

        let rows = collect_root_routing_snapshot_rows(&snapshot, &store)
            .expect("root routing rows should collect");

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].root_pid, 10);
        assert_eq!(rows[0].child_pid, 20);
        assert_eq!(rows[0].child_kind, "leaf");
        assert_eq!(rows[0].child_store_relid, 12345);
    }
}
