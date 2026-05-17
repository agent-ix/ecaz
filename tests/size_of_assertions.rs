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

const _: () = {
    assert!(VAMANA_NODE_HEADER_FIXED_BYTES == 16);
    assert!(VAMANA_NODE_TAG_OFFSET == 0);
    assert!(VAMANA_NODE_FLAGS_OFFSET == 1);
    assert!(VAMANA_NODE_NEIGHBOR_COUNT_OFFSET == 2);
    assert!(VAMANA_NODE_PRIMARY_HEAPTID_OFFSET == 4);
    assert!(VAMANA_NODE_RERANK_TID_OFFSET == 10);
    assert!(VAMANA_NODE_BINARY_WORDS_OFFSET == 16);
    assert!(vamana_node_search_code_offset(2) == 32);
    assert!(vamana_node_neighbors_offset(2, 8) == 40);

    assert!(VAMANA_CODEBOOK_TAG_OFFSET == 0);
    assert!(VAMANA_CODEBOOK_GROUP_INDEX_OFFSET == 1);
    assert!(VAMANA_CODEBOOK_NEXTTID_OFFSET == 3);
    assert!(VAMANA_CODEBOOK_CENTROIDS_OFFSET == 9);
};

const _: () = {
    assert!(EC_IVF_INDEX_FORMAT_VERSION == 1);
    assert!(EC_IVF_METADATA_MAGIC == 0x5649_4345);
    assert!(EC_IVF_METADATA_BYTES == 80);
    assert!(EC_IVF_METADATA_MAGIC_OFFSET == 0);
    assert!(EC_IVF_METADATA_FORMAT_VERSION_OFFSET == 4);
    assert!(EC_IVF_METADATA_DIMENSIONS_OFFSET == 6);
    assert!(EC_IVF_METADATA_NLISTS_OFFSET == 8);
    assert!(EC_IVF_METADATA_NPROBE_OFFSET == 12);
    assert!(EC_IVF_METADATA_TRAINING_SAMPLE_ROWS_OFFSET == 16);
    assert!(EC_IVF_METADATA_TRAINING_VERSION_OFFSET == 20);
    assert!(EC_IVF_METADATA_SEED_OFFSET == 24);
    assert!(EC_IVF_METADATA_STORAGE_FORMAT_OFFSET == 32);
    assert!(EC_IVF_METADATA_RERANK_OFFSET == 33);
    assert!(EC_IVF_METADATA_CENTROID_HEAD_OFFSET == 36);
    assert!(EC_IVF_METADATA_DIRECTORY_HEAD_OFFSET == 42);
    assert!(EC_IVF_METADATA_TOTAL_LIVE_TUPLES_OFFSET == 48);
    assert!(EC_IVF_METADATA_TOTAL_DEAD_TUPLES_OFFSET == 56);
    assert!(EC_IVF_METADATA_INSERTED_SINCE_BUILD_OFFSET == 64);
    assert!(EC_IVF_METADATA_PQ_CODEBOOK_HEAD_OFFSET == 72);
    assert!(EC_IVF_METADATA_PQ_GROUP_SIZE_OFFSET == 78);

    assert!(EC_IVF_BLOCK_REF_BYTES == 4);
    assert!(EC_IVF_BLOCK_REF_BLOCK_NUMBER_OFFSET == 0);
    assert!(EC_IVF_CENTROID_TAG_OFFSET == 0);
    assert!(EC_IVF_CENTROID_LIST_ID_OFFSET == 1);
    assert!(EC_IVF_CENTROID_DIMENSIONS_OFFSET == 5);
    assert!(EC_IVF_CENTROID_VALUES_OFFSET == 7);

    assert!(EC_IVF_LIST_DIRECTORY_BYTES == 37);
    assert!(EC_IVF_LIST_DIRECTORY_TAG_OFFSET == 0);
    assert!(EC_IVF_LIST_DIRECTORY_LIST_ID_OFFSET == 1);
    assert!(EC_IVF_LIST_DIRECTORY_HEAD_BLOCK_OFFSET == 5);
    assert!(EC_IVF_LIST_DIRECTORY_TAIL_BLOCK_OFFSET == 9);
    assert!(EC_IVF_LIST_DIRECTORY_LIVE_COUNT_OFFSET == 13);
    assert!(EC_IVF_LIST_DIRECTORY_DEAD_COUNT_OFFSET == 21);
    assert!(EC_IVF_LIST_DIRECTORY_INSERTED_SINCE_BUILD_OFFSET == 29);

    assert!(EC_IVF_POSTING_TAG_OFFSET == 0);
    assert!(EC_IVF_POSTING_LIST_ID_OFFSET == 1);
    assert!(EC_IVF_POSTING_FLAGS_OFFSET == 5);
    assert!(EC_IVF_POSTING_HEAPTID_COUNT_OFFSET == 6);
    assert!(EC_IVF_POSTING_HEAPTIDS_OFFSET == 7);
    assert!(EC_IVF_POSTING_GAMMA_OFFSET == 67);
    assert!(EC_IVF_POSTING_RERANK_TID_OFFSET == 71);
    assert!(EC_IVF_POSTING_PAYLOAD_OFFSET == 77);

    assert!(EC_IVF_PQ_CODEBOOK_TAG_OFFSET == 0);
    assert!(EC_IVF_PQ_CODEBOOK_GROUP_INDEX_OFFSET == 1);
    assert!(EC_IVF_PQ_CODEBOOK_NEXT_TID_OFFSET == 3);
    assert!(EC_IVF_PQ_CODEBOOK_CENTROIDS_OFFSET == 9);
};

