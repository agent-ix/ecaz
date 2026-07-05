    #[test]
    fn local_scheduled_split_replacement_execution_parts_preserve_write_evidence() {
        let decision = scheduled_split_decision(7);
        let pid_plan = SpireLeafReplacementPidPlan {
            replacement_pids: vec![21, 22],
            reuses_existing_pid: false,
            next_pid: 23,
        };

        let parts = build_local_scheduled_split_replacement_execution_parts(
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
            placement_write_evidence_for_pids(&[1, 21, 22]),
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
        assert_eq!(
            parts
                .leaf_inputs
                .iter()
                .map(|input| input.pid)
                .collect::<Vec<_>>(),
            vec![21, 22]
        );
        assert_eq!(
            parts
                .placement_write_evidence
                .iter()
                .map(|evidence| evidence.pid)
                .collect::<Vec<_>>(),
            vec![1, 21, 22]
        );
    }

    #[test]
    fn local_scheduled_split_replacement_execution_input_uses_publish_plan() {
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

        let input = build_local_scheduled_split_replacement_execution_input(
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
    fn local_scheduled_split_replacement_execution_input_rejects_plan_drift() {
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

        assert!(build_local_scheduled_split_replacement_execution_input(
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
            placement_write_evidence_for_pids(&[1, 21, 22]),
        )
        .unwrap_err()
        .contains("next_pid"));
    }
