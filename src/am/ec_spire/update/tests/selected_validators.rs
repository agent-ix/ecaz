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
