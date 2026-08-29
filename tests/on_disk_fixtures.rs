//! Golden on-disk fixture decode checks.

use sha2::{Digest, Sha256};

use ecaz::bench_api::{
    distann_restore_owner_stream_hash_state, spire_decode_delta_partition_object_fixture,
    spire_decode_leaf_partition_object_fixture, spire_decode_leaf_v2_meta_fixture,
    spire_decode_leaf_v2_segment_fixture, spire_decode_partition_object_v2_chain_meta_fixture,
    spire_decode_partition_object_v2_chain_segment_fixture,
    spire_decode_routing_partition_object_fixture, spire_decode_top_graph_partition_object_fixture,
    vamana_decode_overflow_tuple_fixture, DistannAbandonBindingAuditV1,
    DistannAbandonedBindingSetV1, DistannBuildCandidateV1, DistannBuildSpec,
    DistannCancelPublishAuditV1, DistannCodecArtifact, DistannEpochFingerprint,
    DistannEpochManifestV2, DistannGenerationDescriptor, DistannHandoffBatch, DistannHandoffEntry,
    DistannHandoffShape, DistannManifestBuildOptions, DistannManifestCodecParameters,
    DistannMetadataPage, DistannNodeTuple, DistannReadyReceipt, DistannRetireDecisionV1,
    DistannRowSchemaDescriptor, DistannRowTierLayoutDescriptorV1, DistannSourceSnapshot,
    DistannSuccessorActivationV1, ItemPointer, IvfBlockRef, IvfCentroidTuple,
    IvfListDirectoryTuple, IvfMetadataPage, IvfPostingTuple, IvfPqCodebookTuple, IvfRerankMode,
    IvfRerankScoreMode, IvfStorageFormat, MetadataPage, SpireConsistencyMode, SpireEpochManifest,
    SpireEpochState, SpireLocalStoreConfig, SpireLocalStoreState, SpireManifestEntry,
    SpireObjectManifest, SpirePlacementDirectory, SpirePlacementEntry, SpirePlacementState,
    TqElementTuple, TqGroupedCodebookTuple, TqGroupedHotTuple, TqNeighborTuple, TqRerankTuple,
    TqTurboHotTuple, VamanaCodebookTuple, VamanaMetadataPage, VamanaNodeTuple,
    DISTANN_CONTROL_METADATA_BYTES, DISTANN_EPOCH_FINGERPRINT_BYTES, DISTANN_METADATA_BYTES,
    DISTANN_METADATA_FORMAT_VERSION_OFFSET, DISTANN_NODE_FORMAT_VERSION_OFFSET,
    DISTANN_OWNER_STREAM_HASH_STATE_BLOCK_COUNT_OFFSET,
    DISTANN_OWNER_STREAM_HASH_STATE_BUFFER_LENGTH_OFFSET,
    DISTANN_OWNER_STREAM_HASH_STATE_BUFFER_OFFSET, DISTANN_OWNER_STREAM_HASH_STATE_BYTES,
    DISTANN_OWNER_STREAM_HASH_STATE_CHAIN_OFFSET,
    DISTANN_OWNER_STREAM_HASH_STATE_IMPLEMENTATION_OFFSET,
    DISTANN_OWNER_STREAM_HASH_STATE_VERSION_OFFSET, EC_IVF_CENTROID_DIMENSIONS_OFFSET,
    EC_IVF_METADATA_FORMAT_VERSION_OFFSET, HNSW_METADATA_FORMAT_VERSION_OFFSET,
    INDEX_FORMAT_V1_DISTANN, INDEX_FORMAT_V3_DISKANN, INDEX_FORMAT_V5_DISTANN_CONTROL,
    SPIRE_EPOCH_MANIFEST_FORMAT_VERSION_OFFSET, SPIRE_LOCAL_STORE_CONFIG_FORMAT_VERSION_OFFSET,
    SPIRE_MANIFEST_ENTRY_FORMAT_VERSION_OFFSET, SPIRE_OBJECT_MANIFEST_FORMAT_VERSION_OFFSET,
    SPIRE_PARTITION_OBJECT_FORMAT_VERSION_OFFSET, SPIRE_PLACEMENT_DIRECTORY_FORMAT_VERSION_OFFSET,
    SPIRE_PLACEMENT_ENTRY_FORMAT_VERSION_OFFSET, VAMANA_METADATA_FORMAT_VERSION_OFFSET,
    VAMANA_NODE_NEIGHBOR_COUNT_OFFSET,
};

fn assert_distann_domain_digest(bytes: &[u8], domain: &[u8], expected_hex: &str) {
    let mut hasher = Sha256::new();
    hasher.update(domain);
    hasher.update(bytes);
    assert_eq!(hex::encode(hasher.finalize()), expected_hex);
}

fn decode_hex_fixture(contents: &str) -> Vec<u8> {
    let hex = contents
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect::<String>();
    hex::decode(hex.trim()).expect("fixture hex should decode")
}

/// Deliberately independent little-endian reader for TC-050. These checks do
/// not call the production canonical decoder, so a matching encoder/decoder
/// bug cannot silently bless every golden fixture.
struct DistannFixtureReader<'a> {
    bytes: &'a [u8],
    position: usize,
}

impl<'a> DistannFixtureReader<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, position: 0 }
    }

    fn take(&mut self, length: usize) -> &'a [u8] {
        let end = self.position.checked_add(length).expect("fixture offset");
        assert!(
            end <= self.bytes.len(),
            "independent fixture decode truncated"
        );
        let value = &self.bytes[self.position..end];
        self.position = end;
        value
    }

    fn u8(&mut self) -> u8 {
        self.take(1)[0]
    }

    fn u16(&mut self) -> u16 {
        u16::from_le_bytes(self.take(2).try_into().unwrap())
    }

    fn u32(&mut self) -> u32 {
        u32::from_le_bytes(self.take(4).try_into().unwrap())
    }

    fn u64(&mut self) -> u64 {
        u64::from_le_bytes(self.take(8).try_into().unwrap())
    }

    fn i64(&mut self) -> i64 {
        i64::from_le_bytes(self.take(8).try_into().unwrap())
    }

    fn len_bytes(&mut self) -> &'a [u8] {
        let length = self.u32() as usize;
        self.take(length)
    }

    fn finish(self) {
        assert_eq!(self.position, self.bytes.len(), "fixture trailing bytes");
    }
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
fn hnsw_metadata_v4_rabitq_fixture_decodes() {
    let bytes = decode_hex_fixture(include_str!(
        "../fixtures/on-disk/hnsw_metadata_v4_rabitq.hex"
    ));

    let metadata = MetadataPage::decode(&bytes).expect("hnsw v4 RaBitQ metadata should decode");

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
    assert_eq!(metadata.format_version, 4);
    assert_eq!(metadata.transform_kind as u8, 1);
    assert_eq!(metadata.search_codec_kind as u8, 3);
    assert_eq!(metadata.payload_flags, 1 << 2);
    assert_eq!(metadata.search_bits, 1);
    assert_eq!(metadata.rerank_codec_kind as u8, 1);
    assert_eq!(metadata.search_subvector_count, 0);
    assert_eq!(metadata.search_subvector_dim, 1);
    assert_eq!(
        metadata.grouped_codebook_head,
        ItemPointer {
            block_number: u32::MAX,
            offset_number: u16::MAX
        }
    );
}

