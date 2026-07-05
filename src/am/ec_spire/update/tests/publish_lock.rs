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
