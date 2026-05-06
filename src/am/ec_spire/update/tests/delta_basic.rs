    #[test]
    fn delta_epoch_draft_writes_delta_object_and_published_snapshot() {
        let mut pid_allocator = SpirePidAllocator::new(50).unwrap();
        let mut local_vec_id_allocator = SpireLocalVecIdAllocator::default();
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();

        let draft = build_delta_epoch_draft(
            delta_input(
                vec![insert_assignment(20, 1), insert_assignment(20, 2)],
                vec![delete_assignment(99, 21, 1)],
            ),
            &mut pid_allocator,
            &mut local_vec_id_allocator,
            &mut object_store,
        )
        .unwrap();

        let placement = draft.placement_directory.get(50).unwrap();
        let stored_delta = object_store.read_delta_object(placement).unwrap();
        let mut expected_delta = draft.delta_object.clone();
        expected_delta.header.published_epoch_backref = draft.epoch_manifest.epoch;

        assert_eq!(stored_delta, expected_delta);
        assert_eq!(draft.epoch_manifest.epoch, 8);
        assert_eq!(draft.delta_object.header.pid, 50);
        assert_eq!(draft.delta_object.header.object_version, 3);
        assert_eq!(draft.delta_object.header.parent_pid, 11);
        assert_eq!(draft.delta_object.assignments.len(), 3);
        assert_eq!(
            draft.delta_object.assignments[0].flags,
            SPIRE_ASSIGNMENT_FLAG_PRIMARY | SPIRE_ASSIGNMENT_FLAG_DELTA_INSERT
        );
        assert_eq!(
            draft.delta_object.assignments[2].flags,
            SPIRE_ASSIGNMENT_FLAG_TOMBSTONE | SPIRE_ASSIGNMENT_FLAG_DELTA_DELETE
        );
        assert_eq!(
            draft.object_manifest.get(50).unwrap().placement_tid,
            tid(80, 1)
        );
        SpirePublishedEpochSnapshot::new(
            &draft.epoch_manifest,
            &draft.object_manifest,
            &draft.placement_directory,
        )
        .unwrap();
        assert_eq!(draft.next_pid, 51);
        assert_eq!(draft.next_local_vec_seq, 3);
        assert_eq!(pid_allocator.next_pid(), draft.next_pid);
        assert_eq!(
            local_vec_id_allocator.next_local_vec_seq(),
            draft.next_local_vec_seq
        );
    }

    #[test]
    fn delta_epoch_draft_encodes_publish_bundle() {
        let mut pid_allocator = SpirePidAllocator::new(50).unwrap();
        let mut local_vec_id_allocator = SpireLocalVecIdAllocator::default();
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let draft = build_delta_epoch_draft(
            delta_input(
                vec![insert_assignment(20, 1)],
                vec![delete_assignment(99, 21, 1)],
            ),
            &mut pid_allocator,
            &mut local_vec_id_allocator,
            &mut object_store,
        )
        .unwrap();

        let encoded = draft.encode_publish_bundle(manifest_locators()).unwrap();
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
        assert_eq!(root_control.next_local_vec_seq, draft.next_local_vec_seq);
        assert_eq!(root_control.epoch_manifest_tid, tid(90, 1));
        assert_eq!(root_control.object_manifest_tid, tid(90, 2));
        assert_eq!(root_control.placement_directory_tid, tid(90, 3));
    }

    #[test]
    fn delta_epoch_draft_rejects_invalid_root_control_locator() {
        let mut pid_allocator = SpirePidAllocator::new(50).unwrap();
        let mut local_vec_id_allocator = SpireLocalVecIdAllocator::default();
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let draft = build_delta_epoch_draft(
            delta_input(vec![insert_assignment(20, 1)], Vec::new()),
            &mut pid_allocator,
            &mut local_vec_id_allocator,
            &mut object_store,
        )
        .unwrap();
        let mut locators = manifest_locators();
        locators.placement_directory_tid = ItemPointer::INVALID;

        assert!(draft.root_control_state(locators).is_err());
    }

