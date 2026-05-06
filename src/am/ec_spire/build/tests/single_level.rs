    #[test]
    fn single_level_draft_rejects_invalid_assignment_without_advancing_allocators() {
        let mut pid_allocator = SpirePidAllocator::default();
        let mut local_vec_id_allocator = SpireLocalVecIdAllocator::default();
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let mut bad_assignment = assignment_input(10, 1);
        bad_assignment.heap_tid = ItemPointer::INVALID;

        assert!(build_single_level_leaf_epoch_draft(
            build_input(vec![bad_assignment]),
            &mut pid_allocator,
            &mut local_vec_id_allocator,
            &mut object_store,
        )
        .is_err());
        assert_eq!(pid_allocator.next_pid(), SPIRE_FIRST_PID);
        assert_eq!(
            local_vec_id_allocator.next_local_vec_seq(),
            SPIRE_FIRST_LOCAL_VEC_SEQ
        );
    }

    #[test]
    fn single_level_draft_rejects_invalid_manifest_locator_without_advancing_allocators() {
        let mut pid_allocator = SpirePidAllocator::default();
        let mut local_vec_id_allocator = SpireLocalVecIdAllocator::default();
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let mut input = build_input(vec![assignment_input(10, 1)]);
        input.placement_tid = ItemPointer::INVALID;

        assert!(build_single_level_leaf_epoch_draft(
            input,
            &mut pid_allocator,
            &mut local_vec_id_allocator,
            &mut object_store,
        )
        .is_err());
        assert_eq!(pid_allocator.next_pid(), SPIRE_FIRST_PID);
        assert_eq!(
            local_vec_id_allocator.next_local_vec_seq(),
            SPIRE_FIRST_LOCAL_VEC_SEQ
        );
    }

    #[test]
    fn single_level_draft_rejects_invalid_publish_timestamp_without_advancing_allocators() {
        let mut pid_allocator = SpirePidAllocator::default();
        let mut local_vec_id_allocator = SpireLocalVecIdAllocator::default();
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let mut input = build_input(vec![assignment_input(10, 1)]);
        input.published_at_micros = 0;

        assert!(build_single_level_leaf_epoch_draft(
            input,
            &mut pid_allocator,
            &mut local_vec_id_allocator,
            &mut object_store,
        )
        .is_err());
        assert_eq!(pid_allocator.next_pid(), SPIRE_FIRST_PID);
        assert_eq!(
            local_vec_id_allocator.next_local_vec_seq(),
            SPIRE_FIRST_LOCAL_VEC_SEQ
        );
    }

    #[test]
    fn partitioned_single_level_draft_writes_leaf_objects_per_centroid() {
        let source_vectors = vec![vec![1.0, 0.0], vec![-1.0, 0.0]];
        let centroid_plan = train_single_level_centroid_plan(2, &source_vectors, 2, 42).unwrap();
        let mut pid_allocator = SpirePidAllocator::default();
        let mut local_vec_id_allocator = SpireLocalVecIdAllocator::default();
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();

        let draft = build_partitioned_single_level_leaf_epoch_draft(
            partitioned_build_input(
                vec![assignment_input(10, 1), assignment_input(10, 2)],
                centroid_plan,
            ),
            &mut pid_allocator,
            &mut local_vec_id_allocator,
            &mut object_store,
        )
        .unwrap();

        assert_eq!(
            draft.centroid_pids,
            vec![SPIRE_FIRST_PID + 1, SPIRE_FIRST_PID + 2]
        );
        assert_eq!(draft.root_pid, SPIRE_FIRST_PID);
        assert_eq!(draft.routing_object.header.pid, SPIRE_FIRST_PID);
        assert_eq!(draft.routing_object.header.child_count, 2);
        assert_eq!(draft.leaf_objects.len(), 2);
        assert_eq!(draft.route_map.entries.len(), 2);
        assert_eq!(draft.object_manifest.entries.len(), 3);
        assert_eq!(draft.placement_directory.entries.len(), 3);
        let root_placement = draft.placement_directory.get(draft.root_pid).unwrap();
        let mut expected_routing_object = draft.routing_object.clone();
        expected_routing_object.header.published_epoch_backref = draft.epoch_manifest.epoch;
        assert_eq!(
            object_store.read_routing_object(root_placement).unwrap(),
            expected_routing_object
        );
        for &pid in &draft.centroid_pids {
            assert!(draft.route_map.entries.iter().any(|entry| entry.pid == pid));
        }
        for leaf_object in &draft.leaf_objects {
            assert_eq!(leaf_object.header.parent_pid, draft.root_pid);
            assert!(draft.object_manifest.get(leaf_object.header.pid).is_some());
            let placement = draft
                .placement_directory
                .get(leaf_object.header.pid)
                .unwrap();
            let stored_leaf = object_store.read_leaf_object_v2(placement).unwrap();
            assert_eq!(stored_leaf.meta.header.pid, leaf_object.header.pid);
            assert_eq!(
                stored_leaf.meta.header.assignment_count,
                leaf_object.assignments.len() as u32
            );
        }
        assert_eq!(
            draft
                .leaf_objects
                .iter()
                .map(|object| object.assignments.len())
                .sum::<usize>(),
            2
        );
        SpirePublishedEpochSnapshot::new(
            &draft.epoch_manifest,
            &draft.object_manifest,
            &draft.placement_directory,
        )
        .unwrap();
        assert_eq!(draft.next_pid, SPIRE_FIRST_PID + 3);
        assert_eq!(draft.next_local_vec_seq, SPIRE_FIRST_LOCAL_VEC_SEQ + 2);
        assert_eq!(pid_allocator.next_pid(), draft.next_pid);
        assert_eq!(
            local_vec_id_allocator.next_local_vec_seq(),
            draft.next_local_vec_seq
        );
    }

    #[test]
    fn partitioned_single_level_draft_hash_routes_object_writes_by_pid() {
        let source_vectors = vec![vec![1.0, 0.0], vec![-1.0, 0.0]];
        let centroid_plan = train_single_level_centroid_plan(2, &source_vectors, 2, 42).unwrap();
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

        let draft = build_partitioned_single_level_leaf_epoch_draft(
            partitioned_build_input(
                vec![assignment_input(10, 1), assignment_input(10, 2)],
                centroid_plan,
            ),
            &mut pid_allocator,
            &mut local_vec_id_allocator,
            &mut object_store,
        )
        .unwrap();

        for placement in &draft.placement_directory.entries {
            let expected_store = store_config.store_for_pid(placement.pid).unwrap();
            assert_eq!(placement.local_store_id, expected_store.local_store_id);
            assert_eq!(placement.store_relid, expected_store.store_relid);
        }
        let root_placement = draft.placement_directory.get(draft.root_pid).unwrap();
        assert_eq!(
            object_store
                .read_routing_object(root_placement)
                .unwrap()
                .header
                .pid,
            draft.root_pid
        );
    }

    #[test]
    fn partitioned_single_level_draft_preserves_empty_centroid_leaf() {
        let centroid_plan = SpireSingleLevelCentroidPlan {
            dimensions: 2,
            centroids: vec![vec![1.0, 0.0], vec![-1.0, 0.0]],
            assignment_indexes: vec![0],
        };
        let mut pid_allocator = SpirePidAllocator::default();
        let mut local_vec_id_allocator = SpireLocalVecIdAllocator::default();
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();

        let draft = build_partitioned_single_level_leaf_epoch_draft(
            partitioned_build_input(vec![assignment_input(10, 1)], centroid_plan),
            &mut pid_allocator,
            &mut local_vec_id_allocator,
            &mut object_store,
        )
        .unwrap();

        assert_eq!(draft.leaf_objects.len(), 2);
        assert_eq!(draft.leaf_objects[0].assignments.len(), 1);
        assert!(draft.leaf_objects[1].assignments.is_empty());
        assert_eq!(draft.leaf_objects[1].header.pid, SPIRE_FIRST_PID + 2);
        assert_eq!(draft.route_map.get(1).unwrap().pid, SPIRE_FIRST_PID + 2);
        assert!(draft.object_manifest.get(SPIRE_FIRST_PID + 2).is_some());
        assert!(draft.placement_directory.get(SPIRE_FIRST_PID + 2).is_some());
        assert_eq!(draft.next_pid, SPIRE_FIRST_PID + 3);
        assert_eq!(draft.next_local_vec_seq, SPIRE_FIRST_LOCAL_VEC_SEQ + 1);
    }

    #[test]
    fn partitioned_single_level_draft_rejects_bad_plan_without_advancing_allocators() {
        let centroid_plan = SpireSingleLevelCentroidPlan {
            dimensions: 2,
            centroids: vec![vec![1.0, 0.0], vec![-1.0, 0.0]],
            assignment_indexes: vec![2],
        };
        let mut pid_allocator = SpirePidAllocator::default();
        let mut local_vec_id_allocator = SpireLocalVecIdAllocator::default();
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let initial_page_count = object_store.page_count();

        assert!(build_partitioned_single_level_leaf_epoch_draft(
            partitioned_build_input(vec![assignment_input(10, 1)], centroid_plan),
            &mut pid_allocator,
            &mut local_vec_id_allocator,
            &mut object_store,
        )
        .is_err());
        assert_eq!(pid_allocator.next_pid(), SPIRE_FIRST_PID);
        assert_eq!(
            local_vec_id_allocator.next_local_vec_seq(),
            SPIRE_FIRST_LOCAL_VEC_SEQ
        );
        assert_eq!(object_store.page_count(), initial_page_count);
    }

    #[test]
    fn partitioned_single_level_draft_rejects_late_bad_assignment_without_store_write() {
        let centroid_plan = SpireSingleLevelCentroidPlan {
            dimensions: 2,
            centroids: vec![vec![1.0, 0.0], vec![-1.0, 0.0]],
            assignment_indexes: vec![0, 1],
        };
        let mut pid_allocator = SpirePidAllocator::default();
        let mut local_vec_id_allocator = SpireLocalVecIdAllocator::default();
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let initial_page_count = object_store.page_count();
        let mut bad_assignment = assignment_input(10, 2);
        bad_assignment.heap_tid = ItemPointer::INVALID;

        assert!(build_partitioned_single_level_leaf_epoch_draft(
            partitioned_build_input(vec![assignment_input(10, 1), bad_assignment], centroid_plan),
            &mut pid_allocator,
            &mut local_vec_id_allocator,
            &mut object_store,
        )
        .is_err());
        assert_eq!(pid_allocator.next_pid(), SPIRE_FIRST_PID);
        assert_eq!(
            local_vec_id_allocator.next_local_vec_seq(),
            SPIRE_FIRST_LOCAL_VEC_SEQ
        );
        assert_eq!(object_store.page_count(), initial_page_count);
    }
