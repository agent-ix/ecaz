//! Golden on-disk fixture decode checks.

use ecaz::bench_api::{
    ItemPointer, MetadataPage, VamanaMetadataPage, HNSW_METADATA_FORMAT_VERSION_OFFSET,
    INDEX_FORMAT_V3_DISKANN, VAMANA_METADATA_FORMAT_VERSION_OFFSET,
};

fn decode_hex_fixture(contents: &str) -> Vec<u8> {
    let hex = contents
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect::<String>();
    hex::decode(hex.trim()).expect("fixture hex should decode")
}

#[test]
fn hnsw_metadata_v3_fixture_decodes() {
    let bytes = decode_hex_fixture(include_str!("../fixtures/on-disk/hnsw_metadata_v3.hex"));

    let metadata = MetadataPage::decode(&bytes).expect("hnsw metadata fixture should decode");

    assert_eq!(metadata.m, 16);
    assert_eq!(metadata.ef_construction, 200);
    assert_eq!(
        metadata.entry_point,
        ItemPointer {
            block_number: 5,
            offset_number: 2
        }
    );
    assert_eq!(metadata.dimensions, 128);
    assert_eq!(metadata.bits, 4);
    assert_eq!(metadata.max_level, 3);
    assert_eq!(metadata.seed, 0x0102_0304_0506_0708);
    assert_eq!(metadata.inserted_since_rebuild, 42);
    assert_eq!(metadata.format_version, 3);
    assert_eq!(metadata.payload_flags, 1 << 2);
    assert_eq!(
        metadata.grouped_codebook_head,
        ItemPointer {
            block_number: u32::MAX,
            offset_number: u16::MAX
        }
    );
}

#[test]
fn hnsw_metadata_v3_byteswapped_version_is_rejected() {
    let mut bytes = decode_hex_fixture(include_str!("../fixtures/on-disk/hnsw_metadata_v3.hex"));
    bytes.swap(
        HNSW_METADATA_FORMAT_VERSION_OFFSET,
        HNSW_METADATA_FORMAT_VERSION_OFFSET + 1,
    );

    let err = MetadataPage::decode(&bytes).expect_err("byte-swapped version should fail");

    assert!(
        err.contains("invalid metadata format version"),
        "unexpected error: {err}"
    );
}

#[test]
fn diskann_metadata_v3_fixture_decodes() {
    let bytes = decode_hex_fixture(include_str!(
        "../fixtures/on-disk/diskann_vamana_metadata_v3.hex"
    ));

    let metadata =
        VamanaMetadataPage::decode(&bytes).expect("diskann metadata fixture should decode");

    assert_eq!(metadata.format_version, INDEX_FORMAT_V3_DISKANN);
    assert_eq!(metadata.entry_point, ItemPointer::INVALID);
    assert_eq!(metadata.graph_degree_r, 32);
    assert_eq!(metadata.build_list_size_l, 100);
    assert_eq!(metadata.alpha.to_bits(), 1.2_f32.to_bits());
    assert_eq!(metadata.dimensions, 128);
    assert_eq!(metadata.seed, 0x0102_0304_0506_0708);
    assert_eq!(metadata.inserted_since_rebuild, 42);
    assert!(!metadata.needs_medoid_refresh);
    assert_eq!(metadata.transform_kind, 1);
    assert_eq!(metadata.search_codec_kind, 2);
    assert_eq!(metadata.payload_flags, 1 << 1);
    assert_eq!(metadata.search_subvector_count, 16);
    assert_eq!(metadata.search_subvector_dim, 8);
    assert_eq!(
        metadata.grouped_codebook_head,
        ItemPointer {
            block_number: 7,
            offset_number: 1
        }
    );
}

#[test]
fn diskann_metadata_v3_byteswapped_version_is_rejected() {
    let mut bytes = decode_hex_fixture(include_str!(
        "../fixtures/on-disk/diskann_vamana_metadata_v3.hex"
    ));
    bytes.swap(
        VAMANA_METADATA_FORMAT_VERSION_OFFSET,
        VAMANA_METADATA_FORMAT_VERSION_OFFSET + 1,
    );

    let err = VamanaMetadataPage::decode(&bytes).expect_err("byte-swapped version should fail");

    assert!(
        err.contains("invalid vamana metadata format version"),
        "unexpected error: {err}"
    );
}
