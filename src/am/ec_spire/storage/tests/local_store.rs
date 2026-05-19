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
    fn local_object_store_writes_and_reads_top_graph_object() {
        let object = SpireTopGraphPartitionObject::new(
            90,
            3,
            11,
            2,
            2,
            2,
            4,
            1.2,
            0,
            vec![
                top_graph_node(21, 0, vec![1]),
                top_graph_node(22, 1, vec![0]),
            ],
        )
        .unwrap();
        let expected_bytes = object.encode().unwrap().len() as u32;
        let mut store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();

        let placement = store.insert_top_graph_object(7, &object).unwrap();
        let decoded = store.read_top_graph_object(&placement).unwrap();
        let mut expected = object.clone();
        expected.header.published_epoch_backref = 7;

        assert_eq!(decoded, expected);
        assert_eq!(placement.epoch, 7);
        assert_eq!(placement.pid, 90);
        assert_eq!(placement.object_version, 3);
        assert_eq!(placement.object_bytes, expected_bytes);
        assert_eq!(store.page_count(), 1);
    }

    #[test]
    fn local_object_store_reads_object_headers_for_dispatch() {
        let leaf = SpireLeafPartitionObject::new(17, 3, 0, Vec::new()).unwrap();
        let delta = SpireDeltaPartitionObject::new(19, 4, 17, Vec::new()).unwrap();
        let root = SpireRoutingPartitionObject::root(11, 3, 2, routing_children()).unwrap();
        let top_graph = SpireTopGraphPartitionObject::new(
            90,
            3,
            11,
            2,
            2,
            2,
            4,
            1.2,
            0,
            vec![top_graph_node(21, 0, vec![])],
        )
        .unwrap();
        let mut store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();

        let leaf_placement = store.insert_leaf_object(7, &leaf).unwrap();
        let delta_placement = store.insert_delta_object(7, &delta).unwrap();
        let root_placement = store.insert_routing_object(7, &root).unwrap();
        let top_graph_placement = store.insert_top_graph_object(7, &top_graph).unwrap();
        let leaf_header = store.read_object_header(&leaf_placement).unwrap();
        let delta_header = store.read_object_header(&delta_placement).unwrap();
        let root_header = store.read_object_header(&root_placement).unwrap();
        let top_graph_header = store.read_object_header(&top_graph_placement).unwrap();

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
        assert_eq!(top_graph_header.kind, SpirePartitionObjectKind::TopGraph);
        assert_eq!(top_graph_header.pid, 90);
        assert_eq!(top_graph_header.parent_pid, 11);
        assert_eq!(top_graph_header.published_epoch_backref, 7);
    }

    #[test]
    fn local_object_store_rejects_invalid_store_and_epoch() {
        assert!(SpireLocalObjectStore::with_default_page_size(0).is_err());
        // The other branch in new_for_store: a valid relid with page_size == 0.
        assert!(SpireLocalObjectStore::new(12345, 0).is_err());
        let object = SpireLeafPartitionObject::new(17, 3, 0, Vec::new()).unwrap();
        let mut store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();

        assert!(store.insert_leaf_object(0, &object).is_err());
        assert_eq!(store.page_count(), 1);

        let delta = SpireDeltaPartitionObject::new(19, 4, 17, Vec::new()).unwrap();
        assert!(store.insert_delta_object(0, &delta).is_err());

        let root = SpireRoutingPartitionObject::root(11, 3, 2, routing_children()).unwrap();
        assert!(store.insert_routing_object(0, &root).is_err());

        // Top-graph insert shares the epoch == 0 guard with the other inserts.
        let top_graph = SpireTopGraphPartitionObject::new(
            90,
            3,
            11,
            2,
            2,
            2,
            4,
            1.2,
            0,
            vec![top_graph_node(21, 0, vec![])],
        )
        .unwrap();
        assert!(store.insert_top_graph_object(0, &top_graph).is_err());
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

    #[test]
    fn local_object_store_set_from_config_rejects_duplicate_local_store_id() {
        // Two descriptors with the same local_store_id must error before
        // any store is constructed.
        let config = SpireLocalStoreConfig::from_stores(
            1,
            vec![
                SpireLocalStoreDescriptor::available(0, 10_000, 0).unwrap(),
                SpireLocalStoreDescriptor::available(0, 10_001, 0).unwrap(),
            ],
        );
        // from_stores itself may reject the duplicate; if it does, accept
        // that as the matching guard. If it does not, drive
        // SpireLocalObjectStoreSet::from_config directly.
        match config {
            Ok(config) => {
                let err = SpireLocalObjectStoreSet::from_config(config, 512)
                    .err()
                    .expect("duplicate local_store_id must be rejected");
                assert!(
                    err.contains("duplicate local_store_id"),
                    "unexpected error: {err}"
                );
            }
            Err(err) => {
                assert!(
                    err.contains("local_store_id") || err.contains("duplicate"),
                    "unexpected from_stores error: {err}"
                );
            }
        }
    }

    #[test]
    fn local_object_store_set_round_trips_leaf_v1() {
        // The set's `read_leaf_object` (V1) delegate at line 132 was not
        // covered by 028-031; this exercises it end-to-end alongside the
        // matching insert path on a single-store config.
        let config = SpireLocalStoreConfig::from_stores(
            1,
            vec![SpireLocalStoreDescriptor::available(0, 10_000, 0).unwrap()],
        )
        .unwrap();
        let mut store_set = SpireLocalObjectStoreSet::from_config(config, 512).unwrap();
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
                payload_format: SPIRE_PAYLOAD_FORMAT_TURBOQUANT,
                gamma: 0.25,
                encoded_payload: vec![1, 2],
            }],
        )
        .unwrap();
        // V1 insert delegates through the single-store path; route via
        // store_mut_for_pid + SpireLocalObjectStore::insert_leaf_object.
        let placement = store_set
            .store_mut_for_pid(17)
            .unwrap()
            .insert_leaf_object(7, &object)
            .unwrap();
        let decoded = store_set.read_leaf_object(&placement).unwrap();
        assert_eq!(decoded.header.pid, 17);
        assert_eq!(decoded.header.object_version, 3);
    }

    #[test]
    fn local_object_store_set_object_reader_trait_routes_through_store_for_placement() {
        // Cover the SpireObjectReader-for-SpireLocalObjectStoreSet impls
        // (read_object_header / read_routing / read_delta / read_top_graph
        // / read_leaf_v2) through trait dispatch so a `delegate-to-wrong-store`
        // mutation is observable.
        let config = SpireLocalStoreConfig::from_stores(
            1,
            vec![SpireLocalStoreDescriptor::available(0, 10_000, 0).unwrap()],
        )
        .unwrap();
        let mut store_set = SpireLocalObjectStoreSet::from_config(config, 1024).unwrap();
        let routing =
            SpireRoutingPartitionObject::root(11, 3, 2, routing_children()).unwrap();
        let routing_placement = store_set.insert_routing_object(7, &routing).unwrap();
        let leaf_placement = store_set
            .insert_leaf_object_v2_from_rows(
                7,
                17,
                3,
                5,
                &[leaf_v2_assignment(1, 8), leaf_v2_assignment(2, 8)],
            )
            .unwrap();
        let delta = SpireDeltaPartitionObject::new(
            19,
            4,
            17,
            vec![SpireLeafAssignmentRow {
                flags: SPIRE_ASSIGNMENT_FLAG_PRIMARY | SPIRE_ASSIGNMENT_FLAG_DELTA_INSERT,
                vec_id: SpireVecId::local(1),
                heap_tid: ItemPointer {
                    block_number: 1,
                    offset_number: 1,
                },
                payload_format: SPIRE_PAYLOAD_FORMAT_TURBOQUANT,
                gamma: 0.5,
                encoded_payload: vec![1, 2, 3, 4],
            }],
        )
        .unwrap();
        let delta_placement = store_set.insert_delta_object(7, &delta).unwrap();
        let top_graph = SpireTopGraphPartitionObject::new(
            90,
            3,
            11,
            2,
            2,
            2,
            4,
            1.2,
            0,
            vec![SpireTopGraphNodeRecord {
                child_pid: 21,
                centroid_ordinal: 0,
                neighbors: vec![],
            }],
        )
        .unwrap();
        let top_placement = store_set.insert_top_graph_object(7, &top_graph).unwrap();

        let reader: &dyn SpireObjectReader = &store_set;
        assert_eq!(reader.read_object_header(&routing_placement).unwrap().pid, 11);
        assert_eq!(
            reader.read_routing_object(&routing_placement).unwrap().header.pid,
            11,
        );
        assert_eq!(
            reader
                .read_leaf_object_v2(&leaf_placement)
                .unwrap()
                .meta
                .header
                .pid,
            17,
        );
        assert_eq!(reader.read_delta_object(&delta_placement).unwrap().header.pid, 19);
        assert_eq!(
            reader.read_top_graph_object(&top_placement).unwrap().header.pid,
            90,
        );
    }

    #[test]
    fn local_object_store_trait_dispatch_covers_all_read_methods() {
        // Cover the SpireObjectReader-for-SpireLocalObjectStore impl block
        // (165-207). Existing tests call the inherent methods; trait
        // dispatch on a `&dyn SpireObjectReader` exercises the impl block.
        let mut store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let routing =
            SpireRoutingPartitionObject::root(11, 3, 2, routing_children()).unwrap();
        let routing_placement = store.insert_routing_object(7, &routing).unwrap();

        let leaf_v1 = SpireLeafPartitionObject::new(
            17,
            3,
            0,
            vec![SpireLeafAssignmentRow {
                flags: SPIRE_ASSIGNMENT_FLAG_PRIMARY,
                vec_id: SpireVecId::local(1),
                heap_tid: ItemPointer {
                    block_number: 1,
                    offset_number: 1,
                },
                payload_format: SPIRE_PAYLOAD_FORMAT_TURBOQUANT,
                gamma: 0.5,
                encoded_payload: vec![1, 2, 3, 4],
            }],
        )
        .unwrap();
        let leaf_v1_placement = store.insert_leaf_object(7, &leaf_v1).unwrap();

        let leaf_v2_placement = store
            .insert_leaf_object_v2_from_rows(
                7,
                18,
                1,
                17,
                &[leaf_v2_assignment(1, 8)],
            )
            .unwrap();

        let delta = SpireDeltaPartitionObject::new(
            19,
            1,
            17,
            vec![SpireLeafAssignmentRow {
                flags: SPIRE_ASSIGNMENT_FLAG_PRIMARY | SPIRE_ASSIGNMENT_FLAG_DELTA_INSERT,
                vec_id: SpireVecId::local(1),
                heap_tid: ItemPointer {
                    block_number: 1,
                    offset_number: 1,
                },
                payload_format: SPIRE_PAYLOAD_FORMAT_TURBOQUANT,
                gamma: 0.5,
                encoded_payload: vec![9, 9, 9, 9],
            }],
        )
        .unwrap();
        let delta_placement = store.insert_delta_object(7, &delta).unwrap();

        let top_graph = SpireTopGraphPartitionObject::new(
            90,
            3,
            11,
            2,
            2,
            2,
            4,
            1.2,
            0,
            vec![SpireTopGraphNodeRecord {
                child_pid: 21,
                centroid_ordinal: 0,
                neighbors: vec![],
            }],
        )
        .unwrap();
        let top_placement = store.insert_top_graph_object(7, &top_graph).unwrap();

        let reader: &dyn SpireObjectReader = &store;
        assert_eq!(reader.read_object_header(&routing_placement).unwrap().pid, 11);
        assert_eq!(
            reader.read_routing_object(&routing_placement).unwrap().header.pid,
            11
        );
        assert_eq!(
            reader.read_leaf_object(&leaf_v1_placement).unwrap().header.pid,
            17
        );
        assert_eq!(
            reader
                .read_leaf_object_v2(&leaf_v2_placement)
                .unwrap()
                .meta
                .header
                .pid,
            18
        );
        assert_eq!(reader.read_delta_object(&delta_placement).unwrap().header.pid, 19);
        assert_eq!(
            reader.read_top_graph_object(&top_placement).unwrap().header.pid,
            90
        );
    }
