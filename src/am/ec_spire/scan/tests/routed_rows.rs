    #[test]
    fn collect_snapshot_routed_leaf_rows_routes_query_to_leaf_pid() {
        let mut pid_allocator = SpirePidAllocator::default();
        let mut local_vec_id_allocator = SpireLocalVecIdAllocator::default();
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let draft = build_partitioned_single_level_leaf_epoch_draft(
            partitioned_build_input(
                vec![assignment_input(10, 1), assignment_input(10, 2)],
                vec![0, 1],
            ),
            &mut pid_allocator,
            &mut local_vec_id_allocator,
            &mut object_store,
        )
        .unwrap();
        let snapshot = SpirePublishedEpochSnapshot::new(
            &draft.epoch_manifest,
            &draft.object_manifest,
            &draft.placement_directory,
        )
        .unwrap();

        let positive_rows =
            collect_snapshot_routed_leaf_rows(&snapshot, &object_store, &[1.0, 0.0]).unwrap();
        let negative_rows =
            collect_snapshot_routed_leaf_rows(&snapshot, &object_store, &[-1.0, 0.0]).unwrap();

        assert_eq!(positive_rows.root_pid, SPIRE_FIRST_PID);
        assert_eq!(positive_rows.leaf_pid, SPIRE_FIRST_PID + 1);
        assert_eq!(positive_rows.rows.len(), 1);
        assert_eq!(positive_rows.rows[0].assignment.heap_tid, tid(10, 1));
        assert_eq!(negative_rows.root_pid, SPIRE_FIRST_PID);
        assert_eq!(negative_rows.leaf_pid, SPIRE_FIRST_PID + 2);
        assert_eq!(negative_rows.rows.len(), 1);
        assert_eq!(negative_rows.rows[0].assignment.heap_tid, tid(10, 2));
    }

    #[test]
    fn collect_snapshot_routed_probe_leaf_rows_routes_top_nprobe_leaf_pids() {
        let mut pid_allocator = SpirePidAllocator::default();
        let mut local_vec_id_allocator = SpireLocalVecIdAllocator::default();
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let draft = build_partitioned_single_level_leaf_epoch_draft(
            partitioned_build_input(
                vec![assignment_input(10, 1), assignment_input(10, 2)],
                vec![0, 1],
            ),
            &mut pid_allocator,
            &mut local_vec_id_allocator,
            &mut object_store,
        )
        .unwrap();
        let snapshot = SpirePublishedEpochSnapshot::new(
            &draft.epoch_manifest,
            &draft.object_manifest,
            &draft.placement_directory,
        )
        .unwrap();

        let routed =
            collect_snapshot_routed_probe_leaf_rows(&snapshot, &object_store, &[1.0, 0.0], 2)
                .unwrap();

        assert_eq!(routed.len(), 2);
        assert_eq!(routed[0].leaf_pid, SPIRE_FIRST_PID + 1);
        assert_eq!(routed[0].rows[0].assignment.heap_tid, tid(10, 1));
        assert_eq!(routed[1].leaf_pid, SPIRE_FIRST_PID + 2);
        assert_eq!(routed[1].rows[0].assignment.heap_tid, tid(10, 2));
    }

    #[test]
    fn collect_scan_plan_selected_leaf_pids_does_not_read_remote_leaf_payloads() {
        let mut pid_allocator = SpirePidAllocator::default();
        let mut local_vec_id_allocator = SpireLocalVecIdAllocator::default();
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let draft = build_partitioned_single_level_leaf_epoch_draft(
            partitioned_build_input(
                vec![assignment_input(10, 1), assignment_input(10, 2)],
                vec![0, 1],
            ),
            &mut pid_allocator,
            &mut local_vec_id_allocator,
            &mut object_store,
        )
        .unwrap();
        let remote_leaf_pid = draft.centroid_pids[0];
        let mut placements = draft.placement_directory.entries.clone();
        let remote_placement = placements
            .iter_mut()
            .find(|placement| placement.pid == remote_leaf_pid)
            .expect("remote leaf placement should exist");
        remote_placement.node_id = 2;
        remote_placement.store_relid = 999;
        remote_placement.object_tid = tid(99, 9);
        remote_placement.object_bytes = 1;
        let placement_directory = SpirePlacementDirectory::from_entries(7, placements).unwrap();
        let snapshot = SpirePublishedEpochSnapshot::new(
            &draft.epoch_manifest,
            &draft.object_manifest,
            &placement_directory,
        )
        .unwrap();
        let options = EcSpireOptions {
            nprobe: 2,
            ..EcSpireOptions::DEFAULT
        };
        let scan_plan = resolve_single_level_scan_plan_values(2, options.clone(), -1, -1).unwrap();
        let query = SpireScanQuery::new(vec![1.0, 0.0]).unwrap();

        let selected = collect_scan_plan_selected_leaf_pids(
            &snapshot,
            &object_store,
            &query,
            scan_plan,
            options.top_graph_plan().unwrap(),
        )
        .unwrap();

        assert_eq!(selected, draft.centroid_pids);
        assert!(collect_snapshot_routed_probe_leaf_rows(
            &snapshot,
            &object_store,
            &[1.0, 0.0],
            2,
        )
        .is_err());
    }

    #[test]
    fn collect_snapshot_routed_probe_leaf_rows_accepts_recursive_leaf_parent() {
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let root_pid = SPIRE_FIRST_PID;
        let internal_pid = SPIRE_FIRST_PID + 1;
        let first_leaf_pid = SPIRE_FIRST_PID + 2;
        let second_leaf_pid = SPIRE_FIRST_PID + 3;
        let root = SpireRoutingPartitionObject::root_at_level(
            root_pid,
            1,
            2,
            2,
            vec![routing_child(0, internal_pid, vec![1.0, 0.0])],
        )
        .unwrap();
        let internal = SpireRoutingPartitionObject::internal(
            internal_pid,
            1,
            1,
            root_pid,
            2,
            vec![
                routing_child(0, first_leaf_pid, vec![0.5, 0.0]),
                routing_child(1, second_leaf_pid, vec![1.5, 0.0]),
            ],
        )
        .unwrap();
        let first_leaf_rows = vec![assignment_row(SPIRE_ASSIGNMENT_FLAG_PRIMARY, 1)];
        let second_leaf_rows = vec![assignment_row(SPIRE_ASSIGNMENT_FLAG_PRIMARY, 2)];
        let placements = vec![
            object_store.insert_routing_object(7, &root).unwrap(),
            object_store.insert_routing_object(7, &internal).unwrap(),
            object_store
                .insert_leaf_object_v2_from_rows(
                    7,
                    first_leaf_pid,
                    1,
                    internal_pid,
                    &first_leaf_rows,
                )
                .unwrap(),
            object_store
                .insert_leaf_object_v2_from_rows(
                    7,
                    second_leaf_pid,
                    1,
                    internal_pid,
                    &second_leaf_rows,
                )
                .unwrap(),
        ];
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

        let routed =
            collect_snapshot_routed_probe_leaf_rows(&snapshot, &object_store, &[1.0, 0.0], 1)
                .unwrap();

        assert_eq!(routed.len(), 1);
        assert_eq!(routed[0].root_pid, root_pid);
        assert_eq!(routed[0].leaf_pid, second_leaf_pid);
        assert_eq!(routed[0].rows.len(), 1);
        assert_eq!(routed[0].rows[0].assignment.heap_tid, tid(10, 2));
    }
