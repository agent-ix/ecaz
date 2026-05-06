    #[test]
    fn recursive_routing_build_keeps_single_level_shape_when_under_fanout() {
        let mut pid_allocator = SpirePidAllocator::default();

        let draft = super::build_recursive_routing_hierarchy_draft(
            SpireRecursiveRoutingBuildInput {
                object_version: 3,
                dimensions: 2,
                target_fanout: 4,
                seed: 42,
                children: vec![
                    recursive_child(11, vec![1.0, 0.0]),
                    recursive_child(12, vec![-1.0, 0.0]),
                ],
            },
            &mut pid_allocator,
        )
        .expect("recursive routing draft should build");

        assert_eq!(draft.root_pid, SPIRE_FIRST_PID);
        assert_eq!(draft.root_level, 1);
        assert_eq!(draft.next_pid, SPIRE_FIRST_PID + 1);
        assert_eq!(pid_allocator.next_pid(), draft.next_pid);
        assert_eq!(draft.routing_objects.len(), 1);
        assert_eq!(draft.centroid_records.len(), 2);
        let root = &draft.routing_objects[0];
        assert_eq!(root.header.pid, draft.root_pid);
        assert_eq!(root.header.level, 1);
        assert_eq!(
            root.children()
                .map(|child| child.child_pid)
                .collect::<Vec<_>>(),
            vec![11, 12]
        );
        assert_eq!(
            draft
                .centroid_records
                .iter()
                .map(|record| (
                    record.parent_pid,
                    record.child_pid,
                    record.child_level,
                    record.centroid_ordinal
                ))
                .collect::<Vec<_>>(),
            vec![(draft.root_pid, 11, 0, 0), (draft.root_pid, 12, 0, 1)]
        );
    }

    #[test]
    fn recursive_routing_build_validation_rejects_sparse_centroid_ordinals() {
        let mut pid_allocator = SpirePidAllocator::default();
        let mut draft = super::build_recursive_routing_hierarchy_draft(
            SpireRecursiveRoutingBuildInput {
                object_version: 3,
                dimensions: 2,
                target_fanout: 4,
                seed: 42,
                children: vec![
                    recursive_child(11, vec![1.0, 0.0]),
                    recursive_child(12, vec![-1.0, 0.0]),
                ],
            },
            &mut pid_allocator,
        )
        .expect("recursive routing draft should build");
        draft.centroid_records[1].centroid_ordinal = 7;

        let error = super::validate_recursive_routing_build_draft(&draft).unwrap_err();

        assert!(error.contains("centroid ordinals"));
    }

    #[test]
    fn recursive_routing_build_materializes_internal_level() {
        let mut pid_allocator = SpirePidAllocator::default();

        let draft = super::build_recursive_routing_hierarchy_draft(
            SpireRecursiveRoutingBuildInput {
                object_version: 3,
                dimensions: 2,
                target_fanout: 2,
                seed: 42,
                children: vec![
                    recursive_child(11, vec![1.0, 0.0]),
                    recursive_child(12, vec![0.9, 0.1]),
                    recursive_child(13, vec![-1.0, 0.0]),
                    recursive_child(14, vec![-0.9, 0.1]),
                ],
            },
            &mut pid_allocator,
        )
        .expect("recursive routing draft should build");

        assert_eq!(draft.root_level, 2);
        assert_eq!(draft.routing_objects.len(), 3);
        assert_eq!(draft.centroid_records.len(), 6);
        let root = &draft.routing_objects[0];
        assert_eq!(root.header.pid, draft.root_pid);
        assert_eq!(root.header.level, 2);
        assert_eq!(root.header.parent_pid, 0);
        assert_eq!(root.child_count(), 2);
        let root_child_pids = root
            .children()
            .map(|child| child.child_pid)
            .collect::<Vec<_>>();
        let internal_objects = draft.routing_objects.iter().skip(1).collect::<Vec<_>>();
        assert_eq!(
            internal_objects
                .iter()
                .map(|object| object.header.pid)
                .collect::<Vec<_>>(),
            root_child_pids
        );
        for object in internal_objects {
            assert_eq!(object.header.kind, SpirePartitionObjectKind::Internal);
            assert_eq!(object.header.level, 1);
            assert_eq!(object.header.parent_pid, draft.root_pid);
            assert!(object.child_count() >= 1);
            assert!(object
                .children()
                .all(|child| [11, 12, 13, 14].contains(&child.child_pid)));
        }
        let root_centroid_records = draft
            .centroid_records
            .iter()
            .filter(|record| record.parent_pid == draft.root_pid)
            .collect::<Vec<_>>();
        assert_eq!(root_centroid_records.len(), 2);
        assert!(root_centroid_records
            .iter()
            .all(|record| record.child_level == 1 && record.source_count >= 1));
        assert!(draft
            .centroid_records
            .iter()
            .filter(|record| record.child_level == 0)
            .all(|record| [11, 12, 13, 14].contains(&record.child_pid)));
    }

    #[test]
    fn local_recursive_routing_epoch_draft_combines_routing_and_leaf_placements() {
        let mut pid_allocator = SpirePidAllocator::default();
        let routing_draft = super::build_recursive_routing_hierarchy_draft(
            SpireRecursiveRoutingBuildInput {
                object_version: 3,
                dimensions: 2,
                target_fanout: 2,
                seed: 42,
                children: vec![
                    recursive_child(11, vec![1.0, 0.0]),
                    recursive_child(12, vec![0.9, 0.1]),
                    recursive_child(13, vec![-1.0, 0.0]),
                    recursive_child(14, vec![-0.9, 0.1]),
                ],
            },
            &mut pid_allocator,
        )
        .expect("recursive routing draft should build");
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let mut leaf_placements = Vec::new();
        for object in routing_draft
            .routing_objects
            .iter()
            .filter(|object| object.header.level == 1)
        {
            for child in object.children() {
                leaf_placements.push(
                    object_store
                        .insert_leaf_object_v2_from_rows(
                            7,
                            child.child_pid,
                            routing_draft.routing_objects[0].header.object_version,
                            object.header.pid,
                            &[],
                        )
                        .unwrap(),
                );
            }
        }

        let draft = build_local_recursive_routing_epoch_draft(
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

        assert_eq!(draft.root_pid, SPIRE_FIRST_PID + 2);
        assert_eq!(draft.routing_objects.len(), 3);
        assert_eq!(draft.centroid_records.len(), 6);
        assert_eq!(draft.object_manifest.entries.len(), 7);
        assert_eq!(draft.placement_directory.entries.len(), 7);
        assert!(draft.next_pid >= 15);
        let root_placement = draft.placement_directory.get(draft.root_pid).unwrap();
        let stored_root = object_store.read_routing_object(root_placement).unwrap();
        assert_eq!(stored_root.header.kind, SpirePartitionObjectKind::Root);
        assert_eq!(stored_root.header.level, 2);
        SpirePublishedEpochSnapshot::new(
            &draft.epoch_manifest,
            &draft.object_manifest,
            &draft.placement_directory,
        )
        .unwrap();
    }

    #[test]
    fn local_recursive_routing_epoch_draft_rejects_missing_leaf_placement() {
        let mut pid_allocator = SpirePidAllocator::default();
        let routing_draft = super::build_recursive_routing_hierarchy_draft(
            SpireRecursiveRoutingBuildInput {
                object_version: 3,
                dimensions: 2,
                target_fanout: 4,
                seed: 42,
                children: vec![
                    recursive_child(11, vec![1.0, 0.0]),
                    recursive_child(12, vec![-1.0, 0.0]),
                ],
            },
            &mut pid_allocator,
        )
        .expect("recursive routing draft should build");
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let leaf_placement = object_store
            .insert_leaf_object_v2_from_rows(7, 11, 3, routing_draft.root_pid, &[])
            .unwrap();

        let error = build_local_recursive_routing_epoch_draft(
            SpireRecursiveRoutingEpochInput {
                epoch: 7,
                published_at_micros: 1000,
                retain_until_micros: 2000,
                consistency_mode: SpireConsistencyMode::Strict,
                routing_draft,
                leaf_placements: vec![leaf_placement],
            },
            &mut object_store,
        )
        .unwrap_err();

        assert!(error.contains("leaf placement mismatch"));
    }

    #[test]
    fn local_recursive_routing_epoch_draft_rejects_leaf_parent_drift() {
        let mut pid_allocator = SpirePidAllocator::default();
        let routing_draft = super::build_recursive_routing_hierarchy_draft(
            SpireRecursiveRoutingBuildInput {
                object_version: 3,
                dimensions: 2,
                target_fanout: 4,
                seed: 42,
                children: vec![
                    recursive_child(11, vec![1.0, 0.0]),
                    recursive_child(12, vec![-1.0, 0.0]),
                ],
            },
            &mut pid_allocator,
        )
        .expect("recursive routing draft should build");
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let first_leaf = object_store
            .insert_leaf_object_v2_from_rows(7, 11, 3, routing_draft.root_pid + 99, &[])
            .unwrap();
        let second_leaf = object_store
            .insert_leaf_object_v2_from_rows(7, 12, 3, routing_draft.root_pid, &[])
            .unwrap();

        let error = build_local_recursive_routing_epoch_draft(
            SpireRecursiveRoutingEpochInput {
                epoch: 7,
                published_at_micros: 1000,
                retain_until_micros: 2000,
                consistency_mode: SpireConsistencyMode::Strict,
                routing_draft,
                leaf_placements: vec![first_leaf, second_leaf],
            },
            &mut object_store,
        )
        .unwrap_err();

        assert!(error.contains("parent"));
        assert!(error.contains("does not match routing parent"));
    }

    #[test]
    fn local_recursive_routing_epoch_from_leaf_inputs_writes_leaf_objects() {
        let mut pid_allocator = SpirePidAllocator::default();
        let routing_draft = super::build_recursive_routing_hierarchy_draft(
            SpireRecursiveRoutingBuildInput {
                object_version: 3,
                dimensions: 2,
                target_fanout: 4,
                seed: 42,
                children: vec![
                    recursive_child(11, vec![1.0, 0.0]),
                    recursive_child(12, vec![-1.0, 0.0]),
                ],
            },
            &mut pid_allocator,
        )
        .expect("recursive routing draft should build");
        let root_pid = routing_draft.root_pid;
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();

        let draft = super::build_local_recursive_routing_epoch_from_leaf_inputs(
            SpireRecursiveRoutingEpochObjectInput {
                epoch: 7,
                published_at_micros: 1000,
                retain_until_micros: 2000,
                consistency_mode: SpireConsistencyMode::Strict,
                routing_draft,
                leaf_inputs: vec![
                    SpireRecursiveLeafObjectInput {
                        pid: 11,
                        object_version: 3,
                        parent_pid: root_pid,
                        rows: vec![primary_row(1, 10, 1)],
                    },
                    SpireRecursiveLeafObjectInput {
                        pid: 12,
                        object_version: 3,
                        parent_pid: root_pid,
                        rows: vec![primary_row(2, 10, 2)],
                    },
                ],
            },
            &mut object_store,
        )
        .unwrap();

        assert_eq!(draft.object_manifest.entries.len(), 3);
        assert_eq!(draft.centroid_records.len(), 2);
        let first_leaf_placement = draft.placement_directory.get(11).unwrap();
        let first_leaf = object_store
            .read_leaf_object_v2(first_leaf_placement)
            .unwrap();
        assert_eq!(first_leaf.meta.header.parent_pid, root_pid);
        assert_eq!(
            first_leaf.assignment_rows().unwrap()[0].heap_tid,
            tid(10, 1)
        );
        SpirePublishedEpochSnapshot::new(
            &draft.epoch_manifest,
            &draft.object_manifest,
            &draft.placement_directory,
        )
        .unwrap();
    }

    #[test]
    fn local_recursive_routing_epoch_from_leaf_inputs_rejects_parent_drift() {
        let mut pid_allocator = SpirePidAllocator::default();
        let routing_draft = super::build_recursive_routing_hierarchy_draft(
            SpireRecursiveRoutingBuildInput {
                object_version: 3,
                dimensions: 2,
                target_fanout: 4,
                seed: 42,
                children: vec![
                    recursive_child(11, vec![1.0, 0.0]),
                    recursive_child(12, vec![-1.0, 0.0]),
                ],
            },
            &mut pid_allocator,
        )
        .expect("recursive routing draft should build");
        let root_pid = routing_draft.root_pid;
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();

        let error = super::build_local_recursive_routing_epoch_from_leaf_inputs(
            SpireRecursiveRoutingEpochObjectInput {
                epoch: 7,
                published_at_micros: 1000,
                retain_until_micros: 2000,
                consistency_mode: SpireConsistencyMode::Strict,
                routing_draft,
                leaf_inputs: vec![
                    SpireRecursiveLeafObjectInput {
                        pid: 11,
                        object_version: 3,
                        parent_pid: root_pid + 99,
                        rows: Vec::new(),
                    },
                    SpireRecursiveLeafObjectInput {
                        pid: 12,
                        object_version: 3,
                        parent_pid: root_pid,
                        rows: Vec::new(),
                    },
                ],
            },
            &mut object_store,
        )
        .unwrap_err();

        assert!(error.contains("does not match routing parent"));
    }

    #[test]
    fn recursive_build_coordinator_assembles_epoch_input_from_centroid_plan() {
        let mut pid_allocator = SpirePidAllocator::default();
        let mut local_vec_id_allocator = SpireLocalVecIdAllocator::default();
        let centroid_plan = SpireSingleLevelCentroidPlan {
            dimensions: 2,
            centroids: vec![
                vec![1.0, 0.0],
                vec![0.9, 0.1],
                vec![-1.0, 0.0],
                vec![-0.9, 0.1],
            ],
            assignment_indexes: vec![0, 1, 2, 3],
        };

        let draft = super::build_recursive_epoch_input_from_centroid_plan(
            SpireRecursiveBuildCoordinatorInput {
                epoch: 7,
                object_version: 3,
                published_at_micros: 1000,
                retain_until_micros: 2000,
                consistency_mode: SpireConsistencyMode::Strict,
                target_fanout: 2,
                seed: 42,
                assignments: vec![
                    assignment_input(10, 1),
                    assignment_input(10, 2),
                    assignment_input(10, 3),
                    assignment_input(10, 4),
                ],
                centroid_plan,
            },
            &mut pid_allocator,
            &mut local_vec_id_allocator,
        )
        .unwrap();

        assert_eq!(
            draft.leaf_pids,
            vec![
                SPIRE_FIRST_PID,
                SPIRE_FIRST_PID + 1,
                SPIRE_FIRST_PID + 2,
                SPIRE_FIRST_PID + 3
            ]
        );
        assert_eq!(draft.epoch_input.routing_draft.root_level, 2);
        assert_eq!(draft.epoch_input.leaf_inputs.len(), 4);
        assert!(draft
            .epoch_input
            .leaf_inputs
            .iter()
            .all(|leaf_input| leaf_input.parent_pid != 0 && leaf_input.rows.len() == 1));
        assert_eq!(
            draft
                .epoch_input
                .leaf_inputs
                .iter()
                .map(|leaf_input| leaf_input.rows[0].vec_id.clone())
                .collect::<Vec<_>>(),
            vec![
                SpireVecId::local(SPIRE_FIRST_LOCAL_VEC_SEQ),
                SpireVecId::local(SPIRE_FIRST_LOCAL_VEC_SEQ + 1),
                SpireVecId::local(SPIRE_FIRST_LOCAL_VEC_SEQ + 2),
                SpireVecId::local(SPIRE_FIRST_LOCAL_VEC_SEQ + 3),
            ]
        );

        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let next_pid = draft.next_pid;
        let next_local_vec_seq = draft.next_local_vec_seq;
        let epoch_draft = super::build_local_recursive_routing_epoch_from_leaf_inputs(
            draft.epoch_input,
            &mut object_store,
        )
        .unwrap();
        assert_eq!(epoch_draft.root_pid, SPIRE_FIRST_PID + 6);
        assert_eq!(epoch_draft.object_manifest.entries.len(), 7);
        assert_eq!(epoch_draft.centroid_records.len(), 6);
        assert_eq!(pid_allocator.next_pid(), next_pid);
        assert_eq!(
            local_vec_id_allocator.next_local_vec_seq(),
            next_local_vec_seq
        );
    }

    #[test]
    fn recursive_build_coordinator_rejects_assignment_count_mismatch() {
        let mut pid_allocator = SpirePidAllocator::default();
        let mut local_vec_id_allocator = SpireLocalVecIdAllocator::default();
        let centroid_plan = SpireSingleLevelCentroidPlan {
            dimensions: 2,
            centroids: vec![vec![1.0, 0.0], vec![-1.0, 0.0]],
            assignment_indexes: vec![0, 1],
        };

        let error = super::build_recursive_epoch_input_from_centroid_plan(
            SpireRecursiveBuildCoordinatorInput {
                epoch: 7,
                object_version: 3,
                published_at_micros: 1000,
                retain_until_micros: 2000,
                consistency_mode: SpireConsistencyMode::Strict,
                target_fanout: 2,
                seed: 42,
                assignments: vec![assignment_input(10, 1)],
                centroid_plan,
            },
            &mut pid_allocator,
            &mut local_vec_id_allocator,
        )
        .unwrap_err();

        assert!(error.contains("assignment count"));
        assert_eq!(pid_allocator.next_pid(), SPIRE_FIRST_PID);
        assert_eq!(
            local_vec_id_allocator.next_local_vec_seq(),
            SPIRE_FIRST_LOCAL_VEC_SEQ
        );
    }

    #[test]
    fn recursive_epoch_draft_encodes_publish_bundle_with_allocator_cursor() {
        let mut pid_allocator = SpirePidAllocator::default();
        let mut local_vec_id_allocator = SpireLocalVecIdAllocator::default();
        let centroid_plan = SpireSingleLevelCentroidPlan {
            dimensions: 2,
            centroids: vec![vec![1.0, 0.0], vec![-1.0, 0.0]],
            assignment_indexes: vec![0, 1],
        };
        let coordinator = super::build_recursive_epoch_input_from_centroid_plan(
            SpireRecursiveBuildCoordinatorInput {
                epoch: 7,
                object_version: 3,
                published_at_micros: 1000,
                retain_until_micros: 2000,
                consistency_mode: SpireConsistencyMode::Strict,
                target_fanout: 2,
                seed: 42,
                assignments: vec![assignment_input(10, 1), assignment_input(10, 2)],
                centroid_plan,
            },
            &mut pid_allocator,
            &mut local_vec_id_allocator,
        )
        .unwrap();
        let next_local_vec_seq = coordinator.next_local_vec_seq;
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let draft = super::build_local_recursive_routing_epoch_from_leaf_inputs(
            coordinator.epoch_input,
            &mut object_store,
        )
        .unwrap();

        let encoded = draft
            .encode_publish_bundle(next_local_vec_seq, manifest_locators())
            .unwrap();
        let root_control = SpireRootControlState::decode(&encoded.root_control_state).unwrap();

        assert_eq!(
            SpireEpochManifest::decode(&encoded.manifests.epoch_manifest).unwrap(),
            draft.epoch_manifest
        );
        assert_eq!(
            SpireObjectManifest::decode(&encoded.manifests.object_manifest).unwrap(),
            draft.object_manifest
        );
        assert_eq!(
            SpirePlacementDirectory::decode(&encoded.manifests.placement_directory).unwrap(),
            draft.placement_directory
        );
        assert_eq!(root_control.active_epoch, draft.epoch_manifest.epoch);
        assert_eq!(root_control.next_pid, draft.next_pid);
        assert_eq!(root_control.next_local_vec_seq, next_local_vec_seq);
        assert_eq!(root_control.epoch_manifest_tid, tid(70, 1));
        assert_eq!(root_control.object_manifest_tid, tid(70, 2));
        assert_eq!(root_control.placement_directory_tid, tid(70, 3));
    }

    #[test]
    fn recursive_epoch_relation_publish_input_uses_durable_placement_manifest() {
        let mut pid_allocator = SpirePidAllocator::default();
        let mut local_vec_id_allocator = SpireLocalVecIdAllocator::default();
        let centroid_plan = SpireSingleLevelCentroidPlan {
            dimensions: 2,
            centroids: vec![vec![1.0, 0.0], vec![-1.0, 0.0]],
            assignment_indexes: vec![0, 1],
        };
        let coordinator = super::build_recursive_epoch_input_from_centroid_plan(
            SpireRecursiveBuildCoordinatorInput {
                epoch: 7,
                object_version: 3,
                published_at_micros: 1000,
                retain_until_micros: 2000,
                consistency_mode: SpireConsistencyMode::Strict,
                target_fanout: 2,
                seed: 42,
                assignments: vec![assignment_input(10, 1), assignment_input(10, 2)],
                centroid_plan,
            },
            &mut pid_allocator,
            &mut local_vec_id_allocator,
        )
        .unwrap();
        let next_local_vec_seq = coordinator.next_local_vec_seq;
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let draft = super::build_local_recursive_routing_epoch_from_leaf_inputs(
            coordinator.epoch_input,
            &mut object_store,
        )
        .unwrap();
        let placement_evidence = draft
            .placement_directory
            .entries
            .iter()
            .enumerate()
            .map(|(index, placement)| SpirePublishPlacementWriteEvidence {
                pid: placement.pid,
                placement_tid: tid(90, (index + 1) as u16),
            })
            .collect::<Vec<_>>();
        let durable_manifest = object_manifest_from_placement_writes(
            draft.epoch_manifest.epoch,
            &draft.placement_directory,
            &placement_evidence,
        )
        .unwrap();

        let encoded = encode_publish_bundle_for_publish(
            draft.relation_publish_input(&durable_manifest, next_local_vec_seq),
            manifest_locators(),
        )
        .unwrap();

        assert_eq!(
            SpireObjectManifest::decode(&encoded.manifests.object_manifest).unwrap(),
            durable_manifest
        );
        assert!(durable_manifest
            .entries
            .iter()
            .all(|entry| entry.placement_tid.block_number == 90));
        let root_control = SpireRootControlState::decode(&encoded.root_control_state).unwrap();
        assert_eq!(root_control.next_pid, draft.next_pid);
        assert_eq!(root_control.next_local_vec_seq, next_local_vec_seq);
    }

    #[test]
    fn recursive_routing_build_rejects_mixed_child_levels() {
        let mut pid_allocator = SpirePidAllocator::default();
        let mut internal_child = recursive_child(12, vec![-1.0, 0.0]);
        internal_child.child_level = 1;

        let error = super::build_recursive_routing_hierarchy_draft(
            SpireRecursiveRoutingBuildInput {
                object_version: 3,
                dimensions: 2,
                target_fanout: 2,
                seed: 42,
                children: vec![recursive_child(11, vec![1.0, 0.0]), internal_child],
            },
            &mut pid_allocator,
        )
        .unwrap_err();

        assert!(error.contains("does not match expected level"));
        assert_eq!(pid_allocator.next_pid(), SPIRE_FIRST_PID);
    }
