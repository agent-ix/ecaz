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

