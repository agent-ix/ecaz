    #[test]
    fn leaf_partition_object_round_trips_assignments() {
        let assignments = vec![
            SpireLeafAssignmentRow {
                flags: SPIRE_ASSIGNMENT_FLAG_PRIMARY,
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
                flags: SPIRE_ASSIGNMENT_FLAG_PRIMARY | SPIRE_ASSIGNMENT_FLAG_BOUNDARY_REPLICA,
                vec_id: SpireVecId::local(2),
                heap_tid: ItemPointer {
                    block_number: 10,
                    offset_number: 2,
                },
                payload_format: 1,
                gamma: 0.75,
                encoded_payload: vec![3, 4],
            },
        ];
        let object = SpireLeafPartitionObject::new(17, 3, 5, assignments).unwrap();

        let decoded = SpireLeafPartitionObject::decode(&object.encode().unwrap()).unwrap();

        assert_eq!(decoded, object);
        assert_eq!(decoded.header.pid, 17);
        assert_eq!(decoded.header.assignment_count, 2);
    }

    #[test]
    fn leaf_partition_object_v2_store_segments_large_leaf() {
        let mut store = SpireLocalObjectStore::new(99, 512).unwrap();
        let assignments = (1..=13)
            .map(|local_vec_seq| leaf_v2_assignment(local_vec_seq, 64))
            .collect::<Vec<_>>();

        let placement = store
            .insert_leaf_object_v2_from_rows(7, 17, 3, 5, &assignments)
            .unwrap();
        let decoded = store.read_leaf_object_v2(&placement).unwrap();
        let header = store.read_object_header(&placement).unwrap();

        assert_eq!(header.kind, SpirePartitionObjectKind::Leaf);
        assert_eq!(header.pid, 17);
        assert_eq!(header.object_version, 3);
        assert_eq!(header.published_epoch_backref, 7);
        assert_eq!(header.parent_pid, 5);
        assert_eq!(header.assignment_count, assignments.len() as u32);
        assert_eq!(decoded.meta.header.pid, 17);
        assert_eq!(decoded.meta.header.object_version, 3);
        assert_eq!(decoded.meta.header.published_epoch_backref, 7);
        assert_eq!(decoded.meta.header.parent_pid, 5);
        assert_eq!(
            decoded.meta.header.assignment_count,
            assignments.len() as u32
        );
        assert_eq!(
            decoded.meta.object_bytes_total,
            u64::from(placement.object_bytes)
        );
        assert!(decoded.meta.segment_count > 1);
        assert_ne!(decoded.meta.first_segment_locator, ItemPointer::INVALID);
        assert!(store.page_count() > 1);

        let mut decoded_row_count = 0_usize;
        for segment in &decoded.segments {
            decoded_row_count += segment.flags.len();
            assert_eq!(segment.flags.len(), segment.heap_tids.len());
            assert_eq!(segment.flags.len(), segment.gammas.len());
            assert_eq!(segment.vec_ids.len(), segment.flags.len() * 16);
            assert_eq!(segment.payloads.len(), segment.flags.len() * 64);
        }
        assert_eq!(decoded_row_count, assignments.len());

        let column_segments = decoded
            .column_segments()
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(column_segments.len(), decoded.segments.len());
        assert_eq!(
            column_segments[0].payload_format,
            SPIRE_PAYLOAD_FORMAT_TURBOQUANT
        );
        assert_eq!(column_segments[0].payload_stride, 64);
        assert_eq!(column_segments[0].vec_id_kind, SpireVecIdKind::LocalU64);
        assert_eq!(column_segments[0].vec_id_stride, 16);
        let first_row = column_segments[0].row(0).unwrap();
        assert_eq!(first_row.row_index, 0);
        assert_eq!(first_row.flags, SPIRE_ASSIGNMENT_FLAG_PRIMARY);
        assert_eq!(first_row.local_vec_seq().unwrap(), 1);
        assert_eq!(first_row.heap_tid, assignments[0].heap_tid);
        assert_eq!(first_row.gamma, assignments[0].gamma);
        assert_eq!(first_row.encoded_payload, assignments[0].encoded_payload);

        let first_vec_id = decode_leaf_v2_local_vec_id(&decoded.segments[0].vec_ids[0..16])
            .expect("first local vec_id decodes");
        assert_eq!(first_vec_id, 1);
        let last = decoded.segments.last().expect("segments are present");
        let last_columns = column_segments.last().expect("column segments are present");
        let last_row = last_columns.row(last_columns.row_count() - 1).unwrap();
        assert_eq!(last_row.local_vec_seq().unwrap(), 13);
        assert_eq!(last_row.heap_tid, assignments[12].heap_tid);
        assert!(last_columns.row(last_columns.row_count()).is_err());
        let last_vec_id_start = (last.flags.len() - 1) * 16;
        let last_vec_id =
            decode_leaf_v2_local_vec_id(&last.vec_ids[last_vec_id_start..last_vec_id_start + 16])
                .expect("last local vec_id decodes");
        assert_eq!(last_vec_id, 13);
        assert_eq!(last.next_segment_locator, ItemPointer::INVALID);
    }

    #[test]
    fn leaf_partition_object_v2_store_preserves_empty_leaf_without_segments() {
        let mut store = SpireLocalObjectStore::new(99, 512).unwrap();

        let placement = store
            .insert_leaf_object_v2_from_rows(7, 17, 3, 5, &[])
            .unwrap();
        let decoded = store.read_leaf_object_v2(&placement).unwrap();

        assert_eq!(decoded.meta.header.assignment_count, 0);
        assert_eq!(decoded.meta.segment_count, 0);
        assert_eq!(decoded.meta.first_segment_locator, ItemPointer::INVALID);
        assert_eq!(decoded.meta.payload_format, SPIRE_PAYLOAD_FORMAT_NONE);
        assert!(decoded.segments.is_empty());
        assert_eq!(decoded.column_segments().unwrap().count(), 0);
    }

    #[test]
    fn leaf_partition_object_v2_rejects_mixed_payload_or_global_vec_id() {
        let mut store = SpireLocalObjectStore::new(99, 512).unwrap();
        let mut mixed_stride = vec![leaf_v2_assignment(1, 8), leaf_v2_assignment(2, 16)];
        assert!(store
            .insert_leaf_object_v2_from_rows(7, 17, 3, 5, &mixed_stride)
            .is_err());

        mixed_stride[1] = leaf_v2_assignment(2, 8);
        mixed_stride[1].payload_format = SPIRE_PAYLOAD_FORMAT_PQ_FASTSCAN;
        assert!(store
            .insert_leaf_object_v2_from_rows(7, 17, 3, 5, &mixed_stride)
            .is_err());

        let mut global_row = leaf_v2_assignment(1, 8);
        global_row.vec_id = SpireVecId::global(&[9, 9, 9]).unwrap();
        let err = store
            .insert_leaf_object_v2_from_rows(7, 17, 3, 5, &[global_row])
            .unwrap_err();
        assert!(err.contains("global writer IDs need a future variable-width Leaf V2 format"));
    }

    #[test]
    fn leaf_partition_object_rejects_non_leaf_header_and_children() {
        let row = SpireLeafAssignmentRow {
            flags: SPIRE_ASSIGNMENT_FLAG_PRIMARY,
            vec_id: SpireVecId::local(1),
            heap_tid: ItemPointer {
                block_number: 10,
                offset_number: 1,
            },
            payload_format: 1,
            gamma: 0.5,
            encoded_payload: vec![1, 2],
        };
        let mut object = SpireLeafPartitionObject::new(17, 3, 0, vec![row]).unwrap();

        object.header.kind = SpirePartitionObjectKind::Internal;
        assert!(object.encode().is_err());

        object.header.kind = SpirePartitionObjectKind::Leaf;
        object.header.child_count = 1;
        assert!(object.encode().is_err());
    }

    #[test]
    fn leaf_partition_object_rejects_delta_flags() {
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

        assert!(SpireLeafPartitionObject::new(17, 3, 0, vec![row.clone()]).is_err());

        let header = SpirePartitionObjectHeader {
            kind: SpirePartitionObjectKind::Leaf,
            pid: 17,
            object_version: 3,
            published_epoch_backref: 7,
            level: 0,
            parent_pid: 0,
            child_count: 0,
            assignment_count: 1,
            flags: 0,
        };
        let mut encoded = header.encode().unwrap();
        encoded.extend_from_slice(&row.encode().unwrap());

        assert!(SpireLeafPartitionObject::decode(&encoded).is_err());
    }

    #[test]
    fn leaf_partition_object_rejects_missing_role_flags() {
        let row = SpireLeafAssignmentRow {
            flags: 0,
            vec_id: SpireVecId::local(1),
            heap_tid: ItemPointer {
                block_number: 10,
                offset_number: 1,
            },
            payload_format: 1,
            gamma: 0.5,
            encoded_payload: vec![1, 2],
        };

        assert!(SpireLeafPartitionObject::new(17, 3, 0, vec![row.clone()]).is_err());

        let header = SpirePartitionObjectHeader {
            kind: SpirePartitionObjectKind::Leaf,
            pid: 17,
            object_version: 3,
            published_epoch_backref: 7,
            level: 0,
            parent_pid: 0,
            child_count: 0,
            assignment_count: 1,
            flags: 0,
        };
        let mut encoded = header.encode().unwrap();
        encoded.extend_from_slice(&row.encode().unwrap());

        assert!(SpireLeafPartitionObject::decode(&encoded).is_err());
    }

    #[test]
    fn leaf_partition_object_rejects_scored_assignments_without_payload() {
        let valid_row = SpireLeafAssignmentRow {
            flags: SPIRE_ASSIGNMENT_FLAG_PRIMARY,
            vec_id: SpireVecId::local(1),
            heap_tid: ItemPointer {
                block_number: 10,
                offset_number: 1,
            },
            payload_format: SPIRE_PAYLOAD_FORMAT_TURBOQUANT,
            gamma: 0.5,
            encoded_payload: vec![1, 2],
        };

        let mut row = valid_row.clone();
        row.payload_format = SPIRE_PAYLOAD_FORMAT_NONE;
        assert!(SpireLeafPartitionObject::new(17, 3, 0, vec![row]).is_err());

        row = valid_row;
        row.encoded_payload.clear();
        assert!(SpireLeafPartitionObject::new(17, 3, 0, vec![row]).is_err());
    }

    #[test]
    fn leaf_partition_object_rejects_count_mismatch_and_trailing_bytes() {
        let row = SpireLeafAssignmentRow {
            flags: SPIRE_ASSIGNMENT_FLAG_PRIMARY,
            vec_id: SpireVecId::local(1),
            heap_tid: ItemPointer {
                block_number: 10,
                offset_number: 1,
            },
            payload_format: 1,
            gamma: 0.5,
            encoded_payload: vec![1, 2],
        };
        let mut object = SpireLeafPartitionObject::new(17, 3, 0, vec![row]).unwrap();

        object.header.assignment_count = 2;
        assert!(object.encode().is_err());

        object.header.assignment_count = 1;
        let mut encoded = object.encode().unwrap();
        encoded.push(99);
        assert!(SpireLeafPartitionObject::decode(&encoded).is_err());
    }

    #[test]
    fn leaf_partition_object_rejects_duplicate_vec_ids() {
        let primary_row = SpireLeafAssignmentRow {
            flags: SPIRE_ASSIGNMENT_FLAG_PRIMARY,
            vec_id: SpireVecId::local(1),
            heap_tid: ItemPointer {
                block_number: 10,
                offset_number: 1,
            },
            payload_format: 1,
            gamma: 0.5,
            encoded_payload: vec![1, 2],
        };
        let boundary_row = SpireLeafAssignmentRow {
            flags: SPIRE_ASSIGNMENT_FLAG_PRIMARY | SPIRE_ASSIGNMENT_FLAG_BOUNDARY_REPLICA,
            vec_id: SpireVecId::local(1),
            heap_tid: ItemPointer {
                block_number: 10,
                offset_number: 2,
            },
            payload_format: 1,
            gamma: 0.75,
            encoded_payload: vec![3, 4],
        };

        assert!(SpireLeafPartitionObject::new(
            17,
            3,
            0,
            vec![primary_row.clone(), boundary_row.clone()],
        )
        .is_err());

        let header = SpirePartitionObjectHeader {
            kind: SpirePartitionObjectKind::Leaf,
            pid: 17,
            object_version: 3,
            published_epoch_backref: 7,
            level: 0,
            parent_pid: 0,
            child_count: 0,
            assignment_count: 2,
            flags: 0,
        };
        let mut encoded = header.encode().unwrap();
        encoded.extend_from_slice(&primary_row.encode().unwrap());
        encoded.extend_from_slice(&boundary_row.encode().unwrap());

        assert!(SpireLeafPartitionObject::decode(&encoded).is_err());
    }
