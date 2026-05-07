    fn scheduled_replacement_snapshot_fixture(
        object_store: &mut SpireLocalObjectStore,
        active_epoch: u64,
        root: &SpireRoutingPartitionObject,
    ) -> ScheduledReplacementSnapshotFixture {
        let root_placement = object_store
            .insert_routing_object(active_epoch, root)
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
        let placements = vec![root_placement, leaf_11, leaf_12, leaf_13];
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
            placements
                .iter()
                .map(manifest_entry_for)
                .collect::<Vec<_>>(),
        )
        .unwrap();
        let placement_directory =
            SpirePlacementDirectory::from_entries(active_epoch, placements).unwrap();
        ScheduledReplacementSnapshotFixture {
            epoch_manifest,
            object_manifest,
            placement_directory,
        }
    }

    #[test]
    fn scheduled_replacement_parent_loader_reads_decision_parent() {
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let root = root_routing_object();
        let fixture = scheduled_replacement_snapshot_fixture(&mut object_store, 7, &root);
        let snapshot = fixture.snapshot();
        let decision = scheduled_split_decision(7);

        let parent =
            load_scheduled_replacement_parent_routing(&snapshot, &object_store, &decision).unwrap();
        let mut expected = root;
        expected.header.published_epoch_backref = 7;

        assert_eq!(parent, expected);
    }

    #[test]
    fn scheduled_replacement_parent_loader_rejects_stale_inputs() {
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let root = root_routing_object();
        let fixture = scheduled_replacement_snapshot_fixture(&mut object_store, 7, &root);
        let snapshot = fixture.snapshot();
        let mut stale_decision = scheduled_split_decision(8);
        assert!(load_scheduled_replacement_parent_routing(
            &snapshot,
            &object_store,
            &stale_decision,
        )
        .unwrap_err()
        .contains("snapshot epoch"));

        stale_decision.active_epoch = 7;
        stale_decision.replaced_parent_pid = 12;
        assert!(load_scheduled_replacement_parent_routing(
            &snapshot,
            &object_store,
            &stale_decision,
        )
        .is_err());

        let missing_child_root = SpireRoutingPartitionObject::root(
            1,
            2,
            2,
            vec![
                routing_child(0, 11, vec![1.0, 0.0]),
                routing_child(1, 13, vec![-1.0, 0.0]),
            ],
        )
        .unwrap();
        let mut missing_child_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let missing_child_fixture = scheduled_replacement_snapshot_fixture(
            &mut missing_child_store,
            7,
            &missing_child_root,
        );
        let missing_child_snapshot = missing_child_fixture.snapshot();
        let decision = scheduled_split_decision(7);
        assert!(load_scheduled_replacement_parent_routing(
            &missing_child_snapshot,
            &missing_child_store,
            &decision,
        )
        .unwrap_err()
        .contains("missing affected leaf"));
    }

    #[test]
    fn selected_scheduled_replacement_parent_loader_uses_lock_plan() {
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

        let parent =
            load_selected_scheduled_replacement_parent_routing(&snapshot, &object_store, &selected)
                .unwrap();

        assert_eq!(parent.header.pid, 1);
        assert_eq!(
            parent
                .children()
                .map(|child| child.child_pid)
                .collect::<Vec<_>>(),
            vec![11, 12, 13]
        );
    }

    #[test]
    fn selected_scheduled_replacement_parent_loader_rejects_snapshot_drift() {
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let root = root_routing_object();
        let fixture = scheduled_replacement_snapshot_fixture(&mut object_store, 7, &root);
        let snapshot = fixture.snapshot();
        let selected = SpireSelectedScheduledReplacementPublishLockPlan {
            decision: SpireLeafReplacementScheduleDecision {
                active_epoch: 6,
                ..scheduled_split_decision(7)
            },
            lock_plan: SpireScheduledReplacementPublishLockPlan {
                pid_plan: SpireLeafReplacementPidPlan {
                    replacement_pids: vec![21, 22],
                    reuses_existing_pid: false,
                    next_pid: 23,
                },
                publish_plan: SpireScheduledReplacementPublishPlan {
                    epoch: 7,
                    consistency_mode: SpireConsistencyMode::Strict,
                    next_pid: 23,
                    next_local_vec_seq: 7,
                },
            },
        };

        assert!(load_selected_scheduled_replacement_parent_routing(
            &snapshot,
            &object_store,
            &selected
        )
        .unwrap_err()
        .contains("snapshot epoch"));
    }