#[test]
fn hnsw_metadata_v4_rabitq_byteswapped_version_is_rejected() {
    let mut bytes = decode_hex_fixture(include_str!(
        "../fixtures/on-disk/hnsw_metadata_v4_rabitq.hex"
    ));
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
fn distann_metadata_v4_and_control_v5_fixtures_decode_independently() {
    let legacy = decode_hex_fixture(include_str!("../fixtures/on-disk/distann_metadata_v4.hex"));
    assert_eq!(legacy.len(), DISTANN_METADATA_BYTES);
    assert_eq!(
        u16::from_le_bytes(legacy[0..2].try_into().unwrap()),
        INDEX_FORMAT_V1_DISTANN
    );
    assert_eq!(u64::from_le_bytes(legacy[36..44].try_into().unwrap()), 42);
    let legacy_decoded = DistannMetadataPage::decode(&legacy).unwrap();
    assert!(!legacy_decoded.is_distributed_control());
    assert_eq!(legacy_decoded.dimensions, 128);
    assert_eq!(legacy_decoded.content_digest, 0xA1A2_A3A4_A5A6_A7A8);

    let control = decode_hex_fixture(include_str!(
        "../fixtures/on-disk/distann_control_metadata_v5.hex"
    ));
    let mut expected_uuid = [0xA5; 16];
    expected_uuid[6] = 0x45;
    expected_uuid[8] = 0x85;
    assert_eq!(control.len(), DISTANN_CONTROL_METADATA_BYTES);
    assert_eq!(
        u16::from_le_bytes(control[0..2].try_into().unwrap()),
        INDEX_FORMAT_V5_DISTANN_CONTROL
    );
    assert_eq!(&control[97..113], &expected_uuid);
    let control_decoded = DistannMetadataPage::decode(&control).unwrap();
    assert!(control_decoded.is_distributed_control());
    assert_eq!(control_decoded.logical_index_uuid, expected_uuid);
    assert_eq!(control_decoded.node_count, 0);
    assert_eq!(control_decoded.active_epoch, 0);
}

#[test]
fn distann_metadata_versions_reject_byte_swap() {
    for fixture in [
        include_str!("../fixtures/on-disk/distann_metadata_v4.hex"),
        include_str!("../fixtures/on-disk/distann_control_metadata_v5.hex"),
    ] {
        let mut bytes = decode_hex_fixture(fixture);
        bytes.swap(
            DISTANN_METADATA_FORMAT_VERSION_OFFSET,
            DISTANN_METADATA_FORMAT_VERSION_OFFSET + 1,
        );
        assert!(DistannMetadataPage::decode(&bytes).is_err());
    }
}

#[test]
fn distann_physical_graph_record_v1_fixture_decodes_and_rejects_swap() {
    let bytes = decode_hex_fixture(include_str!(
        "../fixtures/on-disk/distann_graph_record_v1.hex"
    ));
    let mut independent = DistannFixtureReader::new(&bytes);
    assert_eq!(independent.u16(), 1);
    assert_eq!(independent.u16(), 0);
    assert_eq!(independent.u64(), 0x1122_3344_5566_7788);
    assert_eq!(independent.u32(), 9);
    assert_eq!(independent.u16(), 3);
    assert_eq!(independent.u16(), 2);
    assert_eq!(independent.take(2), [0xA1, 0xA2]);
    assert_eq!(independent.u64(), 101);
    assert_eq!(independent.u64(), 202);
    assert_eq!(independent.u64(), 0);
    assert_eq!(independent.u64(), 0);
    assert_eq!(independent.take(8), [1, 2, 3, 4, 0, 0, 0, 0]);
    independent.finish();

    let record = DistannNodeTuple::decode_physical_v1(&bytes, 4, 2).unwrap();
    assert_eq!(record.vec_id, 0x1122_3344_5566_7788);
    assert_eq!(record.neighbor_count, 2);
    assert_eq!(record.neighbor_vec_ids, vec![101, 202, 0, 0]);

    let mut swapped = bytes;
    swapped.swap(
        DISTANN_NODE_FORMAT_VERSION_OFFSET,
        DISTANN_NODE_FORMAT_VERSION_OFFSET + 1,
    );
    assert!(DistannNodeTuple::decode_physical_v1(&swapped, 4, 2).is_err());
}

#[test]
fn distann_physical_graph_record_v2_fixture_appends_cold_tid() {
    let v1 = decode_hex_fixture(include_str!(
        "../fixtures/on-disk/distann_graph_record_v1.hex"
    ));
    let v2 = decode_hex_fixture(include_str!(
        "../fixtures/on-disk/distann_graph_record_v2.hex"
    ));
    let mut independent = DistannFixtureReader::new(&v2);
    assert_eq!(independent.u16(), 2);
    assert_eq!(independent.u16(), 0);
    assert_eq!(independent.u64(), 0x1122_3344_5566_7788);
    assert_eq!(independent.u32(), 9);
    assert_eq!(independent.u16(), 3);
    assert_eq!(independent.u16(), 2);
    assert_eq!(independent.take(2), [0xA1, 0xA2]);
    assert_eq!(independent.u64(), 101);
    assert_eq!(independent.u64(), 202);
    assert_eq!(independent.u64(), 0);
    assert_eq!(independent.u64(), 0);
    assert_eq!(independent.take(8), [1, 2, 3, 4, 0, 0, 0, 0]);
    assert_eq!(independent.u32(), 29);
    assert_eq!(independent.u16(), 7);
    independent.finish();

    assert_eq!(&v2[2..v1.len()], &v1[2..]);
    let record = DistannNodeTuple::decode_physical_v2(&v2, 4, 2).unwrap();
    assert_eq!(
        record.cold_tid,
        Some(ItemPointer {
            block_number: 29,
            offset_number: 7,
        })
    );
    assert!(DistannNodeTuple::decode_physical_v1(&v2, 4, 2).is_err());
}

#[test]
fn distann_row_schema_v1_fixture_decodes_independently_and_rejects_swap() {
    let bytes = decode_hex_fixture(include_str!(
        "../fixtures/on-disk/distann_row_schema_v1.hex"
    ));
    let mut independent = DistannFixtureReader::new(&bytes);
    assert_eq!(independent.u16(), 1);
    let count = independent.u16();
    assert_eq!(count, 3);
    let mut attnums = Vec::new();
    for _ in 0..count {
        attnums.push(independent.u16());
        independent.len_bytes(); // attribute name
        independent.len_bytes(); // type namespace
        independent.len_bytes(); // type name
        independent.take(4); // typmod
        independent.len_bytes(); // collation namespace
        independent.len_bytes(); // collation name
        independent.u8(); // dropped
        independent.u8(); // generated
        independent.len_bytes(); // send function
        independent.len_bytes(); // receive function
    }
    independent.finish();
    assert_eq!(attnums, vec![1, 2, 3]);

    let schema = DistannRowSchemaDescriptor::decode(&bytes).unwrap();
    assert_eq!(schema.attributes.len(), 3);
    assert!(schema.attributes[1].dropped);
    assert_eq!(schema.attributes[2].type_name, "ecvector");

    let mut swapped = bytes;
    swapped.swap(0, 1);
    assert!(DistannRowSchemaDescriptor::decode(&swapped).is_err());
}

#[test]
fn distann_codec_artifact_v1_fixture_decodes_independently_and_rejects_swap() {
    let bytes = decode_hex_fixture(include_str!(
        "../fixtures/on-disk/distann_codec_artifact_v1.hex"
    ));
    let mut independent = DistannFixtureReader::new(&bytes);
    assert_eq!(independent.u16(), 1);
    assert_eq!(independent.u8(), 1);
    assert_eq!(independent.u16(), 4);
    assert_eq!(independent.u64(), 99);
    assert_eq!(independent.u32(), 4);
    let sign_count = independent.u32();
    assert_eq!(sign_count, 4);
    independent.take(sign_count as usize * 4);
    let group_count = independent.u32();
    assert_eq!(group_count, 2);
    assert_eq!(independent.u32(), 2);
    assert_eq!(independent.u16(), 16);
    for _ in 0..group_count {
        let value_count = independent.u32();
        assert_eq!(value_count, 32);
        independent.take(value_count as usize * 4);
    }
    independent.finish();

    match DistannCodecArtifact::decode(&bytes).unwrap() {
        DistannCodecArtifact::GroupedPq4 {
            dimensions, model, ..
        } => {
            assert_eq!(dimensions, 4);
            assert_eq!(model.group_count, 2);
            assert_eq!(model.codebooks.len(), 2);
        }
        other => panic!("unexpected codec fixture variant: {other:?}"),
    }

    let mut swapped = bytes;
    swapped.swap(0, 1);
    assert!(DistannCodecArtifact::decode(&swapped).is_err());
}

#[test]
fn distann_generation_descriptor_v1_fixture_is_rebuild_only_and_rejected() {
    let bytes = decode_hex_fixture(include_str!(
        "../fixtures/on-disk/distann_generation_descriptor_v1.hex"
    ));
    let mut independent = DistannFixtureReader::new(&bytes);
    assert_eq!(independent.u16(), 1);
    assert_eq!(independent.u16(), 5);
    assert_eq!(independent.u16(), 1);
    assert_eq!(independent.u16(), 1);
    assert_eq!(independent.u16(), 8);
    assert_eq!(independent.u16(), 4);
    assert_eq!(independent.u16(), 1);
    let roster_count = independent.u32();
    assert_eq!(roster_count, 2);
    for expected_node in [10, 20] {
        assert_eq!(independent.u32(), expected_node);
        independent.take(16);
        independent.len_bytes();
    }
    assert_eq!(independent.u8(), 2);
    independent.len_bytes();
    independent.len_bytes();
    independent.take(32);
    independent.finish();

    assert!(DistannGenerationDescriptor::decode(&bytes).is_err());

    let mut swapped = bytes;
    swapped.swap(0, 1);
    assert!(DistannGenerationDescriptor::decode(&swapped).is_err());
}

#[test]
fn distann_generation_descriptor_v2_fixture_decodes_independently_and_rejects_swap() {
    let bytes = decode_hex_fixture(include_str!(
        "../fixtures/on-disk/distann_generation_descriptor_v2.hex"
    ));
    let mut independent = DistannFixtureReader::new(&bytes);
    assert_eq!(independent.u16(), 2);
    let coordinator_uuid = independent.take(16);
    assert_eq!(coordinator_uuid[6] >> 4, 4);
    assert_eq!(coordinator_uuid[8] >> 6, 2);
    assert_eq!(independent.u16(), 5);
    assert_eq!(independent.u16(), 1);
    assert_eq!(independent.u16(), 1);
    assert_eq!(independent.u16(), 8);
    assert_eq!(independent.u16(), 4);
    assert_eq!(independent.u16(), 1);
    let roster_count = independent.u32();
    assert_eq!(roster_count, 2);
    for expected_node in [10, 20] {
        assert_eq!(independent.u32(), expected_node);
        independent.take(16);
        independent.len_bytes();
    }
    assert_eq!(independent.u8(), 2);
    independent.len_bytes();
    independent.len_bytes();
    independent.take(32);
    independent.finish();

    let descriptor = DistannGenerationDescriptor::decode(&bytes).unwrap();
    assert_eq!(descriptor.roster.len(), 2);
    assert_eq!(descriptor.roster[1].node_id, 20);
    assert_eq!(descriptor.dimensions, 8);
    assert_eq!(descriptor.coordinator_logical_index_uuid, coordinator_uuid);
    assert_eq!(descriptor.encode().unwrap(), bytes);

    let mut swapped = bytes;
    swapped.swap(0, 1);
    assert!(DistannGenerationDescriptor::decode(&swapped).is_err());
}

#[test]
fn distann_row_tier_layout_v1_fixture_decodes_independently() {
    let bytes = decode_hex_fixture(include_str!(
        "../fixtures/on-disk/distann_row_tier_layout_v1.hex"
    ));
    let mut independent = DistannFixtureReader::new(&bytes);
    assert_eq!(independent.u16(), 1);
    assert_eq!(independent.u16(), 16);
    assert_eq!(independent.u16(), 8);
    assert_eq!(independent.u32(), 84);
    assert_eq!(independent.u16(), 16);
    independent.take(32); // row-schema fingerprint
    assert_eq!(independent.u16(), 3); // indexed vector attnum
    assert_eq!(independent.u16(), 1); // source identity attnum
    assert_eq!(independent.u16(), 0); // optional hot scalars
    assert_eq!(independent.u16(), 3); // complete live partition
    assert_eq!(
        (independent.u16(), independent.u8(), independent.u16()),
        (1, 1, 2)
    );
    assert_eq!(
        (independent.u16(), independent.u8(), independent.u16()),
        (3, 1, 3)
    );
    assert_eq!(
        (independent.u16(), independent.u8(), independent.u16()),
        (4, 2, 2)
    );
    independent.finish();

    let layout = DistannRowTierLayoutDescriptorV1::decode(&bytes).unwrap();
    assert_eq!(layout.encode().unwrap(), bytes);
    assert_distann_domain_digest(
        &bytes,
        b"ec_distann_row_tier_layout_v1\0",
        "73bc8ed94216f4e75afa1cadcc9abdf49b3e948bba8606dd7747bec097f115fc",
    );
}

#[test]
fn distann_generation_descriptor_v4_fixture_binds_layout_and_graph_v2() {
    let bytes = decode_hex_fixture(include_str!(
        "../fixtures/on-disk/distann_generation_descriptor_v4.hex"
    ));
    let layout_bytes = decode_hex_fixture(include_str!(
        "../fixtures/on-disk/distann_row_tier_layout_v1.hex"
    ));
    let mut independent = DistannFixtureReader::new(&bytes);
    assert_eq!(independent.u16(), 4);
    independent.take(16); // coordinator UUID
    assert_eq!(independent.u16(), 5); // index format
    assert_eq!(independent.u16(), 2); // graph record V2
    assert_eq!(independent.u16(), 1); // handoff wire
    assert_eq!(independent.u16(), 8); // dimensions
    assert_eq!(independent.u16(), 4); // graph degree
    assert_eq!(independent.u16(), 1); // placement hash
    assert_eq!(independent.u32(), 2);
    for expected_node in [10, 20] {
        assert_eq!(independent.u32(), expected_node);
        independent.take(16);
        independent.len_bytes();
    }
    assert_eq!(independent.u8(), 2);
    independent.len_bytes(); // codec artifact
    independent.len_bytes(); // row schema
    independent.take(32); // row-schema fingerprint
    assert_eq!(independent.len_bytes(), layout_bytes);
    assert_eq!(
        independent.take(32),
        hex::decode("73bc8ed94216f4e75afa1cadcc9abdf49b3e948bba8606dd7747bec097f115fc").unwrap()
    );
    independent.finish();

    let descriptor = DistannGenerationDescriptor::decode(&bytes).unwrap();
    assert_eq!(descriptor.graph_record_version, 2);
    assert!(descriptor.payload_cover().is_none());
    assert!(descriptor.row_tier_layout().is_some());
    assert_eq!(descriptor.encode().unwrap(), bytes);

    let mut corrupt_layout_digest = bytes;
    *corrupt_layout_digest.last_mut().unwrap() ^= 1;
    assert!(DistannGenerationDescriptor::decode(&corrupt_layout_digest).is_err());
}

#[test]
fn distann_build_registration_v1_fixture_decodes_and_digests_independently() {
    let bytes = decode_hex_fixture(include_str!(
        "../fixtures/on-disk/distann_build_registration_v1.hex"
    ));
    let mut independent = DistannFixtureReader::new(&bytes);
    assert_eq!(independent.u16(), 1);
    assert_eq!(independent.u32(), 1234);
    let coordinator_uuid = independent.take(16);
    assert_eq!(coordinator_uuid[6] >> 4, 4);
    assert_eq!(coordinator_uuid[8] >> 6, 2);
    assert_eq!(independent.u32(), 5678);
    assert_eq!(independent.u64(), 7);
    let build_id = independent.take(16);
    assert_eq!(build_id[6] >> 4, 4);
    assert_eq!(build_id[8] >> 6, 2);
    assert_eq!(independent.u64(), 9);

    let roster = independent.len_bytes();
    let mut roster_reader = DistannFixtureReader::new(roster);
    assert_eq!(roster_reader.u16(), 1);
    assert_eq!(roster_reader.u32(), 1);
    assert_eq!(roster_reader.u32(), 17);
    roster_reader.take(16);
    assert_eq!(roster_reader.len_bytes(), b"registration/node-17");
    roster_reader.finish();

    independent.take(32); // public roster digest
    assert_eq!(independent.take(32), [0x11; 32]);
    assert_eq!(independent.take(32), [0x22; 32]);
    assert_eq!(independent.u32(), 1);
    assert_eq!(independent.u32(), 0); // roster ordinal
    assert_eq!(independent.u32(), 17);
    assert_eq!(independent.len_bytes(), b"registration/node-17");
    assert_eq!(independent.len_bytes(), b"REGISTRATION_SECRET");
    assert_eq!(independent.len_bytes(), b"public.registration_idx");
    let participant_uuid = independent.take(16);
    assert_eq!(participant_uuid[6] >> 4, 4);
    assert_eq!(participant_uuid[8] >> 6, 2);
    assert_eq!(independent.take(32), [0x22; 32]);
    assert_eq!(independent.u8(), 1);
    independent.finish();

    let mut hasher = Sha256::new();
    hasher.update(b"ec_distann_build_registration_v1\0");
    hasher.update(&bytes);
    assert_eq!(
        hex::encode(hasher.finalize()),
        "c5a90122402eb68d6f443d63fe3e5744c07ff902a27e02d02125494c290f25ab"
    );
}

#[test]
fn distann_build_candidate_v1_fixture_decodes_independently_and_rejects_version_swap() {
    let bytes = decode_hex_fixture(include_str!(
        "../fixtures/on-disk/distann_build_candidate_v1.hex"
    ));
    let mut independent = DistannFixtureReader::new(&bytes);
    assert_eq!(independent.u16(), 1);
    assert_eq!(independent.take(32), [0xA0; 32]);
    assert!(!independent.len_bytes().is_empty()); // build specification
    independent.take(32);
    assert!(!independent.len_bytes().is_empty()); // generation descriptor
    independent.take(32);
    assert!(!independent.len_bytes().is_empty()); // source snapshot
    independent.take(32);
    let receipt_set = independent.len_bytes();
    let mut receipts = DistannFixtureReader::new(receipt_set);
    assert_eq!(receipts.u32(), 2);
    assert!(!receipts.len_bytes().is_empty());
    assert!(!receipts.len_bytes().is_empty());
    receipts.finish();
    let receipt_set_digest = independent.take(32);
    assert_distann_domain_digest(
        receipt_set,
        b"ec_distann_ready_receipt_set_v1\0",
        "778aca82955691a96a3f94d14b27e66c8f6eec017a9d86ab478ede930785ec6a",
    );
    assert_eq!(
        hex::encode(receipt_set_digest),
        "778aca82955691a96a3f94d14b27e66c8f6eec017a9d86ab478ede930785ec6a"
    );
    assert!(!independent.len_bytes().is_empty()); // epoch manifest
    let manifest_digest = independent.take(32);
    let fingerprint = independent.take(DISTANN_EPOCH_FINGERPRINT_BYTES);
    assert_eq!(&fingerprint[..2], &[2, 0]);
    assert_eq!(&fingerprint[2..], manifest_digest);
    independent.finish();

    let candidate = DistannBuildCandidateV1::decode(&bytes).unwrap();
    assert_eq!(candidate.encode().unwrap(), bytes);
    assert_distann_domain_digest(
        &bytes,
        b"ec_distann_build_candidate_v1\0",
        "5f1795a1534db0a694c6a9588b56dfed47824fee5ea35c4ee5145c17fd2c723a",
    );
    assert_eq!(
        hex::encode(candidate.digest().unwrap()),
        "5f1795a1534db0a694c6a9588b56dfed47824fee5ea35c4ee5145c17fd2c723a"
    );

    let mut swapped = bytes;
    swapped.swap(0, 1);
    assert!(DistannBuildCandidateV1::decode(&swapped).is_err());
}

#[test]
fn distann_successor_activation_v1_fixture_decodes_independently_and_rejects_version_swap() {
    let bytes = decode_hex_fixture(include_str!(
        "../fixtures/on-disk/distann_successor_activation_v1.hex"
    ));
    let mut independent = DistannFixtureReader::new(&bytes);
    assert_eq!(independent.u16(), 1);
    independent.take(16); // coordinator UUID
    assert_eq!(independent.u8(), 1);
    independent.take(16); // predecessor build
    assert_eq!(independent.u64(), 7);
    let predecessor_fingerprint = independent.len_bytes();
    let predecessor_digest = independent.take(32);
    assert_eq!(&predecessor_fingerprint[2..], predecessor_digest);
    independent.take(16); // successor build
    assert_eq!(independent.u64(), 8);
    let successor_fingerprint = independent.len_bytes();
    let successor_digest = independent.take(32);
    assert_eq!(&successor_fingerprint[2..], successor_digest);
    independent.finish();

    let activation = DistannSuccessorActivationV1::decode(&bytes).unwrap();
    assert_eq!(activation.predecessor.unwrap().epoch, 7);
    assert_eq!(activation.successor.epoch, 8);
    assert_distann_domain_digest(
        &bytes,
        b"ec_distann_successor_activation_v1\0",
        "7e899375e04da53713908a66393f358079fbf157798bb82b7c3f4eb969e3289f",
    );
    assert_eq!(
        hex::encode(activation.digest().unwrap()),
        "7e899375e04da53713908a66393f358079fbf157798bb82b7c3f4eb969e3289f"
    );

    let mut swapped = bytes;
    swapped.swap(0, 1);
    assert!(DistannSuccessorActivationV1::decode(&swapped).is_err());
}

#[test]
fn distann_abandon_binding_audit_v1_fixture_decodes_independently_and_rejects_version_swap() {
    let bytes = decode_hex_fixture(include_str!(
        "../fixtures/on-disk/distann_abandon_binding_audit_v1.hex"
    ));
    let mut independent = DistannFixtureReader::new(&bytes);
    assert_eq!(independent.u16(), 1);
    independent.take(16); // coordinator UUID
    independent.take(16); // successor build
    assert_eq!(independent.u64(), 8);
    independent.len_bytes(); // successor fingerprint
    independent.take(16); // predecessor build
    assert_eq!(independent.u64(), 7);
    let predecessor_fingerprint = independent.len_bytes();
    let predecessor_manifest = independent.take(32);
    assert_eq!(&predecessor_fingerprint[2..], predecessor_manifest);
    assert_eq!(independent.u32(), 1);
    assert_eq!(independent.u32(), 20);
    independent.take(16); // participant UUID
    assert_eq!(independent.len_bytes(), b"cluster-a/node-20");
    assert_eq!(independent.len_bytes(), b"public.distann_idx");
    independent.take(32); // activation digest
    assert_eq!(independent.i64(), 1_750_000_000_123_456);
    assert_eq!(independent.len_bytes(), b"ecaz_operator");
    assert_eq!(
        independent.len_bytes(),
        b"participant permanently unavailable"
    );
    independent.finish();

    let audit = DistannAbandonBindingAuditV1::decode(&bytes).unwrap();
    assert_eq!(audit.node_id, 20);
    assert_distann_domain_digest(
        &bytes,
        b"ec_distann_abandon_predecessor_binding_v1\0",
        "6de563bc944a6bed733aa317fdd96c2955c707b0141a41adcee40a358e0f0bee",
    );
    assert_eq!(
        hex::encode(audit.digest().unwrap()),
        "6de563bc944a6bed733aa317fdd96c2955c707b0141a41adcee40a358e0f0bee"
    );

    let mut swapped = bytes;
    swapped.swap(0, 1);
    assert!(DistannAbandonBindingAuditV1::decode(&swapped).is_err());
}

#[test]
fn distann_cancel_publish_audit_v1_fixture_decodes_independently_and_rejects_version_swap() {
    let bytes = decode_hex_fixture(include_str!(
        "../fixtures/on-disk/distann_cancel_publish_audit_v1.hex"
    ));
    let mut independent = DistannFixtureReader::new(&bytes);
    assert_eq!(independent.u16(), 1);
    independent.take(16); // coordinator UUID
    independent.take(16); // cancelled build id
    assert_eq!(independent.u64(), 8);
    let fingerprint = independent.len_bytes();
    let manifest_digest = independent.take(32);
    assert_eq!(&fingerprint[2..], manifest_digest);
    assert_eq!(independent.i64(), 1_750_000_000_654_321);
    assert_eq!(independent.len_bytes(), b"ecaz_operator");
    assert_eq!(
        independent.len_bytes(),
        b"successor participant permanently unavailable"
    );
    independent.finish();

    let audit = DistannCancelPublishAuditV1::decode(&bytes).unwrap();
    assert_eq!(audit.epoch, 8);
    assert_distann_domain_digest(
        &bytes,
        b"ec_distann_cancel_epoch_publish_v1\0",
        "42a9358f6ec4998673293572fffba5db37127c328d5e7fd2141ac34a9dc2bb53",
    );
    assert_eq!(
        hex::encode(audit.digest().unwrap()),
        "42a9358f6ec4998673293572fffba5db37127c328d5e7fd2141ac34a9dc2bb53"
    );

    let mut swapped = bytes;
    swapped.swap(0, 1);
    assert!(DistannCancelPublishAuditV1::decode(&swapped).is_err());
}

#[test]
fn distann_abandoned_binding_set_v1_fixture_decodes_independently_and_rejects_count_swap() {
    let bytes = decode_hex_fixture(include_str!(
        "../fixtures/on-disk/distann_abandoned_binding_set_v1.hex"
    ));
    let mut independent = DistannFixtureReader::new(&bytes);
    assert_eq!(independent.u32(), 2);
    assert_eq!(independent.u32(), 0);
    assert_eq!(independent.take(32), [0xA1; 32]);
    assert_eq!(independent.u32(), 1);
    independent.take(32);
    independent.finish();

    let set = DistannAbandonedBindingSetV1::decode(&bytes).unwrap();
    assert_eq!(set.entries.len(), 2);
    assert_distann_domain_digest(
        &bytes,
        b"ec_distann_abandoned_binding_set_v1\0",
        "5d261a123049966c026d2b91cec2635d69e8ab1a5015516f33a5ebf0360f26e0",
    );
    assert_eq!(
        hex::encode(set.digest().unwrap()),
        "5d261a123049966c026d2b91cec2635d69e8ab1a5015516f33a5ebf0360f26e0"
    );

    // This domain-versioned segment intentionally begins with count, not an
    // in-band version word. Swapping the count endian must still fail closed.
    let mut swapped = bytes;
    swapped.swap(0, 1);
    assert!(DistannAbandonedBindingSetV1::decode(&swapped).is_err());
}

#[test]
fn distann_retire_decision_v1_fixture_decodes_independently_and_rejects_version_swap() {
    let bytes = decode_hex_fixture(include_str!(
        "../fixtures/on-disk/distann_retire_decision_v1.hex"
    ));
    let mut independent = DistannFixtureReader::new(&bytes);
    assert_eq!(independent.u16(), 1);
    independent.take(16); // coordinator UUID
    independent.take(16); // target build
    assert_eq!(independent.u64(), 7);
    let fingerprint = independent.len_bytes();
    let manifest_digest = independent.take(32);
    assert_eq!(&fingerprint[2..], manifest_digest);
    assert!(!independent.len_bytes().is_empty()); // target roster snapshot
    independent.take(32); // roster digest
    assert_eq!(independent.u32(), 2);
    assert_eq!(independent.u32(), 0);
    independent.take(32);
    assert_eq!(independent.u32(), 1);
    independent.take(32);
    assert_eq!(independent.u8(), 1);
    assert_eq!(independent.u64(), 3);
    assert_eq!(independent.i64(), 1_750_000_001_654_321);
    assert_eq!(independent.len_bytes(), b"ecaz_operator");
    assert_eq!(
        independent.len_bytes(),
        b"forced after audited drain timeout"
    );
    independent.finish();

    let decision = DistannRetireDecisionV1::decode(&bytes).unwrap();
    assert!(decision.forced);
    assert_eq!(decision.abandoned_bindings.entries.len(), 2);
    assert_distann_domain_digest(
        &bytes,
        b"ec_distann_retire_decision_v1\0",
        "393d5ee8f174606e2639e6bb05cbe72966e90bc3306db5001e57fcdf2bd070f8",
    );
    assert_eq!(
        hex::encode(decision.digest().unwrap()),
        "393d5ee8f174606e2639e6bb05cbe72966e90bc3306db5001e57fcdf2bd070f8"
    );

    let mut swapped = bytes;
    swapped.swap(0, 1);
    assert!(DistannRetireDecisionV1::decode(&swapped).is_err());
}

#[test]
fn distann_build_spec_v1_fixture_decodes_independently_and_rejects_swap() {
    let bytes = decode_hex_fixture(include_str!(
        "../fixtures/on-disk/distann_build_spec_v1.hex"
    ));
    let mut independent = DistannFixtureReader::new(&bytes);
    assert_eq!(independent.u16(), 1);
    assert_eq!(independent.u64(), 7);
    independent.take(16);
    assert!(independent.len_bytes().is_empty());
    independent.take(32);
    independent.take(32);
    assert_eq!(independent.len_bytes().len(), 26);
    assert_eq!(independent.u64(), 10);
    independent.take(32 * 3);
    let owner_count = independent.u32();
    assert_eq!(owner_count, 2);
    for expected_node in [10, 20] {
        assert_eq!(independent.u32(), expected_node);
        independent.u64();
        independent.take(32);
    }
    independent.finish();

    let build_spec = DistannBuildSpec::decode(&bytes).unwrap();
    assert_eq!(build_spec.epoch, 7);
    assert_eq!(build_spec.expected_global_count, 10);
    assert_eq!(build_spec.build_options.build_shards, 0);

    let mut swapped = bytes;
    swapped.swap(0, 1);
    assert!(DistannBuildSpec::decode(&swapped).is_err());
}

#[test]
fn distann_handoff_entry_and_batch_v1_fixtures_decode_independently() {
    let shape = DistannHandoffShape {
        code_stride: 2,
        graph_degree: 4,
        non_dropped_attribute_count: 3,
    };
    let entry_bytes = decode_hex_fixture(include_str!(
        "../fixtures/on-disk/distann_handoff_entry_v1.hex"
    ));
    let mut independent = DistannFixtureReader::new(&entry_bytes);
    assert_eq!(independent.u16(), 1);
    assert_eq!(independent.u64(), 7);
    assert_eq!(independent.len_bytes(), &[7; 16]);
    assert_eq!(independent.u16(), 0);
    assert_eq!(independent.len_bytes(), &[0xA7, 0x0F]);
    let neighbor_count = independent.u32();
    assert_eq!(neighbor_count, 2);
    assert_eq!(independent.u64(), 17);
    assert_eq!(independent.u64(), 27);
    assert_eq!(independent.len_bytes(), &[1, 2, 3, 4]);
    assert_eq!(independent.len_bytes(), &[0b0000_0010]);
    let value_count = independent.u32();
    assert_eq!(value_count, 2);
    assert_eq!(independent.len_bytes(), &[0x11, 0x22]);
    assert_eq!(independent.len_bytes(), &[0x33]);
    independent.finish();
    let entry = DistannHandoffEntry::decode(&entry_bytes, shape).unwrap();
    assert_eq!(entry.vec_id, 7);
    assert_eq!(entry.row_values.len(), 2);

    let batch_bytes = decode_hex_fixture(include_str!(
        "../fixtures/on-disk/distann_handoff_batch_v1.hex"
    ));
    let mut independent = DistannFixtureReader::new(&batch_bytes);
    assert_eq!(independent.u16(), 1);
    assert_eq!(independent.u64(), 7);
    independent.take(16);
    assert_eq!(independent.u64(), 0);
    independent.take(64);
    assert_eq!(independent.u16(), 5);
    assert_eq!(independent.u8(), 2);
    assert_eq!(independent.u32(), 2);
    let entry_section_bytes = independent.u32() as usize;
    let entry_section = independent.take(entry_section_bytes);
    let mut entries = DistannFixtureReader::new(entry_section);
    assert_eq!(entries.len_bytes().len(), entry_bytes.len());
    assert_eq!(entries.len_bytes().len(), entry_bytes.len());
    entries.finish();
    independent.take(32);
    independent.finish();
    let batch = DistannHandoffBatch::decode(&batch_bytes, shape).unwrap();
    assert_eq!(batch.entries.len(), 2);
    assert_eq!(batch.entries[1].vec_id, 8);

    let mut entry_swapped = entry_bytes;
    entry_swapped.swap(0, 1);
    assert!(DistannHandoffEntry::decode(&entry_swapped, shape).is_err());
    let mut batch_swapped = batch_bytes;
    batch_swapped.swap(0, 1);
    assert!(DistannHandoffBatch::decode(&batch_swapped, shape).is_err());
}

#[test]
fn distann_owner_stream_hash_state_v1_fixture_is_independent_and_fixed() {
    let bytes = decode_hex_fixture(include_str!(
        "../fixtures/on-disk/distann_owner_stream_hash_state_v1.hex"
    ));
    assert_eq!(bytes.len(), DISTANN_OWNER_STREAM_HASH_STATE_BYTES);
    assert_eq!(DISTANN_OWNER_STREAM_HASH_STATE_BYTES, 107);
    assert_eq!(DISTANN_OWNER_STREAM_HASH_STATE_VERSION_OFFSET, 0);
    assert_eq!(DISTANN_OWNER_STREAM_HASH_STATE_IMPLEMENTATION_OFFSET, 2);
    assert_eq!(DISTANN_OWNER_STREAM_HASH_STATE_CHAIN_OFFSET, 3);
    assert_eq!(DISTANN_OWNER_STREAM_HASH_STATE_BLOCK_COUNT_OFFSET, 35);
    assert_eq!(DISTANN_OWNER_STREAM_HASH_STATE_BUFFER_LENGTH_OFFSET, 43);
    assert_eq!(DISTANN_OWNER_STREAM_HASH_STATE_BUFFER_OFFSET, 44);

    let mut independent = DistannFixtureReader::new(&bytes);
    assert_eq!(independent.u16(), 1, "state format version");
    assert_eq!(independent.u8(), 1, "sha2 0.11 implementation tag");
    assert_eq!(
        independent.take(32),
        hex::decode("67e6096a85ae67bb72f36e3c3af54fa57f520e518c68059babd9831f19cde05b").unwrap(),
        "no full block has been compressed, so the SHA-256 chain is its IV"
    );
    assert_eq!(independent.u64(), 0, "compressed block count");
    let buffered_bytes = independent.u8() as usize;
    let eager_buffer = independent.take(63);
    let domain = b"ec_distann_owner_stream_v1\0";
    assert_eq!(buffered_bytes, domain.len());
    assert_eq!(&eager_buffer[..buffered_bytes], domain);
    assert!(
        eager_buffer[buffered_bytes..].iter().all(|byte| *byte == 0),
        "unused eager-buffer bytes must be canonical zeroes"
    );
    independent.finish();

    let expected_digest: [u8; 32] =
        hex::decode("5f25ef3436224c6f7777c23f9a673cdcfab00a719d11db3c3bec157f63bd8ad6")
            .unwrap()
            .try_into()
            .unwrap();
    assert_eq!(
        distann_restore_owner_stream_hash_state(&bytes, expected_digest).unwrap(),
        expected_digest
    );

    let mut swapped = bytes;
    swapped.swap(
        DISTANN_OWNER_STREAM_HASH_STATE_VERSION_OFFSET,
        DISTANN_OWNER_STREAM_HASH_STATE_VERSION_OFFSET + 1,
    );
    assert!(distann_restore_owner_stream_hash_state(&swapped, expected_digest).is_err());
}

#[test]
fn distann_source_snapshot_v1_fixture_decodes_independently_and_rejects_swap() {
    let bytes = decode_hex_fixture(include_str!(
        "../fixtures/on-disk/distann_source_snapshot_v1.hex"
    ));
    let mut independent = DistannFixtureReader::new(&bytes);
    assert_eq!(independent.u16(), 1);
    assert_eq!(independent.u64(), 0x0102_0304_0506_0708);
    assert_eq!(independent.len_bytes(), b"ecaz");
    assert_eq!(independent.u64(), 100);
    assert_eq!(independent.u64(), 200);
    assert_eq!(independent.u32(), 3);
    let xip_count = independent.u32();
    assert_eq!(xip_count, 3);
    assert_eq!(
        (0..xip_count)
            .map(|_| independent.u64())
            .collect::<Vec<_>>(),
        vec![101, 103, 107]
    );
    let subxip_count = independent.u32();
    assert_eq!(subxip_count, 2);
    assert_eq!(independent.u64(), 109);
    assert_eq!(independent.u64(), 113);
    assert_eq!(independent.u8(), 0);
    assert_eq!(independent.u8(), 1);
    independent.finish();
    let snapshot = DistannSourceSnapshot::decode(&bytes).unwrap();
    assert_eq!(snapshot.database_name, "ecaz");
    assert!(snapshot.taken_during_recovery);

    let mut swapped = bytes;
    swapped.swap(0, 1);
    assert!(DistannSourceSnapshot::decode(&swapped).is_err());
}

#[test]
fn distann_ready_receipt_v1_fixture_decodes_independently_and_rejects_swap() {
    let bytes = decode_hex_fixture(include_str!(
        "../fixtures/on-disk/distann_ready_receipt_v1.hex"
    ));
    let mut independent = DistannFixtureReader::new(&bytes);
    assert_eq!(independent.u16(), 1);
    assert_eq!(independent.u32(), 10);
    assert_eq!(independent.u64(), 7);
    independent.take(16);
    independent.take(64);
    assert_eq!(independent.u64(), 10);
    assert_eq!(independent.u64(), 6);
    assert_eq!(independent.u64(), 6);
    independent.take(32 * 4);
    assert_eq!(independent.u64(), 600);
    assert_eq!(independent.u64(), 1200);
    assert_eq!(independent.u64(), 60);
    assert_eq!(independent.u8(), 1);
    independent.take(32);
    independent.finish();
    let receipt = DistannReadyReceipt::decode(&bytes).unwrap();
    assert_eq!(receipt.node_id, 10);
    assert_eq!(receipt.owned_record_count, receipt.row_count);
    assert_eq!(receipt.encode().unwrap(), bytes);

    let mut swapped = bytes;
    swapped.swap(0, 1);
    assert!(DistannReadyReceipt::decode(&swapped).is_err());
}

#[test]
fn distann_ready_receipt_v3_fixture_decodes_independently_and_rejects_swap() {
    let bytes = decode_hex_fixture(include_str!(
        "../fixtures/on-disk/distann_ready_receipt_v3.hex"
    ));
    assert_eq!(bytes.len(), 383);
    let mut independent = DistannFixtureReader::new(&bytes);
    assert_eq!(independent.u16(), 3);
    assert_eq!(independent.u32(), 10);
    assert_eq!(independent.u64(), 7);
    independent.take(16);
    independent.take(64);
    assert_eq!(independent.u64(), 10);
    assert_eq!(independent.u64(), 6);
    assert_eq!(independent.u64(), 6);
    independent.take(32 * 4);
    assert_eq!(independent.u64(), 600);
    assert_eq!(independent.u64(), 1200);
    assert_eq!(independent.u64(), 60);
    assert_eq!(independent.u8(), 1);
    assert_eq!(independent.take(32), &[0xB1; 32]);
    assert_eq!(independent.take(32), &[0xB2; 32]);
    assert_eq!(independent.u64(), 12_288);
    assert_eq!(independent.u64(), 24_576);
    independent.take(32);
    independent.finish();

    let receipt = DistannReadyReceipt::decode(&bytes).unwrap();
    assert_eq!(receipt.version(), 3);
    let hot_cold = receipt.hot_cold.as_ref().unwrap();
    assert_eq!(hot_cold.hot_initial_content_digest, [0xB1; 32]);
    assert_eq!(hot_cold.cold_initial_content_digest, [0xB2; 32]);
    assert_eq!(receipt.encode().unwrap(), bytes);

    let mut swapped = bytes;
    swapped.swap(0, 1);
    assert!(DistannReadyReceipt::decode(&swapped).is_err());
}

#[test]
fn distann_manifest_subrecord_fixtures_decode_and_reject_swap() {
    let codec_bytes = decode_hex_fixture(include_str!(
        "../fixtures/on-disk/distann_manifest_codec_parameters_v1.hex"
    ));
    let mut codec_reader = DistannFixtureReader::new(&codec_bytes);
    assert_eq!(codec_reader.u16(), 1);
    assert_eq!(codec_reader.u8(), 2);
    assert_eq!(codec_reader.u16(), 8);
    assert_eq!(codec_reader.u32(), 13);
    assert_eq!(codec_reader.u64(), 42);
    assert_eq!(codec_reader.u32(), 0);
    assert_eq!(codec_reader.u32(), 0);
    assert_eq!(codec_reader.u32(), 0);
    assert_eq!(codec_reader.u16(), 0);
    codec_reader.finish();
    let codec = DistannManifestCodecParameters::decode(&codec_bytes).unwrap();
    assert_eq!(codec.code_stride, 13);

    let build_bytes = decode_hex_fixture(include_str!(
        "../fixtures/on-disk/distann_manifest_build_options_v1.hex"
    ));
    let mut build_reader = DistannFixtureReader::new(&build_bytes);
    assert_eq!(build_reader.u16(), 1);
    assert_eq!(build_reader.u16(), 4);
    assert_eq!(build_reader.take(26).len(), 26);
    build_reader.finish();
    let build = DistannManifestBuildOptions::decode(&build_bytes).unwrap();
    assert_eq!(build.graph_degree, 4);
    assert_eq!(build.options.build_shards, 0);

    let mut codec_swapped = codec_bytes;
    codec_swapped.swap(0, 1);
    assert!(DistannManifestCodecParameters::decode(&codec_swapped).is_err());
    let mut build_swapped = build_bytes;
    build_swapped.swap(0, 1);
    assert!(DistannManifestBuildOptions::decode(&build_swapped).is_err());
}

#[test]
fn distann_epoch_manifest_v2_fixture_decodes_independently_and_rejects_swap() {
    let bytes = decode_hex_fixture(include_str!(
        "../fixtures/on-disk/distann_epoch_manifest_v2.hex"
    ));
    let mut independent = DistannFixtureReader::new(&bytes);
    assert_eq!(independent.u16(), 2);
    assert_eq!(independent.u64(), 7);
    independent.take(16);
    assert!(independent.len_bytes().is_empty());
    independent.take(32 * 3);
    assert_eq!(independent.u16(), 1);
    let roster_count = independent.u32();
    assert_eq!(roster_count, 2);
    for expected_node in [10, 20] {
        assert_eq!(independent.u32(), expected_node);
        independent.take(16);
        independent.len_bytes();
    }
    assert_eq!(independent.u16(), 5);
    assert_eq!(independent.u16(), 1);
    assert_eq!(independent.u16(), 1);
    independent.len_bytes();
    independent.len_bytes();
    independent.take(32 * 2);
    assert_eq!(independent.u64(), 10);
    independent.take(32 * 2);
    let receipt_count = independent.u32();
    assert_eq!(receipt_count, 2);
    for _ in 0..receipt_count {
        assert_eq!(independent.len_bytes().len(), 303);
    }
    independent.finish();

    let manifest = DistannEpochManifestV2::decode(&bytes).unwrap();
    assert_eq!(manifest.encode().unwrap(), bytes);
    assert_eq!(manifest.roster.len(), 2);
    assert_eq!(manifest.participant_receipts.len(), 2);
    let fingerprint = manifest.fingerprint().unwrap();
    assert_eq!(
        fingerprint.as_bytes().len(),
        DISTANN_EPOCH_FINGERPRINT_BYTES
    );
    assert_eq!(
        DistannEpochFingerprint::decode(fingerprint.as_bytes()).unwrap(),
        fingerprint
    );

    let mut swapped = bytes;
    swapped.swap(0, 1);
    assert!(DistannEpochManifestV2::decode(&swapped).is_err());
    let mut swapped_fingerprint = *fingerprint.as_bytes();
    swapped_fingerprint.swap(0, 1);
    assert!(DistannEpochFingerprint::decode(&swapped_fingerprint).is_err());
}

#[test]
fn distann_epoch_manifest_v4_fixture_decodes_independently_and_rejects_swap() {
    let bytes = decode_hex_fixture(include_str!(
        "../fixtures/on-disk/distann_epoch_manifest_v4.hex"
    ));
    let mut independent = DistannFixtureReader::new(&bytes);
    assert_eq!(independent.u16(), 4);
    assert_eq!(independent.u64(), 7);
    independent.take(16);
    assert!(independent.len_bytes().is_empty());
    independent.take(32 * 3);
    assert_eq!(independent.u16(), 1);
    let roster_count = independent.u32();
    assert_eq!(roster_count, 2);
    for expected_node in [10, 20] {
        assert_eq!(independent.u32(), expected_node);
        independent.take(16);
        independent.len_bytes();
    }
    assert_eq!(independent.u16(), 5);
    assert_eq!(independent.u16(), 2);
    assert_eq!(independent.u16(), 1);
    independent.len_bytes();
    independent.len_bytes();
    independent.take(32 * 2);
    assert_eq!(independent.u64(), 10);
    independent.take(32 * 2);
    assert_eq!(independent.take(32), &[0xD7; 32]);
    independent.take(32 * 2);
    let receipt_count = independent.u32();
    assert_eq!(receipt_count, 2);
    for _ in 0..receipt_count {
        assert_eq!(independent.len_bytes().len(), 383);
    }
    independent.finish();

    let manifest = DistannEpochManifestV2::decode(&bytes).unwrap();
    assert_eq!(manifest.version(), 4);
    assert_eq!(manifest.graph_record_version, 2);
    assert_eq!(manifest.row_tier_layout_descriptor_digest, Some([0xD7; 32]));
    assert!(manifest
        .participant_receipts
        .iter()
        .all(|receipt| receipt.version() == 3));
    assert_eq!(manifest.encode().unwrap(), bytes);
    let fingerprint = manifest.fingerprint().unwrap();
    assert_eq!(fingerprint.version(), 4);
    assert_eq!(
        DistannEpochFingerprint::decode(fingerprint.as_bytes()).unwrap(),
        fingerprint
    );

    let mut swapped = bytes;
    swapped.swap(0, 1);
    assert!(DistannEpochManifestV2::decode(&swapped).is_err());
    let mut swapped_fingerprint = *fingerprint.as_bytes();
    swapped_fingerprint.swap(0, 1);
    assert!(DistannEpochFingerprint::decode(&swapped_fingerprint).is_err());
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
fn hnsw_grouped_hot_tuple_v2_fixture_decodes() {
    let bytes = decode_hex_fixture(include_str!(
        "../fixtures/on-disk/hnsw_grouped_hot_tuple_v2.hex"
    ));

    let hot = TqGroupedHotTuple::decode(&bytes, 1, 3).expect("hnsw grouped hot tuple decodes");

    assert_eq!(hot.level, 2);
    assert!(!hot.deleted);
    assert_eq!(
        hot.heaptids,
        vec![
            ItemPointer {
                block_number: 70,
                offset_number: 1
            },
            ItemPointer {
                block_number: 71,
                offset_number: 2
            }
        ]
    );
    assert_eq!(
        hot.neighbortid,
        ItemPointer {
            block_number: 80,
            offset_number: 1
        }
    );
    assert_eq!(
        hot.reranktid,
        ItemPointer {
            block_number: 81,
            offset_number: 2
        }
    );
    assert_eq!(hot.binary_words, vec![0x0102_0304_0506_0708]);
    assert_eq!(hot.search_code, vec![0xaa, 0xbb, 0xcc]);
}

#[test]
fn hnsw_turbo_hot_tuple_v3_fixture_decodes() {
    let bytes = decode_hex_fixture(include_str!(
        "../fixtures/on-disk/hnsw_turbo_hot_tuple_v3.hex"
    ));

    let hot = TqTurboHotTuple::decode(&bytes, 2).expect("hnsw turbo hot tuple decodes");

    assert_eq!(hot.level, 3);
    assert!(hot.deleted);
    assert_eq!(
        hot.heaptids,
        vec![ItemPointer {
            block_number: 90,
            offset_number: 1
        }]
    );
    assert_eq!(
        hot.neighbortid,
        ItemPointer {
            block_number: 91,
            offset_number: 1
        }
    );
    assert_eq!(
        hot.reranktid,
        ItemPointer {
            block_number: 92,
            offset_number: 2
        }
    );
    assert_eq!(
        hot.binary_words,
        vec![0x1112_1314_1516_1718, 0x2122_2324_2526_2728]
    );
}

#[test]
fn hnsw_rerank_tuple_v3_fixture_decodes() {
    let bytes = decode_hex_fixture(include_str!("../fixtures/on-disk/hnsw_rerank_tuple_v3.hex"));

    let rerank = TqRerankTuple::decode(&bytes, 4).expect("hnsw rerank tuple decodes");

    assert_eq!(rerank.gamma.to_bits(), 0.75_f32.to_bits());
    assert_eq!(rerank.code, vec![0xde, 0xad, 0xbe, 0xef]);
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
fn diskann_vamana_overflow_tuple_v3_fixture_decodes() {
    let bytes = decode_hex_fixture(include_str!(
        "../fixtures/on-disk/diskann_vamana_overflow_tuple_v3.hex"
    ));

    let overflow =
        vamana_decode_overflow_tuple_fixture(&bytes).expect("diskann overflow tuple decodes");

    assert_eq!(
        overflow.owner_tid,
        ItemPointer {
            block_number: 100,
            offset_number: 1
        }
    );
    assert_eq!(
        overflow.nexttid,
        ItemPointer {
            block_number: 200,
            offset_number: 2
        }
    );
    assert_eq!(overflow.heap_tid_count, 2);
    assert_eq!(
        &overflow.heap_tids[..2],
        &[
            ItemPointer {
                block_number: 300,
                offset_number: 1
            },
            ItemPointer {
                block_number: 301,
                offset_number: 2
            }
        ]
    );
    assert!(overflow.heap_tids[2..]
        .iter()
        .all(|tid| *tid == ItemPointer::INVALID));
}

#[test]
fn diskann_vamana_overflow_tuple_v3_byteswapped_count_is_rejected() {
    let mut bytes = decode_hex_fixture(include_str!(
        "../fixtures/on-disk/diskann_vamana_overflow_tuple_v3.hex"
    ));
    bytes.swap(1, 2);

    let err = vamana_decode_overflow_tuple_fixture(&bytes)
        .expect_err("byte-swapped heap tid count should fail");

    assert!(
        err.contains("ec_diskann overflow tuple heap_tid_count 512 exceeds capacity 10"),
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
fn ivf_metadata_v9_fixture_decodes() {
    // Task 111h: current IVF format is v9 (92 bytes; compact rerank scorer
    // mode persisted at byte 22, RaBitQ clip at byte 23, rerank sidecar head
    // points at packed 0x2B rerank group headers when index placement is used).
    let bytes = decode_hex_fixture(include_str!("../fixtures/on-disk/ivf_metadata_v9.hex"));

    let metadata = IvfMetadataPage::decode(&bytes).expect("ivf metadata fixture should decode");

    assert_eq!(metadata.format_version, 9);
    assert_eq!(metadata.dimensions, 128);
    assert_eq!(metadata.nlists, 16);
    assert_eq!(metadata.nprobe, 4);
    assert_eq!(metadata.training_sample_rows, 1_000);
    assert_eq!(metadata.training_version, 3);
    assert_eq!(metadata.seed, 0x0102_0304_0506_0708);
    assert_eq!(metadata.storage_format, IvfStorageFormat::PqFastScan);
    assert_eq!(metadata.rerank, IvfRerankMode::HeapF32);
    assert_eq!(
        metadata.rabitq_rerank_score_mode,
        IvfRerankScoreMode::Estimator
    );
    assert_eq!(metadata.rabitq_rerank_clip, 2);
    assert_eq!(metadata.quant_bits, 4);
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
    // This fixture carries no rerank sidecar (head = INVALID), the legitimate
    // "no sidecar -> table/heap source" runtime state for table placement / f32.
    assert_eq!(metadata.rerank_sidecar_head, ItemPointer::INVALID);
    // ADR-079: no sidecar directory either (head = INVALID -> full-chain fallback).
    assert_eq!(metadata.rerank_sidecar_directory_head, ItemPointer::INVALID);
}

#[test]
fn ivf_metadata_byteswapped_version_is_rejected() {
    let mut bytes = decode_hex_fixture(include_str!("../fixtures/on-disk/ivf_metadata_v9.hex"));
    bytes.swap(
        EC_IVF_METADATA_FORMAT_VERSION_OFFSET,
        EC_IVF_METADATA_FORMAT_VERSION_OFFSET + 1,
    );

    let err = IvfMetadataPage::decode(&bytes).expect_err("byte-swapped version should fail");

    assert!(
        err.contains("unsupported ec_ivf metadata format version: 2304"),
        "unexpected error: {err}"
    );
}

#[test]
fn ivf_metadata_v8_is_rejected_by_version() {
    // Task 111h / NFR-016: v8 persisted byte 22 as a two-value RaBitQ
    // estimator/least-squares flag. v9 expands it to a compact rerank score
    // mode enum including exact-dequant diagnostics, so v8 is rejected rather
    // than sharing an ambiguous format tag.
    let bytes = decode_hex_fixture(include_str!("../fixtures/on-disk/ivf_metadata_v8.hex"));
    let err = IvfMetadataPage::decode(&bytes).expect_err("old v8 layout should be rejected");
    assert!(
        err.contains("unsupported ec_ivf metadata format version: 8"),
        "unexpected error: {err}"
    );
}

#[test]
fn ivf_metadata_v7_is_rejected_by_version() {
    // Task 111h / NFR-016: v7 had the current 92-byte packed 0x2B layout but
    // did not persist RaBitQ rerank score/clip; v9 rejects it so ALTERed
    // reloptions cannot silently reinterpret existing sidecar bytes.
    let bytes = decode_hex_fixture(include_str!("../fixtures/on-disk/ivf_metadata_v7.hex"));
    let err = IvfMetadataPage::decode(&bytes).expect_err("old v7 layout should be rejected");
    assert!(
        err.contains("unsupported ec_ivf metadata format version: 7"),
        "unexpected error: {err}"
    );
}

#[test]
fn ivf_metadata_v6_is_rejected_by_version() {
    // Task 111h TurboQuant centroid-relative follow-up / NFR-016: v6 used the
    // same packed 0x2B layout, but TurboQuant sidecar payloads encoded whole
    // source vectors. v9 rejects it so old sidecar bytes cannot be silently
    // scored as centroid-relative payloads.
    let bytes = decode_hex_fixture(include_str!("../fixtures/on-disk/ivf_metadata_v6.hex"));
    let err = IvfMetadataPage::decode(&bytes).expect_err("old v6 layout should be rejected");
    assert!(
        err.contains("unsupported ec_ivf metadata format version: 6"),
        "unexpected error: {err}"
    );
}

#[test]
fn ivf_metadata_v5_is_rejected_by_version() {
    // Task 111h residual rerank follow-up / NFR-016: v5 used the same packed
    // 0x2B layout but RaBitQ rerank payloads were non-residual. v9 rejects it
    // so old sidecar bytes cannot be silently scored as residual payloads.
    let bytes = decode_hex_fixture(include_str!("../fixtures/on-disk/ivf_metadata_v5.hex"));
    let err = IvfMetadataPage::decode(&bytes).expect_err("old v5 layout should be rejected");
    assert!(
        err.contains("unsupported ec_ivf metadata format version: 5"),
        "unexpected error: {err}"
    );
}

#[test]
fn ivf_metadata_v4_is_rejected_by_version() {
    // Task 111h / NFR-016: v4 used the legacy 0x2A heap-TID sidecar. The v9
    // writer emits packed 0x2B/0x2C rerank groups, so v4 is an explicit rebuild
    // boundary in this research branch.
    let bytes = decode_hex_fixture(include_str!("../fixtures/on-disk/ivf_metadata_v4.hex"));
    let err = IvfMetadataPage::decode(&bytes).expect_err("old v4 layout should be rejected");
    assert!(
        err.contains("unsupported ec_ivf metadata format version: 4"),
        "unexpected error: {err}"
    );
}

#[test]
fn ivf_metadata_v3_is_rejected_by_version() {
    // ADR-079 / NFR-016: the old 86-byte v3 layout is rejected by version (and
    // width), not silently mis-decoded. Research project => clean break + rebuild.
    let bytes = decode_hex_fixture(include_str!("../fixtures/on-disk/ivf_metadata_v3.hex"));
    let err = IvfMetadataPage::decode(&bytes).expect_err("old v3 layout should be rejected");
    assert!(
        err.contains("format version") || err.contains("length mismatch"),
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

#[test]
fn spire_local_store_config_v1_fixture_decodes() {
    let bytes = decode_hex_fixture(include_str!(
        "../fixtures/on-disk/spire_local_store_config_v1.hex"
    ));

    let config =
        SpireLocalStoreConfig::decode(&bytes).expect("spire local store config should decode");

    assert_eq!(config.generation, 7);
    assert_eq!(config.stores.len(), 2);
    assert_eq!(config.stores[0].local_store_id, 2);
    assert_eq!(config.stores[0].store_relid, 502);
    assert_eq!(config.stores[0].tablespace_oid, 1002);
    assert_eq!(config.stores[0].state, SpireLocalStoreState::Available);
    assert_eq!(config.stores[1].local_store_id, 5);
    assert_eq!(config.stores[1].store_relid, 505);
    assert_eq!(config.stores[1].tablespace_oid, 1005);
    assert_eq!(config.stores[1].state, SpireLocalStoreState::Available);
}

#[test]
fn spire_local_store_config_v1_byteswapped_version_is_rejected() {
    let mut bytes = decode_hex_fixture(include_str!(
        "../fixtures/on-disk/spire_local_store_config_v1.hex"
    ));
    bytes.swap(
        SPIRE_LOCAL_STORE_CONFIG_FORMAT_VERSION_OFFSET,
        SPIRE_LOCAL_STORE_CONFIG_FORMAT_VERSION_OFFSET + 1,
    );

    let err = SpireLocalStoreConfig::decode(&bytes).expect_err("byte-swapped version should fail");

    assert!(
        err.contains("ec_spire unsupported metadata format version: 256"),
        "unexpected error: {err}"
    );
}

#[test]
fn spire_placement_entry_v1_fixture_decodes() {
    let bytes = decode_hex_fixture(include_str!(
        "../fixtures/on-disk/spire_placement_entry_v1.hex"
    ));

    let entry = SpirePlacementEntry::decode(&bytes).expect("spire placement entry should decode");

    assert_eq!(entry.state, SpirePlacementState::Available);
    assert_eq!(entry.epoch, 7);
    assert_eq!(entry.pid, 17);
    assert_eq!(entry.node_id, 0);
    assert_eq!(entry.local_store_id, 2);
    assert_eq!(entry.store_relid, 502);
    assert_eq!(entry.object_version, 3);
    assert_eq!(
        entry.object_tid,
        ItemPointer {
            block_number: 20,
            offset_number: 1
        }
    );
    assert_eq!(entry.object_bytes, 108);
}

#[test]
fn spire_placement_entry_v1_byteswapped_version_is_rejected() {
    let mut bytes = decode_hex_fixture(include_str!(
        "../fixtures/on-disk/spire_placement_entry_v1.hex"
    ));
    bytes.swap(
        SPIRE_PLACEMENT_ENTRY_FORMAT_VERSION_OFFSET,
        SPIRE_PLACEMENT_ENTRY_FORMAT_VERSION_OFFSET + 1,
    );

    let err = SpirePlacementEntry::decode(&bytes).expect_err("byte-swapped version should fail");

    assert!(
        err.contains("ec_spire unsupported metadata format version: 256"),
        "unexpected error: {err}"
    );
}

#[test]
fn spire_placement_directory_v1_fixture_decodes() {
    let bytes = decode_hex_fixture(include_str!(
        "../fixtures/on-disk/spire_placement_directory_v1.hex"
    ));

    let directory =
        SpirePlacementDirectory::decode(&bytes).expect("spire placement directory should decode");

    assert_eq!(directory.epoch, 7);
    assert_eq!(directory.entries.len(), 1);
    assert_eq!(directory.entries[0].pid, 17);
    assert_eq!(directory.entries[0].local_store_id, 2);
}

#[test]
fn spire_placement_directory_v1_byteswapped_version_is_rejected() {
    let mut bytes = decode_hex_fixture(include_str!(
        "../fixtures/on-disk/spire_placement_directory_v1.hex"
    ));
    bytes.swap(
        SPIRE_PLACEMENT_DIRECTORY_FORMAT_VERSION_OFFSET,
        SPIRE_PLACEMENT_DIRECTORY_FORMAT_VERSION_OFFSET + 1,
    );

    let err =
        SpirePlacementDirectory::decode(&bytes).expect_err("byte-swapped version should fail");

    assert!(
        err.contains("ec_spire unsupported metadata format version: 256"),
        "unexpected error: {err}"
    );
}

#[test]
fn spire_epoch_manifest_v1_fixture_decodes() {
    let bytes = decode_hex_fixture(include_str!(
        "../fixtures/on-disk/spire_epoch_manifest_v1.hex"
    ));

    let manifest = SpireEpochManifest::decode(&bytes).expect("spire epoch manifest should decode");

    assert_eq!(manifest.state, SpireEpochState::Published);
    assert_eq!(manifest.consistency_mode, SpireConsistencyMode::Strict);
    assert_eq!(manifest.epoch, 7);
    assert_eq!(manifest.published_at_micros, 1_000);
    assert_eq!(manifest.retain_until_micros, 2_000);
    assert_eq!(manifest.active_query_count, 3);
}

#[test]
fn spire_epoch_manifest_v1_byteswapped_version_is_rejected() {
    let mut bytes = decode_hex_fixture(include_str!(
        "../fixtures/on-disk/spire_epoch_manifest_v1.hex"
    ));
    bytes.swap(
        SPIRE_EPOCH_MANIFEST_FORMAT_VERSION_OFFSET,
        SPIRE_EPOCH_MANIFEST_FORMAT_VERSION_OFFSET + 1,
    );

    let err = SpireEpochManifest::decode(&bytes).expect_err("byte-swapped version should fail");

    assert!(
        err.contains("ec_spire unsupported metadata format version: 256"),
        "unexpected error: {err}"
    );
}

#[test]
fn spire_manifest_entry_v1_fixture_decodes() {
    let bytes = decode_hex_fixture(include_str!(
        "../fixtures/on-disk/spire_manifest_entry_v1.hex"
    ));

    let entry = SpireManifestEntry::decode(&bytes).expect("spire manifest entry should decode");

    assert_eq!(entry.epoch, 7);
    assert_eq!(entry.pid, 17);
    assert_eq!(entry.object_version, 3);
    assert_eq!(
        entry.placement_tid,
        ItemPointer {
            block_number: 30,
            offset_number: 2
        }
    );
}

#[test]
fn spire_manifest_entry_v1_byteswapped_version_is_rejected() {
    let mut bytes = decode_hex_fixture(include_str!(
        "../fixtures/on-disk/spire_manifest_entry_v1.hex"
    ));
    bytes.swap(
        SPIRE_MANIFEST_ENTRY_FORMAT_VERSION_OFFSET,
        SPIRE_MANIFEST_ENTRY_FORMAT_VERSION_OFFSET + 1,
    );

    let err = SpireManifestEntry::decode(&bytes).expect_err("byte-swapped version should fail");

    assert!(
        err.contains("ec_spire unsupported metadata format version: 256"),
        "unexpected error: {err}"
    );
}

#[test]
fn spire_object_manifest_v1_fixture_decodes() {
    let bytes = decode_hex_fixture(include_str!(
        "../fixtures/on-disk/spire_object_manifest_v1.hex"
    ));

    let manifest =
        SpireObjectManifest::decode(&bytes).expect("spire object manifest should decode");

    assert_eq!(manifest.epoch, 7);
    assert_eq!(manifest.entries.len(), 1);
    assert_eq!(manifest.entries[0].pid, 17);
    assert_eq!(manifest.entries[0].object_version, 3);
}

#[test]
fn spire_object_manifest_v1_byteswapped_version_is_rejected() {
    let mut bytes = decode_hex_fixture(include_str!(
        "../fixtures/on-disk/spire_object_manifest_v1.hex"
    ));
    bytes.swap(
        SPIRE_OBJECT_MANIFEST_FORMAT_VERSION_OFFSET,
        SPIRE_OBJECT_MANIFEST_FORMAT_VERSION_OFFSET + 1,
    );

    let err = SpireObjectManifest::decode(&bytes).expect_err("byte-swapped version should fail");

    assert!(
        err.contains("ec_spire unsupported metadata format version: 256"),
        "unexpected error: {err}"
    );
}

#[test]
fn spire_leaf_partition_object_v1_fixture_decodes() {
    let bytes = decode_hex_fixture(include_str!(
        "../fixtures/on-disk/spire_leaf_partition_object_v1.hex"
    ));

    let object =
        spire_decode_leaf_partition_object_fixture(&bytes).expect("spire leaf object decodes");

    assert_eq!(object.header.kind, 3);
    assert_eq!(object.header.pid, 17);
    assert_eq!(object.header.object_version, 3);
    assert_eq!(object.header.published_epoch_backref, 7);
    assert_eq!(object.header.parent_pid, 5);
    assert_eq!(object.header.assignment_count, 1);
    assert_eq!(object.assignments.len(), 1);
    assert_eq!(object.assignments[0].flags, 1);
    assert_eq!(
        object.assignments[0].vec_id,
        vec![1, 5, 0, 0, 0, 0, 0, 0, 0]
    );
    assert_eq!(
        object.assignments[0].heap_tid,
        ItemPointer {
            block_number: 100,
            offset_number: 1
        }
    );
    assert_eq!(object.assignments[0].payload_format, 1);
    assert_eq!(object.assignments[0].gamma.to_bits(), 0.5_f32.to_bits());
    assert_eq!(object.assignments[0].encoded_payload, vec![0xaa, 0xbb]);
}

#[test]
fn spire_leaf_partition_object_v1_byteswapped_version_is_rejected() {
    let mut bytes = decode_hex_fixture(include_str!(
        "../fixtures/on-disk/spire_leaf_partition_object_v1.hex"
    ));
    bytes.swap(
        SPIRE_PARTITION_OBJECT_FORMAT_VERSION_OFFSET,
        SPIRE_PARTITION_OBJECT_FORMAT_VERSION_OFFSET + 1,
    );

    let err = spire_decode_leaf_partition_object_fixture(&bytes)
        .expect_err("byte-swapped version should fail");

    assert!(
        err.contains("ec_spire unsupported partition object format version: 256"),
        "unexpected error: {err}"
    );
}

#[test]
fn spire_routing_root_partition_object_v1_fixture_decodes() {
    let bytes = decode_hex_fixture(include_str!(
        "../fixtures/on-disk/spire_routing_root_partition_object_v1.hex"
    ));

    let object = spire_decode_routing_partition_object_fixture(&bytes)
        .expect("spire routing object decodes");

    assert_eq!(object.header.kind, 1);
    assert_eq!(object.header.pid, 11);
    assert_eq!(object.header.object_version, 2);
    assert_eq!(object.header.level, 1);
    assert_eq!(object.header.child_count, 2);
    assert_eq!(object.dimensions, 2);
    assert_eq!(object.centroid_ordinals, vec![0, 1]);
    assert_eq!(object.child_pids, vec![17, 18]);
    assert_eq!(
        object
            .centroids
            .iter()
            .map(|value| value.to_bits())
            .collect::<Vec<_>>(),
        vec![
            1.0_f32.to_bits(),
            0.0_f32.to_bits(),
            (-1.0_f32).to_bits(),
            0.0_f32.to_bits()
        ]
    );
}

#[test]
fn spire_delta_partition_object_v1_fixture_decodes() {
    let bytes = decode_hex_fixture(include_str!(
        "../fixtures/on-disk/spire_delta_partition_object_v1.hex"
    ));

    let object =
        spire_decode_delta_partition_object_fixture(&bytes).expect("spire delta object decodes");

    assert_eq!(object.header.kind, 4);
    assert_eq!(object.header.pid, 19);
    assert_eq!(object.header.object_version, 2);
    assert_eq!(object.header.parent_pid, 17);
    assert_eq!(object.header.assignment_count, 1);
    assert_eq!(object.assignments.len(), 1);
    assert_eq!(object.assignments[0].flags, 1 | 8);
    assert_eq!(
        object.assignments[0].vec_id,
        vec![1, 6, 0, 0, 0, 0, 0, 0, 0]
    );
    assert_eq!(
        object.assignments[0].heap_tid,
        ItemPointer {
            block_number: 101,
            offset_number: 1
        }
    );
    assert_eq!(object.assignments[0].gamma.to_bits(), 1.0_f32.to_bits());
    assert_eq!(object.assignments[0].encoded_payload, vec![0xcc, 0xdd]);
}

#[test]
fn spire_top_graph_partition_object_v1_fixture_decodes() {
    let bytes = decode_hex_fixture(include_str!(
        "../fixtures/on-disk/spire_top_graph_partition_object_v1.hex"
    ));

    let object = spire_decode_top_graph_partition_object_fixture(&bytes)
        .expect("spire top graph object decodes");

    assert_eq!(object.header.kind, 5);
    assert_eq!(object.header.pid, 21);
    assert_eq!(object.header.object_version, 2);
    assert_eq!(object.header.level, 2);
    assert_eq!(object.header.parent_pid, 11);
    assert_eq!(object.root_pid, 11);
    assert_eq!(object.dimensions, 2);
    assert_eq!(object.graph_degree, 2);
    assert_eq!(object.build_list_size, 16);
    assert_eq!(object.alpha.to_bits(), 1.0_f32.to_bits());
    assert_eq!(object.entry_node, 1);
    assert_eq!(object.nodes.len(), 2);
    assert_eq!(object.nodes[0].child_pid, 17);
    assert_eq!(object.nodes[0].centroid_ordinal, 0);
    assert_eq!(object.nodes[0].neighbors, vec![1]);
    assert_eq!(object.nodes[1].child_pid, 18);
    assert_eq!(object.nodes[1].centroid_ordinal, 1);
    assert_eq!(object.nodes[1].neighbors, vec![0]);
}

#[test]
fn spire_leaf_v2_meta_fixture_decodes() {
    let bytes = decode_hex_fixture(include_str!(
        "../fixtures/on-disk/spire_leaf_v2_meta_v2.hex"
    ));

    let meta = spire_decode_leaf_v2_meta_fixture(&bytes).expect("spire leaf V2 meta decodes");

    assert_eq!(meta.header.kind, 3);
    assert_eq!(meta.header.pid, 23);
    assert_eq!(meta.header.object_version, 4);
    assert_eq!(meta.header.published_epoch_backref, 8);
    assert_eq!(meta.header.parent_pid, 5);
    assert_eq!(meta.header.assignment_count, 2);
    assert_eq!(meta.header.flags, 1);
    assert_eq!(meta.payload_format, 1);
    assert_eq!(meta.payload_stride, 2);
    assert_eq!(meta.vec_id_kind, 1);
    assert_eq!(meta.vec_id_stride, 16);
    assert_eq!(meta.segment_count, 1);
    assert_eq!(
        meta.first_segment_locator,
        ItemPointer {
            block_number: 300,
            offset_number: 4
        }
    );
    assert_eq!(meta.object_bytes_total, 132);
}

#[test]
fn spire_leaf_v2_meta_byteswapped_version_is_rejected() {
    let mut bytes = decode_hex_fixture(include_str!(
        "../fixtures/on-disk/spire_leaf_v2_meta_v2.hex"
    ));
    bytes.swap(
        SPIRE_PARTITION_OBJECT_FORMAT_VERSION_OFFSET,
        SPIRE_PARTITION_OBJECT_FORMAT_VERSION_OFFSET + 1,
    );

    let err =
        spire_decode_leaf_v2_meta_fixture(&bytes).expect_err("byte-swapped version should fail");

    assert!(
        err.contains("ec_spire unsupported partition object format version: 512"),
        "unexpected error: {err}"
    );
}

#[test]
fn spire_leaf_v2_segment_fixture_decodes() {
    let meta_bytes = decode_hex_fixture(include_str!(
        "../fixtures/on-disk/spire_leaf_v2_meta_v2.hex"
    ));
    let segment_bytes = decode_hex_fixture(include_str!(
        "../fixtures/on-disk/spire_leaf_v2_segment_v2.hex"
    ));

    let segment = spire_decode_leaf_v2_segment_fixture(&meta_bytes, &segment_bytes)
        .expect("spire leaf V2 segment decodes");

    assert_eq!(segment.header.kind, 3);
    assert_eq!(segment.header.pid, 23);
    assert_eq!(segment.header.object_version, 4);
    assert_eq!(segment.header.parent_pid, 5);
    assert_eq!(segment.header.assignment_count, 2);
    assert_eq!(segment.header.flags, 2);
    assert_eq!(segment.segment_no, 0);
    assert_eq!(segment.row_base, 0);
    assert_eq!(segment.next_segment_locator, ItemPointer::INVALID);
    assert_eq!(segment.flags, vec![1, 2]);
    assert_eq!(
        &segment.vec_ids[..16],
        &[1, 9, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
    );
    assert_eq!(
        &segment.vec_ids[16..],
        &[1, 10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
    );
    assert_eq!(
        segment.heap_tids,
        vec![
            ItemPointer {
                block_number: 101,
                offset_number: 1
            },
            ItemPointer {
                block_number: 102,
                offset_number: 2
            }
        ]
    );
    assert_eq!(
        segment
            .gammas
            .iter()
            .map(|value| value.to_bits())
            .collect::<Vec<_>>(),
        vec![0.5_f32.to_bits(), 1.0_f32.to_bits()]
    );
    assert_eq!(segment.payloads, vec![0xaa, 0xbb, 0xcc, 0xdd]);
}

#[test]
fn spire_partition_object_v2_chain_meta_fixture_decodes() {
    let bytes = decode_hex_fixture(include_str!(
        "../fixtures/on-disk/spire_partition_object_v2_chain_meta.hex"
    ));

    let meta =
        spire_decode_partition_object_v2_chain_meta_fixture(&bytes).expect("chain meta decodes");

    assert_eq!(meta.header.kind, 1);
    assert_eq!(meta.header.pid, 11);
    assert_eq!(meta.header.object_version, 2);
    assert_eq!(meta.header.level, 1);
    assert_eq!(meta.header.child_count, 2);
    assert_eq!(meta.header.flags, 4);
    assert_eq!(meta.dimensions, 2);
    assert_eq!(meta.segment_count, 1);
    assert_eq!(
        meta.first_segment_locator,
        ItemPointer {
            block_number: 400,
            offset_number: 5
        }
    );
    assert_eq!(meta.object_bytes_total, 32);
}

#[test]
fn spire_partition_object_v2_chain_meta_byteswapped_version_is_rejected() {
    let mut bytes = decode_hex_fixture(include_str!(
        "../fixtures/on-disk/spire_partition_object_v2_chain_meta.hex"
    ));
    bytes.swap(
        SPIRE_PARTITION_OBJECT_FORMAT_VERSION_OFFSET,
        SPIRE_PARTITION_OBJECT_FORMAT_VERSION_OFFSET + 1,
    );

    let err = spire_decode_partition_object_v2_chain_meta_fixture(&bytes)
        .expect_err("byte-swapped version should fail");

    assert!(
        err.contains("ec_spire unsupported partition object format version: 512"),
        "unexpected error: {err}"
    );
}

#[test]
fn spire_partition_object_v2_chain_segment_fixture_decodes() {
    let meta_bytes = decode_hex_fixture(include_str!(
        "../fixtures/on-disk/spire_partition_object_v2_chain_meta.hex"
    ));
    let segment_bytes = decode_hex_fixture(include_str!(
        "../fixtures/on-disk/spire_partition_object_v2_chain_segment.hex"
    ));

    let segment =
        spire_decode_partition_object_v2_chain_segment_fixture(&meta_bytes, &segment_bytes)
            .expect("chain segment decodes");

    assert_eq!(segment.header.kind, 1);
    assert_eq!(segment.header.pid, 11);
    assert_eq!(segment.header.object_version, 2);
    assert_eq!(segment.header.level, 1);
    assert_eq!(segment.header.child_count, 0);
    assert_eq!(segment.header.flags, 8);
    assert_eq!(segment.segment_no, 0);
    assert_eq!(segment.byte_base, 0);
    assert_eq!(segment.next_segment_locator, ItemPointer::INVALID);
    assert_eq!(segment.payload, vec![1, 2, 3, 4, 0xaa, 0xbb, 0xcc, 0xdd]);
}
