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

