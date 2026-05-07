    #[test]
    fn delta_partition_object_round_trips_insert_and_delete_rows() {
        let object = SpireDeltaPartitionObject::new(
            19,
            4,
            17,
            vec![
                SpireLeafAssignmentRow {
                    flags: SPIRE_ASSIGNMENT_FLAG_PRIMARY | SPIRE_ASSIGNMENT_FLAG_DELTA_INSERT,
                    vec_id: SpireVecId::local(1),
                    heap_tid: ItemPointer {
                        block_number: 10,
                        offset_number: 1,
                    },
                    payload_format: 1,
                    gamma: 0.5,
                    encoded_payload: vec![1, 2],
                },
                SpireLeafAssignmentRow {
                    flags: SPIRE_ASSIGNMENT_FLAG_TOMBSTONE | SPIRE_ASSIGNMENT_FLAG_DELTA_DELETE,
                    vec_id: SpireVecId::local(2),
                    heap_tid: ItemPointer {
                        block_number: 10,
                        offset_number: 2,
                    },
                    payload_format: SPIRE_PAYLOAD_FORMAT_NONE,
                    gamma: 0.0,
                    encoded_payload: Vec::new(),
                },
            ],
        )
        .unwrap();

        let decoded = SpireDeltaPartitionObject::decode(&object.encode().unwrap()).unwrap();

        assert_eq!(decoded, object);
        assert_eq!(decoded.header.kind, SpirePartitionObjectKind::Delta);
        assert_eq!(decoded.header.pid, 19);
        assert_eq!(decoded.header.parent_pid, 17);
        assert_eq!(decoded.header.assignment_count, 2);
    }

    #[test]
    fn delta_partition_object_rejects_invalid_header_shape() {
        assert!(SpireDeltaPartitionObject::new(19, 4, 0, Vec::new()).is_err());

        let row = SpireLeafAssignmentRow {
            flags: SPIRE_ASSIGNMENT_FLAG_PRIMARY | SPIRE_ASSIGNMENT_FLAG_DELTA_INSERT,
            vec_id: SpireVecId::local(1),
            heap_tid: ItemPointer {
                block_number: 10,
                offset_number: 1,
            },
            payload_format: 1,
            gamma: 0.5,
            encoded_payload: vec![1, 2],
        };
        let mut object = SpireDeltaPartitionObject::new(19, 4, 17, vec![row]).unwrap();

        object.header.kind = SpirePartitionObjectKind::Leaf;
        assert!(object.encode().is_err());

        object.header.kind = SpirePartitionObjectKind::Delta;
        object.header.child_count = 1;
        assert!(object.encode().is_err());

        object.header.child_count = 0;
        object.header.assignment_count = 2;
        assert!(object.encode().is_err());
    }

    #[test]
    fn delta_partition_object_rejects_invalid_delta_flags() {
        let valid_row = SpireLeafAssignmentRow {
            flags: SPIRE_ASSIGNMENT_FLAG_PRIMARY | SPIRE_ASSIGNMENT_FLAG_DELTA_INSERT,
            vec_id: SpireVecId::local(1),
            heap_tid: ItemPointer {
                block_number: 10,
                offset_number: 1,
            },
            payload_format: 1,
            gamma: 0.5,
            encoded_payload: vec![1, 2],
        };

        let mut row = valid_row.clone();
        row.flags = SPIRE_ASSIGNMENT_FLAG_PRIMARY;
        assert!(SpireDeltaPartitionObject::new(19, 4, 17, vec![row]).is_err());

        row = valid_row.clone();
        row.flags = SPIRE_ASSIGNMENT_FLAG_DELTA_INSERT | SPIRE_ASSIGNMENT_FLAG_DELTA_DELETE;
        assert!(SpireDeltaPartitionObject::new(19, 4, 17, vec![row]).is_err());

        row = valid_row.clone();
        row.flags = SPIRE_ASSIGNMENT_FLAG_DELTA_INSERT;
        assert!(SpireDeltaPartitionObject::new(19, 4, 17, vec![row]).is_err());

        row = valid_row.clone();
        row.flags = SPIRE_ASSIGNMENT_FLAG_DELTA_INSERT | SPIRE_ASSIGNMENT_FLAG_TOMBSTONE;
        assert!(SpireDeltaPartitionObject::new(19, 4, 17, vec![row]).is_err());

        row = valid_row.clone();
        row.flags = SPIRE_ASSIGNMENT_FLAG_PRIMARY
            | SPIRE_ASSIGNMENT_FLAG_TOMBSTONE
            | SPIRE_ASSIGNMENT_FLAG_DELTA_DELETE;
        assert!(SpireDeltaPartitionObject::new(19, 4, 17, vec![row]).is_err());

        row = valid_row.clone();
        row.flags = SPIRE_ASSIGNMENT_FLAG_DELTA_DELETE;
        assert!(SpireDeltaPartitionObject::new(19, 4, 17, vec![row]).is_err());

        row = valid_row.clone();
        row.flags = SPIRE_ASSIGNMENT_FLAG_DELTA_INSERT | SPIRE_ASSIGNMENT_FLAG_BOUNDARY_REPLICA;
        assert!(SpireDeltaPartitionObject::new(19, 4, 17, vec![row]).is_err());

        row = valid_row.clone();
        row.flags = SPIRE_ASSIGNMENT_FLAG_PRIMARY
            | SPIRE_ASSIGNMENT_FLAG_DELTA_INSERT
            | SPIRE_ASSIGNMENT_FLAG_STALE_LOCATOR;
        assert!(SpireDeltaPartitionObject::new(19, 4, 17, vec![row]).is_err());

        row = valid_row.clone();
        row.payload_format = SPIRE_PAYLOAD_FORMAT_NONE;
        assert!(SpireDeltaPartitionObject::new(19, 4, 17, vec![row]).is_err());

        row = valid_row;
        row.encoded_payload.clear();
        assert!(SpireDeltaPartitionObject::new(19, 4, 17, vec![row]).is_err());
    }

    #[test]
    fn delta_partition_object_rejects_delete_payloads() {
        let valid_delete_row = SpireLeafAssignmentRow {
            flags: SPIRE_ASSIGNMENT_FLAG_TOMBSTONE | SPIRE_ASSIGNMENT_FLAG_DELTA_DELETE,
            vec_id: SpireVecId::local(1),
            heap_tid: ItemPointer {
                block_number: 10,
                offset_number: 1,
            },
            payload_format: SPIRE_PAYLOAD_FORMAT_NONE,
            gamma: 0.0,
            encoded_payload: Vec::new(),
        };

        let mut row = valid_delete_row.clone();
        row.payload_format = 1;
        assert!(SpireDeltaPartitionObject::new(19, 4, 17, vec![row]).is_err());

        row = valid_delete_row.clone();
        row.gamma = 0.5;
        assert!(SpireDeltaPartitionObject::new(19, 4, 17, vec![row]).is_err());

        row = valid_delete_row;
        row.encoded_payload = vec![1, 2];
        assert!(SpireDeltaPartitionObject::new(19, 4, 17, vec![row]).is_err());
    }

    #[test]
    fn delta_partition_object_rejects_duplicate_vec_ids() {
        let insert_row = SpireLeafAssignmentRow {
            flags: SPIRE_ASSIGNMENT_FLAG_PRIMARY | SPIRE_ASSIGNMENT_FLAG_DELTA_INSERT,
            vec_id: SpireVecId::local(1),
            heap_tid: ItemPointer {
                block_number: 10,
                offset_number: 1,
            },
            payload_format: 1,
            gamma: 0.5,
            encoded_payload: vec![1, 2],
        };
        let delete_row = SpireLeafAssignmentRow {
            flags: SPIRE_ASSIGNMENT_FLAG_TOMBSTONE | SPIRE_ASSIGNMENT_FLAG_DELTA_DELETE,
            vec_id: SpireVecId::local(1),
            heap_tid: ItemPointer {
                block_number: 10,
                offset_number: 1,
            },
            payload_format: SPIRE_PAYLOAD_FORMAT_NONE,
            gamma: 0.0,
            encoded_payload: Vec::new(),
        };

        assert!(SpireDeltaPartitionObject::new(
            19,
            4,
            17,
            vec![insert_row.clone(), delete_row.clone()],
        )
        .is_err());

        let header = SpirePartitionObjectHeader {
            kind: SpirePartitionObjectKind::Delta,
            pid: 19,
            object_version: 4,
            published_epoch_backref: 7,
            level: 0,
            parent_pid: 17,
            child_count: 0,
            assignment_count: 2,
            flags: 0,
        };
        let mut encoded = header.encode().unwrap();
        encoded.extend_from_slice(&insert_row.encode().unwrap());
        encoded.extend_from_slice(&delete_row.encode().unwrap());

        assert!(SpireDeltaPartitionObject::decode(&encoded).is_err());
    }
