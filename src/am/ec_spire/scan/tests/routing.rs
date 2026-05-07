    #[test]
    fn route_root_object_to_leaf_pids_keeps_bounded_best_routes() {
        let root = SpireRoutingPartitionObject::root(
            SPIRE_FIRST_PID,
            1,
            2,
            vec![
                routing_child(0, SPIRE_FIRST_PID + 9, vec![-2.0, 0.0]),
                routing_child(1, SPIRE_FIRST_PID + 1, vec![1.0, 1.0]),
                routing_child(2, SPIRE_FIRST_PID + 2, vec![1.0, 0.0]),
                routing_child(3, SPIRE_FIRST_PID + 4, vec![2.0, 0.0]),
                routing_child(4, SPIRE_FIRST_PID + 7, vec![0.25, 0.0]),
            ],
        )
        .unwrap();

        assert_eq!(
            route_root_object_to_leaf_pids(&root, &[1.0, 0.0], 3).unwrap(),
            vec![
                SPIRE_FIRST_PID + 4,
                SPIRE_FIRST_PID + 1,
                SPIRE_FIRST_PID + 2
            ]
        );
    }

    #[test]
    fn route_routing_object_to_child_pids_routes_internal_level() {
        let internal = SpireRoutingPartitionObject::internal(
            SPIRE_FIRST_PID + 10,
            1,
            1,
            SPIRE_FIRST_PID,
            2,
            vec![
                routing_child(0, SPIRE_FIRST_PID + 1, vec![0.0, 1.0]),
                routing_child(1, SPIRE_FIRST_PID + 2, vec![1.0, 0.0]),
                routing_child(2, SPIRE_FIRST_PID + 3, vec![0.5, 0.0]),
            ],
        )
        .unwrap();

        assert_eq!(
            route_routing_object_to_child_pids(&internal, &[1.0, 0.0], 2).unwrap(),
            vec![SPIRE_FIRST_PID + 2, SPIRE_FIRST_PID + 3]
        );
    }

    #[test]
    fn route_top_graph_to_child_pids_uses_graph_frontier_with_deterministic_routes() {
        let root = SpireRoutingPartitionObject::root(
            SPIRE_FIRST_PID,
            1,
            2,
            vec![
                routing_child(0, SPIRE_FIRST_PID + 1, vec![1.0, 0.0]),
                routing_child(1, SPIRE_FIRST_PID + 2, vec![0.8, 0.2]),
                routing_child(2, SPIRE_FIRST_PID + 3, vec![-1.0, 0.0]),
                routing_child(3, SPIRE_FIRST_PID + 4, vec![-0.8, 0.2]),
            ],
        )
        .unwrap();
        let top_graph = build_spire_top_graph_draft_from_routing_object(
            &root,
            SpireTopGraphBuildParams {
                graph_degree: 2,
                build_list_size: 4,
                alpha: 1.2,
                seed: 42,
            },
        )
        .expect("top graph should build");

        let child_pids =
            route_top_graph_to_child_pids(&root, &top_graph, &[1.0, 0.0], 4, 2).unwrap();

        assert_eq!(
            child_pids,
            vec![SPIRE_FIRST_PID + 1, SPIRE_FIRST_PID + 2]
        );
    }

    #[test]
    fn route_top_graph_object_to_child_pids_uses_durable_graph_object() {
        let root = SpireRoutingPartitionObject::root(
            SPIRE_FIRST_PID,
            1,
            2,
            vec![
                routing_child(0, SPIRE_FIRST_PID + 1, vec![1.0, 0.0]),
                routing_child(1, SPIRE_FIRST_PID + 2, vec![0.8, 0.2]),
                routing_child(2, SPIRE_FIRST_PID + 3, vec![-1.0, 0.0]),
                routing_child(3, SPIRE_FIRST_PID + 4, vec![-0.8, 0.2]),
            ],
        )
        .unwrap();
        let top_graph_draft = build_spire_top_graph_draft_from_routing_object(
            &root,
            SpireTopGraphBuildParams {
                graph_degree: 2,
                build_list_size: 4,
                alpha: 1.2,
                seed: 42,
            },
        )
        .expect("top graph should build");
        let top_graph_object = spire_top_graph_partition_object_from_build_draft(
            SPIRE_FIRST_PID + 90,
            1,
            root.header.level,
            &top_graph_draft,
        )
        .unwrap();

        let child_pids =
            route_top_graph_object_to_child_pids(&root, &top_graph_object, &[1.0, 0.0], 4, 2)
                .unwrap();

        assert_eq!(
            child_pids,
            vec![SPIRE_FIRST_PID + 1, SPIRE_FIRST_PID + 2]
        );
    }

    #[test]
    fn route_top_graph_to_child_pids_rejects_root_mismatch() {
        let root = SpireRoutingPartitionObject::root(
            SPIRE_FIRST_PID,
            1,
            2,
            vec![
                routing_child(0, SPIRE_FIRST_PID + 1, vec![1.0, 0.0]),
                routing_child(1, SPIRE_FIRST_PID + 2, vec![-1.0, 0.0]),
            ],
        )
        .unwrap();
        let mut top_graph = build_spire_top_graph_draft_from_routing_object(
            &root,
            SpireTopGraphBuildParams {
                graph_degree: 2,
                build_list_size: 2,
                alpha: 1.2,
                seed: 42,
            },
        )
        .expect("top graph should build");
        top_graph.root_pid = SPIRE_FIRST_PID + 99;

        let error =
            route_top_graph_to_child_pids(&root, &top_graph, &[1.0, 0.0], 2, 1).unwrap_err();

        assert!(error.contains("does not match routing root pid"));
    }

    #[test]
    fn route_root_object_to_leaf_pids_still_rejects_internal_parent() {
        let internal = SpireRoutingPartitionObject::internal(
            SPIRE_FIRST_PID + 10,
            1,
            1,
            SPIRE_FIRST_PID,
            2,
            vec![routing_child(0, SPIRE_FIRST_PID + 1, vec![1.0, 0.0])],
        )
        .unwrap();

        let error = route_root_object_to_leaf_pids(&internal, &[1.0, 0.0], 1).unwrap_err();

        assert!(error.contains("root routing object"));
    }

    #[test]
    fn route_recursive_routing_objects_to_leaf_pids_descends_to_leaf_level() {
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
        let internal_a = SpireRoutingPartitionObject::internal(
            SPIRE_FIRST_PID + 10,
            1,
            1,
            SPIRE_FIRST_PID,
            2,
            vec![
                routing_child(0, SPIRE_FIRST_PID + 11, vec![0.5, 0.0]),
                routing_child(1, SPIRE_FIRST_PID + 12, vec![1.5, 0.0]),
            ],
        )
        .unwrap();
        let internal_b = SpireRoutingPartitionObject::internal(
            SPIRE_FIRST_PID + 20,
            1,
            1,
            SPIRE_FIRST_PID,
            2,
            vec![
                routing_child(0, SPIRE_FIRST_PID + 21, vec![-1.5, 0.0]),
                routing_child(1, SPIRE_FIRST_PID + 22, vec![-0.5, 0.0]),
            ],
        )
        .unwrap();
        let routing_objects_by_pid = HashMap::from([
            (internal_a.header.pid, internal_a),
            (internal_b.header.pid, internal_b),
        ]);

        assert_eq!(
            route_recursive_routing_objects_to_leaf_pids(
                &root,
                &routing_objects_by_pid,
                &[1.0, 0.0],
                1
            )
            .unwrap(),
            vec![SPIRE_FIRST_PID + 12]
        );
    }

    #[test]
    fn route_recursive_routing_objects_to_leaf_pids_rejects_missing_internal_child() {
        let root = SpireRoutingPartitionObject::root_at_level(
            SPIRE_FIRST_PID,
            1,
            2,
            2,
            vec![routing_child(0, SPIRE_FIRST_PID + 10, vec![1.0, 0.0])],
        )
        .unwrap();
        let routing_objects_by_pid = HashMap::new();

        let error = route_recursive_routing_objects_to_leaf_pids(
            &root,
            &routing_objects_by_pid,
            &[1.0, 0.0],
            1,
        )
        .unwrap_err();

        assert!(error.contains("missing internal routing child"));
    }

    #[test]
    fn route_recursive_routing_objects_to_leaf_pids_rejects_wrong_child_level() {
        let root = SpireRoutingPartitionObject::root_at_level(
            SPIRE_FIRST_PID,
            1,
            2,
            2,
            vec![routing_child(0, SPIRE_FIRST_PID + 10, vec![1.0, 0.0])],
        )
        .unwrap();
        let wrong_level_child = SpireRoutingPartitionObject::internal(
            SPIRE_FIRST_PID + 10,
            1,
            2,
            SPIRE_FIRST_PID,
            2,
            vec![routing_child(0, SPIRE_FIRST_PID + 11, vec![1.0, 0.0])],
        )
        .unwrap();
        let routing_objects_by_pid =
            HashMap::from([(wrong_level_child.header.pid, wrong_level_child)]);

        let error = route_recursive_routing_objects_to_leaf_pids(
            &root,
            &routing_objects_by_pid,
            &[1.0, 0.0],
            1,
        )
        .unwrap_err();

        assert!(error.contains("is not one below parent level"));
    }

    #[test]
    fn route_recursive_routing_objects_to_leaf_pids_uses_conservative_upper_level_nprobe() {
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
        let internal_a = SpireRoutingPartitionObject::internal(
            SPIRE_FIRST_PID + 10,
            1,
            1,
            SPIRE_FIRST_PID,
            2,
            vec![
                routing_child(0, SPIRE_FIRST_PID + 11, vec![0.5, 0.0]),
                routing_child(1, SPIRE_FIRST_PID + 12, vec![1.5, 0.0]),
            ],
        )
        .unwrap();
        let internal_b = SpireRoutingPartitionObject::internal(
            SPIRE_FIRST_PID + 20,
            1,
            1,
            SPIRE_FIRST_PID,
            2,
            vec![
                routing_child(0, SPIRE_FIRST_PID + 21, vec![-0.5, 0.0]),
                routing_child(1, SPIRE_FIRST_PID + 22, vec![-1.5, 0.0]),
            ],
        )
        .unwrap();
        let routing_objects_by_pid = HashMap::from([
            (internal_a.header.pid, internal_a),
            (internal_b.header.pid, internal_b),
        ]);

        let leaf_pids = route_recursive_routing_objects_to_leaf_pids(
            &root,
            &routing_objects_by_pid,
            &[1.0, 0.0],
            2,
        )
        .unwrap();

        assert_eq!(leaf_pids, vec![SPIRE_FIRST_PID + 12, SPIRE_FIRST_PID + 11]);
    }

    #[test]
    fn route_recursive_routing_objects_to_leaf_pids_descends_three_levels_conservatively() {
        let root = SpireRoutingPartitionObject::root_at_level(
            SPIRE_FIRST_PID,
            1,
            3,
            2,
            vec![
                routing_child(0, SPIRE_FIRST_PID + 100, vec![1.0, 0.0]),
                routing_child(1, SPIRE_FIRST_PID + 200, vec![-1.0, 0.0]),
            ],
        )
        .unwrap();
        let level_2_a = SpireRoutingPartitionObject::internal(
            SPIRE_FIRST_PID + 100,
            1,
            2,
            SPIRE_FIRST_PID,
            2,
            vec![
                routing_child(0, SPIRE_FIRST_PID + 110, vec![0.5, 0.0]),
                routing_child(1, SPIRE_FIRST_PID + 120, vec![0.4, 0.0]),
            ],
        )
        .unwrap();
        let level_2_b = SpireRoutingPartitionObject::internal(
            SPIRE_FIRST_PID + 200,
            1,
            2,
            SPIRE_FIRST_PID,
            2,
            vec![routing_child(0, SPIRE_FIRST_PID + 210, vec![-0.5, 0.0])],
        )
        .unwrap();
        let level_1_a = SpireRoutingPartitionObject::internal(
            SPIRE_FIRST_PID + 110,
            1,
            1,
            SPIRE_FIRST_PID + 100,
            2,
            vec![
                routing_child(0, SPIRE_FIRST_PID + 111, vec![2.0, 0.0]),
                routing_child(1, SPIRE_FIRST_PID + 112, vec![1.0, 0.0]),
            ],
        )
        .unwrap();
        let level_1_b = SpireRoutingPartitionObject::internal(
            SPIRE_FIRST_PID + 120,
            1,
            1,
            SPIRE_FIRST_PID + 100,
            2,
            vec![routing_child(0, SPIRE_FIRST_PID + 121, vec![3.0, 0.0])],
        )
        .unwrap();
        let level_1_c = SpireRoutingPartitionObject::internal(
            SPIRE_FIRST_PID + 210,
            1,
            1,
            SPIRE_FIRST_PID + 200,
            2,
            vec![routing_child(0, SPIRE_FIRST_PID + 211, vec![-2.0, 0.0])],
        )
        .unwrap();
        let routing_objects_by_pid = HashMap::from([
            (level_2_a.header.pid, level_2_a),
            (level_2_b.header.pid, level_2_b),
            (level_1_a.header.pid, level_1_a),
            (level_1_b.header.pid, level_1_b),
            (level_1_c.header.pid, level_1_c),
        ]);

        let leaf_pids = route_recursive_routing_objects_to_leaf_pids(
            &root,
            &routing_objects_by_pid,
            &[1.0, 0.0],
            2,
        )
        .unwrap();

        assert_eq!(
            leaf_pids,
            vec![SPIRE_FIRST_PID + 111, SPIRE_FIRST_PID + 112]
        );
    }

    #[test]
    fn load_snapshot_routing_hierarchy_loads_root_and_internal_objects() {
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let root = SpireRoutingPartitionObject::root_at_level(
            SPIRE_FIRST_PID,
            1,
            2,
            2,
            vec![routing_child(0, SPIRE_FIRST_PID + 1, vec![1.0, 0.0])],
        )
        .unwrap();
        let internal = SpireRoutingPartitionObject::internal(
            SPIRE_FIRST_PID + 1,
            1,
            1,
            SPIRE_FIRST_PID,
            2,
            vec![routing_child(0, SPIRE_FIRST_PID + 2, vec![1.0, 0.0])],
        )
        .unwrap();
        let root_placement = object_store.insert_routing_object(7, &root).unwrap();
        let internal_placement = object_store.insert_routing_object(7, &internal).unwrap();
        let leaf_placement = object_store
            .insert_leaf_object_v2_from_rows(7, SPIRE_FIRST_PID + 2, 1, SPIRE_FIRST_PID + 1, &[])
            .unwrap();
        let epoch_manifest = SpireEpochManifest {
            epoch: 7,
            state: SpireEpochState::Published,
            consistency_mode: SpireConsistencyMode::Strict,
            published_at_micros: 1000,
            retain_until_micros: 2000,
            active_query_count: 0,
        };
        let placements = vec![root_placement, internal_placement, leaf_placement];
        let object_manifest = SpireObjectManifest::from_entries(
            7,
            placements.iter().map(manifest_entry_for).collect(),
        )
        .unwrap();
        let placement_directory = SpirePlacementDirectory::from_entries(7, placements).unwrap();
        let snapshot = SpireValidatedEpochSnapshot::new(
            &epoch_manifest,
            &object_manifest,
            &placement_directory,
        )
        .unwrap();

        let hierarchy = load_snapshot_routing_hierarchy(&snapshot, &object_store)
            .expect("routing hierarchy should load");

        assert_eq!(hierarchy.root_pid, SPIRE_FIRST_PID);
        assert_eq!(hierarchy.root_object.header.level, 2);
        assert_eq!(hierarchy.internal_objects_by_pid.len(), 1);
        assert_eq!(
            hierarchy
                .internal_objects_by_pid
                .get(&(SPIRE_FIRST_PID + 1))
                .unwrap()
                .header
                .parent_pid,
            SPIRE_FIRST_PID
        );
    }

    #[test]
    fn load_snapshot_top_graph_object_loads_available_graph_object() {
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let root = SpireRoutingPartitionObject::root(
            SPIRE_FIRST_PID,
            1,
            2,
            vec![
                routing_child(0, SPIRE_FIRST_PID + 1, vec![1.0, 0.0]),
                routing_child(1, SPIRE_FIRST_PID + 2, vec![0.8, 0.2]),
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
            SPIRE_FIRST_PID + 20,
            1,
            root.header.level,
            &top_graph_draft,
        )
        .unwrap();
        let root_placement = object_store.insert_routing_object(7, &root).unwrap();
        let top_graph_placement = object_store
            .insert_top_graph_object(7, &top_graph_object)
            .unwrap();
        let first_leaf_placement = object_store
            .insert_leaf_object_v2_from_rows(7, SPIRE_FIRST_PID + 1, 1, SPIRE_FIRST_PID, &[])
            .unwrap();
        let second_leaf_placement = object_store
            .insert_leaf_object_v2_from_rows(7, SPIRE_FIRST_PID + 2, 1, SPIRE_FIRST_PID, &[])
            .unwrap();
        let epoch_manifest = SpireEpochManifest {
            epoch: 7,
            state: SpireEpochState::Published,
            consistency_mode: SpireConsistencyMode::Strict,
            published_at_micros: 1000,
            retain_until_micros: 2000,
            active_query_count: 0,
        };
        let placements = vec![
            root_placement,
            top_graph_placement,
            first_leaf_placement,
            second_leaf_placement,
        ];
        let object_manifest = SpireObjectManifest::from_entries(
            7,
            placements.iter().map(manifest_entry_for).collect(),
        )
        .unwrap();
        let placement_directory = SpirePlacementDirectory::from_entries(7, placements).unwrap();
        let snapshot = SpireValidatedEpochSnapshot::new(
            &epoch_manifest,
            &object_manifest,
            &placement_directory,
        )
        .unwrap();

        let (top_graph_pid, loaded_top_graph) =
            load_snapshot_top_graph_object(&snapshot, &object_store)
                .unwrap()
                .expect("top graph should load");

        assert_eq!(top_graph_pid, SPIRE_FIRST_PID + 20);
        assert_eq!(loaded_top_graph.header.kind, SpirePartitionObjectKind::TopGraph);
        assert_eq!(loaded_top_graph.header.parent_pid, SPIRE_FIRST_PID);
        assert_eq!(loaded_top_graph.header.published_epoch_backref, 7);
        assert_eq!(
            route_top_graph_object_to_child_pids(&root, &loaded_top_graph, &[1.0, 0.0], 2, 1)
                .unwrap(),
            vec![SPIRE_FIRST_PID + 1]
        );
    }

    #[test]
    fn top_graph_object_routes_recursive_children_to_leaf_routes() {
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
            vec![
                routing_child(0, SPIRE_FIRST_PID + 11, vec![1.0, 0.0]),
                routing_child(1, SPIRE_FIRST_PID + 12, vec![0.8, 0.2]),
            ],
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
        let routing_objects_by_pid = HashMap::from([
            (positive_internal.header.pid, positive_internal),
            (negative_internal.header.pid, negative_internal),
        ]);

        let leaf_routes = route_top_graph_object_to_leaf_routes(
            &root,
            &routing_objects_by_pid,
            &top_graph_object,
            &[1.0, 0.0],
            2,
            1,
            1,
        )
        .unwrap();

        assert_eq!(
            leaf_routes
                .iter()
                .map(|route| (route.parent_pid, route.leaf_pid))
                .collect::<Vec<_>>(),
            vec![(SPIRE_FIRST_PID + 10, SPIRE_FIRST_PID + 11)]
        );
    }

    #[test]
    fn collect_snapshot_top_graph_routed_probe_leaf_rows_uses_loaded_graph() {
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
                .insert_leaf_object_v2_from_rows(
                    7,
                    SPIRE_FIRST_PID + 11,
                    1,
                    SPIRE_FIRST_PID + 10,
                    &[assignment_row(SPIRE_ASSIGNMENT_FLAG_PRIMARY, 1)],
                )
                .unwrap(),
            object_store
                .insert_leaf_object_v2_from_rows(
                    7,
                    SPIRE_FIRST_PID + 21,
                    1,
                    SPIRE_FIRST_PID + 20,
                    &[assignment_row(SPIRE_ASSIGNMENT_FLAG_PRIMARY, 2)],
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
        let snapshot =
            snapshot_for_placement(&epoch_manifest, &object_manifest, &placement_directory);

        let routed = collect_snapshot_top_graph_routed_probe_leaf_rows(
            &snapshot,
            &object_store,
            &[1.0, 0.0],
            2,
            1,
            1,
        )
        .unwrap();

        assert_eq!(routed.len(), 1);
        assert_eq!(routed[0].root_pid, SPIRE_FIRST_PID);
        assert_eq!(routed[0].leaf_pid, SPIRE_FIRST_PID + 11);
        assert_eq!(routed[0].rows.len(), 1);
        assert_eq!(routed[0].rows[0].assignment.heap_tid, tid(10, 1));
    }

    #[test]
    fn recursive_routed_leaf_rows_skip_degraded_unavailable_unselected_internal() {
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
        let available_internal = SpireRoutingPartitionObject::internal(
            SPIRE_FIRST_PID + 10,
            1,
            1,
            SPIRE_FIRST_PID,
            2,
            vec![routing_child(0, SPIRE_FIRST_PID + 11, vec![1.0, 0.0])],
        )
        .unwrap();
        let unavailable_internal = SpireRoutingPartitionObject::internal(
            SPIRE_FIRST_PID + 20,
            1,
            1,
            SPIRE_FIRST_PID,
            2,
            vec![routing_child(0, SPIRE_FIRST_PID + 21, vec![-1.0, 0.0])],
        )
        .unwrap();
        let root_placement = object_store.insert_routing_object(7, &root).unwrap();
        let available_internal_placement = object_store
            .insert_routing_object(7, &available_internal)
            .unwrap();
        let mut unavailable_internal_placement = object_store
            .insert_routing_object(7, &unavailable_internal)
            .unwrap();
        unavailable_internal_placement.state = SpirePlacementState::Unavailable;
        let available_leaf_placement = object_store
            .insert_leaf_object_v2_from_rows(7, SPIRE_FIRST_PID + 11, 1, SPIRE_FIRST_PID + 10, &[])
            .unwrap();
        let unavailable_leaf_placement = object_store
            .insert_leaf_object_v2_from_rows(7, SPIRE_FIRST_PID + 21, 1, SPIRE_FIRST_PID + 20, &[])
            .unwrap();
        let epoch_manifest = SpireEpochManifest {
            epoch: 7,
            state: SpireEpochState::Published,
            consistency_mode: SpireConsistencyMode::Degraded,
            published_at_micros: 1000,
            retain_until_micros: 2000,
            active_query_count: 0,
        };
        let placements = vec![
            root_placement,
            available_internal_placement,
            unavailable_internal_placement,
            available_leaf_placement,
            unavailable_leaf_placement,
        ];
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
                .expect("degraded recursive route should skip unselected unavailable internal");

        assert_eq!(routed.len(), 1);
        assert_eq!(routed[0].root_pid, SPIRE_FIRST_PID);
        assert_eq!(routed[0].leaf_pid, SPIRE_FIRST_PID + 11);
    }

    #[test]
    fn load_snapshot_routing_hierarchy_rejects_multiple_roots() {
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let first_root = SpireRoutingPartitionObject::root(
            SPIRE_FIRST_PID,
            1,
            2,
            vec![routing_child(0, SPIRE_FIRST_PID + 10, vec![1.0, 0.0])],
        )
        .unwrap();
        let second_root = SpireRoutingPartitionObject::root(
            SPIRE_FIRST_PID + 1,
            1,
            2,
            vec![routing_child(0, SPIRE_FIRST_PID + 11, vec![-1.0, 0.0])],
        )
        .unwrap();
        let placements = vec![
            object_store.insert_routing_object(7, &first_root).unwrap(),
            object_store.insert_routing_object(7, &second_root).unwrap(),
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
        let snapshot = SpireValidatedEpochSnapshot::new(
            &epoch_manifest,
            &object_manifest,
            &placement_directory,
        )
        .unwrap();

        let error = load_snapshot_routing_hierarchy(&snapshot, &object_store).unwrap_err();

        assert!(error.contains("multiple root routing objects"));
    }

    #[test]
    fn recursive_route_matches_flat_single_level_on_small_hierarchy() {
        let flat_root = SpireRoutingPartitionObject::root(
            SPIRE_FIRST_PID,
            1,
            2,
            vec![
                routing_child(0, SPIRE_FIRST_PID + 11, vec![0.5, 0.0]),
                routing_child(1, SPIRE_FIRST_PID + 12, vec![1.5, 0.0]),
                routing_child(2, SPIRE_FIRST_PID + 21, vec![-1.5, 0.0]),
                routing_child(3, SPIRE_FIRST_PID + 22, vec![-0.5, 0.0]),
            ],
        )
        .unwrap();
        let recursive_root = SpireRoutingPartitionObject::root_at_level(
            SPIRE_FIRST_PID + 100,
            1,
            2,
            2,
            vec![
                routing_child(0, SPIRE_FIRST_PID + 10, vec![1.0, 0.0]),
                routing_child(1, SPIRE_FIRST_PID + 20, vec![-1.0, 0.0]),
            ],
        )
        .unwrap();
        let internal_a = SpireRoutingPartitionObject::internal(
            SPIRE_FIRST_PID + 10,
            1,
            1,
            SPIRE_FIRST_PID + 100,
            2,
            vec![
                routing_child(0, SPIRE_FIRST_PID + 11, vec![0.5, 0.0]),
                routing_child(1, SPIRE_FIRST_PID + 12, vec![1.5, 0.0]),
            ],
        )
        .unwrap();
        let internal_b = SpireRoutingPartitionObject::internal(
            SPIRE_FIRST_PID + 20,
            1,
            1,
            SPIRE_FIRST_PID + 100,
            2,
            vec![
                routing_child(0, SPIRE_FIRST_PID + 21, vec![-1.5, 0.0]),
                routing_child(1, SPIRE_FIRST_PID + 22, vec![-0.5, 0.0]),
            ],
        )
        .unwrap();
        let routing_objects_by_pid = HashMap::from([
            (internal_a.header.pid, internal_a),
            (internal_b.header.pid, internal_b),
        ]);

        let query = [1.0, 0.0];
        let flat_best = route_root_object_to_leaf_pids(&flat_root, &query, 1).unwrap();
        let recursive_best = route_recursive_routing_objects_to_leaf_pids(
            &recursive_root,
            &routing_objects_by_pid,
            &query,
            1,
        )
        .unwrap();

        assert_eq!(flat_best, vec![SPIRE_FIRST_PID + 12]);
        assert_eq!(recursive_best, flat_best);
    }
