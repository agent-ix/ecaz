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
    fn collect_resolved_scan_plan_selection_loads_routing_hierarchy_once() {
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
        let options = EcSpireOptions {
            nprobe: 2,
            ..EcSpireOptions::DEFAULT
        };
        let query = SpireScanQuery::new(vec![1.0, 0.0]).unwrap();

        let selection =
            collect_resolved_scan_plan_selection(&snapshot, &object_store, &options, &query)
                .unwrap();

        assert_eq!(selection.scan_plan.leaf_count, 2);
        assert_eq!(selection.scan_plan.nprobe, 2);
        assert_eq!(selection.selected_leaf_pids, draft.centroid_pids);
        assert_eq!(selection.routing_hierarchy_load_count, 1);
        assert_eq!(selection.top_graph_load_count, 0);
    }

    #[test]
    fn collect_resolved_scan_plan_selection_reuses_loaded_top_graph() {
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let root = SpireRoutingPartitionObject::root_at_level(
            SPIRE_FIRST_PID,
            1,
            2,
            2,
            vec![
                routing_child(0, SPIRE_FIRST_PID + 10, vec![1.0, 0.0]),
                routing_child(1, SPIRE_FIRST_PID + 20, vec![-1.0, 0.0]),
            ],
        )
        .unwrap();
        let top_graph_draft = build_spire_top_graph_draft_from_routing_object(
            &root,
            SpireTopGraphBuildParams {
                graph_degree: 1,
                build_list_size: 2,
                alpha: 1.2,
                seed: 42,
            },
        )
        .unwrap();
        let top_graph_object = spire_top_graph_partition_object_from_build_draft(
            SPIRE_FIRST_PID + 90,
            1,
            root.header.level,
            &top_graph_draft,
        )
        .unwrap();
        let positive_internal = SpireRoutingPartitionObject::internal(
            SPIRE_FIRST_PID + 10,
            1,
            1,
            SPIRE_FIRST_PID,
            2,
            vec![routing_child(0, SPIRE_FIRST_PID + 11, vec![1.0, 0.0])],
        )
        .unwrap();
        let negative_internal = SpireRoutingPartitionObject::internal(
            SPIRE_FIRST_PID + 20,
            1,
            1,
            SPIRE_FIRST_PID,
            2,
            vec![routing_child(0, SPIRE_FIRST_PID + 21, vec![-1.0, 0.0])],
        )
        .unwrap();
        let placements = vec![
            object_store.insert_routing_object(7, &root).unwrap(),
            object_store
                .insert_routing_object(7, &positive_internal)
                .unwrap(),
            object_store
                .insert_routing_object(7, &negative_internal)
                .unwrap(),
            object_store
                .insert_top_graph_object(7, &top_graph_object)
                .unwrap(),
            object_store
                .insert_leaf_object_v2_from_rows(7, SPIRE_FIRST_PID + 11, 1, SPIRE_FIRST_PID + 10, &[])
                .unwrap(),
            object_store
                .insert_leaf_object_v2_from_rows(7, SPIRE_FIRST_PID + 21, 1, SPIRE_FIRST_PID + 20, &[])
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
        let options = EcSpireOptions {
            nprobe: 1,
            top_graph_enabled: 1,
            top_graph_degree: 1,
            top_graph_build_list_size: 2,
            top_graph_search_list_size: 2,
            ..EcSpireOptions::DEFAULT
        };
        let query = SpireScanQuery::new(vec![1.0, 0.0]).unwrap();

        let selection =
            collect_resolved_scan_plan_selection(&snapshot, &object_store, &options, &query)
                .unwrap();

        assert_eq!(selection.scan_plan.leaf_count, 2);
        assert_eq!(selection.selected_leaf_pids, vec![SPIRE_FIRST_PID + 11]);
        assert_eq!(selection.routing_hierarchy_load_count, 1);
        assert_eq!(selection.top_graph_load_count, 1);
    }

    #[test]
    fn collect_cached_resolved_scan_plan_selection_reuses_epoch_hierarchy() {
        reset_coordinator_routing_hierarchy_cache_for_test();
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let root = SpireRoutingPartitionObject::root_at_level(
            SPIRE_FIRST_PID,
            1,
            2,
            2,
            vec![
                routing_child(0, SPIRE_FIRST_PID + 10, vec![1.0, 0.0]),
                routing_child(1, SPIRE_FIRST_PID + 20, vec![-1.0, 0.0]),
            ],
        )
        .unwrap();
        let top_graph_draft = build_spire_top_graph_draft_from_routing_object(
            &root,
            SpireTopGraphBuildParams {
                graph_degree: 1,
                build_list_size: 2,
                alpha: 1.2,
                seed: 42,
            },
        )
        .unwrap();
        let top_graph_object = spire_top_graph_partition_object_from_build_draft(
            SPIRE_FIRST_PID + 90,
            1,
            root.header.level,
            &top_graph_draft,
        )
        .unwrap();
        let positive_internal = SpireRoutingPartitionObject::internal(
            SPIRE_FIRST_PID + 10,
            1,
            1,
            SPIRE_FIRST_PID,
            2,
            vec![routing_child(0, SPIRE_FIRST_PID + 11, vec![1.0, 0.0])],
        )
        .unwrap();
        let negative_internal = SpireRoutingPartitionObject::internal(
            SPIRE_FIRST_PID + 20,
            1,
            1,
            SPIRE_FIRST_PID,
            2,
            vec![routing_child(0, SPIRE_FIRST_PID + 21, vec![-1.0, 0.0])],
        )
        .unwrap();
        let placements = vec![
            object_store.insert_routing_object(7, &root).unwrap(),
            object_store
                .insert_routing_object(7, &positive_internal)
                .unwrap(),
            object_store
                .insert_routing_object(7, &negative_internal)
                .unwrap(),
            object_store
                .insert_top_graph_object(7, &top_graph_object)
                .unwrap(),
            object_store
                .insert_leaf_object_v2_from_rows(7, SPIRE_FIRST_PID + 11, 1, SPIRE_FIRST_PID + 10, &[])
                .unwrap(),
            object_store
                .insert_leaf_object_v2_from_rows(7, SPIRE_FIRST_PID + 21, 1, SPIRE_FIRST_PID + 20, &[])
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
        let options = EcSpireOptions {
            nprobe: 1,
            top_graph_enabled: 1,
            top_graph_degree: 1,
            top_graph_build_list_size: 2,
            top_graph_search_list_size: 2,
            ..EcSpireOptions::DEFAULT
        };
        let cache_key = SpireRoutingHierarchyCacheKey {
            index_relid: 12345,
            active_epoch: 7,
        };
        let query = SpireScanQuery::new(vec![1.0, 0.0]).unwrap();

        let first =
            collect_cached_resolved_scan_plan_selection(cache_key, &snapshot, &object_store, &options, &query)
                .unwrap();
        let second =
            collect_cached_resolved_scan_plan_selection(cache_key, &snapshot, &object_store, &options, &query)
                .unwrap();

        assert_eq!(first.selected_leaf_pids, second.selected_leaf_pids);
        assert_eq!(first.routing_hierarchy_load_count, 1);
        assert_eq!(first.top_graph_load_count, 1);
        assert_eq!(second.routing_hierarchy_load_count, 0);
        assert_eq!(second.top_graph_load_count, 0);
    }

    #[test]
    fn collect_cached_resolved_scan_plan_selection_reloads_on_epoch_change() {
        reset_coordinator_routing_hierarchy_cache_for_test();
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();

        let epoch7_root = SpireRoutingPartitionObject::root_at_level(
            SPIRE_FIRST_PID,
            1,
            2,
            2,
            vec![
                routing_child(0, SPIRE_FIRST_PID + 10, vec![1.0, 0.0]),
                routing_child(1, SPIRE_FIRST_PID + 20, vec![-1.0, 0.0]),
            ],
        )
        .unwrap();
        let epoch7_positive_internal = SpireRoutingPartitionObject::internal(
            SPIRE_FIRST_PID + 10,
            1,
            1,
            SPIRE_FIRST_PID,
            2,
            vec![routing_child(0, SPIRE_FIRST_PID + 11, vec![1.0, 0.0])],
        )
        .unwrap();
        let epoch7_negative_internal = SpireRoutingPartitionObject::internal(
            SPIRE_FIRST_PID + 20,
            1,
            1,
            SPIRE_FIRST_PID,
            2,
            vec![routing_child(0, SPIRE_FIRST_PID + 21, vec![-1.0, 0.0])],
        )
        .unwrap();
        let epoch7_placements = vec![
            object_store.insert_routing_object(7, &epoch7_root).unwrap(),
            object_store
                .insert_routing_object(7, &epoch7_positive_internal)
                .unwrap(),
            object_store
                .insert_routing_object(7, &epoch7_negative_internal)
                .unwrap(),
            object_store
                .insert_leaf_object_v2_from_rows(
                    7,
                    SPIRE_FIRST_PID + 11,
                    1,
                    SPIRE_FIRST_PID + 10,
                    &[],
                )
                .unwrap(),
            object_store
                .insert_leaf_object_v2_from_rows(
                    7,
                    SPIRE_FIRST_PID + 21,
                    1,
                    SPIRE_FIRST_PID + 20,
                    &[],
                )
                .unwrap(),
        ];
        let epoch7_manifest = SpireEpochManifest {
            epoch: 7,
            state: SpireEpochState::Published,
            consistency_mode: SpireConsistencyMode::Strict,
            published_at_micros: 1000,
            retain_until_micros: 2000,
            active_query_count: 0,
        };
        let epoch7_object_manifest = SpireObjectManifest::from_entries(
            7,
            epoch7_placements.iter().map(manifest_entry_for).collect(),
        )
        .unwrap();
        let epoch7_placement_directory =
            SpirePlacementDirectory::from_entries(7, epoch7_placements).unwrap();
        let epoch7_snapshot = SpirePublishedEpochSnapshot::new(
            &epoch7_manifest,
            &epoch7_object_manifest,
            &epoch7_placement_directory,
        )
        .unwrap();

        let epoch8_root = SpireRoutingPartitionObject::root_at_level(
            SPIRE_FIRST_PID + 100,
            1,
            2,
            2,
            vec![
                routing_child(0, SPIRE_FIRST_PID + 110, vec![-1.0, 0.0]),
                routing_child(1, SPIRE_FIRST_PID + 120, vec![1.0, 0.0]),
            ],
        )
        .unwrap();
        let epoch8_negative_internal = SpireRoutingPartitionObject::internal(
            SPIRE_FIRST_PID + 110,
            1,
            1,
            SPIRE_FIRST_PID + 100,
            2,
            vec![routing_child(0, SPIRE_FIRST_PID + 111, vec![-1.0, 0.0])],
        )
        .unwrap();
        let epoch8_positive_internal = SpireRoutingPartitionObject::internal(
            SPIRE_FIRST_PID + 120,
            1,
            1,
            SPIRE_FIRST_PID + 100,
            2,
            vec![routing_child(0, SPIRE_FIRST_PID + 121, vec![1.0, 0.0])],
        )
        .unwrap();
        let epoch8_placements = vec![
            object_store.insert_routing_object(8, &epoch8_root).unwrap(),
            object_store
                .insert_routing_object(8, &epoch8_negative_internal)
                .unwrap(),
            object_store
                .insert_routing_object(8, &epoch8_positive_internal)
                .unwrap(),
            object_store
                .insert_leaf_object_v2_from_rows(
                    8,
                    SPIRE_FIRST_PID + 111,
                    1,
                    SPIRE_FIRST_PID + 110,
                    &[],
                )
                .unwrap(),
            object_store
                .insert_leaf_object_v2_from_rows(
                    8,
                    SPIRE_FIRST_PID + 121,
                    1,
                    SPIRE_FIRST_PID + 120,
                    &[],
                )
                .unwrap(),
        ];
        let epoch8_manifest = SpireEpochManifest {
            epoch: 8,
            state: SpireEpochState::Published,
            consistency_mode: SpireConsistencyMode::Strict,
            published_at_micros: 3000,
            retain_until_micros: 4000,
            active_query_count: 0,
        };
        let epoch8_object_manifest = SpireObjectManifest::from_entries(
            8,
            epoch8_placements.iter().map(manifest_entry_for).collect(),
        )
        .unwrap();
        let epoch8_placement_directory =
            SpirePlacementDirectory::from_entries(8, epoch8_placements).unwrap();
        let epoch8_snapshot = SpirePublishedEpochSnapshot::new(
            &epoch8_manifest,
            &epoch8_object_manifest,
            &epoch8_placement_directory,
        )
        .unwrap();

        let options = EcSpireOptions {
            nprobe: 1,
            ..EcSpireOptions::DEFAULT
        };
        let query = SpireScanQuery::new(vec![1.0, 0.0]).unwrap();
        let index_relid = 12345;

        let epoch7_selection = collect_cached_resolved_scan_plan_selection(
            SpireRoutingHierarchyCacheKey {
                index_relid,
                active_epoch: 7,
            },
            &epoch7_snapshot,
            &object_store,
            &options,
            &query,
        )
        .unwrap();
        let epoch8_selection = collect_cached_resolved_scan_plan_selection(
            SpireRoutingHierarchyCacheKey {
                index_relid,
                active_epoch: 8,
            },
            &epoch8_snapshot,
            &object_store,
            &options,
            &query,
        )
        .unwrap();

        assert_eq!(epoch7_selection.selected_leaf_pids, vec![SPIRE_FIRST_PID + 11]);
        assert_eq!(epoch8_selection.selected_leaf_pids, vec![SPIRE_FIRST_PID + 121]);
        assert_eq!(epoch7_selection.routing_hierarchy_load_count, 1);
        assert_eq!(epoch8_selection.routing_hierarchy_load_count, 1);
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
