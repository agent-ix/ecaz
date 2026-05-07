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