const _: () = {
    assert!(SPIRE_PARTITION_OBJECT_MAGIC == 0x4f50_5345);
    assert!(SPIRE_PARTITION_OBJECT_FORMAT_VERSION_V1 == 1);
    assert!(SPIRE_PARTITION_OBJECT_FORMAT_VERSION_V2 == 2);
    assert!(SPIRE_PARTITION_OBJECT_HEADER_BYTES == 54);
    assert!(SPIRE_PARTITION_OBJECT_MAGIC_OFFSET == 0);
    assert!(SPIRE_PARTITION_OBJECT_FORMAT_VERSION_OFFSET == 4);
    assert!(SPIRE_PARTITION_OBJECT_KIND_OFFSET == 6);
    assert!(SPIRE_PARTITION_OBJECT_RESERVED_OFFSET == 7);
    assert!(SPIRE_PARTITION_OBJECT_PID_OFFSET == 8);
    assert!(SPIRE_PARTITION_OBJECT_OBJECT_VERSION_OFFSET == 16);
    assert!(SPIRE_PARTITION_OBJECT_PUBLISHED_EPOCH_BACKREF_OFFSET == 24);
    assert!(SPIRE_PARTITION_OBJECT_LEVEL_OFFSET == 32);
    assert!(SPIRE_PARTITION_OBJECT_PARENT_PID_OFFSET == 34);
    assert!(SPIRE_PARTITION_OBJECT_CHILD_COUNT_OFFSET == 42);
    assert!(SPIRE_PARTITION_OBJECT_ASSIGNMENT_COUNT_OFFSET == 46);
    assert!(SPIRE_PARTITION_OBJECT_FLAGS_OFFSET == 50);

    assert!(SPIRE_ASSIGNMENT_ROW_FIXED_PREFIX_BYTES == 3);
    assert!(SPIRE_ASSIGNMENT_ROW_FIXED_TAIL_BYTES == 15);
    assert!(SPIRE_ASSIGNMENT_ROW_FLAGS_OFFSET == 0);
    assert!(SPIRE_ASSIGNMENT_ROW_VEC_ID_LEN_OFFSET == 2);
    assert!(SPIRE_ASSIGNMENT_ROW_VEC_ID_OFFSET == 3);
    assert!(spire_assignment_row_heap_tid_offset(9) == 12);
    assert!(spire_assignment_row_payload_format_offset(9) == 18);
    assert!(spire_assignment_row_gamma_offset(9) == 19);
    assert!(spire_assignment_row_payload_len_offset(9) == 23);
    assert!(spire_assignment_row_payload_offset(9) == 27);

    assert!(SPIRE_LEAF_V2_META_FLAG == 0x0000_0001);
    assert!(SPIRE_LEAF_V2_SEGMENT_FLAG == 0x0000_0002);
    assert!(SPIRE_PARTITION_OBJECT_V2_CHAIN_META_FLAG == 0x0000_0004);
    assert!(SPIRE_PARTITION_OBJECT_V2_CHAIN_SEGMENT_FLAG == 0x0000_0008);
    assert!(SPIRE_LEAF_V2_LOCAL_VEC_ID_STRIDE == 16);
    assert!(SPIRE_LEAF_V2_META_BODY_BYTES == 30);
    assert!(SPIRE_LEAF_V2_META_PAYLOAD_FORMAT_OFFSET == 0);
    assert!(SPIRE_LEAF_V2_META_VEC_ID_KIND_OFFSET == 1);
    assert!(SPIRE_LEAF_V2_META_RESERVED_OFFSET == 2);
    assert!(SPIRE_LEAF_V2_META_PAYLOAD_STRIDE_OFFSET == 4);
    assert!(SPIRE_LEAF_V2_META_VEC_ID_STRIDE_OFFSET == 8);
    assert!(SPIRE_LEAF_V2_META_RESERVED2_OFFSET == 10);
    assert!(SPIRE_LEAF_V2_META_SEGMENT_COUNT_OFFSET == 12);
    assert!(SPIRE_LEAF_V2_META_FIRST_SEGMENT_LOCATOR_OFFSET == 16);
    assert!(SPIRE_LEAF_V2_META_OBJECT_BYTES_TOTAL_OFFSET == 22);

    assert!(SPIRE_LEAF_V2_SEGMENT_PREFIX_BYTES == 18);
    assert!(SPIRE_LEAF_V2_SEGMENT_NO_OFFSET == 0);
    assert!(SPIRE_LEAF_V2_SEGMENT_ROW_BASE_OFFSET == 4);
    assert!(SPIRE_LEAF_V2_SEGMENT_ROW_COUNT_OFFSET == 8);
    assert!(SPIRE_LEAF_V2_SEGMENT_NEXT_LOCATOR_OFFSET == 12);
    assert!(SPIRE_LEAF_V2_SEGMENT_FLAGS_OFFSET == 18);
    assert!(spire_leaf_v2_segment_vec_ids_offset(2) == 22);
    assert!(spire_leaf_v2_segment_heap_tids_offset(2, 16) == 54);
    assert!(spire_leaf_v2_segment_gammas_offset(2, 16) == 66);
    assert!(spire_leaf_v2_segment_payloads_offset(2, 16) == 74);

    assert!(SPIRE_PARTITION_OBJECT_V2_CHAIN_META_BODY_BYTES == 22);
    assert!(SPIRE_PARTITION_OBJECT_V2_CHAIN_META_DIMENSIONS_OFFSET == 0);
    assert!(SPIRE_PARTITION_OBJECT_V2_CHAIN_META_RESERVED_OFFSET == 2);
    assert!(SPIRE_PARTITION_OBJECT_V2_CHAIN_META_SEGMENT_COUNT_OFFSET == 4);
    assert!(SPIRE_PARTITION_OBJECT_V2_CHAIN_META_FIRST_SEGMENT_LOCATOR_OFFSET == 8);
    assert!(SPIRE_PARTITION_OBJECT_V2_CHAIN_META_OBJECT_BYTES_TOTAL_OFFSET == 14);
    assert!(SPIRE_PARTITION_OBJECT_V2_CHAIN_SEGMENT_PREFIX_BYTES == 14);
    assert!(SPIRE_PARTITION_OBJECT_V2_CHAIN_SEGMENT_NO_OFFSET == 0);
    assert!(SPIRE_PARTITION_OBJECT_V2_CHAIN_SEGMENT_BYTE_BASE_OFFSET == 4);
    assert!(SPIRE_PARTITION_OBJECT_V2_CHAIN_SEGMENT_NEXT_LOCATOR_OFFSET == 8);
    assert!(SPIRE_PARTITION_OBJECT_V2_CHAIN_SEGMENT_PAYLOAD_OFFSET == 14);
};

