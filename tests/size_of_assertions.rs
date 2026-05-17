//! Layout stability assertions for tqvector storage formats.
//! Ensures that payload sizes, struct sizes, and wire formats
//! remain stable across code changes.

use ecaz::bench_api::*;

const _: () = {
    assert!(ITEM_POINTER_BYTES == 6);
    assert!(ITEM_POINTER_BLOCK_NUMBER_OFFSET == 0);
    assert!(ITEM_POINTER_OFFSET_NUMBER_OFFSET == 4);
    assert!(std::mem::size_of::<ItemPointer>() == 8);
    assert!(std::mem::align_of::<ItemPointer>() == 4);
};

const _: () = {
    assert!(HNSW_LEGACY_METADATA_BYTES == 30);
    assert!(HNSW_METADATA_BYTES == 47);
    assert!(HNSW_METADATA_M_OFFSET == 0);
    assert!(HNSW_METADATA_EF_CONSTRUCTION_OFFSET == 2);
    assert!(HNSW_METADATA_ENTRY_POINT_OFFSET == 4);
    assert!(HNSW_METADATA_DIMENSIONS_OFFSET == 10);
    assert!(HNSW_METADATA_BITS_OFFSET == 12);
    assert!(HNSW_METADATA_MAX_LEVEL_OFFSET == 13);
    assert!(HNSW_METADATA_SEED_OFFSET == 14);
    assert!(HNSW_METADATA_INSERTED_SINCE_REBUILD_OFFSET == 22);
    assert!(HNSW_METADATA_FORMAT_VERSION_OFFSET == 30);
    assert!(HNSW_METADATA_TRANSFORM_KIND_OFFSET == 32);
    assert!(HNSW_METADATA_SEARCH_CODEC_KIND_OFFSET == 33);
    assert!(HNSW_METADATA_PAYLOAD_FLAGS_OFFSET == 34);
    assert!(HNSW_METADATA_SEARCH_BITS_OFFSET == 35);
    assert!(HNSW_METADATA_RERANK_CODEC_KIND_OFFSET == 36);
    assert!(HNSW_METADATA_SEARCH_SUBVECTOR_COUNT_OFFSET == 37);
    assert!(HNSW_METADATA_SEARCH_SUBVECTOR_DIM_OFFSET == 39);
    assert!(HNSW_METADATA_GROUPED_CODEBOOK_HEAD_OFFSET == 41);
};

const _: () = {
    assert!(TQ_ELEMENT_TAG_OFFSET == 0);
    assert!(TQ_ELEMENT_LEVEL_OFFSET == 1);
    assert!(TQ_ELEMENT_DELETED_OFFSET == 2);
    assert!(TQ_ELEMENT_HEAPTIDS_OFFSET == 3);
    assert!(TQ_ELEMENT_HEAPTID_COUNT_OFFSET == 63);
    assert!(TQ_ELEMENT_GAMMA_OFFSET == 64);
    assert!(TQ_ELEMENT_NEIGHBORTID_OFFSET == 68);
    assert!(TQ_ELEMENT_CODE_OFFSET == 74);

    assert!(TQ_GROUPED_HOT_TAG_OFFSET == 0);
    assert!(TQ_GROUPED_HOT_LEVEL_OFFSET == 1);
    assert!(TQ_GROUPED_HOT_DELETED_OFFSET == 2);
    assert!(TQ_GROUPED_HOT_HEAPTIDS_OFFSET == 3);
    assert!(TQ_GROUPED_HOT_HEAPTID_COUNT_OFFSET == 63);
    assert!(TQ_GROUPED_HOT_NEIGHBORTID_OFFSET == 64);
    assert!(TQ_GROUPED_HOT_RERANKTID_OFFSET == 70);
    assert!(TQ_GROUPED_HOT_BINARY_WORDS_OFFSET == 76);

    assert!(TQ_TURBO_HOT_TAG_OFFSET == 0);
    assert!(TQ_TURBO_HOT_LEVEL_OFFSET == 1);
    assert!(TQ_TURBO_HOT_DELETED_OFFSET == 2);
    assert!(TQ_TURBO_HOT_HEAPTIDS_OFFSET == 3);
    assert!(TQ_TURBO_HOT_HEAPTID_COUNT_OFFSET == 63);
    assert!(TQ_TURBO_HOT_NEIGHBORTID_OFFSET == 64);
    assert!(TQ_TURBO_HOT_RERANKTID_OFFSET == 70);
    assert!(TQ_TURBO_HOT_BINARY_WORDS_OFFSET == 76);

    assert!(TQ_RERANK_TAG_OFFSET == 0);
    assert!(TQ_RERANK_GAMMA_OFFSET == 1);
    assert!(TQ_RERANK_CODE_OFFSET == 5);

    assert!(TQ_GROUPED_CODEBOOK_TAG_OFFSET == 0);
    assert!(TQ_GROUPED_CODEBOOK_GROUP_INDEX_OFFSET == 1);
    assert!(TQ_GROUPED_CODEBOOK_NEXTTID_OFFSET == 3);
    assert!(TQ_GROUPED_CODEBOOK_CENTROIDS_OFFSET == 9);

    assert!(TQ_NEIGHBOR_TAG_OFFSET == 0);
    assert!(TQ_NEIGHBOR_COUNT_OFFSET == 1);
    assert!(TQ_NEIGHBOR_TIDS_OFFSET == 3);
};

