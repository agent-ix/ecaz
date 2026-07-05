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