const _: () = {
    assert!(SPIRE_META_FORMAT_VERSION == 1);
    assert!(SPIRE_LOCAL_STORE_CONFIG_MAGIC == 0x534c_5345);
    assert!(SPIRE_LOCAL_STORE_CONFIG_HEADER_BYTES == 20);
    assert!(SPIRE_LOCAL_STORE_CONFIG_MAGIC_OFFSET == 0);
    assert!(SPIRE_LOCAL_STORE_CONFIG_FORMAT_VERSION_OFFSET == 4);
    assert!(SPIRE_LOCAL_STORE_CONFIG_RESERVED_OFFSET == 6);
    assert!(SPIRE_LOCAL_STORE_CONFIG_GENERATION_OFFSET == 8);
    assert!(SPIRE_LOCAL_STORE_CONFIG_STORE_COUNT_OFFSET == 16);
    assert!(SPIRE_LOCAL_STORE_DESCRIPTOR_BYTES == 16);
    assert!(SPIRE_LOCAL_STORE_DESCRIPTOR_LOCAL_STORE_ID_OFFSET == 0);
    assert!(SPIRE_LOCAL_STORE_DESCRIPTOR_STORE_RELID_OFFSET == 4);
    assert!(SPIRE_LOCAL_STORE_DESCRIPTOR_TABLESPACE_OID_OFFSET == 8);
    assert!(SPIRE_LOCAL_STORE_DESCRIPTOR_STATE_OFFSET == 12);
    assert!(SPIRE_LOCAL_STORE_DESCRIPTOR_RESERVED_OFFSET == 13);

    assert!(SPIRE_PLACEMENT_ENTRY_BYTES == 50);
    assert!(SPIRE_PLACEMENT_ENTRY_FORMAT_VERSION_OFFSET == 0);
    assert!(SPIRE_PLACEMENT_ENTRY_STATE_OFFSET == 2);
    assert!(SPIRE_PLACEMENT_ENTRY_RESERVED_OFFSET == 3);
    assert!(SPIRE_PLACEMENT_ENTRY_EPOCH_OFFSET == 4);
    assert!(SPIRE_PLACEMENT_ENTRY_PID_OFFSET == 12);
    assert!(SPIRE_PLACEMENT_ENTRY_NODE_ID_OFFSET == 20);
    assert!(SPIRE_PLACEMENT_ENTRY_LOCAL_STORE_ID_OFFSET == 24);
    assert!(SPIRE_PLACEMENT_ENTRY_STORE_RELID_OFFSET == 28);
    assert!(SPIRE_PLACEMENT_ENTRY_OBJECT_VERSION_OFFSET == 32);
    assert!(SPIRE_PLACEMENT_ENTRY_OBJECT_TID_OFFSET == 40);
    assert!(SPIRE_PLACEMENT_ENTRY_OBJECT_BYTES_OFFSET == 46);

    assert!(SPIRE_PLACEMENT_DIRECTORY_MAGIC == 0x4450_5345);
    assert!(SPIRE_PLACEMENT_DIRECTORY_HEADER_BYTES == 20);
    assert!(SPIRE_PLACEMENT_DIRECTORY_MAGIC_OFFSET == 0);
    assert!(SPIRE_PLACEMENT_DIRECTORY_FORMAT_VERSION_OFFSET == 4);
    assert!(SPIRE_PLACEMENT_DIRECTORY_RESERVED_OFFSET == 6);
    assert!(SPIRE_PLACEMENT_DIRECTORY_EPOCH_OFFSET == 8);
    assert!(SPIRE_PLACEMENT_DIRECTORY_ENTRY_COUNT_OFFSET == 16);

    assert!(SPIRE_EPOCH_MANIFEST_MAGIC == 0x454d_5345);
    assert!(SPIRE_EPOCH_MANIFEST_BYTES == 40);
    assert!(SPIRE_EPOCH_MANIFEST_MAGIC_OFFSET == 0);
    assert!(SPIRE_EPOCH_MANIFEST_FORMAT_VERSION_OFFSET == 4);
    assert!(SPIRE_EPOCH_MANIFEST_STATE_OFFSET == 6);
    assert!(SPIRE_EPOCH_MANIFEST_CONSISTENCY_MODE_OFFSET == 7);
    assert!(SPIRE_EPOCH_MANIFEST_EPOCH_OFFSET == 8);
    assert!(SPIRE_EPOCH_MANIFEST_PUBLISHED_AT_MICROS_OFFSET == 16);
    assert!(SPIRE_EPOCH_MANIFEST_RETAIN_UNTIL_MICROS_OFFSET == 24);
    assert!(SPIRE_EPOCH_MANIFEST_ACTIVE_QUERY_COUNT_OFFSET == 32);

    assert!(SPIRE_OBJECT_MANIFEST_MAGIC == 0x4d4f_5345);
    assert!(SPIRE_OBJECT_MANIFEST_HEADER_BYTES == 20);
    assert!(SPIRE_OBJECT_MANIFEST_MAGIC_OFFSET == 0);
    assert!(SPIRE_OBJECT_MANIFEST_FORMAT_VERSION_OFFSET == 4);
    assert!(SPIRE_OBJECT_MANIFEST_RESERVED_OFFSET == 6);
    assert!(SPIRE_OBJECT_MANIFEST_EPOCH_OFFSET == 8);
    assert!(SPIRE_OBJECT_MANIFEST_ENTRY_COUNT_OFFSET == 16);
    assert!(SPIRE_MANIFEST_ENTRY_BYTES == 34);
    assert!(SPIRE_MANIFEST_ENTRY_FORMAT_VERSION_OFFSET == 0);
    assert!(SPIRE_MANIFEST_ENTRY_RESERVED_OFFSET == 2);
    assert!(SPIRE_MANIFEST_ENTRY_EPOCH_OFFSET == 4);
    assert!(SPIRE_MANIFEST_ENTRY_PID_OFFSET == 12);
    assert!(SPIRE_MANIFEST_ENTRY_OBJECT_VERSION_OFFSET == 20);
    assert!(SPIRE_MANIFEST_ENTRY_PLACEMENT_TID_OFFSET == 28);
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
