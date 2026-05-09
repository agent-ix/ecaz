    #[test]
    fn collect_ranked_routed_probe_candidates_scores_and_limits() {
        let mut pid_allocator = SpirePidAllocator::default();
        let mut local_vec_id_allocator = SpireLocalVecIdAllocator::default();
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let draft = build_partitioned_single_level_leaf_epoch_draft(
            partitioned_build_input(
                vec![
                    assignment_input_with_payload(10, 1, vec![1]),
                    assignment_input_with_payload(10, 2, vec![9]),
                ],
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

        let candidates = collect_ranked_routed_probe_candidates(
            &snapshot,
            &object_store,
            &[1.0, 0.0],
            2,
            |row| Ok(f32::from(row.encoded_payload[0])),
            SpireCandidateDedupeMode::NoReplicaDedupeDisabled,
            Some(1),
        )
        .unwrap();

        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].pid, SPIRE_FIRST_PID + 2);
        assert_eq!(candidates[0].object_version, 1);
        assert_eq!(candidates[0].row_index, 0);
        assert_eq!(candidates[0].heap_tid, tid(10, 2));
        assert_eq!(candidates[0].vec_id.local_sequence(), Some(2));
        assert_eq!(candidates[0].score, -9.0);
    }

    #[test]
    fn collect_quantized_routed_probe_candidates_matches_prepared_assignment_scorer() {
        for payload_format in [
            SpireAssignmentPayloadFormat::TurboQuant,
            SpireAssignmentPayloadFormat::RaBitQ,
        ] {
            let mut pid_allocator = SpirePidAllocator::default();
            let mut local_vec_id_allocator = SpireLocalVecIdAllocator::default();
            let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
            let query = [1.0, 0.0];
            let draft = build_partitioned_single_level_leaf_epoch_draft(
                partitioned_build_input(
                    vec![
                        quantized_assignment_input(10, 1, payload_format, &[1.0, 0.0]),
                        quantized_assignment_input(10, 2, payload_format, &[-1.0, 0.0]),
                    ],
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
            let scorer =
                SpirePreparedAssignmentScorer::prepare(payload_format, query.len(), &query)
                    .unwrap();
            let expected = collect_ranked_routed_probe_candidates(
                &snapshot,
                &object_store,
                &query,
                2,
                |row| scorer.score_assignment_ip(row),
                SpireCandidateDedupeMode::NoReplicaDedupeDisabled,
                Some(2),
            )
            .unwrap();

            let observed = collect_quantized_routed_probe_candidates(
                &snapshot,
                &object_store,
                &query,
                2,
                payload_format,
                SpireCandidateDedupeMode::NoReplicaDedupeDisabled,
                Some(2),
            )
            .unwrap();

            assert_eq!(observed, expected);
            assert_eq!(observed.len(), 2);
        }
    }

    #[test]
    fn collect_quantized_routed_probe_candidates_reads_hash_routed_two_store_build() {
        let payload_format = SpireAssignmentPayloadFormat::TurboQuant;
        let store_config = SpireLocalStoreConfig::from_stores(
            1,
            vec![
                SpireLocalStoreDescriptor::available(0, 12345, 10).unwrap(),
                SpireLocalStoreDescriptor::available(1, 12346, 10).unwrap(),
            ],
        )
        .unwrap();
        let mut object_store =
            SpireLocalObjectStoreSet::from_config(store_config.clone(), 8192).unwrap();
        let mut pid_allocator = SpirePidAllocator::default();
        let mut local_vec_id_allocator = SpireLocalVecIdAllocator::default();
        let centroid_count = 8;
        let assignments = (0..centroid_count)
            .map(|index| {
                quantized_assignment_input(
                    10,
                    u16::try_from(index + 1).unwrap(),
                    payload_format,
                    &[index as f32, 0.0],
                )
            })
            .collect::<Vec<_>>();
        let centroid_plan = SpireSingleLevelCentroidPlan {
            dimensions: 2,
            centroids: (0..centroid_count)
                .map(|index| vec![index as f32, 0.0])
                .collect(),
            assignment_indexes: (0..centroid_count)
                .map(|index| u32::try_from(index).unwrap())
                .collect(),
        };
        let draft = build_partitioned_single_level_leaf_epoch_draft(
            SpirePartitionedSingleLevelBuildInput {
                epoch: 7,
                object_version: 1,
                published_at_micros: 1000,
                retain_until_micros: 2000,
                consistency_mode: SpireConsistencyMode::Strict,
                root_placement_tid: tid(60, 100),
                placement_tids: (0..centroid_count)
                    .map(|index| tid(60, u16::try_from(index + 1).unwrap()))
                    .collect(),
                assignments,
                centroid_plan,
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

        let observed = collect_quantized_routed_probe_candidates(
            &snapshot,
            &object_store,
            &[1.0, 0.0],
            u32::try_from(centroid_count).unwrap(),
            payload_format,
            SpireCandidateDedupeMode::NoReplicaDedupeDisabled,
            Some(centroid_count),
        )
        .unwrap();

        let root_placement = draft.placement_directory.get(draft.root_pid).unwrap();
        assert_eq!(
            root_placement.local_store_id,
            store_config
                .store_for_pid(draft.root_pid)
                .unwrap()
                .local_store_id
        );
        assert_eq!(root_placement.local_store_id, 1);
        assert_eq!(observed.len(), centroid_count);

        let candidate_store_ids = observed
            .iter()
            .map(|candidate| {
                draft
                    .placement_directory
                    .get(candidate.pid)
                    .unwrap()
                    .local_store_id
            })
            .collect::<HashSet<_>>();
        assert_eq!(candidate_store_ids, HashSet::from([0, 1]));
    }

    #[test]
    fn collect_quantized_routed_probe_candidates_accepts_recursive_leaf_parent() {
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let root_pid = SPIRE_FIRST_PID;
        let internal_pid = SPIRE_FIRST_PID + 1;
        let first_leaf_pid = SPIRE_FIRST_PID + 2;
        let second_leaf_pid = SPIRE_FIRST_PID + 3;
        let payload_format = SpireAssignmentPayloadFormat::TurboQuant;
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
        let first_input = quantized_assignment_input(10, 1, payload_format, &[-1.0, 0.0]);
        let second_input = quantized_assignment_input(10, 2, payload_format, &[1.0, 0.0]);
        let first_leaf_rows = vec![SpireLeafAssignmentRow {
            flags: SPIRE_ASSIGNMENT_FLAG_PRIMARY,
            vec_id: SpireVecId::local(1),
            heap_tid: first_input.heap_tid,
            payload_format: first_input.payload_format,
            gamma: first_input.gamma,
            encoded_payload: first_input.encoded_payload,
        }];
        let second_leaf_rows = vec![SpireLeafAssignmentRow {
            flags: SPIRE_ASSIGNMENT_FLAG_PRIMARY,
            vec_id: SpireVecId::local(2),
            heap_tid: second_input.heap_tid,
            payload_format: second_input.payload_format,
            gamma: second_input.gamma,
            encoded_payload: second_input.encoded_payload,
        }];
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

        let observed = collect_quantized_routed_probe_candidates(
            &snapshot,
            &object_store,
            &[1.0, 0.0],
            1,
            payload_format,
            SpireCandidateDedupeMode::NoReplicaDedupeDisabled,
            Some(1),
        )
        .unwrap();

        assert_eq!(observed.len(), 1);
        assert_eq!(observed[0].pid, second_leaf_pid);
        assert_eq!(observed[0].heap_tid, tid(10, 2));
        assert_eq!(observed[0].vec_id.local_sequence(), Some(2));
    }

    #[test]
    fn group_leaf_and_delta_reads_by_local_store_orders_stores_and_preserves_leaf_route_order() {
        let epoch_manifest = SpireEpochManifest {
            epoch: 7,
            state: SpireEpochState::Published,
            consistency_mode: SpireConsistencyMode::Strict,
            published_at_micros: 1000,
            retain_until_micros: 2000,
            active_query_count: 0,
        };
        let placements = vec![
            SpirePlacementEntry::local_store_available_by_id(
                7,
                SPIRE_FIRST_PID + 3,
                1,
                501,
                1,
                tid(60, 3),
                100,
            ),
            SpirePlacementEntry::local_store_available_by_id(
                7,
                SPIRE_FIRST_PID + 1,
                0,
                500,
                1,
                tid(60, 1),
                100,
            ),
            SpirePlacementEntry::local_store_available_by_id(
                7,
                SPIRE_FIRST_PID + 2,
                1,
                501,
                1,
                tid(60, 2),
                100,
            ),
        ];
        let object_manifest = SpireObjectManifest::from_entries(
            7,
            placements.iter().map(manifest_entry_for).collect(),
        )
        .unwrap();
        let placement_directory = SpirePlacementDirectory::from_entries(7, placements).unwrap();
        let snapshot =
            snapshot_for_placement(&epoch_manifest, &object_manifest, &placement_directory);
        let snapshot = SpireValidatedEpochSnapshot::from_snapshot(snapshot).unwrap();

        let mut observer = SpireNoopRoutedScanObserver;
        let groups = group_leaf_and_delta_reads_by_local_store(
            &snapshot,
            vec![
                SpireRecursiveLeafRoute {
                    leaf_pid: SPIRE_FIRST_PID + 3,
                    parent_pid: SPIRE_FIRST_PID,
                },
                SpireRecursiveLeafRoute {
                    leaf_pid: SPIRE_FIRST_PID + 1,
                    parent_pid: SPIRE_FIRST_PID,
                },
                SpireRecursiveLeafRoute {
                    leaf_pid: SPIRE_FIRST_PID + 2,
                    parent_pid: SPIRE_FIRST_PID,
                },
            ],
            Vec::new(),
            &mut observer,
        )
        .unwrap();

        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].node_id, 0);
        assert_eq!(groups[0].local_store_id, 0);
        assert_eq!(
            groups[0]
                .leaf_routes
                .iter()
                .map(|route| route.leaf_pid)
                .collect::<Vec<_>>(),
            vec![SPIRE_FIRST_PID + 1]
        );
        assert_eq!(groups[1].node_id, 0);
        assert_eq!(groups[1].local_store_id, 1);
        assert_eq!(
            groups[1]
                .leaf_routes
                .iter()
                .map(|route| route.leaf_pid)
                .collect::<Vec<_>>(),
            vec![SPIRE_FIRST_PID + 3, SPIRE_FIRST_PID + 2]
        );
    }

    #[test]
    fn group_leaf_and_delta_reads_by_local_store_groups_deltas_by_own_store() {
        let epoch_manifest = SpireEpochManifest {
            epoch: 7,
            state: SpireEpochState::Published,
            consistency_mode: SpireConsistencyMode::Strict,
            published_at_micros: 1000,
            retain_until_micros: 2000,
            active_query_count: 0,
        };
        let selected_leaf_pid = SPIRE_FIRST_PID + 1;
        let other_leaf_pid = SPIRE_FIRST_PID + 2;
        let selected_delta_pid = SPIRE_FIRST_PID + 11;
        let other_delta_pid = SPIRE_FIRST_PID + 12;
        let placements = vec![
            SpirePlacementEntry::local_store_available_by_id(
                7,
                selected_leaf_pid,
                0,
                500,
                1,
                tid(60, 1),
                100,
            ),
            SpirePlacementEntry::local_store_available_by_id(
                7,
                other_leaf_pid,
                1,
                501,
                1,
                tid(60, 2),
                100,
            ),
            SpirePlacementEntry::local_store_available_by_id(
                7,
                selected_delta_pid,
                2,
                502,
                1,
                tid(60, 3),
                100,
            ),
            SpirePlacementEntry::local_store_available_by_id(
                7,
                other_delta_pid,
                1,
                501,
                1,
                tid(60, 4),
                100,
            ),
        ];
        let object_manifest = SpireObjectManifest::from_entries(
            7,
            placements.iter().map(manifest_entry_for).collect(),
        )
        .unwrap();
        let placement_directory = SpirePlacementDirectory::from_entries(7, placements).unwrap();
        let snapshot =
            snapshot_for_placement(&epoch_manifest, &object_manifest, &placement_directory);
        let snapshot = SpireValidatedEpochSnapshot::from_snapshot(snapshot).unwrap();

        let mut observer = SpireScanPlacementDiagnosticsObserver::new();
        let groups = group_leaf_and_delta_reads_by_local_store(
            &snapshot,
            vec![SpireRecursiveLeafRoute {
                leaf_pid: selected_leaf_pid,
                parent_pid: SPIRE_FIRST_PID,
            }],
            vec![
                SpireDeltaObjectRoute {
                    delta_pid: selected_delta_pid,
                    parent_leaf_pid: selected_leaf_pid,
                    placement: SpirePlacementEntry::local_store_available_by_id(
                        7,
                        selected_delta_pid,
                        2,
                        502,
                        1,
                        tid(60, 3),
                        100,
                    ),
                    object_version: 1,
                },
                SpireDeltaObjectRoute {
                    delta_pid: other_delta_pid,
                    parent_leaf_pid: other_leaf_pid,
                    placement: SpirePlacementEntry::local_store_available_by_id(
                        7,
                        other_delta_pid,
                        1,
                        501,
                        1,
                        tid(60, 4),
                        100,
                    ),
                    object_version: 1,
                },
            ],
            &mut observer,
        )
        .unwrap();

        let stores = observer.into_stores();
        assert_eq!(stores.len(), 1);
        assert_eq!(stores[0].local_store_id, 1);
        assert_eq!(stores[0].dropped_unselected_delta_route_count, 1);

        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].local_store_id, 0);
        assert_eq!(groups[0].leaf_routes.len(), 1);
        assert_eq!(groups[0].leaf_routes[0].leaf_pid, selected_leaf_pid);
        assert!(groups[0].delta_routes.is_empty());
        assert_eq!(groups[1].local_store_id, 2);
        assert!(groups[1].leaf_routes.is_empty());
        assert_eq!(groups[1].delta_routes.len(), 1);
        assert_eq!(groups[1].delta_routes[0].delta_pid, selected_delta_pid);
        assert_eq!(groups[1].delta_routes[0].parent_leaf_pid, selected_leaf_pid);
    }

    #[test]
    fn prefetch_store_object_read_groups_prefetches_leaf_and_delta_routes() {
        struct RecordingPrefetchReader {
            prefetched_pids: RefCell<Vec<u64>>,
        }

        impl SpireObjectReader for RecordingPrefetchReader {
            fn prefetch_object(&self, placement: &SpirePlacementEntry) -> Result<(), String> {
                self.prefetched_pids.borrow_mut().push(placement.pid);
                Ok(())
            }

            fn read_object_header(
                &self,
                _placement: &SpirePlacementEntry,
            ) -> Result<SpirePartitionObjectHeader, String> {
                unreachable!("prefetch test should not read object headers")
            }

            fn read_routing_object(
                &self,
                _placement: &SpirePlacementEntry,
            ) -> Result<SpireRoutingPartitionObject, String> {
                unreachable!("prefetch test should not read routing objects")
            }

            fn read_leaf_object(
                &self,
                _placement: &SpirePlacementEntry,
            ) -> Result<SpireLeafPartitionObject, String> {
                unreachable!("prefetch test should not read leaf objects")
            }

            fn read_leaf_object_v2(
                &self,
                _placement: &SpirePlacementEntry,
            ) -> Result<crate::am::ec_spire::storage::SpireLeafPartitionObjectV2, String>
            {
                unreachable!("prefetch test should not read leaf V2 objects")
            }

            fn read_delta_object(
                &self,
                _placement: &SpirePlacementEntry,
            ) -> Result<SpireDeltaPartitionObject, String> {
                unreachable!("prefetch test should not read delta objects")
            }
        }

        let epoch_manifest = SpireEpochManifest {
            epoch: 7,
            state: SpireEpochState::Published,
            consistency_mode: SpireConsistencyMode::Strict,
            published_at_micros: 1000,
            retain_until_micros: 2000,
            active_query_count: 0,
        };
        let leaf_pid = SPIRE_FIRST_PID + 1;
        let delta_pid = SPIRE_FIRST_PID + 11;
        let placements = vec![
            SpirePlacementEntry::local_store_available_by_id(
                7,
                leaf_pid,
                0,
                500,
                1,
                tid(60, 1),
                100,
            ),
            SpirePlacementEntry::local_store_available_by_id(
                7,
                delta_pid,
                0,
                500,
                1,
                tid(61, 1),
                100,
            ),
        ];
        let object_manifest = SpireObjectManifest::from_entries(
            7,
            placements.iter().map(manifest_entry_for).collect(),
        )
        .unwrap();
        let placement_directory = SpirePlacementDirectory::from_entries(7, placements).unwrap();
        let snapshot =
            snapshot_for_placement(&epoch_manifest, &object_manifest, &placement_directory);
        let snapshot = SpireValidatedEpochSnapshot::from_snapshot(snapshot).unwrap();

        let mut observer = SpireNoopRoutedScanObserver;
        let groups = group_leaf_and_delta_reads_by_local_store(
            &snapshot,
            vec![SpireRecursiveLeafRoute {
                leaf_pid,
                parent_pid: SPIRE_FIRST_PID,
            }],
            vec![SpireDeltaObjectRoute {
                delta_pid,
                parent_leaf_pid: leaf_pid,
                placement: SpirePlacementEntry::local_store_available_by_id(
                    7,
                    delta_pid,
                    0,
                    500,
                    1,
                    tid(61, 1),
                    100,
                ),
                object_version: 1,
            }],
            &mut observer,
        )
        .unwrap();
        let reader = RecordingPrefetchReader {
            prefetched_pids: RefCell::new(Vec::new()),
        };

        prefetch_store_object_read_groups(&reader, std::slice::from_ref(&groups[0])).unwrap();

        assert_eq!(*reader.prefetched_pids.borrow(), vec![leaf_pid, delta_pid]);
    }

    #[test]
    fn prefetch_store_object_read_groups_prefetches_every_store_before_scoring() {
        struct RecordingPrefetchReader {
            prefetched_pids: RefCell<Vec<u64>>,
        }

        impl SpireObjectReader for RecordingPrefetchReader {
            fn prefetch_object(&self, placement: &SpirePlacementEntry) -> Result<(), String> {
                self.prefetched_pids.borrow_mut().push(placement.pid);
                Ok(())
            }

            fn read_object_header(
                &self,
                _placement: &SpirePlacementEntry,
            ) -> Result<SpirePartitionObjectHeader, String> {
                unreachable!("prefetch-all test should not read object headers")
            }

            fn read_routing_object(
                &self,
                _placement: &SpirePlacementEntry,
            ) -> Result<SpireRoutingPartitionObject, String> {
                unreachable!("prefetch-all test should not read routing objects")
            }

            fn read_leaf_object(
                &self,
                _placement: &SpirePlacementEntry,
            ) -> Result<SpireLeafPartitionObject, String> {
                unreachable!("prefetch-all test should not read leaf objects")
            }

            fn read_leaf_object_v2(
                &self,
                _placement: &SpirePlacementEntry,
            ) -> Result<crate::am::ec_spire::storage::SpireLeafPartitionObjectV2, String>
            {
                unreachable!("prefetch-all test should not read leaf V2 objects")
            }

            fn read_delta_object(
                &self,
                _placement: &SpirePlacementEntry,
            ) -> Result<SpireDeltaPartitionObject, String> {
                unreachable!("prefetch-all test should not read delta objects")
            }
        }

        let leaf_pid = SPIRE_FIRST_PID + 1;
        let delta_pid = SPIRE_FIRST_PID + 11;
        let groups = vec![
            SpireStoreObjectReadGroup {
                node_id: 0,
                local_store_id: 0,
                leaf_routes: vec![SpireLeafObjectReadRoute {
                    leaf_pid,
                    parent_pid: SPIRE_FIRST_PID,
                    placement: SpirePlacementEntry::local_store_available_by_id(
                        7,
                        leaf_pid,
                        0,
                        500,
                        1,
                        tid(60, 1),
                        100,
                    ),
                    object_version: 1,
                }],
                delta_routes: Vec::new(),
            },
            SpireStoreObjectReadGroup {
                node_id: 0,
                local_store_id: 2,
                leaf_routes: Vec::new(),
                delta_routes: vec![SpireDeltaObjectRoute {
                    delta_pid,
                    parent_leaf_pid: leaf_pid,
                    placement: SpirePlacementEntry::local_store_available_by_id(
                        7,
                        delta_pid,
                        2,
                        502,
                        1,
                        tid(61, 1),
                        100,
                    ),
                    object_version: 1,
                }],
            },
        ];
        let reader = RecordingPrefetchReader {
            prefetched_pids: RefCell::new(Vec::new()),
        };

        prefetch_store_object_read_groups(&reader, &groups).unwrap();

        assert_eq!(*reader.prefetched_pids.borrow(), vec![leaf_pid, delta_pid]);
    }

    #[test]
    fn collect_quantized_routed_probe_candidates_rejects_deferred_and_bad_payloads() {
        let mut pid_allocator = SpirePidAllocator::default();
        let mut local_vec_id_allocator = SpireLocalVecIdAllocator::default();
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let draft = build_partitioned_single_level_leaf_epoch_draft(
            partitioned_build_input(
                vec![
                    assignment_input_with_payload(10, 1, vec![1]),
                    assignment_input_with_payload(10, 2, vec![2]),
                ],
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

        assert!(collect_quantized_routed_probe_candidates(
            &snapshot,
            &object_store,
            &[1.0, 0.0],
            2,
            SpireAssignmentPayloadFormat::PqFastScan,
            SpireCandidateDedupeMode::NoReplicaDedupeDisabled,
            Some(2),
        )
        .unwrap_err()
        .contains("PQ-FastScan"));
        assert!(collect_quantized_routed_probe_candidates(
            &snapshot,
            &object_store,
            &[1.0, 0.0],
            2,
            SpireAssignmentPayloadFormat::TurboQuant,
            SpireCandidateDedupeMode::NoReplicaDedupeDisabled,
            Some(2),
        )
        .unwrap_err()
        .contains("payload stride mismatch"));
    }

    #[test]
    fn collect_reranked_quantized_routed_probe_candidates_rescores_prefix() {
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
        let snapshot = SpirePublishedEpochSnapshot::new(
            &draft.epoch_manifest,
            &draft.object_manifest,
            &draft.placement_directory,
        )
        .unwrap();

        let candidates = collect_reranked_quantized_routed_probe_candidates(
            &snapshot,
            &object_store,
            &[1.0, 0.0],
            2,
            SpireAssignmentPayloadFormat::TurboQuant,
            SpireCandidateDedupeMode::NoReplicaDedupeDisabled,
            Some(2),
            2,
            |candidate| {
                Ok(Some(match candidate.vec_id.local_sequence().unwrap() {
                    1 => 1.0,
                    2 => 10.0,
                    other => panic!("unexpected rerank candidate {other}"),
                }))
            },
        )
        .unwrap();

        assert_eq!(candidates.len(), 2);
        assert_eq!(candidates[0].vec_id.local_sequence(), Some(2));
        assert_eq!(candidates[0].score, -10.0);
        assert_eq!(candidates[1].vec_id.local_sequence(), Some(1));
        assert_eq!(candidates[1].score, -1.0);
    }

    #[test]
    fn collect_single_level_scan_plan_reranked_candidates_uses_plan_knobs() {
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
        let snapshot = SpirePublishedEpochSnapshot::new(
            &draft.epoch_manifest,
            &draft.object_manifest,
            &draft.placement_directory,
        )
        .unwrap();
        let scan_plan = SpireSingleLevelScanPlan {
            leaf_count: 2,
            nprobe: 2,
            nprobe_source: "relation",
            recursive_nprobe_policy: SpireRecursiveNprobePolicy::conservative(2).unwrap(),
            recursive_route_budget: SpireRecursiveRouteBudget::unbounded(),
            payload_format: SpireAssignmentPayloadFormat::TurboQuant,
            rerank_width: 2,
            rerank_width_source: "relation",
            candidate_limit: Some(2),
            dedupe_mode: SpireCandidateDedupeMode::NoReplicaDedupeDisabled,
        };

        let candidates = collect_single_level_scan_plan_reranked_candidates(
            &snapshot,
            &object_store,
            &[1.0, 0.0],
            scan_plan,
            |candidate| {
                Ok(Some(match candidate.vec_id.local_sequence().unwrap() {
                    1 => 1.0,
                    2 => 10.0,
                    other => panic!("unexpected rerank candidate {other}"),
                }))
            },
        )
        .unwrap();

        assert_eq!(candidates.len(), 2);
        assert_eq!(candidates[0].vec_id.local_sequence(), Some(2));
        assert_eq!(candidates[0].score, -10.0);
        assert_eq!(candidates[1].vec_id.local_sequence(), Some(1));
        assert_eq!(candidates[1].score, -1.0);
    }

    #[test]
    fn prepare_single_level_snapshot_scan_candidates_resolves_plan_and_candidates() {
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
        let snapshot = SpirePublishedEpochSnapshot::new(
            &draft.epoch_manifest,
            &draft.object_manifest,
            &draft.placement_directory,
        )
        .unwrap();
        let options = EcSpireOptions {
            nlists: 2,
            recursive_fanout: 0,
            local_store_count: 1,
            boundary_replica_count: 0,
            nprobe: 2,
            rerank_width: 2,
            max_candidate_rows: 0,
            training_sample_rows: 0,
            seed: 0,
            pq_group_size: 0,
            top_graph_enabled: 0,
            top_graph_degree: 32,
            top_graph_build_list_size: 100,
            top_graph_alpha: 1.2,
            top_graph_search_list_size: 0,
            nprobe_per_level: None,
            storage_format: SpireStorageFormat::TurboQuant,
            local_store_tablespaces: None,
        };
        let query = SpireScanQuery::new(vec![1.0, 0.0]).unwrap();

        let leaf_count = count_snapshot_single_level_leaf_pids(&snapshot, &object_store).unwrap();
        let scan_plan =
            resolve_single_level_scan_plan_values(leaf_count, options, -1, -1).unwrap();
        let candidates = collect_single_level_scan_plan_reranked_candidates(
            &snapshot,
            &object_store,
            query.values(),
            scan_plan,
            |candidate| {
                Ok(Some(match candidate.vec_id.local_sequence().unwrap() {
                    1 => 1.0,
                    2 => 10.0,
                    other => panic!("unexpected rerank candidate {other}"),
                }))
            },
        )
        .unwrap();

        assert_eq!(scan_plan.leaf_count, 2);
        assert_eq!(scan_plan.nprobe, 2);
        assert_eq!(scan_plan.nprobe_source, "relation");
        assert_eq!(candidates.len(), 2);
        assert_eq!(candidates[0].vec_id.local_sequence(), Some(2));
        assert_eq!(candidates[0].score, -10.0);
        assert_eq!(candidates[1].vec_id.local_sequence(), Some(1));
        assert_eq!(candidates[1].score, -1.0);
    }

    #[test]
    fn prepare_single_level_snapshot_scan_candidates_uses_top_graph_when_enabled() {
        fn quantized_row(
            vec_seq: u64,
            block_number: u32,
            offset_number: u16,
            source_vector: &[f32],
        ) -> SpireLeafAssignmentRow {
            let input = quantized_assignment_input(
                block_number,
                offset_number,
                SpireAssignmentPayloadFormat::TurboQuant,
                source_vector,
            );
            SpireLeafAssignmentRow {
                flags: SPIRE_ASSIGNMENT_FLAG_PRIMARY,
                vec_id: SpireVecId::local(vec_seq),
                heap_tid: input.heap_tid,
                payload_format: input.payload_format,
                gamma: input.gamma,
                encoded_payload: input.encoded_payload,
            }
        }

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
                    &[quantized_row(1, 10, 1, &[1.0, 0.0])],
                )
                .unwrap(),
            object_store
                .insert_leaf_object_v2_from_rows(
                    7,
                    SPIRE_FIRST_PID + 21,
                    1,
                    SPIRE_FIRST_PID + 20,
                    &[quantized_row(2, 10, 2, &[-1.0, 0.0])],
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
        let options = EcSpireOptions {
            nlists: 2,
            recursive_fanout: 2,
            local_store_count: 1,
            boundary_replica_count: 0,
            nprobe: 1,
            rerank_width: 0,
            max_candidate_rows: 0,
            training_sample_rows: 0,
            seed: 42,
            pq_group_size: 0,
            top_graph_enabled: 1,
            top_graph_degree: 1,
            top_graph_build_list_size: 2,
            top_graph_alpha: 1.2,
            top_graph_search_list_size: 2,
            nprobe_per_level: None,
            storage_format: SpireStorageFormat::TurboQuant,
            local_store_tablespaces: None,
        };
        let query = SpireScanQuery::new(vec![1.0, 0.0]).unwrap();

        let top_graph_plan = options.top_graph_plan().unwrap();
        let leaf_count = count_snapshot_single_level_leaf_pids(&snapshot, &object_store).unwrap();
        let scan_plan =
            resolve_single_level_scan_plan_values(leaf_count, options, -1, -1).unwrap();
        let candidates = collect_top_graph_scan_plan_reranked_candidates(
            &snapshot,
            &object_store,
            query.values(),
            scan_plan,
            top_graph_plan,
            |candidate| {
                Ok(Some(match candidate.vec_id.local_sequence().unwrap() {
                    1 => 10.0,
                    other => panic!("unexpected rerank candidate {other}"),
                }))
            },
        )
        .unwrap();

        assert_eq!(scan_plan.nprobe, 1);
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].vec_id.local_sequence(), Some(1));
        assert_eq!(candidates[0].heap_tid, tid(10, 1));
        assert_eq!(candidates[0].score, -10.0);
    }

    #[test]
    fn collect_single_level_scan_plan_reranked_candidates_allows_empty_plan() {
        let epoch_manifest = SpireEpochManifest {
            epoch: 7,
            state: SpireEpochState::Published,
            consistency_mode: SpireConsistencyMode::Strict,
            published_at_micros: 1000,
            retain_until_micros: 2000,
            active_query_count: 0,
        };
        let object_manifest = SpireObjectManifest::from_entries(7, Vec::new()).unwrap();
        let placement_directory = SpirePlacementDirectory::from_entries(7, Vec::new()).unwrap();
        let snapshot =
            snapshot_for_placement(&epoch_manifest, &object_manifest, &placement_directory);
        let object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let scan_plan = SpireSingleLevelScanPlan {
            leaf_count: 0,
            nprobe: 0,
            nprobe_source: "none",
            recursive_nprobe_policy: SpireRecursiveNprobePolicy::conservative(0).unwrap(),
            recursive_route_budget: SpireRecursiveRouteBudget::unbounded(),
            payload_format: SpireAssignmentPayloadFormat::TurboQuant,
            rerank_width: 0,
            rerank_width_source: "relation",
            candidate_limit: None,
            dedupe_mode: SpireCandidateDedupeMode::NoReplicaDedupeDisabled,
        };

        let candidates = collect_single_level_scan_plan_reranked_candidates(
            &snapshot,
            &object_store,
            &[1.0, 0.0],
            scan_plan,
            |_| panic!("empty scan plan should not call exact scorer"),
        )
        .unwrap();

        assert!(candidates.is_empty());
    }

    #[test]
    fn rank_routed_leaf_rows_by_ip_keeps_best_visible_vec_id_candidate() {
        let same_vec_id_low_score =
            assignment_row_with_payload(SPIRE_ASSIGNMENT_FLAG_PRIMARY, 7, 20, 2, vec![1]);
        let same_vec_id_high_score =
            assignment_row_with_payload(SPIRE_ASSIGNMENT_FLAG_PRIMARY, 7, 10, 1, vec![9]);
        let better_boundary_replica = assignment_row_with_payload(
            SPIRE_ASSIGNMENT_FLAG_BOUNDARY_REPLICA,
            7,
            30,
            3,
            vec![100],
        );
        let routed = vec![SpireRoutedLeafScanRows {
            epoch: 1,
            root_pid: SPIRE_FIRST_PID,
            leaf_pid: SPIRE_FIRST_PID + 1,
            rows: vec![
                SpireLeafScanRow {
                    pid: SPIRE_FIRST_PID + 1,
                    object_version: 1,
                    row_index: 0,
                    assignment: same_vec_id_low_score,
                },
                SpireLeafScanRow {
                    pid: SPIRE_FIRST_PID + 2,
                    object_version: 1,
                    row_index: 0,
                    assignment: same_vec_id_high_score,
                },
                SpireLeafScanRow {
                    pid: SPIRE_FIRST_PID + 3,
                    object_version: 1,
                    row_index: 0,
                    assignment: better_boundary_replica,
                },
            ],
        }];

        let candidates = rank_routed_leaf_rows_by_ip(
            routed,
            |row| Ok(f32::from(row.encoded_payload[0])),
            SpireCandidateDedupeMode::VecIdDedupeEnabled,
            None,
        )
        .unwrap();

        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].vec_id.local_sequence(), Some(7));
        assert_eq!(candidates[0].pid, SPIRE_FIRST_PID + 3);
        assert_eq!(candidates[0].heap_tid, tid(30, 3));
        assert_eq!(candidates[0].score, -100.0);
    }

    #[test]
    fn rank_routed_leaf_rows_by_ip_can_skip_vec_id_dedupe() {
        let routed = vec![SpireRoutedLeafScanRows {
            epoch: 1,
            root_pid: SPIRE_FIRST_PID,
            leaf_pid: SPIRE_FIRST_PID + 1,
            rows: vec![
                SpireLeafScanRow {
                    pid: SPIRE_FIRST_PID + 1,
                    object_version: 1,
                    row_index: 0,
                    assignment: assignment_row_with_payload(
                        SPIRE_ASSIGNMENT_FLAG_PRIMARY,
                        7,
                        20,
                        2,
                        vec![1],
                    ),
                },
                SpireLeafScanRow {
                    pid: SPIRE_FIRST_PID + 2,
                    object_version: 1,
                    row_index: 0,
                    assignment: assignment_row_with_payload(
                        SPIRE_ASSIGNMENT_FLAG_PRIMARY,
                        7,
                        10,
                        1,
                        vec![9],
                    ),
                },
            ],
        }];

        let candidates = rank_routed_leaf_rows_by_ip(
            routed,
            |row| Ok(f32::from(row.encoded_payload[0])),
            SpireCandidateDedupeMode::NoReplicaDedupeDisabled,
            None,
        )
        .unwrap();

        assert_eq!(candidates.len(), 2);
        assert_eq!(candidates[0].vec_id.local_sequence(), Some(7));
        assert_eq!(candidates[0].score, -9.0);
        assert_eq!(candidates[1].vec_id.local_sequence(), Some(7));
        assert_eq!(candidates[1].score, -1.0);
    }

    #[test]
    fn rank_routed_leaf_rows_by_ip_keeps_bounded_best_candidates() {
        let routed = vec![SpireRoutedLeafScanRows {
            epoch: 1,
            root_pid: SPIRE_FIRST_PID,
            leaf_pid: SPIRE_FIRST_PID + 1,
            rows: vec![
                SpireLeafScanRow {
                    pid: SPIRE_FIRST_PID + 1,
                    object_version: 1,
                    row_index: 0,
                    assignment: assignment_row_with_payload(
                        SPIRE_ASSIGNMENT_FLAG_PRIMARY,
                        1,
                        10,
                        1,
                        vec![3],
                    ),
                },
                SpireLeafScanRow {
                    pid: SPIRE_FIRST_PID + 1,
                    object_version: 1,
                    row_index: 1,
                    assignment: assignment_row_with_payload(
                        SPIRE_ASSIGNMENT_FLAG_PRIMARY,
                        2,
                        10,
                        2,
                        vec![10],
                    ),
                },
                SpireLeafScanRow {
                    pid: SPIRE_FIRST_PID + 1,
                    object_version: 1,
                    row_index: 2,
                    assignment: assignment_row_with_payload(
                        SPIRE_ASSIGNMENT_FLAG_PRIMARY,
                        3,
                        10,
                        3,
                        vec![5],
                    ),
                },
                SpireLeafScanRow {
                    pid: SPIRE_FIRST_PID + 1,
                    object_version: 1,
                    row_index: 3,
                    assignment: assignment_row_with_payload(
                        SPIRE_ASSIGNMENT_FLAG_PRIMARY,
                        4,
                        10,
                        4,
                        vec![7],
                    ),
                },
            ],
        }];

        let candidates = rank_routed_leaf_rows_by_ip(
            routed,
            |row| Ok(f32::from(row.encoded_payload[0])),
            SpireCandidateDedupeMode::NoReplicaDedupeDisabled,
            Some(2),
        )
        .unwrap();

        assert_eq!(candidates.len(), 2);
        assert_eq!(candidates[0].vec_id.local_sequence(), Some(2));
        assert_eq!(candidates[0].score, -10.0);
        assert_eq!(candidates[1].vec_id.local_sequence(), Some(4));
        assert_eq!(candidates[1].score, -7.0);
    }

    #[test]
    fn rank_routed_leaf_rows_by_ip_keeps_bounded_best_deduped_candidates() {
        let routed = vec![SpireRoutedLeafScanRows {
            epoch: 1,
            root_pid: SPIRE_FIRST_PID,
            leaf_pid: SPIRE_FIRST_PID + 1,
            rows: vec![
                SpireLeafScanRow {
                    pid: SPIRE_FIRST_PID + 1,
                    object_version: 1,
                    row_index: 0,
                    assignment: assignment_row_with_payload(
                        SPIRE_ASSIGNMENT_FLAG_PRIMARY,
                        1,
                        10,
                        1,
                        vec![4],
                    ),
                },
                SpireLeafScanRow {
                    pid: SPIRE_FIRST_PID + 1,
                    object_version: 1,
                    row_index: 1,
                    assignment: assignment_row_with_payload(
                        SPIRE_ASSIGNMENT_FLAG_PRIMARY,
                        2,
                        10,
                        2,
                        vec![3],
                    ),
                },
                SpireLeafScanRow {
                    pid: SPIRE_FIRST_PID + 1,
                    object_version: 1,
                    row_index: 2,
                    assignment: assignment_row_with_payload(
                        SPIRE_ASSIGNMENT_FLAG_PRIMARY,
                        3,
                        10,
                        3,
                        vec![10],
                    ),
                },
                SpireLeafScanRow {
                    pid: SPIRE_FIRST_PID + 1,
                    object_version: 1,
                    row_index: 3,
                    assignment: assignment_row_with_payload(
                        SPIRE_ASSIGNMENT_FLAG_PRIMARY,
                        2,
                        10,
                        4,
                        vec![9],
                    ),
                },
                SpireLeafScanRow {
                    pid: SPIRE_FIRST_PID + 1,
                    object_version: 1,
                    row_index: 4,
                    assignment: assignment_row_with_payload(
                        SPIRE_ASSIGNMENT_FLAG_PRIMARY,
                        1,
                        10,
                        5,
                        vec![2],
                    ),
                },
            ],
        }];

        let candidates = rank_routed_leaf_rows_by_ip(
            routed,
            |row| Ok(f32::from(row.encoded_payload[0])),
            SpireCandidateDedupeMode::VecIdDedupeEnabled,
            Some(2),
        )
        .unwrap();

        assert_eq!(candidates.len(), 2);
        assert_eq!(candidates[0].vec_id.local_sequence(), Some(3));
        assert_eq!(candidates[0].score, -10.0);
        assert_eq!(candidates[1].vec_id.local_sequence(), Some(2));
        assert_eq!(candidates[1].score, -9.0);
    }

    #[test]
    fn rank_routed_leaf_rows_by_ip_keeps_primary_tie_break_under_bounded_dedupe() {
        let routed = vec![SpireRoutedLeafScanRows {
            epoch: 1,
            root_pid: SPIRE_FIRST_PID,
            leaf_pid: SPIRE_FIRST_PID + 1,
            rows: vec![
                SpireLeafScanRow {
                    pid: SPIRE_FIRST_PID + 1,
                    object_version: 1,
                    row_index: 0,
                    assignment: assignment_row_with_payload(
                        SPIRE_ASSIGNMENT_FLAG_BOUNDARY_REPLICA,
                        7,
                        10,
                        1,
                        vec![5],
                    ),
                },
                SpireLeafScanRow {
                    pid: SPIRE_FIRST_PID + 1,
                    object_version: 1,
                    row_index: 1,
                    assignment: assignment_row_with_payload(
                        SPIRE_ASSIGNMENT_FLAG_PRIMARY,
                        7,
                        10,
                        2,
                        vec![5],
                    ),
                },
            ],
        }];

        let candidates = rank_routed_leaf_rows_by_ip(
            routed,
            |row| Ok(f32::from(row.encoded_payload[0])),
            SpireCandidateDedupeMode::VecIdDedupeEnabled,
            Some(1),
        )
        .unwrap();

        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].vec_id.local_sequence(), Some(7));
        assert_eq!(candidates[0].assignment_flags, SPIRE_ASSIGNMENT_FLAG_PRIMARY);
        assert_eq!(candidates[0].heap_tid, tid(10, 2));
        assert_eq!(candidates[0].score, -5.0);
    }

    #[test]
    fn scored_candidate_tie_break_prefers_newer_epoch_then_primary_role() {
        let older_primary = scored_candidate(1, 10, 1, 1.0);
        let mut newer_replica = scored_candidate(2, 10, 2, 1.0);
        newer_replica.epoch = 2;
        newer_replica.assignment_flags =
            SPIRE_ASSIGNMENT_FLAG_PRIMARY | SPIRE_ASSIGNMENT_FLAG_BOUNDARY_REPLICA;
        let mut newer_primary = scored_candidate(3, 10, 3, 1.0);
        newer_primary.epoch = 2;

        let ranked = super::rank_bounded_scored_candidates(
            vec![older_primary, newer_replica, newer_primary],
            None,
        );

        assert_eq!(ranked[0].vec_id.local_sequence(), Some(3));
        assert_eq!(ranked[1].vec_id.local_sequence(), Some(2));
        assert_eq!(ranked[2].vec_id.local_sequence(), Some(1));
    }

    #[test]
    fn rank_routed_leaf_rows_by_ip_rejects_non_finite_scores() {
        let routed = vec![SpireRoutedLeafScanRows {
            epoch: 1,
            root_pid: SPIRE_FIRST_PID,
            leaf_pid: SPIRE_FIRST_PID + 1,
            rows: vec![SpireLeafScanRow {
                pid: SPIRE_FIRST_PID + 1,
                object_version: 1,
                row_index: 0,
                assignment: assignment_row(SPIRE_ASSIGNMENT_FLAG_PRIMARY, 1),
            }],
        }];

        assert!(rank_routed_leaf_rows_by_ip(
            routed,
            |_| Ok(f32::NAN),
            SpireCandidateDedupeMode::NoReplicaDedupeDisabled,
            None
        )
        .unwrap_err()
        .contains("non-finite"));
    }
