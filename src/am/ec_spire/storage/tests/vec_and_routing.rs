    #[test]
    fn miri_local_vec_id_round_trips_sequence() {
        let vec_id = SpireVecId::local(42);

        assert_eq!(vec_id.discriminator(), SPIRE_LOCAL_VEC_ID_DISCRIMINATOR);
        assert_eq!(vec_id.local_sequence(), Some(42));
        assert_eq!(
            SpireVecId::from_bytes(vec_id.as_bytes())
                .unwrap()
                .local_sequence(),
            Some(42)
        );
    }

    #[test]
    fn miri_vec_id_rejects_invalid_shapes() {
        assert!(SpireVecId::from_bytes(&[]).is_err());
        assert!(SpireVecId::from_bytes(&[0xff, 1, 2]).is_err());
        assert!(SpireVecId::from_bytes(&[SPIRE_LOCAL_VEC_ID_DISCRIMINATOR, 1]).is_err());
        assert!(SpireVecId::from_bytes(SpireVecId::local(0).as_bytes()).is_err());
        assert!(SpireVecId::from_bytes(&[SPIRE_GLOBAL_VEC_ID_DISCRIMINATOR]).is_err());
        assert!(SpireVecId::global(&vec![7; SPIRE_VEC_ID_MAX_BYTES]).is_err());
    }

    #[test]
    fn miri_global_vec_id_preserves_payload() {
        let vec_id = SpireVecId::global(&[9, 8, 7]).unwrap();

        assert_eq!(vec_id.discriminator(), SPIRE_GLOBAL_VEC_ID_DISCRIMINATOR);
        assert_eq!(
            vec_id.as_bytes(),
            &[SPIRE_GLOBAL_VEC_ID_DISCRIMINATOR, 9, 8, 7]
        );
        assert_eq!(SpireVecId::from_bytes(vec_id.as_bytes()).unwrap(), vec_id);
    }

    #[test]
    fn miri_global_vec_id_max_payload_is_accepted() {
        // Boundary: discriminator + payload must fit in SPIRE_VEC_ID_MAX_BYTES.
        // Sister test miri_vec_id_rejects_invalid_shapes pins the just-too-big
        // case; this pins the just-fits case so a `> → >=` mutation on the
        // length guard is observable.
        let max_payload = SPIRE_VEC_ID_MAX_BYTES - 1;
        let vec_id = SpireVecId::global(&vec![7_u8; max_payload]).unwrap();
        assert_eq!(vec_id.as_bytes().len(), SPIRE_VEC_ID_MAX_BYTES);
        assert_eq!(vec_id.discriminator(), SPIRE_GLOBAL_VEC_ID_DISCRIMINATOR);
    }

    #[test]
    fn miri_vec_id_local_sequence_is_none_for_global() {
        let global = SpireVecId::global(&[9, 8, 7]).unwrap();
        assert!(global.local_sequence().is_none());

        // Round-trip through from_bytes preserves the None path.
        let reparsed = SpireVecId::from_bytes(global.as_bytes()).unwrap();
        assert!(reparsed.local_sequence().is_none());
    }

    #[test]
    fn partition_object_header_decodes_prefix_and_payload_tail() {
        let header = SpirePartitionObjectHeader {
            kind: SpirePartitionObjectKind::Leaf,
            pid: 17,
            object_version: 3,
            published_epoch_backref: 7,
            level: 1,
            parent_pid: 5,
            child_count: 0,
            assignment_count: 99,
            flags: 0x10,
        };
        let mut encoded = header.encode().unwrap();
        encoded.extend_from_slice(&[1, 2, 3]);

        let (decoded, tail) = SpirePartitionObjectHeader::decode_prefix(&encoded).unwrap();

        assert_eq!(decoded, header);
        assert_eq!(tail, &[1, 2, 3]);
    }

    #[test]
    fn partition_object_header_rejects_invalid_identity() {
        let mut header = SpirePartitionObjectHeader {
            kind: SpirePartitionObjectKind::Internal,
            pid: 0,
            object_version: 1,
            published_epoch_backref: 7,
            level: 0,
            parent_pid: 0,
            child_count: 1,
            assignment_count: 0,
            flags: 0,
        };
        assert!(header.encode().is_err());
        header.pid = 1;
        header.object_version = 0;
        assert!(header.encode().is_err());
    }

    #[test]
    fn partition_object_constructors_reject_invalid_header_identity() {
        let row = leaf_v2_assignment(1, 8);

        assert!(SpireLeafPartitionObject::new(0, 3, 0, vec![row.clone()]).is_err());
        assert!(SpireLeafPartitionObject::new(17, 0, 0, vec![row]).is_err());
        assert!(SpireDeltaPartitionObject::new(0, 4, 17, Vec::new()).is_err());
        assert!(SpireDeltaPartitionObject::new(19, 0, 17, Vec::new()).is_err());
        assert!(SpireRoutingPartitionObject::root(0, 3, 2, routing_children()).is_err());
        assert!(SpireRoutingPartitionObject::root(11, 0, 2, routing_children()).is_err());
    }

    #[test]
    fn routing_partition_object_round_trips_root_children() {
        let object = SpireRoutingPartitionObject::root(11, 3, 2, routing_children()).unwrap();

        let decoded = SpireRoutingPartitionObject::decode(&object.encode().unwrap()).unwrap();

        assert_eq!(decoded, object);
        assert_eq!(decoded.header.kind, SpirePartitionObjectKind::Root);
        assert_eq!(decoded.header.level, 1);
        assert_eq!(decoded.header.parent_pid, 0);
        assert_eq!(decoded.header.child_count, 2);
        assert_eq!(decoded.header.assignment_count, 0);
        assert_eq!(decoded.child_pids[0], 17);
        assert_eq!(decoded.child_centroid(1).unwrap(), &[-1.0, 0.0]);
    }

    #[test]
    fn routing_partition_object_round_trips_internal_children() {
        let object =
            SpireRoutingPartitionObject::internal(12, 4, 2, 11, 2, routing_children()).unwrap();

        let decoded = SpireRoutingPartitionObject::decode(&object.encode().unwrap()).unwrap();

        assert_eq!(decoded, object);
        assert_eq!(decoded.header.kind, SpirePartitionObjectKind::Internal);
        assert_eq!(decoded.header.level, 2);
        assert_eq!(decoded.header.parent_pid, 11);
    }

    #[test]
    fn local_store_relation_name_is_deterministic() {
        assert_eq!(
            spire_local_store_relation_name(12345, 0).unwrap(),
            "ec_spire_store_12345_0"
        );
        assert_eq!(
            spire_local_store_relation_name(u32::MAX, 16).unwrap(),
            "ec_spire_store_4294967295_16"
        );
        assert!(spire_local_store_relation_name(0, 0).is_err());
    }

    #[test]
    fn miri_vec_id_kind_decode_accepts_known_and_rejects_unknown() {
        assert_eq!(
            super::SpireVecIdKind::decode(1).unwrap(),
            super::SpireVecIdKind::LocalU64,
        );
        assert_eq!(
            super::SpireVecIdKind::decode(2).unwrap(),
            super::SpireVecIdKind::GlobalBytes,
        );
        let err = super::SpireVecIdKind::decode(0)
            .expect_err("0 is not a known vec_id kind");
        assert!(
            err.contains("invalid leaf V2 vec_id kind"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn miri_vec_id_ref_round_trips_local_and_global_through_assignment_row() {
        // SpireVecIdRef::from_bytes and to_owned are reached via
        // SpireLeafAssignmentRow::decode_prefix_ref. Cover both
        // local and global discriminator paths plus the None-for-global
        // local_sequence branch.
        let local_row = SpireLeafAssignmentRow {
            flags: SPIRE_ASSIGNMENT_FLAG_PRIMARY,
            vec_id: SpireVecId::local(99),
            heap_tid: ItemPointer {
                block_number: 12,
                offset_number: 4,
            },
            payload_format: SPIRE_PAYLOAD_FORMAT_TURBOQUANT,
            gamma: 0.5,
            encoded_payload: vec![1, 2, 3],
        };
        let mut local_encoded = local_row.encode().unwrap();
        local_encoded.extend_from_slice(&[0xa, 0xb]);
        let (local_ref, local_tail) =
            SpireLeafAssignmentRow::decode_prefix_ref(&local_encoded).unwrap();
        assert_eq!(
            local_ref.vec_id.discriminator(),
            SPIRE_LOCAL_VEC_ID_DISCRIMINATOR,
        );
        assert_eq!(local_ref.vec_id.as_bytes(), local_row.vec_id.as_bytes());
        assert_eq!(local_ref.vec_id.local_sequence(), Some(99));
        assert_eq!(local_ref.vec_id.to_owned(), local_row.vec_id);
        assert_eq!(local_tail, &[0xa, 0xb]);

        let global_row = SpireLeafAssignmentRow {
            flags: SPIRE_ASSIGNMENT_FLAG_PRIMARY,
            vec_id: SpireVecId::global(&[9, 9, 9]).unwrap(),
            heap_tid: ItemPointer {
                block_number: 24,
                offset_number: 6,
            },
            payload_format: SPIRE_PAYLOAD_FORMAT_TURBOQUANT,
            gamma: 1.25,
            encoded_payload: vec![4, 5],
        };
        let global_encoded = global_row.encode().unwrap();
        let (global_ref, global_tail) =
            SpireLeafAssignmentRow::decode_prefix_ref(&global_encoded).unwrap();
        assert_eq!(
            global_ref.vec_id.discriminator(),
            SPIRE_GLOBAL_VEC_ID_DISCRIMINATOR,
        );
        assert_eq!(global_ref.vec_id.as_bytes(), global_row.vec_id.as_bytes());
        assert!(global_ref.vec_id.local_sequence().is_none());
        assert_eq!(global_ref.vec_id.to_owned(), global_row.vec_id);
        assert!(global_tail.is_empty());
    }

    #[test]
    fn miri_leaf_v2_column_layout_offsets_are_consistent() {
        // Pin the assignment-row and leaf-V2 segment layout const fns so
        // an offset-shift mutation (`+` → `-`, drift in stride math) is
        // observable. The arithmetic is centralised here so production
        // callers can rely on these helpers staying byte-for-byte stable.
        use super::*;
        assert_eq!(spire_assignment_row_heap_tid_offset(9), 3 + 9);
        assert_eq!(spire_assignment_row_payload_format_offset(9), 3 + 9 + ITEM_POINTER_BYTES);
        assert_eq!(
            spire_assignment_row_gamma_offset(9),
            3 + 9 + ITEM_POINTER_BYTES + 1,
        );
        assert_eq!(
            spire_assignment_row_payload_len_offset(9),
            3 + 9 + ITEM_POINTER_BYTES + 1 + 4,
        );
        assert_eq!(
            spire_assignment_row_payload_offset(9),
            3 + 9 + ITEM_POINTER_BYTES + 1 + 4 + 4,
        );

        let prefix = SPIRE_LEAF_V2_SEGMENT_PREFIX_BYTES;
        assert_eq!(spire_leaf_v2_segment_vec_ids_offset(4), prefix + 4 * 2);
        assert_eq!(
            spire_leaf_v2_segment_heap_tids_offset(4, 16),
            prefix + 4 * 2 + 4 * 16,
        );
        assert_eq!(
            spire_leaf_v2_segment_gammas_offset(4, 16),
            prefix + 4 * 2 + 4 * 16 + 4 * ITEM_POINTER_BYTES,
        );
        assert_eq!(
            spire_leaf_v2_segment_payloads_offset(4, 16),
            prefix + 4 * 2 + 4 * 16 + 4 * ITEM_POINTER_BYTES + 4 * 4,
        );
    }

    #[test]
    fn miri_leaf_object_columns_row_rejects_out_of_range_offset() {
        use super::{SpireLeafObjectColumns, SpirePartitionObjectKind};
        let header = SpirePartitionObjectHeader {
            kind: SpirePartitionObjectKind::Leaf,
            pid: 7,
            object_version: 1,
            published_epoch_backref: 1,
            level: 0,
            parent_pid: 0,
            child_count: 0,
            assignment_count: 2,
            flags: 0,
        };
        let columns = SpireLeafObjectColumns {
            header,
            payload_format: SPIRE_PAYLOAD_FORMAT_TURBOQUANT,
            payload_stride: 4,
            vec_id_kind: SpireVecIdKind::LocalU64,
            vec_id_stride: LEAF_V2_LOCAL_VEC_ID_STRIDE,
            row_base: 0,
            flags: &[SPIRE_ASSIGNMENT_FLAG_PRIMARY, SPIRE_ASSIGNMENT_FLAG_PRIMARY],
            // Local vec_ids: two 16-byte slots (discriminator + 8-byte LE seq).
            vec_ids: &[
                SPIRE_LOCAL_VEC_ID_DISCRIMINATOR, 1, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0,
                SPIRE_LOCAL_VEC_ID_DISCRIMINATOR, 2, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0,
            ],
            heap_tids: &[
                ItemPointer { block_number: 100, offset_number: 1 },
                ItemPointer { block_number: 101, offset_number: 2 },
            ],
            gammas: &[0.5, 0.75],
            payloads: &[1, 2, 3, 4, 5, 6, 7, 8],
        };
        assert_eq!(columns.row_count(), 2);
        let row_zero = columns.row(0).unwrap();
        assert_eq!(row_zero.row_index, 0);
        assert_eq!(row_zero.gamma, 0.5);
        assert_eq!(row_zero.heap_tid.block_number, 100);
        assert_eq!(row_zero.vec_id_bytes.len(), LEAF_V2_LOCAL_VEC_ID_STRIDE);
        assert_eq!(row_zero.encoded_payload, &[1, 2, 3, 4]);
        assert_eq!(row_zero.local_vec_seq().unwrap(), 1);
        assert_eq!(row_zero.vec_id().unwrap(), SpireVecId::local(1));

        let err = columns
            .row(2)
            .expect_err("row offset 2 must be rejected at row_count 2");
        assert!(
            err.contains("exceeds row count"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn miri_leaf_object_columns_row_with_row_base_offsets_row_index() {
        use super::SpireLeafObjectColumns;
        let header = SpirePartitionObjectHeader {
            kind: SpirePartitionObjectKind::Leaf,
            pid: 8,
            object_version: 1,
            published_epoch_backref: 1,
            level: 0,
            parent_pid: 0,
            child_count: 0,
            assignment_count: 1,
            flags: 0,
        };
        // Single global row at row_base 10, so row(0).row_index == 10.
        let columns = SpireLeafObjectColumns {
            header,
            payload_format: SPIRE_PAYLOAD_FORMAT_TURBOQUANT,
            payload_stride: 2,
            vec_id_kind: SpireVecIdKind::GlobalBytes,
            vec_id_stride: 3,
            row_base: 10,
            flags: &[SPIRE_ASSIGNMENT_FLAG_PRIMARY],
            vec_ids: &[SPIRE_GLOBAL_VEC_ID_DISCRIMINATOR, 0xab, 0xcd],
            heap_tids: &[ItemPointer { block_number: 1, offset_number: 1 }],
            gammas: &[2.0],
            payloads: &[0x11, 0x22],
        };
        let row = columns.row(0).unwrap();
        assert_eq!(row.row_index, 10);
        assert_eq!(row.encoded_payload, &[0x11, 0x22]);
        let decoded = row.vec_id().unwrap();
        assert_eq!(decoded.discriminator(), SPIRE_GLOBAL_VEC_ID_DISCRIMINATOR);
        assert_eq!(decoded.as_bytes()[1..], [0xab, 0xcd]);
        // GlobalBytes does not have a local sequence; the decoder must
        // error so callers do not silently produce a wrong u64.
        assert!(row.local_vec_seq().is_err());
    }
