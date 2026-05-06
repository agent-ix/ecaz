    #[test]
    fn recursive_quantized_candidates_match_flat_single_level_on_small_hierarchy() {
        let payload_format = SpireAssignmentPayloadFormat::TurboQuant;
        let leaf_specs = [
            (SPIRE_FIRST_PID + 11, 1, tid(10, 1), [0.5, 0.0]),
            (SPIRE_FIRST_PID + 12, 2, tid(10, 2), [1.5, 0.0]),
            (SPIRE_FIRST_PID + 21, 3, tid(10, 3), [-1.5, 0.0]),
            (SPIRE_FIRST_PID + 22, 4, tid(10, 4), [-0.5, 0.0]),
        ];
        let mut flat_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let flat_root = SpireRoutingPartitionObject::root(
            SPIRE_FIRST_PID,
            1,
            2,
            leaf_specs
                .iter()
                .enumerate()
                .map(|(centroid_index, (pid, _, _, centroid))| {
                    routing_child(
                        u32::try_from(centroid_index).unwrap(),
                        *pid,
                        centroid.to_vec(),
                    )
                })
                .collect(),
        )
        .unwrap();
        let mut flat_placements = vec![flat_store.insert_routing_object(7, &flat_root).unwrap()];
        for (pid, vec_seq, heap_tid, source_vector) in &leaf_specs {
            let input = encode_assignment_input(payload_format, *heap_tid, source_vector).unwrap();
            let rows = vec![SpireLeafAssignmentRow {
                flags: SPIRE_ASSIGNMENT_FLAG_PRIMARY,
                vec_id: SpireVecId::local(*vec_seq),
                heap_tid: *heap_tid,
                payload_format: input.payload_format,
                gamma: input.gamma,
                encoded_payload: input.encoded_payload,
            }];
            flat_placements.push(
                flat_store
                    .insert_leaf_object_v2_from_rows(7, *pid, 1, flat_root.header.pid, &rows)
                    .unwrap(),
            );
        }
        let epoch_manifest = SpireEpochManifest {
            epoch: 7,
            state: SpireEpochState::Published,
            consistency_mode: SpireConsistencyMode::Strict,
            published_at_micros: 1000,
            retain_until_micros: 2000,
            active_query_count: 0,
        };
        let flat_object_manifest = SpireObjectManifest::from_entries(
            7,
            flat_placements.iter().map(manifest_entry_for).collect(),
        )
        .unwrap();
        let flat_placement_directory =
            SpirePlacementDirectory::from_entries(7, flat_placements).unwrap();
        let flat_snapshot = SpirePublishedEpochSnapshot::new(
            &epoch_manifest,
            &flat_object_manifest,
            &flat_placement_directory,
        )
        .unwrap();

        let mut recursive_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
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
            recursive_root.header.pid,
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
            recursive_root.header.pid,
            2,
            vec![
                routing_child(0, SPIRE_FIRST_PID + 21, vec![-1.5, 0.0]),
                routing_child(1, SPIRE_FIRST_PID + 22, vec![-0.5, 0.0]),
            ],
        )
        .unwrap();
        let mut recursive_placements = vec![
            recursive_store
                .insert_routing_object(7, &recursive_root)
                .unwrap(),
            recursive_store
                .insert_routing_object(7, &internal_a)
                .unwrap(),
            recursive_store
                .insert_routing_object(7, &internal_b)
                .unwrap(),
        ];
        for (pid, vec_seq, heap_tid, source_vector) in &leaf_specs {
            let input = encode_assignment_input(payload_format, *heap_tid, source_vector).unwrap();
            let rows = vec![SpireLeafAssignmentRow {
                flags: SPIRE_ASSIGNMENT_FLAG_PRIMARY,
                vec_id: SpireVecId::local(*vec_seq),
                heap_tid: *heap_tid,
                payload_format: input.payload_format,
                gamma: input.gamma,
                encoded_payload: input.encoded_payload,
            }];
            let parent_pid = if *pid < SPIRE_FIRST_PID + 20 {
                internal_a.header.pid
            } else {
                internal_b.header.pid
            };
            recursive_placements.push(
                recursive_store
                    .insert_leaf_object_v2_from_rows(7, *pid, 1, parent_pid, &rows)
                    .unwrap(),
            );
        }
        let recursive_object_manifest = SpireObjectManifest::from_entries(
            7,
            recursive_placements
                .iter()
                .map(manifest_entry_for)
                .collect(),
        )
        .unwrap();
        let recursive_placement_directory =
            SpirePlacementDirectory::from_entries(7, recursive_placements).unwrap();
        let recursive_snapshot = SpirePublishedEpochSnapshot::new(
            &epoch_manifest,
            &recursive_object_manifest,
            &recursive_placement_directory,
        )
        .unwrap();

        let flat_candidates = collect_quantized_routed_probe_candidates(
            &flat_snapshot,
            &flat_store,
            &[1.0, 0.0],
            1,
            payload_format,
            SpireCandidateDedupeMode::NoReplicaDedupeDisabled,
            Some(1),
        )
        .unwrap();
        let recursive_candidates = collect_quantized_routed_probe_candidates(
            &recursive_snapshot,
            &recursive_store,
            &[1.0, 0.0],
            1,
            payload_format,
            SpireCandidateDedupeMode::NoReplicaDedupeDisabled,
            Some(1),
        )
        .unwrap();

        assert_eq!(flat_candidates.len(), 1);
        assert_eq!(recursive_candidates, flat_candidates);
        assert_eq!(recursive_candidates[0].pid, SPIRE_FIRST_PID + 12);
        assert_eq!(recursive_candidates[0].heap_tid, tid(10, 2));
    }

    #[test]
    fn materialized_recursive_routing_epoch_scans_quantized_candidates() {
        let payload_format = SpireAssignmentPayloadFormat::TurboQuant;
        let leaf_specs = [
            (SPIRE_FIRST_PID + 20, 1, tid(10, 1), [0.5, 0.0]),
            (SPIRE_FIRST_PID + 21, 2, tid(10, 2), [1.5, 0.0]),
            (SPIRE_FIRST_PID + 22, 3, tid(10, 3), [-1.5, 0.0]),
            (SPIRE_FIRST_PID + 23, 4, tid(10, 4), [-0.5, 0.0]),
        ];
        let mut pid_allocator = SpirePidAllocator::default();
        let routing_draft = build_recursive_routing_hierarchy_draft(
            SpireRecursiveRoutingBuildInput {
                object_version: 1,
                dimensions: 2,
                target_fanout: 2,
                seed: 42,
                children: leaf_specs
                    .iter()
                    .map(|(pid, _, _, centroid)| SpireRecursiveRoutingChildInput {
                        child_pid: *pid,
                        child_level: 0,
                        centroid: centroid.to_vec(),
                        source_count: 1,
                    })
                    .collect(),
            },
            &mut pid_allocator,
        )
        .unwrap();
        let mut leaf_parent_pids = HashMap::new();
        for object in routing_draft
            .routing_objects
            .iter()
            .filter(|object| object.header.level == 1)
        {
            for child in object.children() {
                leaf_parent_pids.insert(child.child_pid, object.header.pid);
            }
        }

        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let mut leaf_placements = Vec::new();
        for (pid, vec_seq, heap_tid, source_vector) in &leaf_specs {
            let input = encode_assignment_input(payload_format, *heap_tid, source_vector).unwrap();
            let rows = vec![SpireLeafAssignmentRow {
                flags: SPIRE_ASSIGNMENT_FLAG_PRIMARY,
                vec_id: SpireVecId::local(*vec_seq),
                heap_tid: *heap_tid,
                payload_format: input.payload_format,
                gamma: input.gamma,
                encoded_payload: input.encoded_payload,
            }];
            leaf_placements.push(
                object_store
                    .insert_leaf_object_v2_from_rows(
                        7,
                        *pid,
                        1,
                        *leaf_parent_pids.get(pid).unwrap(),
                        &rows,
                    )
                    .unwrap(),
            );
        }
        let epoch_draft = build_local_recursive_routing_epoch_draft(
            SpireRecursiveRoutingEpochInput {
                epoch: 7,
                published_at_micros: 1000,
                retain_until_micros: 2000,
                consistency_mode: SpireConsistencyMode::Strict,
                routing_draft,
                leaf_placements,
            },
            &mut object_store,
        )
        .unwrap();
        let snapshot = SpirePublishedEpochSnapshot::new(
            &epoch_draft.epoch_manifest,
            &epoch_draft.object_manifest,
            &epoch_draft.placement_directory,
        )
        .unwrap();

        let candidates = collect_quantized_routed_probe_candidates(
            &snapshot,
            &object_store,
            &[1.0, 0.0],
            2,
            payload_format,
            SpireCandidateDedupeMode::NoReplicaDedupeDisabled,
            Some(2),
        )
        .unwrap();

        assert_eq!(candidates.len(), 2);
        assert_eq!(candidates[0].pid, SPIRE_FIRST_PID + 21);
        assert_eq!(candidates[0].heap_tid, tid(10, 2));
        assert_eq!(candidates[1].pid, SPIRE_FIRST_PID + 20);
        assert_eq!(candidates[1].heap_tid, tid(10, 1));
    }

