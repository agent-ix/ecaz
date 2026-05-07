    #[test]
    fn local_object_store_writes_and_reads_leaf_object() {
        let object = SpireLeafPartitionObject::new(
            17,
            3,
            0,
            vec![SpireLeafAssignmentRow {
                flags: SPIRE_ASSIGNMENT_FLAG_PRIMARY,
                vec_id: SpireVecId::local(1),
                heap_tid: ItemPointer {
                    block_number: 10,
                    offset_number: 1,
                },
                payload_format: 1,
                gamma: 0.5,
                encoded_payload: vec![1, 2],
            }],
        )
        .unwrap();
        let expected_bytes = object.encode().unwrap().len() as u32;
        let mut store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();

        let placement = store.insert_leaf_object(7, &object).unwrap();
        let decoded = store.read_leaf_object(&placement).unwrap();
        let mut expected = object.clone();
        expected.header.published_epoch_backref = 7;

        assert_eq!(decoded, expected);
        assert_eq!(placement.epoch, 7);
        assert_eq!(placement.pid, 17);
        assert_eq!(placement.object_version, 3);
        assert_eq!(placement.store_relid, 12345);
        assert_eq!(placement.node_id, 0);
        assert_eq!(placement.local_store_id, 0);
        assert_eq!(placement.object_bytes, expected_bytes);
        assert_eq!(store.page_count(), 1);
    }

    #[test]
    fn local_object_store_writes_and_reads_delta_object() {
        let object = SpireDeltaPartitionObject::new(
            19,
            4,
            17,
            vec![SpireLeafAssignmentRow {
                flags: SPIRE_ASSIGNMENT_FLAG_PRIMARY | SPIRE_ASSIGNMENT_FLAG_DELTA_INSERT,
                vec_id: SpireVecId::local(1),
                heap_tid: ItemPointer {
                    block_number: 10,
                    offset_number: 1,
                },
                payload_format: 1,
                gamma: 0.5,
                encoded_payload: vec![1, 2],
            }],
        )
        .unwrap();
        let expected_bytes = object.encode().unwrap().len() as u32;
        let mut store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();

        let placement = store.insert_delta_object(7, &object).unwrap();
        let decoded = store.read_delta_object(&placement).unwrap();
        let mut expected = object.clone();
        expected.header.published_epoch_backref = 7;

        assert_eq!(decoded, expected);
        assert_eq!(placement.epoch, 7);
        assert_eq!(placement.pid, 19);
        assert_eq!(placement.object_version, 4);
        assert_eq!(placement.store_relid, 12345);
        assert_eq!(placement.node_id, 0);
        assert_eq!(placement.local_store_id, 0);
        assert_eq!(placement.object_bytes, expected_bytes);
        assert_eq!(store.page_count(), 1);
    }

    #[test]
    fn local_object_store_writes_and_reads_routing_object() {
        let object = SpireRoutingPartitionObject::root(11, 3, 2, routing_children()).unwrap();
        let expected_bytes = object.encode().unwrap().len() as u32;
        let mut store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();

        let placement = store.insert_routing_object(7, &object).unwrap();
        let decoded = store.read_routing_object(&placement).unwrap();
        let mut expected = object.clone();
        expected.header.published_epoch_backref = 7;

        assert_eq!(decoded, expected);
        assert_eq!(placement.epoch, 7);
        assert_eq!(placement.pid, 11);
        assert_eq!(placement.object_version, 3);
        assert_eq!(placement.store_relid, 12345);
        assert_eq!(placement.node_id, 0);
        assert_eq!(placement.local_store_id, 0);
        assert_eq!(placement.object_bytes, expected_bytes);
        assert_eq!(store.page_count(), 1);
    }

    #[test]
    fn local_object_store_reads_object_headers_for_dispatch() {
        let leaf = SpireLeafPartitionObject::new(17, 3, 0, Vec::new()).unwrap();
        let delta = SpireDeltaPartitionObject::new(19, 4, 17, Vec::new()).unwrap();
        let root = SpireRoutingPartitionObject::root(11, 3, 2, routing_children()).unwrap();
        let mut store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();

        let leaf_placement = store.insert_leaf_object(7, &leaf).unwrap();
        let delta_placement = store.insert_delta_object(7, &delta).unwrap();
        let root_placement = store.insert_routing_object(7, &root).unwrap();
        let leaf_header = store.read_object_header(&leaf_placement).unwrap();
        let delta_header = store.read_object_header(&delta_placement).unwrap();
        let root_header = store.read_object_header(&root_placement).unwrap();

        assert_eq!(leaf_header.kind, SpirePartitionObjectKind::Leaf);
        assert_eq!(leaf_header.pid, 17);
        assert_eq!(leaf_header.object_version, 3);
        assert_eq!(leaf_header.published_epoch_backref, 7);
        assert_eq!(delta_header.kind, SpirePartitionObjectKind::Delta);
        assert_eq!(delta_header.pid, 19);
        assert_eq!(delta_header.object_version, 4);
        assert_eq!(delta_header.published_epoch_backref, 7);
        assert_eq!(root_header.kind, SpirePartitionObjectKind::Root);
        assert_eq!(root_header.pid, 11);
        assert_eq!(root_header.object_version, 3);
        assert_eq!(root_header.published_epoch_backref, 7);
    }

    #[test]
    fn local_object_store_rejects_invalid_store_and_epoch() {
        assert!(SpireLocalObjectStore::with_default_page_size(0).is_err());
        let object = SpireLeafPartitionObject::new(17, 3, 0, Vec::new()).unwrap();
        let mut store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();

        assert!(store.insert_leaf_object(0, &object).is_err());
        assert_eq!(store.page_count(), 1);

        let delta = SpireDeltaPartitionObject::new(19, 4, 17, Vec::new()).unwrap();
        assert!(store.insert_delta_object(0, &delta).is_err());

        let root = SpireRoutingPartitionObject::root(11, 3, 2, routing_children()).unwrap();
        assert!(store.insert_routing_object(0, &root).is_err());
    }

    #[test]
    fn local_object_store_rejects_mismatched_placement() {
        let object = SpireLeafPartitionObject::new(17, 3, 0, Vec::new()).unwrap();
        let mut store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let placement = store.insert_leaf_object(7, &object).unwrap();

        let mut wrong_store = placement;
        wrong_store.store_relid = 54321;
        assert!(store.read_leaf_object(&wrong_store).is_err());

        let mut stale = placement;
        stale.state = SpirePlacementState::Stale;
        assert!(store.read_leaf_object(&stale).is_err());

        let mut wrong_pid = placement;
        wrong_pid.pid = 99;
        assert!(store.read_leaf_object(&wrong_pid).is_err());

        let mut wrong_bytes = placement;
        wrong_bytes.object_bytes += 1;
        assert!(store.read_leaf_object(&wrong_bytes).is_err());

        let mut wrong_epoch = placement;
        wrong_epoch.epoch = 6;
        assert!(store.read_leaf_object(&wrong_epoch).is_err());
    }

    #[test]
    fn local_object_store_rejects_mismatched_delta_placement() {
        let object = SpireDeltaPartitionObject::new(19, 4, 17, Vec::new()).unwrap();
        let mut store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let placement = store.insert_delta_object(7, &object).unwrap();

        let mut wrong_store = placement;
        wrong_store.store_relid = 54321;
        assert!(store.read_delta_object(&wrong_store).is_err());

        let mut stale = placement;
        stale.state = SpirePlacementState::Stale;
        assert!(store.read_delta_object(&stale).is_err());

        let mut wrong_pid = placement;
        wrong_pid.pid = 99;
        assert!(store.read_delta_object(&wrong_pid).is_err());

        let mut wrong_version = placement;
        wrong_version.object_version = 99;
        assert!(store.read_delta_object(&wrong_version).is_err());

        let mut wrong_bytes = placement;
        wrong_bytes.object_bytes += 1;
        assert!(store.read_delta_object(&wrong_bytes).is_err());
    }

    #[test]
    fn local_object_store_rejects_mismatched_routing_placement() {
        let object = SpireRoutingPartitionObject::root(11, 3, 2, routing_children()).unwrap();
        let mut store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let placement = store.insert_routing_object(7, &object).unwrap();

        let mut wrong_store = placement;
        wrong_store.store_relid = 54321;
        assert!(store.read_routing_object(&wrong_store).is_err());

        let mut stale = placement;
        stale.state = SpirePlacementState::Stale;
        assert!(store.read_routing_object(&stale).is_err());

        let mut wrong_pid = placement;
        wrong_pid.pid = 99;
        assert!(store.read_routing_object(&wrong_pid).is_err());

        let mut wrong_bytes = placement;
        wrong_bytes.object_bytes += 1;
        assert!(store.read_routing_object(&wrong_bytes).is_err());
    }
