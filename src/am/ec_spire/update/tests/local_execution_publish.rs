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

