    #[test]
    fn delta_epoch_draft_from_snapshot_carries_base_entries() {
        let mut pid_allocator = SpirePidAllocator::default();
        let mut local_vec_id_allocator = SpireLocalVecIdAllocator::default();
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let base_draft = build_single_level_leaf_epoch_draft(
            base_build_input(vec![insert_assignment(10, 1)]),
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

        let mut input = delta_input(
            vec![insert_assignment(20, 1)],
            vec![delete_assignment(1, 10, 1)],
        );
        input.base_pid = 1;
        let draft = build_delta_epoch_draft_from_snapshot(
            input,
            &base_snapshot,
            &mut pid_allocator,
            &mut local_vec_id_allocator,
            &mut object_store,
        )
        .unwrap();

        let base_entry = draft.object_manifest.get(1).unwrap();
        let delta_entry = draft.object_manifest.get(2).unwrap();
        let base_placement = draft.placement_directory.get(1).unwrap();
        let delta_placement = draft.placement_directory.get(2).unwrap();

        assert_eq!(draft.object_manifest.entries.len(), 2);
        assert_eq!(draft.placement_directory.entries.len(), 2);
        assert_eq!(base_entry.epoch, 8);
        assert_eq!(base_entry.object_version, 1);
        assert_eq!(base_entry.placement_tid, tid(70, 1));
        assert_eq!(delta_entry.object_version, 3);
        assert_eq!(base_placement.epoch, 8);
        assert_eq!(base_placement.object_version, 1);
        assert_eq!(delta_placement.epoch, 8);
        assert_eq!(delta_placement.object_version, 3);
        assert_eq!(
            object_store
                .read_leaf_object_v2(base_placement)
                .unwrap()
                .meta
                .header
                .pid,
            1
        );
        assert_eq!(
            object_store
                .read_delta_object(delta_placement)
                .unwrap()
                .header
                .pid,
            2
        );
        SpirePublishedEpochSnapshot::new(
            &draft.epoch_manifest,
            &draft.object_manifest,
            &draft.placement_directory,
        )
        .unwrap();
        assert_eq!(draft.next_pid, 3);
        assert_eq!(draft.next_local_vec_seq, 3);
        assert_eq!(pid_allocator.next_pid(), 3);
        assert_eq!(local_vec_id_allocator.next_local_vec_seq(), 3);
    }

    #[test]
    fn replacement_leaf_rows_fold_active_deltas_into_base_leaf_rows() {
        let mut pid_allocator = SpirePidAllocator::default();
        let mut local_vec_id_allocator = SpireLocalVecIdAllocator::default();
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let base_draft = build_single_level_leaf_epoch_draft(
            base_build_input(vec![insert_assignment(10, 1)]),
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
        let mut input = delta_input(
            vec![insert_assignment(20, 1)],
            vec![delete_assignment(1, 10, 1)],
        );
        input.base_pid = 1;
        let delta_draft = build_delta_epoch_draft_from_snapshot(
            input,
            &base_snapshot,
            &mut pid_allocator,
            &mut local_vec_id_allocator,
            &mut object_store,
        )
        .unwrap();
        let delta_snapshot = SpirePublishedEpochSnapshot::new(
            &delta_draft.epoch_manifest,
            &delta_draft.object_manifest,
            &delta_draft.placement_directory,
        )
        .unwrap();

        let folded = collect_replacement_leaf_rows(&delta_snapshot, &object_store, &[1]).unwrap();

        assert_eq!(folded.len(), 1);
        assert_eq!(folded[0].base_pid, 1);
        assert_eq!(folded[0].rows.len(), 1);
        assert_eq!(folded[0].rows[0].heap_tid, tid(20, 1));
        assert_eq!(folded[0].rows[0].flags, SPIRE_ASSIGNMENT_FLAG_PRIMARY);
    }

    #[test]
    fn delta_epoch_draft_from_snapshot_observes_existing_vec_ids_before_allocating() {
        let mut pid_allocator = SpirePidAllocator::default();
        let mut local_vec_id_allocator = SpireLocalVecIdAllocator::default();
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let base_draft = build_single_level_leaf_epoch_draft(
            base_build_input(vec![insert_assignment(10, 1)]),
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
        let mut stale_pid_allocator = SpirePidAllocator::default();
        let mut stale_local_vec_id_allocator = SpireLocalVecIdAllocator::default();
        let mut input = delta_input(vec![insert_assignment(20, 1)], Vec::new());
        input.base_pid = 1;

        let draft = build_delta_epoch_draft_from_snapshot(
            input,
            &base_snapshot,
            &mut stale_pid_allocator,
            &mut stale_local_vec_id_allocator,
            &mut object_store,
        )
        .unwrap();

        assert_eq!(draft.delta_object.header.pid, 2);
        assert_eq!(
            draft.delta_object.assignments[0].vec_id.local_sequence(),
            Some(2)
        );
        assert_eq!(draft.next_pid, 3);
        assert_eq!(draft.next_local_vec_seq, 3);
        assert_eq!(stale_pid_allocator.next_pid(), 3);
        assert_eq!(stale_local_vec_id_allocator.next_local_vec_seq(), 3);
    }

    #[test]
    fn delta_epoch_draft_from_snapshot_rejects_missing_base_pid() {
        let mut pid_allocator = SpirePidAllocator::default();
        let mut local_vec_id_allocator = SpireLocalVecIdAllocator::default();
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let base_draft = build_single_level_leaf_epoch_draft(
            base_build_input(vec![insert_assignment(10, 1)]),
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
        let initial_page_count = object_store.page_count();
        let mut input = delta_input(vec![insert_assignment(20, 1)], Vec::new());
        input.base_pid = 99;

        assert!(build_delta_epoch_draft_from_snapshot(
            input,
            &base_snapshot,
            &mut pid_allocator,
            &mut local_vec_id_allocator,
            &mut object_store,
        )
        .is_err());
        assert_eq!(pid_allocator.next_pid(), 2);
        assert_eq!(local_vec_id_allocator.next_local_vec_seq(), 2);
        assert_eq!(object_store.page_count(), initial_page_count);
    }

    #[test]
    fn delta_epoch_draft_from_snapshot_rejects_unknown_delete_vec_id() {
        let mut pid_allocator = SpirePidAllocator::default();
        let mut local_vec_id_allocator = SpireLocalVecIdAllocator::default();
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let base_draft = build_single_level_leaf_epoch_draft(
            base_build_input(vec![insert_assignment(10, 1)]),
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
        let initial_page_count = object_store.page_count();
        let mut input = delta_input(
            vec![insert_assignment(20, 1)],
            vec![delete_assignment(99, 10, 1)],
        );
        input.base_pid = 1;

        assert!(build_delta_epoch_draft_from_snapshot(
            input,
            &base_snapshot,
            &mut pid_allocator,
            &mut local_vec_id_allocator,
            &mut object_store,
        )
        .is_err());
        assert_eq!(pid_allocator.next_pid(), 2);
        assert_eq!(local_vec_id_allocator.next_local_vec_seq(), 2);
        assert_eq!(object_store.page_count(), initial_page_count);
    }

    #[test]
    fn delta_epoch_draft_from_snapshot_rejects_mismatched_delete_heap_tid() {
        let mut pid_allocator = SpirePidAllocator::default();
        let mut local_vec_id_allocator = SpireLocalVecIdAllocator::default();
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let base_draft = build_single_level_leaf_epoch_draft(
            base_build_input(vec![insert_assignment(10, 1)]),
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
        let initial_page_count = object_store.page_count();
        let mut input = delta_input(Vec::new(), vec![delete_assignment(1, 10, 2)]);
        input.base_pid = 1;

        assert!(build_delta_epoch_draft_from_snapshot(
            input,
            &base_snapshot,
            &mut pid_allocator,
            &mut local_vec_id_allocator,
            &mut object_store,
        )
        .is_err());
        assert_eq!(pid_allocator.next_pid(), 2);
        assert_eq!(local_vec_id_allocator.next_local_vec_seq(), 2);
        assert_eq!(object_store.page_count(), initial_page_count);
    }

    #[test]
    fn delta_epoch_draft_from_snapshot_rejects_stale_delete_target() {
        let mut pid_allocator = SpirePidAllocator::new(2).unwrap();
        let mut local_vec_id_allocator = SpireLocalVecIdAllocator::new(2).unwrap();
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let stale_assignment = SpireLeafAssignmentRow {
            flags: SPIRE_ASSIGNMENT_FLAG_PRIMARY | SPIRE_ASSIGNMENT_FLAG_STALE_LOCATOR,
            vec_id: SpireVecId::local(1),
            heap_tid: tid(10, 1),
            payload_format: 1,
            gamma: 0.5,
            encoded_payload: vec![1, 2, 3],
        };
        let leaf_object = SpireLeafPartitionObject::new(1, 1, 0, vec![stale_assignment]).unwrap();
        let placement = object_store.insert_leaf_object(7, &leaf_object).unwrap();
        let epoch_manifest = SpireEpochManifest {
            epoch: 7,
            state: SpireEpochState::Published,
            consistency_mode: SpireConsistencyMode::Strict,
            published_at_micros: 900,
            retain_until_micros: 1900,
            active_query_count: 0,
        };
        let object_manifest = SpireObjectManifest::from_entries(
            7,
            vec![SpireManifestEntry {
                epoch: 7,
                pid: 1,
                object_version: 1,
                placement_tid: placement.object_tid,
            }],
        )
        .unwrap();
        let placement_directory =
            SpirePlacementDirectory::from_entries(7, vec![placement]).unwrap();
        let base_snapshot = SpirePublishedEpochSnapshot::new(
            &epoch_manifest,
            &object_manifest,
            &placement_directory,
        )
        .unwrap();
        let initial_page_count = object_store.page_count();
        let mut input = delta_input(Vec::new(), vec![delete_assignment(1, 10, 1)]);
        input.base_pid = 1;

        assert!(build_delta_epoch_draft_from_snapshot(
            input,
            &base_snapshot,
            &mut pid_allocator,
            &mut local_vec_id_allocator,
            &mut object_store,
        )
        .is_err());
        assert_eq!(pid_allocator.next_pid(), 2);
        assert_eq!(local_vec_id_allocator.next_local_vec_seq(), 2);
        assert_eq!(object_store.page_count(), initial_page_count);
    }

    #[test]
    fn delta_epoch_draft_from_snapshot_rejects_duplicate_delete_vec_id() {
        let mut pid_allocator = SpirePidAllocator::default();
        let mut local_vec_id_allocator = SpireLocalVecIdAllocator::default();
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let base_draft = build_single_level_leaf_epoch_draft(
            base_build_input(vec![insert_assignment(10, 1)]),
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
        let initial_page_count = object_store.page_count();
        let mut input = delta_input(
            Vec::new(),
            vec![delete_assignment(1, 10, 1), delete_assignment(1, 10, 1)],
        );
        input.base_pid = 1;

        assert!(build_delta_epoch_draft_from_snapshot(
            input,
            &base_snapshot,
            &mut pid_allocator,
            &mut local_vec_id_allocator,
            &mut object_store,
        )
        .is_err());
        assert_eq!(pid_allocator.next_pid(), 2);
        assert_eq!(local_vec_id_allocator.next_local_vec_seq(), 2);
        assert_eq!(object_store.page_count(), initial_page_count);
    }

    #[test]
    fn delta_epoch_draft_from_snapshot_rejects_already_deleted_vec_id() {
        let mut pid_allocator = SpirePidAllocator::default();
        let mut local_vec_id_allocator = SpireLocalVecIdAllocator::default();
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let base_draft = build_single_level_leaf_epoch_draft(
            base_build_input(vec![insert_assignment(10, 1)]),
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
        let mut first_delete = delta_input(Vec::new(), vec![delete_assignment(1, 10, 1)]);
        first_delete.base_pid = 1;
        let first_delta = build_delta_epoch_draft_from_snapshot(
            first_delete,
            &base_snapshot,
            &mut pid_allocator,
            &mut local_vec_id_allocator,
            &mut object_store,
        )
        .unwrap();
        let deleted_snapshot = SpirePublishedEpochSnapshot::new(
            &first_delta.epoch_manifest,
            &first_delta.object_manifest,
            &first_delta.placement_directory,
        )
        .unwrap();
        let initial_page_count = object_store.page_count();
        let mut duplicate_delete = delta_input(Vec::new(), vec![delete_assignment(1, 10, 1)]);
        duplicate_delete.epoch = 9;
        duplicate_delete.base_pid = 1;

        assert!(build_delta_epoch_draft_from_snapshot(
            duplicate_delete,
            &deleted_snapshot,
            &mut pid_allocator,
            &mut local_vec_id_allocator,
            &mut object_store,
        )
        .is_err());
        assert_eq!(pid_allocator.next_pid(), 3);
        assert_eq!(local_vec_id_allocator.next_local_vec_seq(), 2);
        assert_eq!(object_store.page_count(), initial_page_count);
    }

    #[test]
    fn delta_epoch_draft_from_snapshot_rejects_delta_base_pid() {
        let mut pid_allocator = SpirePidAllocator::default();
        let mut local_vec_id_allocator = SpireLocalVecIdAllocator::default();
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let base_draft = build_single_level_leaf_epoch_draft(
            base_build_input(vec![insert_assignment(10, 1)]),
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
        let mut first_delta_input = delta_input(vec![insert_assignment(20, 1)], Vec::new());
        first_delta_input.base_pid = 1;
        let first_delta = build_delta_epoch_draft_from_snapshot(
            first_delta_input,
            &base_snapshot,
            &mut pid_allocator,
            &mut local_vec_id_allocator,
            &mut object_store,
        )
        .unwrap();
        let delta_snapshot = SpirePublishedEpochSnapshot::new(
            &first_delta.epoch_manifest,
            &first_delta.object_manifest,
            &first_delta.placement_directory,
        )
        .unwrap();
        let initial_page_count = object_store.page_count();
        let mut nested_delta_input = delta_input(vec![insert_assignment(30, 1)], Vec::new());
        nested_delta_input.epoch = 9;
        nested_delta_input.base_pid = first_delta.delta_object.header.pid;

        assert!(build_delta_epoch_draft_from_snapshot(
            nested_delta_input,
            &delta_snapshot,
            &mut pid_allocator,
            &mut local_vec_id_allocator,
            &mut object_store,
        )
        .is_err());
        assert_eq!(pid_allocator.next_pid(), 3);
        assert_eq!(local_vec_id_allocator.next_local_vec_seq(), 3);
        assert_eq!(object_store.page_count(), initial_page_count);
    }

    #[test]
    fn delta_epoch_draft_from_snapshot_rejects_non_newer_epoch() {
        let mut pid_allocator = SpirePidAllocator::default();
        let mut local_vec_id_allocator = SpireLocalVecIdAllocator::default();
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let base_draft = build_single_level_leaf_epoch_draft(
            base_build_input(vec![insert_assignment(10, 1)]),
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
        let initial_page_count = object_store.page_count();
        let mut input = delta_input(vec![insert_assignment(20, 1)], Vec::new());
        input.epoch = base_draft.epoch_manifest.epoch;
        input.base_pid = 1;

        assert!(build_delta_epoch_draft_from_snapshot(
            input,
            &base_snapshot,
            &mut pid_allocator,
            &mut local_vec_id_allocator,
            &mut object_store,
        )
        .is_err());
        assert_eq!(pid_allocator.next_pid(), 2);
        assert_eq!(local_vec_id_allocator.next_local_vec_seq(), 2);
        assert_eq!(object_store.page_count(), initial_page_count);
    }

    #[test]
    fn delta_epoch_draft_from_snapshot_rejects_degraded_base_placements() {
        let mut pid_allocator = SpirePidAllocator::default();
        let mut local_vec_id_allocator = SpireLocalVecIdAllocator::default();
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let base_draft = build_single_level_leaf_epoch_draft(
            base_build_input(vec![insert_assignment(10, 1)]),
            &mut pid_allocator,
            &mut local_vec_id_allocator,
            &mut object_store,
        )
        .unwrap();
        let mut epoch_manifest = base_draft.epoch_manifest;
        epoch_manifest.consistency_mode = SpireConsistencyMode::Degraded;
        let mut placement = *base_draft.placement_directory.get(1).unwrap();
        placement.state = SpirePlacementState::Skipped;
        let placement_directory =
            SpirePlacementDirectory::from_entries(base_draft.epoch_manifest.epoch, vec![placement])
                .unwrap();
        let base_snapshot = SpirePublishedEpochSnapshot::new(
            &epoch_manifest,
            &base_draft.object_manifest,
            &placement_directory,
        )
        .unwrap();
        let initial_page_count = object_store.page_count();
        let mut input = delta_input(vec![insert_assignment(20, 1)], Vec::new());
        input.base_pid = 1;

        let error = build_delta_epoch_draft_from_snapshot(
            input,
            &base_snapshot,
            &mut pid_allocator,
            &mut local_vec_id_allocator,
            &mut object_store,
        )
        .unwrap_err();

        assert!(error.contains("requires available placement"));
        assert_eq!(pid_allocator.next_pid(), 2);
        assert_eq!(local_vec_id_allocator.next_local_vec_seq(), 2);
        assert_eq!(object_store.page_count(), initial_page_count);
    }

    #[test]
    fn delta_epoch_draft_rejects_empty_delta_without_advancing_allocators() {
        let mut pid_allocator = SpirePidAllocator::new(50).unwrap();
        let mut local_vec_id_allocator = SpireLocalVecIdAllocator::default();
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let initial_page_count = object_store.page_count();

        assert!(build_delta_epoch_draft(
            delta_input(Vec::new(), Vec::new()),
            &mut pid_allocator,
            &mut local_vec_id_allocator,
            &mut object_store,
        )
        .is_err());
        assert_eq!(pid_allocator.next_pid(), 50);
        assert_eq!(local_vec_id_allocator.next_local_vec_seq(), 1);
        assert_eq!(object_store.page_count(), initial_page_count);
    }

    #[test]
    fn delta_epoch_draft_rejects_invalid_base_pid_without_advancing_allocators() {
        let mut pid_allocator = SpirePidAllocator::new(50).unwrap();
        let mut local_vec_id_allocator = SpireLocalVecIdAllocator::default();
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let initial_page_count = object_store.page_count();
        let mut input = delta_input(vec![insert_assignment(20, 1)], Vec::new());
        input.base_pid = 0;

        assert!(build_delta_epoch_draft(
            input,
            &mut pid_allocator,
            &mut local_vec_id_allocator,
            &mut object_store,
        )
        .is_err());
        assert_eq!(pid_allocator.next_pid(), 50);
        assert_eq!(local_vec_id_allocator.next_local_vec_seq(), 1);
        assert_eq!(object_store.page_count(), initial_page_count);
    }

    #[test]
    fn delta_epoch_draft_rejects_invalid_assignment_without_advancing_allocators() {
        let mut pid_allocator = SpirePidAllocator::new(50).unwrap();
        let mut local_vec_id_allocator = SpireLocalVecIdAllocator::default();
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let initial_page_count = object_store.page_count();
        let mut bad_assignment = insert_assignment(20, 1);
        bad_assignment.heap_tid = ItemPointer::INVALID;

        assert!(build_delta_epoch_draft(
            delta_input(vec![bad_assignment], vec![delete_assignment(99, 21, 1)]),
            &mut pid_allocator,
            &mut local_vec_id_allocator,
            &mut object_store,
        )
        .is_err());
        assert_eq!(pid_allocator.next_pid(), 50);
        assert_eq!(local_vec_id_allocator.next_local_vec_seq(), 1);
        assert_eq!(object_store.page_count(), initial_page_count);
    }
