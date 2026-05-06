    #[test]
    fn replacement_placement_directory_carries_unaffected_and_drops_old_leaf_and_delta() {
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let active_epoch = 7;
        let new_epoch = 8;
        let root = root_routing_object();
        let root_placement = object_store
            .insert_routing_object(active_epoch, &root)
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
        let delta =
            SpireDeltaPartitionObject::new(40, 1, 12, vec![delta_insert_row(4, 20, 1)]).unwrap();
        let delta_placement = object_store
            .insert_delta_object(active_epoch, &delta)
            .unwrap();
        let active_placements = vec![root_placement, leaf_11, leaf_12, leaf_13, delta_placement];
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
            active_placements
                .iter()
                .map(manifest_entry_for)
                .collect::<Vec<_>>(),
        )
        .unwrap();
        let placement_directory =
            SpirePlacementDirectory::from_entries(active_epoch, active_placements).unwrap();
        let snapshot = SpirePublishedEpochSnapshot::new(
            &epoch_manifest,
            &object_manifest,
            &placement_directory,
        )
        .unwrap();
        let replacement_root = rewrite_routing_partition_for_leaf_replacement(
            &root,
            &[12],
            vec![
                replacement_child(21, vec![0.5, 0.5]),
                replacement_child(22, vec![-0.5, 0.5]),
            ],
            4,
        )
        .unwrap();
        let replacement_root_placement = object_store
            .insert_routing_object(new_epoch, &replacement_root)
            .unwrap();
        let replacement_leaf_21 = object_store
            .insert_leaf_object_v2_from_rows(new_epoch, 21, 1, root.header.pid, &[])
            .unwrap();
        let replacement_leaf_22 = object_store
            .insert_leaf_object_v2_from_rows(new_epoch, 22, 1, root.header.pid, &[])
            .unwrap();

        let replacement_directory = plan_replacement_epoch_placement_directory(
            &snapshot,
            &object_store,
            new_epoch,
            root.header.pid,
            replacement_root_placement,
            &[12],
            vec![replacement_leaf_21, replacement_leaf_22],
        )
        .unwrap();

        let pids = replacement_directory
            .entries
            .iter()
            .map(|entry| entry.pid)
            .collect::<Vec<_>>();
        assert_eq!(pids, vec![1, 11, 13, 21, 22]);
        assert!(replacement_directory
            .entries
            .iter()
            .all(|entry| entry.epoch == new_epoch));
        assert!(replacement_directory.get(12).is_none());
        assert!(replacement_directory.get(40).is_none());
        assert_eq!(
            object_store
                .read_object_header(placement_directory.get(12).unwrap())
                .unwrap()
                .pid,
            12
        );
    }

    #[test]
    fn replacement_epoch_draft_builds_manifest_and_publish_bundle() {
        let placement_directory = SpirePlacementDirectory::from_entries(
            8,
            vec![
                SpirePlacementEntry::local_single_store_available(8, 1, 12345, 4, tid(30, 1), 128),
                SpirePlacementEntry::local_single_store_available(8, 21, 12345, 1, tid(31, 1), 256),
            ],
        )
        .unwrap();
        let draft = build_replacement_epoch_draft(SpireReplacementEpochInput {
            epoch: 8,
            published_at_micros: 1000,
            retain_until_micros: 2000,
            consistency_mode: SpireConsistencyMode::Strict,
            placement_directory,
            placement_write_evidence: vec![
                SpirePublishPlacementWriteEvidence {
                    pid: 21,
                    placement_tid: tid(90, 2),
                },
                SpirePublishPlacementWriteEvidence {
                    pid: 1,
                    placement_tid: tid(90, 1),
                },
            ],
            next_pid: 30,
            next_local_vec_seq: 5,
        })
        .unwrap();

        assert_eq!(draft.epoch_manifest.epoch, 8);
        assert_eq!(
            draft.object_manifest.get(1).unwrap().placement_tid,
            tid(90, 1)
        );
        assert_eq!(
            draft.object_manifest.get(21).unwrap().placement_tid,
            tid(90, 2)
        );
        let root_control = draft.root_control_state(manifest_locators()).unwrap();
        assert_eq!(root_control.active_epoch, 8);
        assert_eq!(root_control.next_pid, 30);
        assert_eq!(root_control.next_local_vec_seq, 5);
        let encoded = draft.encode_publish_bundle(manifest_locators()).unwrap();
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
    }

    #[test]
    fn replacement_leaf_object_inputs_match_replacement_children() {
        let children = vec![
            replacement_child(21, vec![0.5, 0.5]),
            replacement_child(22, vec![-0.5, 0.5]),
        ];
        let inputs = vec![
            SpireReplacementLeafObjectInput {
                pid: 21,
                rows: vec![primary_row(1, 10, 1)],
            },
            SpireReplacementLeafObjectInput {
                pid: 22,
                rows: vec![primary_row(2, 10, 2)],
            },
        ];

        validate_replacement_leaf_object_inputs(&children, &inputs).unwrap();
    }

    #[test]
    fn replacement_leaf_object_inputs_reject_delta_flags_and_pid_mismatch() {
        let children = vec![replacement_child(21, vec![0.5, 0.5])];
        let with_delta = vec![SpireReplacementLeafObjectInput {
            pid: 21,
            rows: vec![delta_insert_row(1, 10, 1)],
        }];
        assert!(
            validate_replacement_leaf_object_inputs(&children, &with_delta)
                .unwrap_err()
                .contains("delta-insert")
        );

        let wrong_pid = vec![SpireReplacementLeafObjectInput {
            pid: 22,
            rows: vec![primary_row(1, 10, 1)],
        }];
        assert!(
            validate_replacement_leaf_object_inputs(&children, &wrong_pid)
                .unwrap_err()
                .contains("no replacement routing child")
        );
    }

    #[test]
    fn local_replacement_object_writer_persists_routing_and_leaf_objects() {
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let root = root_routing_object();
        let children = vec![
            replacement_child(21, vec![0.5, 0.5]),
            replacement_child(22, vec![-0.5, 0.5]),
        ];
        let replacement_root =
            rewrite_routing_partition_for_leaf_replacement(&root, &[12], children.clone(), 4)
                .unwrap();

        let placements = write_local_replacement_objects(
            8,
            &replacement_root,
            &children,
            1,
            vec![
                SpireReplacementLeafObjectInput {
                    pid: 22,
                    rows: vec![primary_row(2, 20, 2)],
                },
                SpireReplacementLeafObjectInput {
                    pid: 21,
                    rows: vec![primary_row(1, 20, 1)],
                },
            ],
            &mut object_store,
        )
        .unwrap();

        let stored_root = object_store
            .read_routing_object(&placements.parent_placement)
            .unwrap();
        let stored_root_children = stored_root.children().collect::<Vec<_>>();
        assert_eq!(placements.parent_placement.epoch, 8);
        assert_eq!(placements.parent_placement.pid, replacement_root.header.pid);
        assert_eq!(stored_root.header.published_epoch_backref, 8);
        assert_eq!(
            stored_root_children
                .iter()
                .map(|child| child.child_pid)
                .collect::<Vec<_>>(),
            vec![11, 21, 22, 13]
        );

        assert_eq!(
            placements
                .leaf_placements
                .iter()
                .map(|placement| placement.pid)
                .collect::<Vec<_>>(),
            vec![21, 22]
        );
        let first_leaf = object_store
            .read_leaf_object_v2(&placements.leaf_placements[0])
            .unwrap();
        let second_leaf = object_store
            .read_leaf_object_v2(&placements.leaf_placements[1])
            .unwrap();
        assert_eq!(
            first_leaf.meta.header.parent_pid,
            replacement_root.header.pid
        );
        assert_eq!(
            second_leaf.meta.header.parent_pid,
            replacement_root.header.pid
        );
        assert_eq!(
            first_leaf.assignment_rows().unwrap()[0].heap_tid,
            tid(20, 1)
        );
        assert_eq!(
            second_leaf.assignment_rows().unwrap()[0].heap_tid,
            tid(20, 2)
        );
    }

    #[test]
    fn local_scheduled_replacement_object_writer_persists_decision_bound_objects() {
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let root = root_routing_object();
        let decision = SpireLeafReplacementScheduleDecision {
            mode: SpireLeafReplacementScheduleMode::Split,
            active_epoch: 7,
            replaced_parent_pid: root.header.pid,
            affected_leaf_pids: vec![12],
            replacement_leaf_count: 2,
            reason: "test_split",
        };
        let children = vec![
            replacement_child(21, vec![0.5, 0.5]),
            replacement_child(22, vec![-0.5, 0.5]),
        ];
        let replacement_root =
            rewrite_scheduled_replacement_parent_routing(&root, &decision, children.clone(), 4)
                .unwrap();

        let placements = write_local_scheduled_replacement_objects(
            8,
            &replacement_root,
            &decision,
            &children,
            1,
            vec![
                SpireReplacementLeafObjectInput {
                    pid: 22,
                    rows: vec![primary_row(2, 20, 2)],
                },
                SpireReplacementLeafObjectInput {
                    pid: 21,
                    rows: vec![primary_row(1, 20, 1)],
                },
            ],
            &mut object_store,
        )
        .unwrap();

        assert_eq!(placements.parent_placement.pid, root.header.pid);
        assert_eq!(
            placements
                .leaf_placements
                .iter()
                .map(|placement| placement.pid)
                .collect::<Vec<_>>(),
            vec![21, 22]
        );
        let stored_root = object_store
            .read_routing_object(&placements.parent_placement)
            .unwrap();
        assert_eq!(
            stored_root
                .children()
                .map(|child| child.child_pid)
                .collect::<Vec<_>>(),
            vec![11, 21, 22, 13]
        );
    }

    #[test]
    fn local_scheduled_replacement_object_writer_rejects_parent_or_child_count_mismatch() {
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let root = root_routing_object();
        let decision = SpireLeafReplacementScheduleDecision {
            mode: SpireLeafReplacementScheduleMode::Split,
            active_epoch: 7,
            replaced_parent_pid: root.header.pid,
            affected_leaf_pids: vec![12],
            replacement_leaf_count: 2,
            reason: "test_split",
        };
        let wrong_parent_decision = SpireLeafReplacementScheduleDecision {
            replaced_parent_pid: 2,
            ..decision.clone()
        };
        let children = vec![
            replacement_child(21, vec![0.5, 0.5]),
            replacement_child(22, vec![-0.5, 0.5]),
        ];
        let replacement_root =
            rewrite_scheduled_replacement_parent_routing(&root, &decision, children.clone(), 4)
                .unwrap();

        assert!(write_local_scheduled_replacement_objects(
            9,
            &replacement_root,
            &decision,
            &children,
            1,
            Vec::new(),
            &mut object_store,
        )
        .unwrap_err()
        .contains("immediate successor"));

        assert!(write_local_scheduled_replacement_objects(
            8,
            &replacement_root,
            &wrong_parent_decision,
            &children,
            1,
            Vec::new(),
            &mut object_store,
        )
        .unwrap_err()
        .contains("does not match decision parent pid"));

        assert!(write_local_scheduled_replacement_objects(
            8,
            &replacement_root,
            &decision,
            &children[..1],
            1,
            Vec::new(),
            &mut object_store,
        )
        .unwrap_err()
        .contains("child count"));
    }

    #[test]
    fn replacement_epoch_draft_from_object_placements_builds_directory_and_manifest() {
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let active_epoch = 7;
        let new_epoch = 8;
        let root = root_routing_object();
        let root_placement = object_store
            .insert_routing_object(active_epoch, &root)
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
        let delta =
            SpireDeltaPartitionObject::new(40, 1, 12, vec![delta_insert_row(4, 20, 1)]).unwrap();
        let delta_placement = object_store
            .insert_delta_object(active_epoch, &delta)
            .unwrap();
        let active_placements = vec![root_placement, leaf_11, leaf_12, leaf_13, delta_placement];
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
            active_placements
                .iter()
                .map(manifest_entry_for)
                .collect::<Vec<_>>(),
        )
        .unwrap();
        let placement_directory =
            SpirePlacementDirectory::from_entries(active_epoch, active_placements).unwrap();
        let snapshot = SpirePublishedEpochSnapshot::new(
            &epoch_manifest,
            &object_manifest,
            &placement_directory,
        )
        .unwrap();
        let children = vec![
            replacement_child(21, vec![0.5, 0.5]),
            replacement_child(22, vec![-0.5, 0.5]),
        ];
        let replacement_root =
            rewrite_routing_partition_for_leaf_replacement(&root, &[12], children.clone(), 4)
                .unwrap();
        let replacement_object_placements = write_local_replacement_objects(
            new_epoch,
            &replacement_root,
            &children,
            2,
            vec![
                SpireReplacementLeafObjectInput {
                    pid: 21,
                    rows: vec![primary_row(5, 30, 1)],
                },
                SpireReplacementLeafObjectInput {
                    pid: 22,
                    rows: vec![primary_row(6, 30, 2)],
                },
            ],
            &mut object_store,
        )
        .unwrap();

        let draft = build_replacement_epoch_draft_from_object_placements(
            &snapshot,
            &object_store,
            SpireReplacementEpochObjectPlacementInput {
                epoch: new_epoch,
                published_at_micros: 3000,
                retain_until_micros: 4000,
                consistency_mode: SpireConsistencyMode::Strict,
                replaced_parent_pid: root.header.pid,
                affected_leaf_pids: vec![12],
                replacement_object_placements,
                placement_write_evidence: placement_write_evidence_for_pids(&[1, 11, 13, 21, 22]),
                next_pid: 30,
                next_local_vec_seq: 7,
            },
        )
        .unwrap();

        let active_pids = draft
            .placement_directory
            .entries
            .iter()
            .map(|entry| entry.pid)
            .collect::<Vec<_>>();
        assert_eq!(active_pids, vec![1, 11, 13, 21, 22]);
        assert!(draft.placement_directory.get(12).is_none());
        assert!(draft.placement_directory.get(40).is_none());
        assert_eq!(
            draft.object_manifest.get(21).unwrap().placement_tid,
            tid(90, 4)
        );
        assert_eq!(
            draft.object_manifest.get(22).unwrap().placement_tid,
            tid(90, 5)
        );
        let root_control = draft.root_control_state(manifest_locators()).unwrap();
        assert_eq!(root_control.active_epoch, new_epoch);
        assert_eq!(root_control.next_pid, 30);
        assert_eq!(root_control.next_local_vec_seq, 7);
        let stored_root = object_store
            .read_routing_object(draft.placement_directory.get(1).unwrap())
            .unwrap();
        assert_eq!(
            stored_root
                .children()
                .map(|child| child.child_pid)
                .collect::<Vec<_>>(),
            vec![11, 21, 22, 13]
        );
    }

    #[test]
    fn scheduled_replacement_epoch_draft_uses_decision_shape_for_publish_assembly() {
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let active_epoch = 7;
        let new_epoch = 8;
        let root = root_routing_object();
        let root_placement = object_store
            .insert_routing_object(active_epoch, &root)
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
        let active_placements = vec![root_placement, leaf_11, leaf_12, leaf_13];
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
            active_placements
                .iter()
                .map(manifest_entry_for)
                .collect::<Vec<_>>(),
        )
        .unwrap();
        let placement_directory =
            SpirePlacementDirectory::from_entries(active_epoch, active_placements).unwrap();
        let snapshot = SpirePublishedEpochSnapshot::new(
            &epoch_manifest,
            &object_manifest,
            &placement_directory,
        )
        .unwrap();
        let decision = SpireLeafReplacementScheduleDecision {
            mode: SpireLeafReplacementScheduleMode::Split,
            active_epoch,
            replaced_parent_pid: root.header.pid,
            affected_leaf_pids: vec![12],
            replacement_leaf_count: 2,
            reason: "test_split",
        };
        let children = vec![
            replacement_child(21, vec![0.5, 0.5]),
            replacement_child(22, vec![-0.5, 0.5]),
        ];
        let replacement_root =
            rewrite_scheduled_replacement_parent_routing(&root, &decision, children.clone(), 4)
                .unwrap();
        let replacement_object_placements = write_local_replacement_objects(
            new_epoch,
            &replacement_root,
            &children,
            2,
            vec![
                SpireReplacementLeafObjectInput {
                    pid: 21,
                    rows: vec![primary_row(5, 30, 1)],
                },
                SpireReplacementLeafObjectInput {
                    pid: 22,
                    rows: vec![primary_row(6, 30, 2)],
                },
            ],
            &mut object_store,
        )
        .unwrap();

        let draft = build_scheduled_replacement_epoch_draft_from_object_placements(
            &snapshot,
            &object_store,
            &decision,
            SpireScheduledReplacementEpochObjectPlacementInput {
                epoch: new_epoch,
                published_at_micros: 3000,
                retain_until_micros: 4000,
                consistency_mode: SpireConsistencyMode::Strict,
                replacement_object_placements,
                placement_write_evidence: placement_write_evidence_for_pids(&[1, 11, 13, 21, 22]),
                next_pid: 30,
                next_local_vec_seq: 7,
            },
        )
        .unwrap();

        assert_eq!(
            draft
                .placement_directory
                .entries
                .iter()
                .map(|entry| entry.pid)
                .collect::<Vec<_>>(),
            vec![1, 11, 13, 21, 22]
        );
        assert!(draft.placement_directory.get(12).is_none());
        assert_eq!(
            draft.object_manifest.get(21).unwrap().placement_tid,
            tid(90, 4)
        );
        assert_eq!(
            draft.object_manifest.get(22).unwrap().placement_tid,
            tid(90, 5)
        );
    }

    #[test]
    fn scheduled_replacement_epoch_draft_rejects_epoch_or_placement_count_mismatch() {
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let active_epoch = 7;
        let root = root_routing_object();
        let root_placement = object_store
            .insert_routing_object(active_epoch, &root)
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
        let active_placements = vec![root_placement, leaf_12];
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
            active_placements
                .iter()
                .map(manifest_entry_for)
                .collect::<Vec<_>>(),
        )
        .unwrap();
        let placement_directory =
            SpirePlacementDirectory::from_entries(active_epoch, active_placements).unwrap();
        let snapshot = SpirePublishedEpochSnapshot::new(
            &epoch_manifest,
            &object_manifest,
            &placement_directory,
        )
        .unwrap();
        let decision = SpireLeafReplacementScheduleDecision {
            mode: SpireLeafReplacementScheduleMode::Split,
            active_epoch,
            replaced_parent_pid: root.header.pid,
            affected_leaf_pids: vec![12],
            replacement_leaf_count: 2,
            reason: "test_split",
        };
        let replacement_parent_placement =
            SpirePlacementEntry::local_single_store_available(8, 1, 12345, 4, tid(40, 1), 128);
        let replacement_leaf_21 =
            SpirePlacementEntry::local_single_store_available(8, 21, 12345, 2, tid(41, 1), 256);
        let replacement_leaf_22 =
            SpirePlacementEntry::local_single_store_available(8, 22, 12345, 2, tid(42, 1), 256);
        let placements = super::SpireReplacementObjectPlacements {
            parent_placement: replacement_parent_placement,
            leaf_placements: vec![replacement_leaf_21, replacement_leaf_22],
        };
        let wrong_epoch_decision = SpireLeafReplacementScheduleDecision {
            active_epoch: active_epoch + 1,
            ..decision.clone()
        };

        assert!(
            build_scheduled_replacement_epoch_draft_from_object_placements(
                &snapshot,
                &object_store,
                &wrong_epoch_decision,
                SpireScheduledReplacementEpochObjectPlacementInput {
                    epoch: 8,
                    published_at_micros: 3000,
                    retain_until_micros: 4000,
                    consistency_mode: SpireConsistencyMode::Strict,
                    replacement_object_placements: placements.clone(),
                    placement_write_evidence: placement_write_evidence_for_pids(&[1, 21, 22]),
                    next_pid: 30,
                    next_local_vec_seq: 7,
                },
            )
            .unwrap_err()
            .contains("snapshot epoch")
        );

        assert!(
            build_scheduled_replacement_epoch_draft_from_object_placements(
                &snapshot,
                &object_store,
                &decision,
                SpireScheduledReplacementEpochObjectPlacementInput {
                    epoch: 9,
                    published_at_micros: 3000,
                    retain_until_micros: 4000,
                    consistency_mode: SpireConsistencyMode::Strict,
                    replacement_object_placements: placements.clone(),
                    placement_write_evidence: placement_write_evidence_for_pids(&[1, 21, 22]),
                    next_pid: 30,
                    next_local_vec_seq: 7,
                },
            )
            .unwrap_err()
            .contains("immediate successor")
        );

        assert!(
            build_scheduled_replacement_epoch_draft_from_object_placements(
                &snapshot,
                &object_store,
                &decision,
                SpireScheduledReplacementEpochObjectPlacementInput {
                    epoch: 8,
                    published_at_micros: 3000,
                    retain_until_micros: 4000,
                    consistency_mode: SpireConsistencyMode::Degraded,
                    replacement_object_placements: placements.clone(),
                    placement_write_evidence: placement_write_evidence_for_pids(&[1, 21, 22]),
                    next_pid: 30,
                    next_local_vec_seq: 7,
                },
            )
            .unwrap_err()
            .contains("consistency mode")
        );

        let mut missing_leaf_placement = placements;
        missing_leaf_placement.leaf_placements.pop();
        assert!(
            build_scheduled_replacement_epoch_draft_from_object_placements(
                &snapshot,
                &object_store,
                &decision,
                SpireScheduledReplacementEpochObjectPlacementInput {
                    epoch: 8,
                    published_at_micros: 3000,
                    retain_until_micros: 4000,
                    consistency_mode: SpireConsistencyMode::Strict,
                    replacement_object_placements: missing_leaf_placement,
                    placement_write_evidence: placement_write_evidence_for_pids(&[1, 21]),
                    next_pid: 30,
                    next_local_vec_seq: 7,
                },
            )
            .unwrap_err()
            .contains("leaf placement count")
        );
    }

    #[test]
    fn scheduled_replacement_pid_plan_output_accepts_matching_placements_and_cursor() {
        let decision = SpireLeafReplacementScheduleDecision {
            mode: SpireLeafReplacementScheduleMode::Split,
            active_epoch: 7,
            replaced_parent_pid: 1,
            affected_leaf_pids: vec![12],
            replacement_leaf_count: 2,
            reason: "test_split",
        };
        let pid_plan = SpireLeafReplacementPidPlan {
            replacement_pids: vec![21, 22],
            reuses_existing_pid: false,
            next_pid: 23,
        };
        let placements = SpireReplacementObjectPlacements {
            parent_placement: SpirePlacementEntry::local_single_store_available(
                8,
                1,
                12345,
                4,
                tid(40, 1),
                128,
            ),
            leaf_placements: vec![
                SpirePlacementEntry::local_single_store_available(8, 21, 12345, 2, tid(41, 1), 256),
                SpirePlacementEntry::local_single_store_available(8, 22, 12345, 2, tid(42, 1), 256),
            ],
        };

        validate_scheduled_replacement_pid_plan_output(&decision, &pid_plan, &placements, 23)
            .unwrap();
    }

    #[test]
    fn scheduled_replacement_pid_plan_output_rejects_mismatched_outputs() {
        let decision = SpireLeafReplacementScheduleDecision {
            mode: SpireLeafReplacementScheduleMode::Split,
            active_epoch: 7,
            replaced_parent_pid: 1,
            affected_leaf_pids: vec![12],
            replacement_leaf_count: 2,
            reason: "test_split",
        };
        let pid_plan = SpireLeafReplacementPidPlan {
            replacement_pids: vec![21, 22],
            reuses_existing_pid: false,
            next_pid: 23,
        };
        let mut placements = SpireReplacementObjectPlacements {
            parent_placement: SpirePlacementEntry::local_single_store_available(
                8,
                1,
                12345,
                4,
                tid(40, 1),
                128,
            ),
            leaf_placements: vec![
                SpirePlacementEntry::local_single_store_available(8, 22, 12345, 2, tid(42, 1), 256),
                SpirePlacementEntry::local_single_store_available(8, 21, 12345, 2, tid(41, 1), 256),
            ],
        };

        assert!(validate_scheduled_replacement_pid_plan_output(
            &decision,
            &pid_plan,
            &placements,
            23
        )
        .unwrap_err()
        .contains("do not match pid plan"));

        placements.parent_placement.pid = 2;
        placements.leaf_placements.swap(0, 1);
        assert!(validate_scheduled_replacement_pid_plan_output(
            &decision,
            &pid_plan,
            &placements,
            23
        )
        .unwrap_err()
        .contains("parent placement pid"));

        placements.parent_placement.pid = 1;
        assert!(validate_scheduled_replacement_pid_plan_output(
            &decision,
            &pid_plan,
            &placements,
            24
        )
        .unwrap_err()
        .contains("next_pid"));
    }

