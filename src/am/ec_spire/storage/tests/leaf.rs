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
fn miri_leaf_v2_meta_rejects_invalid_validate_inputs() {
    // Baseline: a known-valid arg list for SpireLeafPartitionObjectV2Meta::new.
    // Each sub-case clones the baseline and twists one field so a single
    // error branch in validate() is the only thing that fires.
    let valid_segment = ItemPointer {
        block_number: 10,
        offset_number: 1,
    };
    let build = |published_epoch_backref: u64,
                 object_bytes_total: u64,
                 assignment_count: u32,
                 payload_format: u8,
                 payload_stride: u32,
                 vec_id_kind: SpireVecIdKind,
                 vec_id_stride: u16,
                 segment_count: u32,
                 first_segment_locator: ItemPointer| {
        SpireLeafPartitionObjectV2Meta::new(
            17,
            3,
            5,
            assignment_count,
            payload_format,
            payload_stride,
            vec_id_kind,
            vec_id_stride,
            segment_count,
            first_segment_locator,
            object_bytes_total,
            published_epoch_backref,
        )
    };

    // Sanity: the baseline really does succeed.
    assert!(build(
        7,
        256,
        2,
        SPIRE_PAYLOAD_FORMAT_TURBOQUANT,
        2,
        SpireVecIdKind::LocalU64,
        LEAF_V2_LOCAL_VEC_ID_STRIDE as u16,
        1,
        valid_segment,
    )
    .is_ok());

    // published_epoch_backref == 0 is invalid.
    assert!(build(
        0,
        256,
        2,
        SPIRE_PAYLOAD_FORMAT_TURBOQUANT,
        2,
        SpireVecIdKind::LocalU64,
        LEAF_V2_LOCAL_VEC_ID_STRIDE as u16,
        1,
        valid_segment,
    )
    .is_err());

    // object_bytes_total == 0 is invalid.
    assert!(build(
        7,
        0,
        2,
        SPIRE_PAYLOAD_FORMAT_TURBOQUANT,
        2,
        SpireVecIdKind::LocalU64,
        LEAF_V2_LOCAL_VEC_ID_STRIDE as u16,
        1,
        valid_segment,
    )
    .is_err());

    // LocalU64 stride must equal LEAF_V2_LOCAL_VEC_ID_STRIDE.
    assert!(build(
        7,
        256,
        2,
        SPIRE_PAYLOAD_FORMAT_TURBOQUANT,
        2,
        SpireVecIdKind::LocalU64,
        (LEAF_V2_LOCAL_VEC_ID_STRIDE as u16) + 1,
        1,
        valid_segment,
    )
    .is_err());

    // GlobalBytes stride below the 2-byte minimum is rejected.
    assert!(build(
        7,
        256,
        2,
        SPIRE_PAYLOAD_FORMAT_TURBOQUANT,
        2,
        SpireVecIdKind::GlobalBytes,
        1,
        1,
        valid_segment,
    )
    .is_err());

    // GlobalBytes stride above SPIRE_VEC_ID_MAX_BYTES is rejected.
    assert!(build(
        7,
        256,
        2,
        SPIRE_PAYLOAD_FORMAT_TURBOQUANT,
        2,
        SpireVecIdKind::GlobalBytes,
        (SPIRE_VEC_ID_MAX_BYTES as u16) + 1,
        1,
        valid_segment,
    )
    .is_err());

    // Non-empty assignment_count with segment_count == 0 is invalid.
    assert!(build(
        7,
        256,
        2,
        SPIRE_PAYLOAD_FORMAT_TURBOQUANT,
        2,
        SpireVecIdKind::LocalU64,
        LEAF_V2_LOCAL_VEC_ID_STRIDE as u16,
        0,
        valid_segment,
    )
    .is_err());

    // Non-empty meta with INVALID first segment locator is invalid.
    assert!(build(
        7,
        256,
        2,
        SPIRE_PAYLOAD_FORMAT_TURBOQUANT,
        2,
        SpireVecIdKind::LocalU64,
        LEAF_V2_LOCAL_VEC_ID_STRIDE as u16,
        1,
        ItemPointer::INVALID,
    )
    .is_err());

    // Non-empty meta with payload_format == NONE is invalid.
    assert!(build(
        7,
        256,
        2,
        SPIRE_PAYLOAD_FORMAT_NONE,
        2,
        SpireVecIdKind::LocalU64,
        LEAF_V2_LOCAL_VEC_ID_STRIDE as u16,
        1,
        valid_segment,
    )
    .is_err());

    // Non-empty meta with payload_stride == 0 is invalid.
    assert!(build(
        7,
        256,
        2,
        SPIRE_PAYLOAD_FORMAT_TURBOQUANT,
        0,
        SpireVecIdKind::LocalU64,
        LEAF_V2_LOCAL_VEC_ID_STRIDE as u16,
        1,
        valid_segment,
    )
    .is_err());
}

