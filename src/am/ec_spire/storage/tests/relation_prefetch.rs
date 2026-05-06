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
