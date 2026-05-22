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
    fn miri_assignment_row_round_trips() {
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
    fn miri_assignment_row_prefix_decoder_returns_tail() {
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
    fn miri_assignment_row_ref_prefix_decoder_borrows_payload() {
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
    fn miri_assignment_visibility_helpers_match_primary_and_delta_semantics() {
        let mut row = leaf_v2_assignment(1, 8);
        assert!(is_visible_primary_assignment(&row));
        let encoded = row.encode().unwrap();
        let (row_ref, _) =
            SpireLeafAssignmentRow::decode_prefix_ref(&encoded).expect("row ref decodes");
        assert!(is_visible_primary_assignment_ref(&row_ref));

        row.flags = SPIRE_ASSIGNMENT_FLAG_PRIMARY | SPIRE_ASSIGNMENT_FLAG_BOUNDARY_REPLICA;
        assert!(!is_visible_primary_assignment(&row));
        assert!(is_visible_scored_assignment(&row));
        row.flags = SPIRE_ASSIGNMENT_FLAG_BOUNDARY_REPLICA;
        assert!(is_visible_scored_assignment(&row));
        row.flags = SPIRE_ASSIGNMENT_FLAG_PRIMARY | SPIRE_ASSIGNMENT_FLAG_TOMBSTONE;
        assert!(!is_visible_primary_assignment(&row));
        assert!(!is_visible_scored_assignment(&row));
        row.flags = SPIRE_ASSIGNMENT_FLAG_PRIMARY | SPIRE_ASSIGNMENT_FLAG_STALE_LOCATOR;
        assert!(!is_visible_primary_assignment(&row));
        assert!(!is_visible_scored_assignment(&row));

        row.flags = SPIRE_ASSIGNMENT_FLAG_TOMBSTONE | SPIRE_ASSIGNMENT_FLAG_DELTA_DELETE;
        row.payload_format = SPIRE_PAYLOAD_FORMAT_NONE;
        row.gamma = 0.0;
        row.encoded_payload.clear();
        assert!(is_delete_delta_assignment(&row));
        assert!(!is_visible_primary_assignment(&row));
        assert!(!is_visible_scored_assignment(&row));
    }

    #[test]
    fn miri_assignment_row_rejects_unknown_flags_and_invalid_locator() {
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
    fn miri_assignment_row_rejects_unknown_payload_format() {
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
    fn miri_assignment_row_rejects_length_mismatch() {
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

    #[test]
    fn miri_assignment_row_decode_rejects_zero_vec_id_len_at_min_prefix_boundary() {
        // Input length exactly equals
        // SPIRE_ASSIGNMENT_ROW_FIXED_PREFIX_BYTES + SPIRE_ASSIGNMENT_ROW_FIXED_TAIL_BYTES,
        // probing the boundary check on line 106. All bytes zero so
        // vec_id_len == input[2] == 0 → original errors with "vec_id length 0".
        let buf =
            vec![0u8; SPIRE_ASSIGNMENT_ROW_FIXED_PREFIX_BYTES + SPIRE_ASSIGNMENT_ROW_FIXED_TAIL_BYTES];
        let err = SpireLeafAssignmentRow::decode_prefix_ref(&buf).unwrap_err();
        assert!(
            err.contains("vec_id length 0 is invalid"),
            "expected post-boundary vec_id-len error, got {err}"
        );
    }

    #[test]
    fn miri_assignment_row_decode_rejects_vec_id_len_above_max() {
        // input[2] = SPIRE_VEC_ID_MAX_BYTES + 1 (clearly above max) to kill
        // `> -> ==` on line 116:42 (which would only reject len == MAX exactly).
        let mut buf = vec![0u8; SPIRE_ASSIGNMENT_ROW_FIXED_PREFIX_BYTES
            + SPIRE_ASSIGNMENT_ROW_FIXED_TAIL_BYTES
            + SPIRE_VEC_ID_MAX_BYTES
            + 1];
        buf[2] = (SPIRE_VEC_ID_MAX_BYTES + 1) as u8;
        let err = SpireLeafAssignmentRow::decode_prefix_ref(&buf).unwrap_err();
        assert!(
            err.contains("vec_id length"),
            "expected vec_id-length-invalid error, got {err}"
        );
    }

    #[test]
    fn miri_assignment_row_round_trips_with_vec_id_at_max_bytes() {
        // vec_id_len == SPIRE_VEC_ID_MAX_BYTES exactly. Original accepts;
        // `> -> >=` mutant on line 116:42 rejects.
        // Global vec_id wraps a payload with a 1-byte discriminator, so the
        // max payload that yields a 32-byte vec_id is SPIRE_VEC_ID_MAX_BYTES - 1.
        let max_payload: Vec<u8> = (0u8..((SPIRE_VEC_ID_MAX_BYTES - 1) as u8))
            .map(|i| i.saturating_add(1))
            .collect();
        let row = SpireLeafAssignmentRow {
            flags: SPIRE_ASSIGNMENT_FLAG_PRIMARY,
            vec_id: SpireVecId::global(&max_payload).expect("max-length global vec_id"),
            heap_tid: ItemPointer {
                block_number: 1,
                offset_number: 1,
            },
            payload_format: SPIRE_PAYLOAD_FORMAT_NONE,
            gamma: 0.0,
            encoded_payload: vec![],
        };
        let encoded = row.encode().expect("encode should succeed at max vec_id length");
        let decoded = SpireLeafAssignmentRow::decode(&encoded)
            .expect("decode should succeed at max vec_id length");
        assert_eq!(decoded.vec_id.as_bytes().len(), SPIRE_VEC_ID_MAX_BYTES);
    }
