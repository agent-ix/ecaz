#[test]
fn relation_object_prefetch_groups_dedupes_blocks_and_orders_by_store() {
    let placements = vec![
        SpirePlacementEntry::local_store_available_by_id(
            7,
            101,
            2,
            502,
            1,
            ItemPointer {
                block_number: 30,
                offset_number: 1,
            },
            100,
        ),
        SpirePlacementEntry::local_store_available_by_id(
            7,
            102,
            0,
            500,
            1,
            ItemPointer {
                block_number: 20,
                offset_number: 1,
            },
            100,
        ),
        SpirePlacementEntry::local_store_available_by_id(
            7,
            103,
            2,
            502,
            1,
            ItemPointer {
                block_number: 30,
                offset_number: 2,
            },
            100,
        ),
        SpirePlacementEntry::local_store_available_by_id(
            7,
            104,
            2,
            502,
            1,
            ItemPointer {
                block_number: 31,
                offset_number: 1,
            },
            100,
        ),
    ];

    let groups = relation_object_prefetch_groups(&[(0, 500), (2, 502)], &placements).unwrap();

    assert_eq!(groups.len(), 2);
    assert_eq!(groups[0].local_store_id, 0);
    assert_eq!(groups[0].store_relid, 500);
    assert_eq!(groups[0].block_numbers, vec![20]);
    assert_eq!(groups[1].local_store_id, 2);
    assert_eq!(groups[1].store_relid, 502);
    assert_eq!(groups[1].block_numbers, vec![30, 31]);
}

#[test]
fn relation_object_prefetch_groups_rejects_missing_store() {
    let placements = vec![SpirePlacementEntry::local_store_available_by_id(
        7,
        101,
        2,
        502,
        1,
        ItemPointer {
            block_number: 30,
            offset_number: 1,
        },
        100,
    )];

    let error = relation_object_prefetch_groups(&[(0, 500)], &placements).unwrap_err();

    assert!(error.contains("missing local_store_id 2 relid 502"));
}

#[test]
fn relation_object_chain_codecs_accept_top_graph_kind() {
    let mut object = valid_top_graph_object();
    object.header.published_epoch_backref = 7;
    let first_segment_locator = ItemPointer {
        block_number: 42,
        offset_number: 3,
    };

    let encoded_meta = encode_relation_object_chain_meta(
        object.header,
        object.dimensions,
        2,
        first_segment_locator,
        9_000,
    )
    .unwrap();
    let meta = decode_relation_object_chain_meta(&encoded_meta)
        .unwrap()
        .unwrap();

    assert_eq!(meta.header.kind, SpirePartitionObjectKind::TopGraph);
    assert_eq!(meta.header.pid, object.header.pid);
    assert_eq!(meta.header.object_version, object.header.object_version);
    assert_eq!(
        meta.header.flags & PARTITION_OBJECT_V2_CHAIN_META_FLAG,
        PARTITION_OBJECT_V2_CHAIN_META_FLAG
    );
    assert_eq!(meta.dimensions, object.dimensions);
    assert_eq!(meta.segment_count, 2);
    assert_eq!(meta.first_segment_locator, first_segment_locator);
    assert_eq!(meta.object_bytes_total, 9_000);

    let payload = [1_u8, 2, 3, 4];
    let encoded_segment = encode_relation_object_chain_segment(
        object.header,
        1,
        4,
        ItemPointer::INVALID,
        &payload,
    )
    .unwrap();
    let segment = decode_relation_object_chain_segment(&encoded_segment, &meta).unwrap();

    assert_eq!(segment.segment_no, 1);
    assert_eq!(segment.byte_base, 4);
    assert_eq!(segment.next_segment_locator, ItemPointer::INVALID);
    assert_eq!(segment.payload, payload.to_vec());

    let (segment_header, _, _) =
        SpirePartitionObjectHeader::decode_prefix_with_format_version(&encoded_segment).unwrap();
    assert_eq!(
        segment_header.flags & PARTITION_OBJECT_V2_CHAIN_SEGMENT_FLAG,
        PARTITION_OBJECT_V2_CHAIN_SEGMENT_FLAG
    );
}