#[test]
fn miri_leaf_v2_empty_meta_rejects_segment_locator() {
    let meta = SpireLeafPartitionObjectV2Meta::new(
        17,
        3,
        5,
        0,
        SPIRE_PAYLOAD_FORMAT_NONE,
        0,
        SpireVecIdKind::LocalU64,
        LEAF_V2_LOCAL_VEC_ID_STRIDE as u16,
        0,
        ItemPointer {
            block_number: 10,
            offset_number: 1,
        },
        54,
        7,
    );

    assert!(meta.is_err());
}

#[test]
fn miri_leaf_v2_segment_rows_roundtrip_through_columns() {
    let meta = SpireLeafPartitionObjectV2Meta::new(
        17,
        3,
        5,
        2,
        SPIRE_PAYLOAD_FORMAT_TURBOQUANT,
        2,
        SpireVecIdKind::LocalU64,
        LEAF_V2_LOCAL_VEC_ID_STRIDE as u16,
        1,
        ItemPointer {
            block_number: 10,
            offset_number: 1,
        },
        256,
        7,
    )
    .unwrap();
    let rows = vec![
        SpireLeafAssignmentRow {
            flags: SPIRE_ASSIGNMENT_FLAG_PRIMARY,
            vec_id: SpireVecId::local(11),
            heap_tid: ItemPointer {
                block_number: 1,
                offset_number: 1,
            },
            payload_format: SPIRE_PAYLOAD_FORMAT_TURBOQUANT,
            gamma: 1.5,
            encoded_payload: vec![1, 2],
        },
        SpireLeafAssignmentRow {
            flags: SPIRE_ASSIGNMENT_FLAG_PRIMARY,
            vec_id: SpireVecId::local(12),
            heap_tid: ItemPointer {
                block_number: 1,
                offset_number: 2,
            },
            payload_format: SPIRE_PAYLOAD_FORMAT_TURBOQUANT,
            gamma: 2.5,
            encoded_payload: vec![3, 4],
        },
    ];

    let segment =
        SpireLeafPartitionObjectV2Segment::new(&meta, 0, 0, ItemPointer::INVALID, &rows).unwrap();
    let encoded = segment.encode(&meta).unwrap();
    let decoded = SpireLeafPartitionObjectV2Segment::decode(&encoded, &meta).unwrap();
    let columns = decoded.columns(&meta).unwrap();

    assert_eq!(columns.row_count(), 2);
    assert_eq!(
        columns.row(0).unwrap().vec_id().unwrap(),
        SpireVecId::local(11)
    );
    assert_eq!(columns.row(1).unwrap().encoded_payload, &[3, 4]);
}

#[test]
fn leaf_partition_object_v2_store_preserves_fixed_width_global_vec_ids() {
    let mut store = SpireLocalObjectStore::new(99, 512).unwrap();
    let assignments = vec![
        leaf_v2_global_assignment(&[7, 0, 0, 1], 200, 1, 32),
        leaf_v2_global_assignment(&[7, 0, 0, 2], 200, 2, 32),
        leaf_v2_global_assignment(&[7, 0, 0, 3], 200, 3, 32),
    ];

    let placement = store
        .insert_leaf_object_v2_from_rows(7, 17, 3, 5, &assignments)
        .unwrap();
    let decoded = store.read_leaf_object_v2(&placement).unwrap();

    assert_eq!(decoded.meta.vec_id_kind, SpireVecIdKind::GlobalBytes);
    assert_eq!(
        usize::from(decoded.meta.vec_id_stride),
        assignments[0].vec_id.as_bytes().len()
    );
    assert_eq!(decoded.assignment_rows().unwrap(), assignments);

    let column_segments = decoded
        .column_segments()
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    let first_row = column_segments[0].row(0).unwrap();
    assert_eq!(first_row.vec_id().unwrap(), assignments[0].vec_id);
    assert!(first_row.local_vec_seq().is_err());
    assert_eq!(first_row.vec_id_bytes, assignments[0].vec_id.as_bytes());
}

