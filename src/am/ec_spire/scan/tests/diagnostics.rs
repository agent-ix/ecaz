    #[test]
    fn collect_scan_placement_diagnostics_counts_routed_store_rows() {
        let mut pid_allocator = SpirePidAllocator::default();
        let mut local_vec_id_allocator = SpireLocalVecIdAllocator::default();
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let base_draft = build_partitioned_single_level_leaf_epoch_draft(
            partitioned_build_input(
                vec![
                    quantized_assignment_input(
                        10,
                        1,
                        SpireAssignmentPayloadFormat::TurboQuant,
                        &[1.0, 0.0],
                    ),
                    quantized_assignment_input(
                        10,
                        2,
                        SpireAssignmentPayloadFormat::TurboQuant,
                        &[-1.0, 0.0],
                    ),
                ],
                vec![0, 1],
            ),
            &mut pid_allocator,
            &mut local_vec_id_allocator,
            &mut object_store,
        )
        .unwrap();
        let base_snapshot = SpirePublishedEpochSnapshot::new(
            &base_draft.epoch_manifest,
            &base_draft.object_manifest,
            &base_draft.placement_directory,
        )
        .unwrap();
        let mut delta = delta_input(
            vec![quantized_assignment_input(
                30,
                1,
                SpireAssignmentPayloadFormat::TurboQuant,
                &[1.0, 0.0],
            )],
            vec![delete_delta_input(1, 10, 1)],
        );
        delta.base_pid = SPIRE_FIRST_PID + 1;
        let delta_draft = build_delta_epoch_draft_from_snapshot(
            delta,
            &base_snapshot,
            &mut pid_allocator,
            &mut local_vec_id_allocator,
            &mut object_store,
        )
        .unwrap();
        let snapshot = SpirePublishedEpochSnapshot::new(
            &delta_draft.epoch_manifest,
            &delta_draft.object_manifest,
            &delta_draft.placement_directory,
        )
        .unwrap();
        let query = SpireScanQuery::new(vec![1.0, 0.0]).unwrap();
        let scan_plan = SpireSingleLevelScanPlan {
            leaf_count: 2,
            nprobe: 1,
            nprobe_source: "relation",
            recursive_nprobe_policy: SpireRecursiveNprobePolicy::conservative(1).unwrap(),
            recursive_route_budget: SpireRecursiveRouteBudget::unbounded(),
            payload_format: SpireAssignmentPayloadFormat::TurboQuant,
            rerank_width: 10,
            rerank_width_source: "relation",
            candidate_limit: Some(10),
            dedupe_mode: SpireCandidateDedupeMode::NoReplicaDedupeDisabled,
        };

        let diagnostics = collect_single_level_scan_plan_placement_diagnostics(
            &snapshot,
            &object_store,
            &query,
            scan_plan,
        )
        .unwrap();

        assert_eq!(diagnostics.scan_plan.leaf_count, 2);
        assert_eq!(diagnostics.scan_plan.nprobe, 1);
        assert_eq!(diagnostics.scan_plan.nprobe_source, "relation");
        assert_eq!(diagnostics.stores.len(), 1);
        let store = &diagnostics.stores[0];
        assert_eq!(store.epoch, 8);
        assert_eq!(store.node_id, 0);
        assert_eq!(store.local_store_id, 0);
        assert_eq!(store.scanned_pid_count, 2);
        assert_eq!(store.leaf_pid_count, 1);
        assert_eq!(store.delta_pid_count, 1);
        assert_eq!(store.candidate_row_count, 1);
        assert_eq!(store.leaf_candidate_row_count, 0);
        assert_eq!(store.delta_candidate_row_count, 1);
        assert_eq!(store.delete_delta_row_count, 1);
        assert_eq!(store.dropped_unselected_delta_route_count, 0);

        let zero_nprobe_plan = SpireSingleLevelScanPlan {
            nprobe: 0,
            ..scan_plan
        };
        let zero_nprobe_diagnostics = collect_single_level_scan_plan_placement_diagnostics(
            &snapshot,
            &object_store,
            &query,
            zero_nprobe_plan,
        )
        .unwrap();
        assert_eq!(zero_nprobe_diagnostics.scan_plan.nprobe, 0);
        assert!(zero_nprobe_diagnostics.stores.is_empty());

        let stale_leaf_count_plan = SpireSingleLevelScanPlan {
            leaf_count: 3,
            ..scan_plan
        };
        let error = collect_single_level_scan_plan_placement_diagnostics(
            &snapshot,
            &object_store,
            &query,
            stale_leaf_count_plan,
        )
        .unwrap_err();
        assert!(error.contains("does not match snapshot leaf_count 2"));
    }

    #[test]
    fn collect_scan_placement_diagnostics_skips_degraded_unavailable_leaf() {
        let mut pid_allocator = SpirePidAllocator::default();
        let mut local_vec_id_allocator = SpireLocalVecIdAllocator::default();
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let draft = build_partitioned_single_level_leaf_epoch_draft(
            partitioned_build_input(
                vec![
                    quantized_assignment_input(
                        10,
                        1,
                        SpireAssignmentPayloadFormat::TurboQuant,
                        &[1.0, 0.0],
                    ),
                    quantized_assignment_input(
                        10,
                        2,
                        SpireAssignmentPayloadFormat::TurboQuant,
                        &[-1.0, 0.0],
                    ),
                ],
                vec![0, 1],
            ),
            &mut pid_allocator,
            &mut local_vec_id_allocator,
            &mut object_store,
        )
        .unwrap();
        let epoch_manifest = SpireEpochManifest {
            epoch: draft.epoch_manifest.epoch,
            state: SpireEpochState::Published,
            consistency_mode: SpireConsistencyMode::Degraded,
            published_at_micros: draft.epoch_manifest.published_at_micros,
            retain_until_micros: draft.epoch_manifest.retain_until_micros,
            active_query_count: 0,
        };
        let mut placements = draft.placement_directory.entries.clone();
        placements
            .iter_mut()
            .find(|placement| placement.pid == SPIRE_FIRST_PID + 1)
            .unwrap()
            .state = SpirePlacementState::Unavailable;
        let placement_directory =
            SpirePlacementDirectory::from_entries(draft.epoch_manifest.epoch, placements).unwrap();
        let snapshot = SpirePublishedEpochSnapshot::new(
            &epoch_manifest,
            &draft.object_manifest,
            &placement_directory,
        )
        .unwrap();
        let query = SpireScanQuery::new(vec![1.0, 0.0]).unwrap();
        let scan_plan = SpireSingleLevelScanPlan {
            leaf_count: 2,
            nprobe: 2,
            nprobe_source: "relation",
            recursive_nprobe_policy: SpireRecursiveNprobePolicy::conservative(2).unwrap(),
            recursive_route_budget: SpireRecursiveRouteBudget::unbounded(),
            payload_format: SpireAssignmentPayloadFormat::TurboQuant,
            rerank_width: 10,
            rerank_width_source: "relation",
            candidate_limit: Some(10),
            dedupe_mode: SpireCandidateDedupeMode::NoReplicaDedupeDisabled,
        };

        let diagnostics = collect_single_level_scan_plan_placement_diagnostics(
            &snapshot,
            &object_store,
            &query,
            scan_plan,
        )
        .unwrap();

        assert_eq!(diagnostics.stores.len(), 1);
        let store = &diagnostics.stores[0];
        assert_eq!(store.epoch, 7);
        assert_eq!(store.scanned_pid_count, 1);
        assert_eq!(store.leaf_pid_count, 1);
        assert_eq!(store.delta_pid_count, 0);
        assert_eq!(store.candidate_row_count, 1);
        assert_eq!(store.leaf_candidate_row_count, 1);
        assert_eq!(store.delta_candidate_row_count, 0);
        assert_eq!(store.delete_delta_row_count, 0);
    }

    #[test]
    fn collect_scan_routing_diagnostics_reports_recursive_levels_and_truncation() {
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let root = SpireRoutingPartitionObject::root_at_level(
            SPIRE_FIRST_PID,
            1,
            2,
            2,
            vec![
                routing_child(0, SPIRE_FIRST_PID + 10, vec![1.0, 0.0]),
                routing_child(1, SPIRE_FIRST_PID + 20, vec![0.9, 0.0]),
                routing_child(2, SPIRE_FIRST_PID + 30, vec![0.8, 0.0]),
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
                routing_child(0, SPIRE_FIRST_PID + 11, vec![1.5, 0.0]),
                routing_child(1, SPIRE_FIRST_PID + 12, vec![0.5, 0.0]),
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
                routing_child(0, SPIRE_FIRST_PID + 21, vec![1.4, 0.0]),
                routing_child(1, SPIRE_FIRST_PID + 22, vec![0.4, 0.0]),
            ],
        )
        .unwrap();
        let internal_c = SpireRoutingPartitionObject::internal(
            SPIRE_FIRST_PID + 30,
            1,
            1,
            SPIRE_FIRST_PID,
            2,
            vec![
                routing_child(0, SPIRE_FIRST_PID + 31, vec![1.3, 0.0]),
                routing_child(1, SPIRE_FIRST_PID + 32, vec![0.3, 0.0]),
            ],
        )
        .unwrap();
        let placements = vec![
            object_store.insert_routing_object(7, &root).unwrap(),
            object_store.insert_routing_object(7, &internal_a).unwrap(),
            object_store.insert_routing_object(7, &internal_b).unwrap(),
            object_store.insert_routing_object(7, &internal_c).unwrap(),
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
        let query = SpireScanQuery::new(vec![1.0, 0.0]).unwrap();
        let options = EcSpireOptions {
            nprobe: 2,
            nprobe_per_level: Some("3".to_owned()),
            ..EcSpireOptions::DEFAULT
        };

        let diagnostics =
            collect_scan_routing_diagnostics(&snapshot, &object_store, &query, options).unwrap();

        assert_eq!(diagnostics.scan_plan.leaf_count, 6);
        assert_eq!(diagnostics.scan_plan.nprobe, 2);
        assert_eq!(diagnostics.scan_plan.recursive_route_budget.beam_width, 2);
        assert_eq!(
            diagnostics
                .levels
                .iter()
                .map(|level| {
                    (
                        level.level,
                        level.input_frontier_width,
                        level.expanded_parent_count,
                        level.selected_child_count,
                        level.deduped_route_count,
                        level.truncation_reason,
                    )
                })
                .collect::<Vec<_>>(),
            vec![
                (2, 1, 1, 3, 2, "beam_width"),
                (1, 2, 2, 4, 2, "max_leaf_routes"),
            ]
        );
    }

    #[test]
    fn count_snapshot_single_level_leaf_pids_uses_root_routing_children() {
        let mut pid_allocator = SpirePidAllocator::default();
        let mut local_vec_id_allocator = SpireLocalVecIdAllocator::default();
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let draft = build_partitioned_single_level_leaf_epoch_draft(
            SpirePartitionedSingleLevelBuildInput {
                epoch: 7,
                object_version: 1,
                published_at_micros: 1000,
                retain_until_micros: 2000,
                consistency_mode: SpireConsistencyMode::Strict,
                root_placement_tid: tid(60, 3),
                placement_tids: vec![tid(60, 1), tid(60, 2), tid(60, 4)],
                assignments: vec![assignment_input(10, 1), assignment_input(10, 2)],
                centroid_plan: SpireSingleLevelCentroidPlan {
                    dimensions: 2,
                    centroids: vec![vec![1.0, 0.0], vec![0.0, 1.0], vec![-1.0, 0.0]],
                    assignment_indexes: vec![0, 2],
                },
            },
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

        assert_eq!(
            count_snapshot_single_level_leaf_pids(&snapshot, &object_store).unwrap(),
            3
        );
    }

    #[test]
    fn count_snapshot_recursive_leaf_pids_counts_leaf_level_children() {
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let root_pid = SPIRE_FIRST_PID;
        let first_internal_pid = SPIRE_FIRST_PID + 1;
        let second_internal_pid = SPIRE_FIRST_PID + 2;
        let root = SpireRoutingPartitionObject::root_at_level(
            root_pid,
            1,
            2,
            2,
            vec![
                routing_child(0, first_internal_pid, vec![1.0, 0.0]),
                routing_child(1, second_internal_pid, vec![-1.0, 0.0]),
            ],
        )
        .unwrap();
        let first_internal = SpireRoutingPartitionObject::internal(
            first_internal_pid,
            1,
            1,
            root_pid,
            2,
            vec![
                routing_child(0, SPIRE_FIRST_PID + 10, vec![0.25, 0.0]),
                routing_child(1, SPIRE_FIRST_PID + 11, vec![0.75, 0.0]),
            ],
        )
        .unwrap();
        let second_internal = SpireRoutingPartitionObject::internal(
            second_internal_pid,
            1,
            1,
            root_pid,
            2,
            vec![routing_child(0, SPIRE_FIRST_PID + 20, vec![-1.0, 0.0])],
        )
        .unwrap();
        let placements = vec![
            object_store.insert_routing_object(7, &root).unwrap(),
            object_store
                .insert_routing_object(7, &first_internal)
                .unwrap(),
            object_store
                .insert_routing_object(7, &second_internal)
                .unwrap(),
            object_store
                .insert_leaf_object_v2_from_rows(
                    7,
                    SPIRE_FIRST_PID + 10,
                    1,
                    first_internal_pid,
                    &[],
                )
                .unwrap(),
            object_store
                .insert_leaf_object_v2_from_rows(
                    7,
                    SPIRE_FIRST_PID + 11,
                    1,
                    first_internal_pid,
                    &[],
                )
                .unwrap(),
            object_store
                .insert_leaf_object_v2_from_rows(
                    7,
                    SPIRE_FIRST_PID + 20,
                    1,
                    second_internal_pid,
                    &[],
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

        assert_eq!(
            count_snapshot_single_level_leaf_pids(&snapshot, &object_store).unwrap(),
            2
        );
        assert_eq!(
            count_snapshot_recursive_leaf_pids(&snapshot, &object_store).unwrap(),
            3
        );
    }
