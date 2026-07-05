    #[test]
    fn single_level_draft_builds_leaf_object_and_published_snapshot() {
        let (draft, pid_allocator, local_vec_id_allocator, object_store) = build_valid_draft();

        let placement = draft.placement_directory.get(SPIRE_FIRST_PID).unwrap();
        let stored_leaf = object_store.read_leaf_object_v2(placement).unwrap();

        assert_eq!(draft.epoch_manifest.epoch, 7);
        assert_eq!(draft.leaf_object.header.pid, SPIRE_FIRST_PID);
        assert_eq!(draft.leaf_object.header.object_version, 1);
        assert_eq!(draft.leaf_object.assignments.len(), 2);
        assert_eq!(stored_leaf.meta.header.pid, draft.leaf_object.header.pid);
        assert_eq!(
            stored_leaf.meta.header.object_version,
            draft.leaf_object.header.object_version
        );
        assert_eq!(
            stored_leaf.meta.header.assignment_count,
            draft.leaf_object.assignments.len() as u32
        );
        assert_eq!(
            draft
                .object_manifest
                .get(SPIRE_FIRST_PID)
                .unwrap()
                .placement_tid,
            tid(60, 1)
        );
        SpirePublishedEpochSnapshot::new(
            &draft.epoch_manifest,
            &draft.object_manifest,
            &draft.placement_directory,
        )
        .unwrap();
        assert_eq!(draft.next_pid, SPIRE_FIRST_PID + 1);
        assert_eq!(draft.next_local_vec_seq, SPIRE_FIRST_LOCAL_VEC_SEQ + 2);
        assert_eq!(pid_allocator.next_pid(), draft.next_pid);
        assert_eq!(
            local_vec_id_allocator.next_local_vec_seq(),
            draft.next_local_vec_seq
        );
    }

    #[test]
    fn single_level_draft_builds_root_control_state_from_manifest_locators() {
        let (draft, _, _, _) = build_valid_draft();
        let root_control = draft.root_control_state(manifest_locators()).unwrap();

        assert_eq!(root_control.active_epoch, draft.epoch_manifest.epoch);
        assert_eq!(root_control.next_pid, draft.next_pid);
        assert_eq!(root_control.next_local_vec_seq, draft.next_local_vec_seq);
        assert_eq!(root_control.epoch_manifest_tid, tid(70, 1));
        assert_eq!(root_control.object_manifest_tid, tid(70, 2));
        assert_eq!(root_control.placement_directory_tid, tid(70, 3));
        assert_eq!(root_control.local_store_config_tid, tid(70, 4));
    }

    #[test]
    fn single_level_draft_rejects_invalid_root_control_manifest_locator() {
        let (draft, _, _, _) = build_valid_draft();
        let mut locators = manifest_locators();
        locators.object_manifest_tid = ItemPointer::INVALID;

        assert!(draft.root_control_state(locators).is_err());
    }

    #[test]
    fn publish_coordinator_validates_before_active_epoch_publish() {
        let (mut draft, _, _, _) = build_valid_draft();
        draft.placement_directory.entries[0].object_version = 99;

        let error = draft
            .encode_publish_bundle(manifest_locators())
            .unwrap_err();

        assert!(error.contains("Validating failed"));
        assert!(error.contains("object_version mismatch"));
    }

    #[test]
    fn publish_coordinator_rejects_missing_object_write_evidence() {
        let (draft, _, _, _) = build_valid_draft();
        let error = SpirePublishWritingObjects::new(draft.publish_input())
            .objects_written(&[])
            .unwrap_err();

        assert_eq!(error.stage, SpirePublishStage::WritingObjects);
        assert!(error.error.contains("object write evidence count mismatch"));
    }

    #[test]
    fn publish_coordinator_rejects_mismatched_placement_write_evidence() {
        let (draft, _, _, _) = build_valid_draft();
        let object_evidence =
            object_write_evidence_from_placement_directory(&draft.placement_directory);
        let mut placement_evidence =
            placement_write_evidence_from_object_manifest(&draft.object_manifest);
        placement_evidence[0].placement_tid = tid(99, 9);

        let error = SpirePublishWritingObjects::new(draft.publish_input())
            .objects_written(&object_evidence)
            .unwrap()
            .placements_written(&placement_evidence)
            .unwrap_err();

        assert_eq!(error.stage, SpirePublishStage::WritingPlacements);
        assert!(error.error.contains("placement_tid mismatch"));
    }

    #[test]
    fn object_manifest_from_placement_writes_uses_durable_placement_tids() {
        let (draft, _, _, _) = build_valid_draft();
        let evidence = vec![SpirePublishPlacementWriteEvidence {
            pid: SPIRE_FIRST_PID,
            placement_tid: tid(90, 7),
        }];

        let manifest = object_manifest_from_placement_writes(
            draft.epoch_manifest.epoch,
            &draft.placement_directory,
            &evidence,
        )
        .unwrap();

        let entry = manifest.get(SPIRE_FIRST_PID).unwrap();
        assert_eq!(
            entry.object_version,
            draft.leaf_object.header.object_version
        );
        assert_eq!(entry.placement_tid, tid(90, 7));
    }

    #[test]
    fn object_manifest_from_placement_writes_rejects_missing_or_duplicate_evidence() {
        let (draft, _, _, _) = build_valid_draft();

        assert!(object_manifest_from_placement_writes(
            draft.epoch_manifest.epoch,
            &draft.placement_directory,
            &[],
        )
        .unwrap_err()
        .contains("count mismatch"));

        let duplicate = vec![
            SpirePublishPlacementWriteEvidence {
                pid: SPIRE_FIRST_PID,
                placement_tid: tid(90, 7),
            },
            SpirePublishPlacementWriteEvidence {
                pid: SPIRE_FIRST_PID,
                placement_tid: tid(90, 8),
            },
        ];
        let mut duplicate_directory = draft.placement_directory.clone();
        duplicate_directory
            .entries
            .push(duplicate_directory.entries[0]);
        assert!(object_manifest_from_placement_writes(
            draft.epoch_manifest.epoch,
            &duplicate_directory,
            &duplicate,
        )
        .unwrap_err()
        .contains("duplicate pid"));
    }

    #[test]
    fn single_level_draft_encodes_manifest_bundle() {
        let (draft, _, _, _) = build_valid_draft();

        let encoded = draft.encode_manifest_bundle().unwrap();

        assert_eq!(
            SpireEpochManifest::decode(&encoded.epoch_manifest).unwrap(),
            draft.epoch_manifest
        );
        assert_eq!(
            SpireObjectManifest::decode(&encoded.object_manifest).unwrap(),
            draft.object_manifest
        );
        assert_eq!(
            SpirePlacementDirectory::decode(&encoded.placement_directory).unwrap(),
            draft.placement_directory
        );
        assert_eq!(
            SpireLocalStoreConfig::decode(&encoded.local_store_config).unwrap(),
            SpireLocalStoreConfig::from_placement_directory(
                draft.epoch_manifest.epoch,
                &draft.placement_directory
            )
            .unwrap()
        );
    }

    #[test]
    fn single_level_draft_encodes_publish_bundle() {
        let (draft, _, _, _) = build_valid_draft();

        let encoded = draft.encode_publish_bundle(manifest_locators()).unwrap();
        let root_control = SpireRootControlState::decode(&encoded.root_control_state).unwrap();

        assert_eq!(
            SpireEpochManifest::decode(&encoded.manifests.epoch_manifest).unwrap(),
            draft.epoch_manifest
        );
        assert_eq!(root_control.active_epoch, draft.epoch_manifest.epoch);
        assert_eq!(root_control.next_pid, draft.next_pid);
        assert_eq!(root_control.next_local_vec_seq, draft.next_local_vec_seq);
        assert_eq!(root_control.epoch_manifest_tid, tid(70, 1));
        assert_eq!(root_control.object_manifest_tid, tid(70, 2));
        assert_eq!(root_control.placement_directory_tid, tid(70, 3));
        assert_eq!(root_control.local_store_config_tid, tid(70, 4));
    }

    #[test]
    fn retired_epoch_manifest_requires_published_input() {
        let retired_manifest = SpireEpochManifest {
            epoch: 7,
            state: SpireEpochState::Retired,
            consistency_mode: SpireConsistencyMode::Strict,
            published_at_micros: 1000,
            retain_until_micros: 2000,
            active_query_count: 0,
        };

        let error = retired_epoch_manifest_from(retired_manifest)
            .expect_err("retiring an already-retired manifest should fail");

        assert_eq!(
            error,
            "ec_spire can only retire a previously published epoch manifest"
        );
    }