#[test]
fn leaf_partition_object_v2_rejects_mixed_payload_or_vec_id_layout() {
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

    let mut mixed_vec_id_kind = leaf_v2_assignment(1, 8);
    mixed_vec_id_kind.vec_id = SpireVecId::global(&[9, 9, 9]).unwrap();
    let err = store
        .insert_leaf_object_v2_from_rows(
            7,
            17,
            3,
            5,
            &[leaf_v2_assignment(1, 8), mixed_vec_id_kind],
        )
        .unwrap_err();
    assert!(err.contains("requires one vec_id kind per object"));

    let variable_global_lengths = vec![
        leaf_v2_global_assignment(&[9, 9, 1], 200, 1, 8),
        leaf_v2_global_assignment(&[9, 9, 9, 2], 200, 2, 8),
    ];
    let err = store
        .insert_leaf_object_v2_from_rows(7, 17, 3, 5, &variable_global_lengths)
        .unwrap_err();
    assert!(err.contains("requires one vec_id stride per object"));
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

// -------------------------------------------------------------------
// Direct SpireLeafPartitionObjectV2 validation paths. These construct
// an object with a specific invalidity via struct literals so the
// matching `validate()` error branch is observable; the validate()
// is reached through the pub(super) `column_segments()` API.
// -------------------------------------------------------------------

fn leaf_v2_test_meta(segment_count: u32, assignment_count: u32) -> SpireLeafPartitionObjectV2Meta {
    SpireLeafPartitionObjectV2Meta {
        header: SpirePartitionObjectHeader {
            kind: SpirePartitionObjectKind::Leaf,
            pid: 11,
            object_version: 1,
            published_epoch_backref: 1,
            level: 0,
            parent_pid: 0,
            child_count: 0,
            assignment_count,
            flags: super::SPIRE_LEAF_V2_META_FLAG,
        },
        payload_format: SPIRE_PAYLOAD_FORMAT_TURBOQUANT,
        payload_stride: 4,
        vec_id_kind: SpireVecIdKind::LocalU64,
        vec_id_stride: super::SPIRE_LEAF_V2_LOCAL_VEC_ID_STRIDE as u16,
        segment_count,
        first_segment_locator: ItemPointer {
            block_number: 1,
            offset_number: 1,
        },
        object_bytes_total: 200,
    }
}

fn leaf_v2_test_segment(
    meta: &SpireLeafPartitionObjectV2Meta,
    segment_no: u32,
    row_base: u32,
    rows: &[SpireLeafAssignmentRow],
    next_locator: ItemPointer,
) -> SpireLeafPartitionObjectV2Segment {
    let row_count = rows.len() as u32;
    let mut flags = Vec::with_capacity(rows.len());
    let mut vec_ids = Vec::with_capacity(usize::from(meta.vec_id_stride) * rows.len());
    let mut heap_tids = Vec::with_capacity(rows.len());
    let mut gammas = Vec::with_capacity(rows.len());
    let mut payloads = Vec::with_capacity(meta.payload_stride as usize * rows.len());
    for row in rows {
        flags.push(row.flags);
        let seq = row
            .vec_id
            .local_sequence()
            .expect("test rows use local vec_ids");
        vec_ids.push(SPIRE_LOCAL_VEC_ID_DISCRIMINATOR);
        vec_ids.extend_from_slice(&seq.to_le_bytes());
        vec_ids.extend_from_slice(&[0u8; super::SPIRE_LEAF_V2_LOCAL_VEC_ID_STRIDE - 9]);
        heap_tids.push(row.heap_tid);
        gammas.push(row.gamma);
        payloads.extend_from_slice(&row.encoded_payload);
    }
    SpireLeafPartitionObjectV2Segment {
        header: SpirePartitionObjectHeader {
            kind: SpirePartitionObjectKind::Leaf,
            pid: meta.header.pid,
            object_version: meta.header.object_version,
            published_epoch_backref: meta.header.published_epoch_backref,
            level: meta.header.level,
            parent_pid: meta.header.parent_pid,
            child_count: 0,
            assignment_count: row_count,
            flags: super::SPIRE_LEAF_V2_SEGMENT_FLAG,
        },
        segment_no,
        row_base,
        next_segment_locator: next_locator,
        flags,
        vec_ids,
        heap_tids,
        gammas,
        payloads,
    }
}

#[test]
fn miri_leaf_v2_validate_rejects_segment_count_mismatch() {
    let meta = leaf_v2_test_meta(2, 2);
    let object = super::SpireLeafPartitionObjectV2 {
        meta: meta.clone(),
        segments: vec![leaf_v2_test_segment(
            &meta,
            0,
            0,
            &[leaf_v2_assignment(1, 4)],
            ItemPointer::INVALID,
        )],
    };
    let err = object
        .column_segments()
        .err()
        .expect("segment count mismatch must be rejected");
    assert!(
        err.contains("segment count mismatch"),
        "unexpected error: {err}"
    );
}

#[test]
fn miri_leaf_v2_validate_rejects_segment_number_mismatch() {
    let meta = leaf_v2_test_meta(1, 1);
    let object = super::SpireLeafPartitionObjectV2 {
        meta: meta.clone(),
        // Deliberately label segment 0 as segment 5 — validate must catch it.
        segments: vec![leaf_v2_test_segment(
            &meta,
            5,
            0,
            &[leaf_v2_assignment(1, 4)],
            ItemPointer::INVALID,
        )],
    };
    let err = object
        .column_segments()
        .err()
        .expect("segment number mismatch must be rejected");
    assert!(
        err.contains("segment number mismatch"),
        "unexpected error: {err}"
    );
}

#[test]
fn miri_leaf_v2_validate_rejects_row_base_mismatch() {
    let mut meta = leaf_v2_test_meta(1, 1);
    meta.header.assignment_count = 1;
    let object = super::SpireLeafPartitionObjectV2 {
        meta: meta.clone(),
        // row_base=5 on the only segment; first segment must have row_base=0.
        segments: vec![leaf_v2_test_segment(
            &meta,
            0,
            5,
            &[leaf_v2_assignment(1, 4)],
            ItemPointer::INVALID,
        )],
    };
    let err = object
        .column_segments()
        .err()
        .expect("row_base mismatch must be rejected");
    assert!(
        err.contains("row_base mismatch"),
        "unexpected error: {err}"
    );
}

#[test]
fn miri_leaf_v2_validate_rejects_final_segment_with_non_invalid_locator() {
    let meta = leaf_v2_test_meta(1, 1);
    let dangling_locator = ItemPointer {
        block_number: 17,
        offset_number: 3,
    };
    let object = super::SpireLeafPartitionObjectV2 {
        meta: meta.clone(),
        segments: vec![leaf_v2_test_segment(
            &meta,
            0,
            0,
            &[leaf_v2_assignment(1, 4)],
            dangling_locator,
        )],
    };
    let err = object
        .column_segments()
        .err()
        .expect("trailing locator on final segment must be rejected");
    assert!(
        err.contains("final segment next locator must be invalid"),
        "unexpected error: {err}"
    );
}

#[test]
fn miri_leaf_v2_validate_rejects_non_final_segment_missing_locator() {
    let meta = leaf_v2_test_meta(2, 2);
    let object = super::SpireLeafPartitionObjectV2 {
        meta: meta.clone(),
        segments: vec![
            leaf_v2_test_segment(
                &meta,
                0,
                0,
                &[leaf_v2_assignment(1, 4)],
                ItemPointer::INVALID,
            ),
            leaf_v2_test_segment(
                &meta,
                1,
                1,
                &[leaf_v2_assignment(2, 4)],
                ItemPointer::INVALID,
            ),
        ],
    };
    let err = object
        .column_segments()
        .err()
        .expect("non-final segment missing next locator must be rejected");
    assert!(
        err.contains("non-final segment requires next locator"),
        "unexpected error: {err}"
    );
}

#[test]
fn miri_leaf_v2_validate_rejects_meta_assignment_count_mismatch() {
    // segments report 2 rows total, meta says 7.
    let mut meta = leaf_v2_test_meta(1, 7);
    meta.header.assignment_count = 7;
    let object = super::SpireLeafPartitionObjectV2 {
        meta: meta.clone(),
        segments: vec![leaf_v2_test_segment(
            &meta,
            0,
            0,
            &[leaf_v2_assignment(1, 4), leaf_v2_assignment(2, 4)],
            ItemPointer::INVALID,
        )],
    };
    let err = object
        .column_segments()
        .err()
        .expect("meta assignment_count mismatch must be rejected");
    assert!(
        err.contains("assignment count mismatch"),
        "unexpected error: {err}"
    );
}

#[test]
fn miri_leaf_v2_assignment_rows_round_trips_segments_back_to_rows() {
    // Happy path through assignment_rows() so column_segments → row →
    // SpireLeafAssignmentRow reconstruction is observable.
    let mut meta = leaf_v2_test_meta(1, 2);
    meta.header.assignment_count = 2;
    let rows_input = vec![leaf_v2_assignment(11, 4), leaf_v2_assignment(13, 4)];
    let object = super::SpireLeafPartitionObjectV2 {
        meta: meta.clone(),
        segments: vec![leaf_v2_test_segment(
            &meta,
            0,
            0,
            &rows_input,
            ItemPointer::INVALID,
        )],
    };
    let decoded_rows = object.assignment_rows().unwrap();
    assert_eq!(decoded_rows.len(), 2);
    assert_eq!(decoded_rows[0].vec_id, rows_input[0].vec_id);
    assert_eq!(decoded_rows[1].vec_id, rows_input[1].vec_id);
    assert_eq!(decoded_rows[0].heap_tid, rows_input[0].heap_tid);
    assert_eq!(decoded_rows[1].gamma, rows_input[1].gamma);
}

// ----------------------------------------------------------------
// Additional mutation-killing tests for leaf_v2_parts.rs surfaced by
// the packet 047 manual cargo-mutants verification campaign. Each
// test below targets one or more specific operator-swap mutations
// that the earlier round of leaf_v2_parts coverage left undetected.
// ----------------------------------------------------------------

#[test]
fn miri_leaf_v2_meta_rejects_empty_meta_with_nonzero_segment_count() {
    // Targets leaf_v2_parts.rs:146:35 (`!=` -> `==`) inside the
    // assignment_count == 0 branch of Meta::validate. The original
    // guard fires when an empty meta declares a non-zero segment
    // count; the mutant `==` skips the guard, letting the invalid
    // shape pass.
    let meta = SpireLeafPartitionObjectV2Meta::new(
        17,
        3,
        5,
        0,
        SPIRE_PAYLOAD_FORMAT_NONE,
        0,
        SpireVecIdKind::LocalU64,
        super::SPIRE_LEAF_V2_LOCAL_VEC_ID_STRIDE as u16,
        2,
        ItemPointer::INVALID,
        54,
        7,
    );
    let err = meta.expect_err("empty meta with non-zero segment_count must be rejected");
    assert!(
        err.contains("cannot reference segments"),
        "unexpected error: {err}"
    );
}

#[test]
fn miri_leaf_v2_segment_decode_distinguishes_prefix_boundary_via_error_message() {
    // Targets leaf_v2_parts.rs:305:23 (`<` -> `==`, `<` -> `<=`).
    // Crafting an input that has exactly LEAF_V2_SEGMENT_PREFIX_BYTES
    // tail bytes lets us distinguish:
    //   - Original code: `tail.len() < 18` is false, parse continues,
    //     row_count == 0, validate_against_meta errors "row count 0 is
    //     invalid".
    //   - `< -> ==` mutant: errors "segment too short" at line 305
    //     instead.
    //   - `< -> <=` mutant: same as `==` (errors at line 305).
    // Asserting the exact error text distinguishes original vs both
    // mutants.
    let meta = leaf_v2_test_meta(1, 1);
    let header = SpirePartitionObjectHeader {
        kind: SpirePartitionObjectKind::Leaf,
        pid: meta.header.pid,
        object_version: meta.header.object_version,
        published_epoch_backref: meta.header.published_epoch_backref,
        level: meta.header.level,
        parent_pid: meta.header.parent_pid,
        child_count: 0,
        assignment_count: 0,
        flags: super::SPIRE_LEAF_V2_SEGMENT_FLAG,
    };
    let mut input = header
        .encode_after_validation(super::SPIRE_PARTITION_OBJECT_FORMAT_VERSION_V2);
    // Build an 18-byte tail: segment_no(4) + row_base(4) + row_count(4)
    // + locator(6) — all zeros + ItemPointer::INVALID.
    input.extend_from_slice(&0u32.to_le_bytes());
    input.extend_from_slice(&0u32.to_le_bytes());
    input.extend_from_slice(&0u32.to_le_bytes());
    ItemPointer::INVALID.encode_into(&mut input);
    let err = SpireLeafPartitionObjectV2Segment::decode(&input, &meta)
        .expect_err("zero-row segment must be rejected somewhere");
    // The original code reaches validate_against_meta with row_count==0
    // and surfaces a row-count error; mutants `==`/`<=` short-circuit
    // at line 305 with "segment too short".
    assert!(
        !err.contains("segment too short"),
        "boundary mutation surfaced 'segment too short' instead of the row_count error: {err}",
    );
    assert!(
        err.contains("row count 0") || err.contains("0 is invalid"),
        "expected row_count==0 error, got: {err}",
    );
}

fn leaf_v2_segment_with_mismatched_header(
    meta: &SpireLeafPartitionObjectV2Meta,
    pid_override: Option<u64>,
    version_override: Option<u64>,
    parent_pid_override: Option<u64>,
) -> SpireLeafPartitionObjectV2Segment {
    let mut segment = leaf_v2_test_segment(
        meta,
        0,
        0,
        &[leaf_v2_assignment(1, 4)],
        ItemPointer::INVALID,
    );
    if let Some(p) = pid_override {
        segment.header.pid = p;
    }
    if let Some(v) = version_override {
        segment.header.object_version = v;
    }
    if let Some(pp) = parent_pid_override {
        segment.header.parent_pid = pp;
    }
    segment
}

#[test]
fn miri_leaf_v2_segment_validate_rejects_pid_mismatch_only() {
    // Targets leaf_v2_parts.rs:399:13 (`||` -> `&&` in the segment
    // header-against-meta guard). With ONLY pid mismatched, the
    // original `||` chain fires immediately; the mutant turns
    // `A || B || C` into `(A && B) || C`, so `A=true, B=false,
    // C=false` evaluates false and the guard misses.
    let mut meta = leaf_v2_test_meta(1, 1);
    meta.header.assignment_count = 1;
    let segment =
        leaf_v2_segment_with_mismatched_header(&meta, Some(999), None, None);
    let object = super::SpireLeafPartitionObjectV2 {
        meta,
        segments: vec![segment],
    };
    let err = object
        .column_segments()
        .err()
        .expect("pid-only header mismatch must be rejected");
    assert!(
        err.contains("segment header does not match meta"),
        "unexpected error: {err}"
    );
}

#[test]
fn miri_leaf_v2_segment_validate_rejects_object_version_mismatch_only() {
    // Targets leaf_v2_parts.rs:400:13 (`||` -> `&&` further into the
    // chain). With ONLY object_version mismatched, the same
    // priority-of-evaluation defect surfaces — the mutant misses the
    // bad header where the original errors immediately.
    let mut meta = leaf_v2_test_meta(1, 1);
    meta.header.assignment_count = 1;
    let segment =
        leaf_v2_segment_with_mismatched_header(&meta, None, Some(99), None);
    let object = super::SpireLeafPartitionObjectV2 {
        meta,
        segments: vec![segment],
    };
    let err = object
        .column_segments()
        .err()
        .expect("version-only header mismatch must be rejected");
    assert!(
        err.contains("segment header does not match meta"),
        "unexpected error: {err}"
    );
}

#[test]
fn miri_leaf_v2_segment_validate_rejects_row_with_delta_insert_flag() {
    // Targets leaf_v2_parts.rs:418:60 (`|` -> `&` in the
    // DELTA_INSERT | DELTA_DELETE flag mask). With a single row
    // setting DELTA_INSERT alone, the original mask
    // (0x0008 | 0x0010 = 0x0018) catches it; the `&` mutant collapses
    // the mask to 0x0008 & 0x0010 = 0 and the guard silently passes.
    // The `^` mutant (0x0008 ^ 0x0010 = 0x0018) is mathematically
    // equivalent for these non-overlapping bits — see triage.md for
    // the equivalent-mutant rationale.
    let mut meta = leaf_v2_test_meta(1, 1);
    meta.header.assignment_count = 1;
    let mut bad_row = leaf_v2_assignment(1, 4);
    bad_row.flags = SPIRE_ASSIGNMENT_FLAG_PRIMARY | SPIRE_ASSIGNMENT_FLAG_DELTA_INSERT;
    let segment = leaf_v2_test_segment(
        &meta,
        0,
        0,
        &[bad_row],
        ItemPointer::INVALID,
    );
    let object = super::SpireLeafPartitionObjectV2 {
        meta,
        segments: vec![segment],
    };
    let err = object
        .column_segments()
        .err()
        .expect("delta-insert flag on segment row must be rejected");
    assert!(
        err.contains("cannot set delta flags"),
        "unexpected error: {err}"
    );
}