const _: () = {
    assert!(VAMANA_METADATA_BYTES == 48);
    assert!(VAMANA_METADATA_FORMAT_VERSION_OFFSET == 0);
    assert!(VAMANA_METADATA_ENTRY_POINT_OFFSET == 2);
    assert!(VAMANA_METADATA_GRAPH_DEGREE_R_OFFSET == 8);
    assert!(VAMANA_METADATA_BUILD_LIST_SIZE_L_OFFSET == 10);
    assert!(VAMANA_METADATA_ALPHA_OFFSET == 12);
    assert!(VAMANA_METADATA_DIMENSIONS_OFFSET == 16);
    assert!(VAMANA_METADATA_SEED_OFFSET == 18);
    assert!(VAMANA_METADATA_INSERTED_SINCE_REBUILD_OFFSET == 26);
    assert!(VAMANA_METADATA_NEEDS_MEDOID_REFRESH_OFFSET == 34);
    assert!(VAMANA_METADATA_TRANSFORM_KIND_OFFSET == 35);
    assert!(VAMANA_METADATA_SEARCH_CODEC_KIND_OFFSET == 36);
    assert!(VAMANA_METADATA_PAYLOAD_FLAGS_OFFSET == 37);
    assert!(VAMANA_METADATA_SEARCH_SUBVECTOR_COUNT_OFFSET == 38);
    assert!(VAMANA_METADATA_SEARCH_SUBVECTOR_DIM_OFFSET == 40);
    assert!(VAMANA_METADATA_GROUPED_CODEBOOK_HEAD_OFFSET == 42);
};

// --- NFR-002 payload size contracts ---

#[test]
fn payload_len_1536_dim_4bit() {
    // 4 gamma + 768 MSE + 0 QJL = 772
    assert_eq!(payload_len(1536, 4), 772);
}

#[test]
fn payload_len_1536_dim_2bit() {
    // mse: (1536 * 1) / 8 = 192, qjl: 192, gamma: 4
    assert_eq!(mse_code_len(1536, 2), 192);
    assert_eq!(payload_len(1536, 2), 4 + 192 + 192);
}

#[test]
fn payload_len_1536_dim_3bit() {
    // mse: (1536 * 2).div_ceil(8) = 384, qjl: 192, gamma: 4
    assert_eq!(mse_code_len(1536, 3), 384);
    assert_eq!(payload_len(1536, 3), 4 + 384 + 192);
}

#[test]
fn payload_len_1536_dim_6bit() {
    // mse: (1536 * 5).div_ceil(8) = 960, qjl: 192, gamma: 4
    assert_eq!(mse_code_len(1536, 6), 960);
    assert_eq!(payload_len(1536, 6), 4 + 960 + 192);
}

#[test]
fn payload_len_1536_dim_8bit() {
    // mse: (1536 * 7).div_ceil(8) = 1344, qjl: 192, gamma: 4
    assert_eq!(mse_code_len(1536, 8), 1344);
    assert_eq!(payload_len(1536, 8), 4 + 1344 + 192);
}

#[test]
fn mse_code_len_1536_4bit() {
    // 1536 * 4 bits / 8 = 768 (QJL omitted at tiled 1536 @ 4-bit)
    assert_eq!(mse_code_len(1536, 4), 768);
}

#[test]
fn qjl_code_len_1536() {
    // 1536 / 8 = 192
    assert_eq!(qjl_code_len(1536), 192);
}

// --- Struct sizes ---

#[test]
fn item_pointer_struct_size() {
    assert_eq!(std::mem::size_of::<ItemPointer>(), 8); // u32 + u16 + padding
}

#[test]
fn item_pointer_wire_size() {
    assert_eq!(ITEM_POINTER_BYTES, 6);
}

#[test]
fn page_header_size() {
    assert_eq!(PAGE_HEADER_BYTES, 24);
}

#[test]
fn heaptid_inline_capacity() {
    assert_eq!(HEAPTID_INLINE_CAPACITY, 10);
}

// --- Element tuple encoded length ---

#[test]
fn element_tuple_encoded_len_1536_4bit() {
    let code_len = payload_len(1536, 4) - 4; // 768 packed code bytes
    assert_eq!(code_len, 768);

    // tag(1) + level(1) + deleted(1) + 10*ItemPointer(60) + count(1) + gamma(4) + neighbortid(6) + code(768)
    let expected = 1
        + 1
        + 1
        + (HEAPTID_INLINE_CAPACITY * ITEM_POINTER_BYTES)
        + 1
        + 4
        + ITEM_POINTER_BYTES
        + code_len;
    assert_eq!(expected, 842);
    assert_eq!(TqElementTuple::encoded_len(code_len), expected);
}

// --- Compression ratio contract ---

#[test]
fn compression_ratio_1536_4bit() {
    let raw_fp32 = 1536 * 4; // 6144
    let tqvector_datum = 11 + payload_len(1536, 4); // header + payload = 11 + 772 = 783
    let ratio = raw_fp32 as f64 / tqvector_datum as f64;
    assert!(
        ratio >= 7.8,
        "compression ratio = {ratio:.2}x, expected >= 7.8x"
    );
}
