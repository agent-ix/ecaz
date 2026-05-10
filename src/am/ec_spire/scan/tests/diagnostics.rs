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
        assert_eq!(store.route_count, 2);
        assert_eq!(store.leaf_route_count, 1);
        assert_eq!(store.delta_route_count, 1);
        assert_eq!(store.prefetched_object_count, 2);
        assert_eq!(store.scanned_pid_count, 2);
        assert_eq!(store.leaf_pid_count, 1);
        assert_eq!(store.delta_pid_count, 1);
        assert_eq!(store.candidate_row_count, 1);
        assert_eq!(store.leaf_candidate_row_count, 0);
        assert_eq!(store.delta_candidate_row_count, 1);
        assert_eq!(store.primary_candidate_row_count, 1);
        assert_eq!(store.boundary_replica_candidate_row_count, 0);
        assert_eq!(store.deduped_candidate_row_count, 0);
        assert_eq!(store.truncated_candidate_row_count, 0);
        assert_eq!(store.candidate_winner_count, 1);
        assert_eq!(store.primary_candidate_winner_count, 1);
        assert_eq!(store.boundary_replica_candidate_winner_count, 0);
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
        assert_eq!(store.primary_candidate_row_count, 1);
        assert_eq!(store.boundary_replica_candidate_row_count, 0);
        assert_eq!(store.deduped_candidate_row_count, 0);
        assert_eq!(store.truncated_candidate_row_count, 0);
        assert_eq!(store.candidate_winner_count, 1);
        assert_eq!(store.primary_candidate_winner_count, 1);
        assert_eq!(store.boundary_replica_candidate_winner_count, 0);
        assert_eq!(store.delete_delta_row_count, 0);
    }

    #[test]
    fn collect_scan_placement_diagnostics_reports_boundary_dedupe_and_winners() {
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let root = SpireRoutingPartitionObject::root(
            SPIRE_FIRST_PID,
            1,
            2,
            vec![
                routing_child(0, SPIRE_FIRST_PID + 1, vec![1.0, 0.0]),
                routing_child(1, SPIRE_FIRST_PID + 2, vec![0.9, 0.0]),
            ],
        )
        .unwrap();
        let encoded = encode_assignment_input(
            SpireAssignmentPayloadFormat::TurboQuant,
            tid(10, 1),
            &[1.0, 0.0],
        )
        .unwrap();
        let primary_row = SpireLeafAssignmentRow {
            flags: SPIRE_ASSIGNMENT_FLAG_PRIMARY,
            vec_id: SpireVecId::local(7),
            heap_tid: encoded.heap_tid,
            payload_format: encoded.payload_format,
            gamma: encoded.gamma,
            encoded_payload: encoded.encoded_payload.clone(),
        };
        let replica_row = SpireLeafAssignmentRow {
            flags: SPIRE_ASSIGNMENT_FLAG_BOUNDARY_REPLICA,
            vec_id: SpireVecId::local(7),
            heap_tid: encoded.heap_tid,
            payload_format: encoded.payload_format,
            gamma: encoded.gamma,
            encoded_payload: encoded.encoded_payload,
        };
        let placements = vec![
            object_store.insert_routing_object(7, &root).unwrap(),
            object_store
                .insert_leaf_object_v2_from_rows(7, SPIRE_FIRST_PID + 1, 1, SPIRE_FIRST_PID, &[primary_row])
                .unwrap(),
            object_store
                .insert_leaf_object_v2_from_rows(7, SPIRE_FIRST_PID + 2, 1, SPIRE_FIRST_PID, &[replica_row])
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
            dedupe_mode: SpireCandidateDedupeMode::VecIdDedupeEnabled,
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
        assert_eq!(store.candidate_row_count, 2);
        assert_eq!(store.primary_candidate_row_count, 1);
        assert_eq!(store.boundary_replica_candidate_row_count, 1);
        assert_eq!(store.deduped_candidate_row_count, 1);
        assert_eq!(store.deduped_primary_candidate_row_count, 0);
        assert_eq!(store.deduped_boundary_replica_candidate_row_count, 1);
        assert_eq!(store.truncated_candidate_row_count, 0);
        assert_eq!(store.candidate_winner_count, 1);
        assert_eq!(store.primary_candidate_winner_count, 1);
        assert_eq!(store.boundary_replica_candidate_winner_count, 0);
    }

    #[test]
    fn collect_scan_placement_diagnostics_reports_candidate_truncation() {
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let root = SpireRoutingPartitionObject::root(
            SPIRE_FIRST_PID,
            1,
            2,
            vec![
                routing_child(0, SPIRE_FIRST_PID + 1, vec![1.0, 0.0]),
                routing_child(1, SPIRE_FIRST_PID + 2, vec![0.9, 0.0]),
            ],
        )
        .unwrap();
        let first = encode_assignment_input(
            SpireAssignmentPayloadFormat::TurboQuant,
            tid(10, 1),
            &[1.0, 0.0],
        )
        .unwrap();
        let second = encode_assignment_input(
            SpireAssignmentPayloadFormat::TurboQuant,
            tid(10, 2),
            &[0.9, 0.0],
        )
        .unwrap();
        let first_row = SpireLeafAssignmentRow {
            flags: SPIRE_ASSIGNMENT_FLAG_PRIMARY,
            vec_id: SpireVecId::local(1),
            heap_tid: first.heap_tid,
            payload_format: first.payload_format,
            gamma: first.gamma,
            encoded_payload: first.encoded_payload,
        };
        let second_row = SpireLeafAssignmentRow {
            flags: SPIRE_ASSIGNMENT_FLAG_PRIMARY,
            vec_id: SpireVecId::local(2),
            heap_tid: second.heap_tid,
            payload_format: second.payload_format,
            gamma: second.gamma,
            encoded_payload: second.encoded_payload,
        };
        let placements = vec![
            object_store.insert_routing_object(7, &root).unwrap(),
            object_store
                .insert_leaf_object_v2_from_rows(
                    7,
                    SPIRE_FIRST_PID + 1,
                    1,
                    SPIRE_FIRST_PID,
                    &[first_row],
                )
                .unwrap(),
            object_store
                .insert_leaf_object_v2_from_rows(
                    7,
                    SPIRE_FIRST_PID + 2,
                    1,
                    SPIRE_FIRST_PID,
                    &[second_row],
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
            candidate_limit: Some(1),
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
        assert_eq!(store.candidate_row_count, 2);
        assert_eq!(store.deduped_candidate_row_count, 0);
        assert_eq!(store.truncated_candidate_row_count, 1);
        assert_eq!(store.truncated_primary_candidate_row_count, 1);
        assert_eq!(store.truncated_boundary_replica_candidate_row_count, 0);
        assert_eq!(store.candidate_winner_count, 1);
        assert_eq!(
            store.candidate_row_count,
            store.deduped_candidate_row_count
                + store.truncated_candidate_row_count
                + store.candidate_winner_count
        );
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
            nprobe_per_level: Some(vec![3]),
            ..EcSpireOptions::DEFAULT
        };
        let routing_objects_by_pid = HashMap::from([
            (internal_a.header.pid, internal_a),
            (internal_b.header.pid, internal_b),
            (internal_c.header.pid, internal_c),
        ]);

        let diagnostics =
            collect_scan_routing_diagnostics(&snapshot, &object_store, &query, options).unwrap();
        let production_leaf_routes = route_recursive_routing_objects_to_leaf_routes_with_budget(
            &root,
            &routing_objects_by_pid,
            query.values(),
            &diagnostics.scan_plan.recursive_nprobe_policy,
            diagnostics.scan_plan.recursive_route_budget,
        )
        .unwrap();

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

        let root_level = diagnostics
            .levels
            .first()
            .expect("recursive diagnostics should include the root level");
        let leaf_level = diagnostics
            .levels
            .last()
            .expect("recursive diagnostics should include the leaf level");
        let production_parent_pids = production_leaf_routes
            .iter()
            .map(|route| route.parent_pid)
            .collect::<HashSet<_>>();
        let production_leaf_pids = production_leaf_routes
            .iter()
            .map(|route| route.leaf_pid)
            .collect::<Vec<_>>();
        let production_leaf_candidate_count = production_parent_pids
            .iter()
            .map(|parent_pid| {
                let parent = routing_objects_by_pid
                    .get(parent_pid)
                    .expect("production route parent should exist in fixture");
                route_routing_object_to_child_pids(
                    parent,
                    query.values(),
                    diagnostics
                        .scan_plan
                        .recursive_nprobe_policy
                        .nprobe_for_parent_level(parent.header.level),
                )
                .expect("fixture parent should route to leaf children")
                .len()
            })
            .sum::<usize>();

        assert_eq!(root_level.deduped_route_count, production_parent_pids.len());
        assert_eq!(
            root_level.selected_child_count,
            route_routing_object_to_child_pids(
                &root,
                query.values(),
                diagnostics
                    .scan_plan
                    .recursive_nprobe_policy
                    .nprobe_for_parent_level(root.header.level),
            )
            .unwrap()
            .len()
        );
        assert_eq!(leaf_level.selected_child_count, production_leaf_candidate_count);
        assert_eq!(leaf_level.deduped_route_count, production_leaf_routes.len());
        assert_eq!(
            production_leaf_pids,
            vec![SPIRE_FIRST_PID + 11, SPIRE_FIRST_PID + 21]
        );
    }

    #[test]
    fn collect_scan_routing_diagnostics_matches_production_on_three_level_hierarchy() {
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
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
            vec![
                routing_child(0, SPIRE_FIRST_PID + 121, vec![3.0, 0.0]),
                routing_child(1, SPIRE_FIRST_PID + 122, vec![2.5, 0.0]),
            ],
        )
        .unwrap();
        let level_1_c = SpireRoutingPartitionObject::internal(
            SPIRE_FIRST_PID + 210,
            1,
            1,
            SPIRE_FIRST_PID + 200,
            2,
            vec![
                routing_child(0, SPIRE_FIRST_PID + 211, vec![-2.0, 0.0]),
                routing_child(1, SPIRE_FIRST_PID + 212, vec![-3.0, 0.0]),
            ],
        )
        .unwrap();
        let placements = vec![
            object_store.insert_routing_object(7, &root).unwrap(),
            object_store.insert_routing_object(7, &level_2_a).unwrap(),
            object_store.insert_routing_object(7, &level_2_b).unwrap(),
            object_store.insert_routing_object(7, &level_1_a).unwrap(),
            object_store.insert_routing_object(7, &level_1_b).unwrap(),
            object_store.insert_routing_object(7, &level_1_c).unwrap(),
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
            nprobe: 6,
            nprobe_per_level: Some(vec![2, 2]),
            ..EcSpireOptions::DEFAULT
        };
        let routing_objects_by_pid = HashMap::from([
            (level_2_a.header.pid, level_2_a),
            (level_2_b.header.pid, level_2_b),
            (level_1_a.header.pid, level_1_a),
            (level_1_b.header.pid, level_1_b),
            (level_1_c.header.pid, level_1_c),
        ]);

        let diagnostics =
            collect_scan_routing_diagnostics(&snapshot, &object_store, &query, options).unwrap();
        let production_leaf_routes = route_recursive_routing_objects_to_leaf_routes_with_budget(
            &root,
            &routing_objects_by_pid,
            query.values(),
            &diagnostics.scan_plan.recursive_nprobe_policy,
            diagnostics.scan_plan.recursive_route_budget,
        )
        .unwrap();

        assert_eq!(
            diagnostics
                .levels
                .iter()
                .map(|level| level.level)
                .collect::<Vec<_>>(),
            vec![3, 2, 1]
        );
        assert_eq!(
            diagnostics
                .levels
                .iter()
                .map(|level| level.truncation_reason)
                .collect::<Vec<_>>(),
            vec!["none", "none", "none"]
        );

        let root_level = &diagnostics.levels[0];
        let middle_level = &diagnostics.levels[1];
        let leaf_level = &diagnostics.levels[2];
        let production_level_1_parent_pids = production_leaf_routes
            .iter()
            .map(|route| route.parent_pid)
            .collect::<HashSet<_>>();
        let production_level_2_parent_pids = production_level_1_parent_pids
            .iter()
            .map(|parent_pid| {
                routing_objects_by_pid
                    .get(parent_pid)
                    .expect("production leaf parent should exist in fixture")
                    .header
                    .parent_pid
            })
            .collect::<HashSet<_>>();
        let middle_selected_child_count = production_level_2_parent_pids
            .iter()
            .map(|parent_pid| {
                let parent = routing_objects_by_pid
                    .get(parent_pid)
                    .expect("production level-2 parent should exist in fixture");
                route_routing_object_to_child_pids(
                    parent,
                    query.values(),
                    diagnostics
                        .scan_plan
                        .recursive_nprobe_policy
                        .nprobe_for_parent_level(parent.header.level),
                )
                .expect("fixture level-2 parent should route to level-1 children")
                .len()
            })
            .sum::<usize>();
        let leaf_selected_child_count = production_level_1_parent_pids
            .iter()
            .map(|parent_pid| {
                let parent = routing_objects_by_pid
                    .get(parent_pid)
                    .expect("production level-1 parent should exist in fixture");
                route_routing_object_to_child_pids(
                    parent,
                    query.values(),
                    diagnostics
                        .scan_plan
                        .recursive_nprobe_policy
                        .nprobe_for_parent_level(parent.header.level),
                )
                .expect("fixture level-1 parent should route to leaf children")
                .len()
            })
            .sum::<usize>();

        assert_eq!(
            root_level.deduped_route_count,
            production_level_2_parent_pids.len()
        );
        assert_eq!(
            middle_level.deduped_route_count,
            production_level_1_parent_pids.len()
        );
        assert_eq!(leaf_level.deduped_route_count, production_leaf_routes.len());
        assert_eq!(middle_level.selected_child_count, middle_selected_child_count);
        assert_eq!(leaf_level.selected_child_count, leaf_selected_child_count);
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
