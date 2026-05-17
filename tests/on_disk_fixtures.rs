//! Golden on-disk fixture decode checks.

use ecaz::bench_api::{
    ItemPointer, IvfBlockRef, IvfCentroidTuple, IvfListDirectoryTuple, IvfMetadataPage,
    IvfPostingTuple, IvfPqCodebookTuple, IvfRerankMode, IvfStorageFormat, MetadataPage,
    TqElementTuple, TqGroupedCodebookTuple, TqNeighborTuple, VamanaCodebookTuple,
    VamanaMetadataPage, VamanaNodeTuple, EC_IVF_CENTROID_DIMENSIONS_OFFSET,
    EC_IVF_INDEX_FORMAT_VERSION, EC_IVF_METADATA_FORMAT_VERSION_OFFSET,
    HNSW_METADATA_FORMAT_VERSION_OFFSET, INDEX_FORMAT_V3_DISKANN,
    VAMANA_METADATA_FORMAT_VERSION_OFFSET, VAMANA_NODE_NEIGHBOR_COUNT_OFFSET,
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

#[test]
fn hnsw_element_tuple_v3_fixture_decodes() {
    let bytes = decode_hex_fixture(include_str!(
        "../fixtures/on-disk/hnsw_element_tuple_v3.hex"
    ));

    let element = TqElementTuple::decode(&bytes, 4).expect("hnsw element tuple should decode");

    assert_eq!(element.level, 2);
    assert!(!element.deleted);
    assert_eq!(
        element.heaptids,
        vec![
            ItemPointer {
                block_number: 10,
                offset_number: 1
            },
            ItemPointer {
                block_number: 11,
                offset_number: 2
            }
        ]
    );
    assert_eq!(element.gamma.to_bits(), 0.5_f32.to_bits());
    assert_eq!(
        element.neighbortid,
        ItemPointer {
            block_number: 20,
            offset_number: 1
        }
    );
    assert_eq!(element.code, vec![0xaa, 0xbb, 0xcc, 0xdd]);
    assert!(element.binary_words.is_empty());
}

#[test]
fn hnsw_neighbor_tuple_v3_fixture_decodes() {
    let bytes = decode_hex_fixture(include_str!(
        "../fixtures/on-disk/hnsw_neighbor_tuple_v3.hex"
    ));

    let neighbors = TqNeighborTuple::decode(&bytes).expect("hnsw neighbor tuple should decode");

    assert_eq!(neighbors.count, 3);
    assert_eq!(
        neighbors.tids,
        vec![
            ItemPointer {
                block_number: 30,
                offset_number: 1
            },
            ItemPointer {
                block_number: 31,
                offset_number: 2
            },
            ItemPointer {
                block_number: 32,
                offset_number: 3
            }
        ]
    );
}

#[test]
fn hnsw_grouped_codebook_tuple_v3_fixture_decodes() {
    let bytes = decode_hex_fixture(include_str!(
        "../fixtures/on-disk/hnsw_grouped_codebook_tuple_v3.hex"
    ));

    let codebook =
        TqGroupedCodebookTuple::decode(&bytes, 2).expect("hnsw codebook tuple should decode");

    assert_eq!(codebook.group_index, 5);
    assert_eq!(
        codebook.nexttid,
        ItemPointer {
            block_number: 40,
            offset_number: 1
        }
    );
    assert_eq!(
        codebook
            .centroids
            .iter()
            .map(|centroid| centroid.to_bits())
            .collect::<Vec<_>>(),
        vec![1.0_f32.to_bits(), 2.0_f32.to_bits()]
    );
}

#[test]
fn diskann_vamana_node_tuple_v3_fixture_decodes() {
    let bytes = decode_hex_fixture(include_str!(
        "../fixtures/on-disk/diskann_vamana_node_tuple_v3.hex"
    ));

    let node =
        VamanaNodeTuple::decode(&bytes, 4, 1, 3).expect("diskann vamana node tuple should decode");

    assert!(!node.deleted);
    assert!(!node.has_overflow_heaptids);
    assert_eq!(
        node.primary_heaptid,
        ItemPointer {
            block_number: 50,
            offset_number: 1
        }
    );
    assert_eq!(node.rerank_tid, ItemPointer::INVALID);
    assert_eq!(node.binary_words, vec![0x0102_0304_0506_0708]);
    assert_eq!(node.search_code, vec![0xaa, 0xbb, 0xcc]);
    assert_eq!(node.neighbor_count, 2);
    assert_eq!(
        node.neighbors,
        vec![
            ItemPointer {
                block_number: 60,
                offset_number: 1
            },
            ItemPointer {
                block_number: 61,
                offset_number: 2
            },
            ItemPointer::INVALID,
            ItemPointer::INVALID,
        ]
    );
}

#[test]
fn diskann_vamana_node_tuple_v3_byteswapped_neighbor_count_is_rejected() {
    let mut bytes = decode_hex_fixture(include_str!(
        "../fixtures/on-disk/diskann_vamana_node_tuple_v3.hex"
    ));
    bytes.swap(
        VAMANA_NODE_NEIGHBOR_COUNT_OFFSET,
        VAMANA_NODE_NEIGHBOR_COUNT_OFFSET + 1,
    );

    let err = VamanaNodeTuple::decode(&bytes, 4, 1, 3)
        .expect_err("byte-swapped neighbor_count should fail");

    assert!(
        err.contains("neighbor_count 512 exceeds graph_degree_r 4"),
        "unexpected error: {err}"
    );
}

#[test]
fn diskann_vamana_codebook_tuple_v3_fixture_decodes() {
    let bytes = decode_hex_fixture(include_str!(
        "../fixtures/on-disk/diskann_vamana_codebook_tuple_v3.hex"
    ));

    let codebook =
        VamanaCodebookTuple::decode(&bytes, 2).expect("diskann codebook tuple should decode");

    assert_eq!(codebook.group_index, 7);
    assert_eq!(
        codebook.nexttid,
        ItemPointer {
            block_number: 70,
            offset_number: 1
        }
    );
    assert_eq!(
        codebook
            .centroids
            .iter()
            .map(|centroid| centroid.to_bits())
            .collect::<Vec<_>>(),
        vec![1.0_f32.to_bits(), 2.0_f32.to_bits()]
    );
}

#[test]
fn ivf_metadata_v1_fixture_decodes() {
    let bytes = decode_hex_fixture(include_str!("../fixtures/on-disk/ivf_metadata_v1.hex"));

    let metadata = IvfMetadataPage::decode(&bytes).expect("ivf metadata fixture should decode");

    assert_eq!(metadata.format_version, EC_IVF_INDEX_FORMAT_VERSION);
    assert_eq!(metadata.dimensions, 128);
    assert_eq!(metadata.nlists, 16);
    assert_eq!(metadata.nprobe, 4);
    assert_eq!(metadata.training_sample_rows, 1_000);
    assert_eq!(metadata.training_version, 3);
    assert_eq!(metadata.seed, 0x0102_0304_0506_0708);
    assert_eq!(metadata.storage_format, IvfStorageFormat::PqFastScan);
    assert_eq!(metadata.rerank, IvfRerankMode::HeapF32);
    assert_eq!(
        metadata.centroid_head,
        ItemPointer {
            block_number: 10,
            offset_number: 1
        }
    );
    assert_eq!(
        metadata.directory_head,
        ItemPointer {
            block_number: 11,
            offset_number: 2
        }
    );
    assert_eq!(metadata.total_live_tuples, 42);
    assert_eq!(metadata.total_dead_tuples, 5);
    assert_eq!(metadata.inserted_since_build, 7);
    assert_eq!(
        metadata.pq_codebook_head,
        ItemPointer {
            block_number: 12,
            offset_number: 3
        }
    );
    assert_eq!(metadata.pq_group_size, 4);
}

#[test]
fn ivf_metadata_v1_byteswapped_version_is_rejected() {
    let mut bytes = decode_hex_fixture(include_str!("../fixtures/on-disk/ivf_metadata_v1.hex"));
    bytes.swap(
        EC_IVF_METADATA_FORMAT_VERSION_OFFSET,
        EC_IVF_METADATA_FORMAT_VERSION_OFFSET + 1,
    );

    let err = IvfMetadataPage::decode(&bytes).expect_err("byte-swapped version should fail");

    assert!(
        err.contains("unsupported ec_ivf metadata format version: 256"),
        "unexpected error: {err}"
    );
}

#[test]
fn ivf_centroid_tuple_v1_fixture_decodes() {
    let bytes = decode_hex_fixture(include_str!(
        "../fixtures/on-disk/ivf_centroid_tuple_v1.hex"
    ));

    let centroid = IvfCentroidTuple::decode(&bytes, 2).expect("ivf centroid should decode");

    assert_eq!(centroid.list_id, 3);
    assert_eq!(
        centroid
            .centroid
            .iter()
            .map(|value| value.to_bits())
            .collect::<Vec<_>>(),
        vec![0.25_f32.to_bits(), (-0.5_f32).to_bits()]
    );
}

#[test]
fn ivf_centroid_tuple_v1_byteswapped_dimensions_is_rejected() {
    let mut bytes = decode_hex_fixture(include_str!(
        "../fixtures/on-disk/ivf_centroid_tuple_v1.hex"
    ));
    bytes.swap(
        EC_IVF_CENTROID_DIMENSIONS_OFFSET,
        EC_IVF_CENTROID_DIMENSIONS_OFFSET + 1,
    );

    let err = IvfCentroidTuple::decode(&bytes, 2).expect_err("byte-swapped dimensions should fail");

    assert!(
        err.contains("ec_ivf centroid dimensions mismatch: got 512, expected 2"),
        "unexpected error: {err}"
    );
}

#[test]
fn ivf_list_directory_tuple_v1_fixture_decodes() {
    let bytes = decode_hex_fixture(include_str!(
        "../fixtures/on-disk/ivf_list_directory_tuple_v1.hex"
    ));

    let directory =
        IvfListDirectoryTuple::decode(&bytes).expect("ivf list directory should decode");

    assert_eq!(directory.list_id, 9);
    assert_eq!(directory.head_block, IvfBlockRef { block_number: 20 });
    assert_eq!(directory.tail_block, IvfBlockRef { block_number: 25 });
    assert_eq!(directory.live_count, 101);
    assert_eq!(directory.dead_count, 7);
    assert_eq!(directory.inserted_since_build, 11);
}

#[test]
fn ivf_posting_tuple_v1_fixture_decodes() {
    let bytes = decode_hex_fixture(include_str!("../fixtures/on-disk/ivf_posting_tuple_v1.hex"));

    let posting = IvfPostingTuple::decode(&bytes, 5).expect("ivf posting tuple should decode");

    assert_eq!(posting.list_id, 2);
    assert!(!posting.deleted);
    assert_eq!(
        posting.heaptids,
        vec![
            ItemPointer {
                block_number: 1,
                offset_number: 1
            },
            ItemPointer {
                block_number: 1,
                offset_number: 4
            },
            ItemPointer {
                block_number: 2,
                offset_number: 1
            }
        ]
    );
    assert_eq!(posting.gamma.to_bits(), 0.75_f32.to_bits());
    assert_eq!(
        posting.rerank_tid,
        ItemPointer {
            block_number: 7,
            offset_number: 2
        }
    );
    assert_eq!(posting.payload, vec![1, 2, 3, 4, 5]);
}

#[test]
fn ivf_pq_codebook_tuple_v1_fixture_decodes() {
    let bytes = decode_hex_fixture(include_str!(
        "../fixtures/on-disk/ivf_pq_codebook_tuple_v1.hex"
    ));

    let codebook =
        IvfPqCodebookTuple::decode(&bytes, 4).expect("ivf pq codebook tuple should decode");

    assert_eq!(codebook.group_index, 2);
    assert_eq!(
        codebook.next_tid,
        ItemPointer {
            block_number: 9,
            offset_number: 3
        }
    );
    assert_eq!(
        codebook
            .centroids
            .iter()
            .map(|value| value.to_bits())
            .collect::<Vec<_>>(),
        vec![
            0.0_f32.to_bits(),
            0.25_f32.to_bits(),
            (-0.5_f32).to_bits(),
            1.0_f32.to_bits()
        ]
    );
}
