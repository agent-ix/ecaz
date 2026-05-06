    #[test]
    fn routing_partition_object_rejects_invalid_shape() {
        assert!(SpireRoutingPartitionObject::root(11, 3, 0, routing_children()).is_err());
        assert!(SpireRoutingPartitionObject::root(11, 3, 2, Vec::new()).is_err());
        assert!(SpireRoutingPartitionObject::internal(12, 4, 2, 0, 2, routing_children()).is_err());

        let mut children = routing_children();
        children[1].centroid_index = 7;
        assert!(SpireRoutingPartitionObject::root(11, 3, 2, children).is_err());

        let mut children = routing_children();
        children[0].child_pid = 0;
        assert!(SpireRoutingPartitionObject::root(11, 3, 2, children).is_err());

        let mut children = routing_children();
        children[0].centroid = vec![1.0];
        assert!(SpireRoutingPartitionObject::root(11, 3, 2, children).is_err());

        let mut children = routing_children();
        children[0].centroid = vec![f32::NAN, 0.0];
        assert!(SpireRoutingPartitionObject::root(11, 3, 2, children).is_err());
    }

    #[test]
    fn routing_partition_object_rejects_corrupt_header_and_body() {
        let object = SpireRoutingPartitionObject::root(11, 3, 2, routing_children()).unwrap();
        let mut wrong_count = object.clone();
        wrong_count.header.child_count = 1;
        assert!(wrong_count.encode().is_err());

        let mut wrong_kind = object.clone();
        wrong_kind.header.kind = SpirePartitionObjectKind::Leaf;
        assert!(wrong_kind.encode().is_err());

        let mut encoded = object.encode().unwrap();
        encoded.truncate(encoded.len() - 1);
        assert!(SpireRoutingPartitionObject::decode(&encoded).is_err());

        let mut encoded = object.encode().unwrap();
        encoded[48] = 1;
        assert!(SpireRoutingPartitionObject::decode(&encoded).is_err());
    }

    #[test]
    fn assignment_row_round_trips() {
        let row = SpireLeafAssignmentRow {
            flags: SPIRE_ASSIGNMENT_FLAG_PRIMARY | SPIRE_ASSIGNMENT_FLAG_BOUNDARY_REPLICA,
            vec_id: SpireVecId::local(123),
            heap_tid: ItemPointer {
                block_number: 44,
                offset_number: 7,
            },
            payload_format: SPIRE_PAYLOAD_FORMAT_PQ_FASTSCAN,
            gamma: 1.25,
            encoded_payload: vec![4, 5, 6],
        };

        let decoded = SpireLeafAssignmentRow::decode(&row.encode().unwrap()).unwrap();

        assert_eq!(decoded, row);
    }

    #[test]
    fn assignment_row_prefix_decoder_returns_tail() {
        let row = SpireLeafAssignmentRow {
            flags: SPIRE_ASSIGNMENT_FLAG_PRIMARY,
            vec_id: SpireVecId::local(123),
            heap_tid: ItemPointer {
                block_number: 44,
                offset_number: 7,
            },
            payload_format: SPIRE_PAYLOAD_FORMAT_PQ_FASTSCAN,
            gamma: 1.25,
            encoded_payload: vec![4, 5, 6],
        };
        let mut encoded = row.encode().unwrap();
        encoded.extend_from_slice(&[9, 8]);

        let (decoded, tail) = SpireLeafAssignmentRow::decode_prefix(&encoded).unwrap();

        assert_eq!(decoded, row);
        assert_eq!(tail, &[9, 8]);
        assert!(SpireLeafAssignmentRow::decode(&encoded).is_err());
    }

    #[test]
    fn assignment_row_ref_prefix_decoder_borrows_payload() {
        let row = SpireLeafAssignmentRow {
            flags: SPIRE_ASSIGNMENT_FLAG_PRIMARY,
            vec_id: SpireVecId::local(123),
            heap_tid: ItemPointer {
                block_number: 44,
                offset_number: 7,
            },
            payload_format: SPIRE_PAYLOAD_FORMAT_PQ_FASTSCAN,
            gamma: 1.25,
            encoded_payload: vec![4, 5, 6],
        };
        let mut encoded = row.encode().unwrap();
        encoded.extend_from_slice(&[9, 8]);

        let (row_ref, tail) = SpireLeafAssignmentRow::decode_prefix_ref(&encoded).unwrap();

        assert_eq!(row_ref.flags, row.flags);
        assert_eq!(row_ref.vec_id.local_sequence(), Some(123));
        assert_eq!(row_ref.heap_tid, row.heap_tid);
        assert_eq!(row_ref.payload_format, row.payload_format);
        assert_eq!(row_ref.gamma, row.gamma);
        assert_eq!(row_ref.encoded_payload, row.encoded_payload.as_slice());
        assert_eq!(row_ref.to_owned(), row);
        assert_eq!(tail, &[9, 8]);
    }

    #[test]
    fn assignment_visibility_helpers_match_primary_and_delta_semantics() {
        let mut row = leaf_v2_assignment(1, 8);
        assert!(is_visible_primary_assignment(&row));
        let encoded = row.encode().unwrap();
        let (row_ref, _) =
            SpireLeafAssignmentRow::decode_prefix_ref(&encoded).expect("row ref decodes");
        assert!(is_visible_primary_assignment_ref(&row_ref));

        row.flags = SPIRE_ASSIGNMENT_FLAG_PRIMARY | SPIRE_ASSIGNMENT_FLAG_BOUNDARY_REPLICA;
        assert!(!is_visible_primary_assignment(&row));
        row.flags = SPIRE_ASSIGNMENT_FLAG_PRIMARY | SPIRE_ASSIGNMENT_FLAG_TOMBSTONE;
        assert!(!is_visible_primary_assignment(&row));
        row.flags = SPIRE_ASSIGNMENT_FLAG_PRIMARY | SPIRE_ASSIGNMENT_FLAG_STALE_LOCATOR;
        assert!(!is_visible_primary_assignment(&row));

        row.flags = SPIRE_ASSIGNMENT_FLAG_TOMBSTONE | SPIRE_ASSIGNMENT_FLAG_DELTA_DELETE;
        row.payload_format = SPIRE_PAYLOAD_FORMAT_NONE;
        row.gamma = 0.0;
        row.encoded_payload.clear();
        assert!(is_delete_delta_assignment(&row));
        assert!(!is_visible_primary_assignment(&row));
    }

    #[test]
    fn assignment_row_rejects_unknown_flags_and_invalid_locator() {
        let mut row = SpireLeafAssignmentRow {
            flags: 0x8000,
            vec_id: SpireVecId::local(1),
            heap_tid: ItemPointer {
                block_number: 1,
                offset_number: 1,
            },
            payload_format: SPIRE_PAYLOAD_FORMAT_NONE,
            gamma: 0.0,
            encoded_payload: Vec::new(),
        };
        assert!(row.encode().is_err());

        row.flags = SPIRE_ASSIGNMENT_FLAG_PRIMARY;
        row.heap_tid = ItemPointer::INVALID;
        assert!(row.encode().is_err());
    }

    #[test]
    fn assignment_row_rejects_unknown_payload_format() {
        let row = SpireLeafAssignmentRow {
            flags: SPIRE_ASSIGNMENT_FLAG_PRIMARY,
            vec_id: SpireVecId::local(1),
            heap_tid: ItemPointer {
                block_number: 1,
                offset_number: 1,
            },
            payload_format: SPIRE_PAYLOAD_FORMAT_TURBOQUANT,
            gamma: 0.5,
            encoded_payload: vec![1, 2, 3],
        };

        let mut invalid = row.clone();
        invalid.payload_format = 99;
        assert!(invalid.encode().is_err());

        let mut encoded = row.encode().unwrap();
        let payload_format_offset = 3 + row.vec_id.as_bytes().len() + ITEM_POINTER_BYTES;
        encoded[payload_format_offset] = 99;
        assert!(SpireLeafAssignmentRow::decode(&encoded).is_err());
    }

    #[test]
    fn assignment_row_rejects_length_mismatch() {
        let row = SpireLeafAssignmentRow {
            flags: SPIRE_ASSIGNMENT_FLAG_PRIMARY,
            vec_id: SpireVecId::local(1),
            heap_tid: ItemPointer {
                block_number: 1,
                offset_number: 1,
            },
            payload_format: SPIRE_PAYLOAD_FORMAT_NONE,
            gamma: 0.0,
            encoded_payload: vec![1, 2, 3],
        };
        let mut encoded = row.encode().unwrap();
        encoded.pop();

        assert!(SpireLeafAssignmentRow::decode(&encoded).is_err());
    }

