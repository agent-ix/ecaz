//! Property tests for page codec invariants.

use proptest::prelude::*;
use ecaz::bench_api::{
    CurrentFormatMetadata, ItemPointer, MetadataPage, TqElementTuple, TqNeighborTuple,
    HEAPTID_INLINE_CAPACITY, ITEM_POINTER_BYTES,
};

// P9: TqElementTuple encode/decode roundtrip.
proptest! {
    #[test]
    fn element_tuple_roundtrip(
        level in 0..10u8,
        deleted in proptest::bool::ANY,
        heaptid_count in 0..=HEAPTID_INLINE_CAPACITY,
        code_len in prop::sample::select(&[64, 192, 384, 768][..]),
        seed in 0..1000u32,
    ) {
        let heaptids: Vec<ItemPointer> = (0..heaptid_count)
            .map(|i| ItemPointer {
                block_number: seed + i as u32,
                offset_number: (i + 1) as u16,
            })
            .collect();

        let tuple = TqElementTuple {
            level,
            deleted,
            heaptids,
            gamma: 0.42 + seed as f32 * 0.001,
            neighbortid: ItemPointer { block_number: seed + 100, offset_number: 1 },
            code: vec![0xAB; code_len],
            binary_words: Vec::new(),
        };

        let encoded = tuple.encode().unwrap();
        let decoded = TqElementTuple::decode(&encoded, code_len).unwrap();

        prop_assert_eq!(decoded.level, tuple.level);
        prop_assert_eq!(decoded.deleted, tuple.deleted);
        prop_assert_eq!(decoded.heaptids, tuple.heaptids);
        prop_assert!((decoded.gamma - tuple.gamma).abs() < 1e-6);
        prop_assert_eq!(decoded.neighbortid, tuple.neighbortid);
        prop_assert_eq!(decoded.code, tuple.code);
        prop_assert_eq!(decoded.binary_words, tuple.binary_words);
    }
}

// P10: TqNeighborTuple encode/decode roundtrip.
proptest! {
    #[test]
    fn neighbor_tuple_roundtrip(
        count in 1..64u16,
        seed in 0..1000u32,
    ) {
        let tids: Vec<ItemPointer> = (0..count)
            .map(|i| ItemPointer {
                block_number: seed + i as u32,
                offset_number: i + 1,
            })
            .collect();

        let tuple = TqNeighborTuple { count, tids };
        let encoded = tuple.encode().unwrap();
        let decoded = TqNeighborTuple::decode(&encoded).unwrap();

        prop_assert_eq!(decoded.count, tuple.count);
        prop_assert_eq!(decoded.tids, tuple.tids);
    }
}

// P11: MetadataPage encode/decode roundtrip.
proptest! {
    #[test]
    fn metadata_roundtrip(
        m in 1..64u16,
        ef_construction in 1..500u16,
        dimensions in 1..4096u16,
        bits in 2..8u8,
        max_level in 0..20u8,
        seed in 0..10000u64,
    ) {
        let metadata = MetadataPage::current_v1_scalar(CurrentFormatMetadata {
            m,
            ef_construction,
            entry_point: ItemPointer {
                block_number: 1,
                offset_number: 1,
            },
            dimensions,
            bits,
            max_level,
            seed,
            inserted_since_rebuild: seed,
            persisted_binary_sidecar: false,
        });

        let encoded = metadata.encode();
        let decoded = MetadataPage::decode(&encoded).unwrap();

        prop_assert_eq!(decoded.m, metadata.m);
        prop_assert_eq!(decoded.ef_construction, metadata.ef_construction);
        prop_assert_eq!(decoded.entry_point, metadata.entry_point);
        prop_assert_eq!(decoded.dimensions, metadata.dimensions);
        prop_assert_eq!(decoded.bits, metadata.bits);
        prop_assert_eq!(decoded.max_level, metadata.max_level);
        prop_assert_eq!(decoded.seed, metadata.seed);
        prop_assert_eq!(decoded.inserted_since_rebuild, metadata.inserted_since_rebuild);
    }
}

// P12: ItemPointer encode/decode roundtrip.
proptest! {
    #[test]
    fn item_pointer_roundtrip(block in 0..u32::MAX, offset in 0..u16::MAX) {
        let ptr = ItemPointer { block_number: block, offset_number: offset };
        let mut buf = Vec::new();
        ptr.encode_into(&mut buf);
        prop_assert_eq!(buf.len(), ITEM_POINTER_BYTES);

        let decoded = ItemPointer::decode(&buf).unwrap();
        prop_assert_eq!(decoded, ptr);
    }
}

// P13: element_tuple_encoded_len correctness — encode().len() == encoded_len(code_len).
proptest! {
    #[test]
    fn element_encoded_len_matches(code_len in prop::sample::select(&[64, 192, 384, 768, 1536][..])) {
        let tuple = TqElementTuple {
            level: 1,
            deleted: false,
            heaptids: vec![ItemPointer { block_number: 1, offset_number: 1 }],
            gamma: 0.5,
            neighbortid: ItemPointer { block_number: 2, offset_number: 1 },
            code: vec![0xFF; code_len],
            binary_words: Vec::new(),
        };

        let encoded = tuple.encode().unwrap();
        prop_assert_eq!(encoded.len(), TqElementTuple::encoded_len(code_len));
    }
}
