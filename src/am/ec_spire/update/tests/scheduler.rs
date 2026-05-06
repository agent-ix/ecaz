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

