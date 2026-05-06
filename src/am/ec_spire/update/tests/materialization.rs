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

