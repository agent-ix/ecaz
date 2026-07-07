//! On-disk metadata page for `ec_distann` (FR-076 index metadata surface).
//!
//! Block 0 of the index relation holds one fixed-size metadata record in the
//! page special area, following the `VamanaMetadataPage` convention from
//! `ec_diskann`. The record is format-versioned; version bumps follow the
//! NFR-016 research posture (rebuild, no migration).

use crate::storage::page::ItemPointer;

/// ec_distann on-disk format version. Bump on any layout change (NFR-016:
/// rebuild, no migration).
pub const INDEX_FORMAT_V1_DISTANN: u16 = 1;

/// Neighbor-code codec kinds persisted in the metadata page. Mirrors the
/// `neighbor_code_format` reloption (ADR-085 D7; default pinned to RaBitQ
/// by the M0 measurement, task-162 packet 002).
pub const DISTANN_NEIGHBOR_CODEC_GROUPED_PQ: u8 = 1;
pub const DISTANN_NEIGHBOR_CODEC_RABITQ: u8 = 2;
pub const DISTANN_NEIGHBOR_CODEC_TURBOQUANT: u8 = 3;

pub const DISTANN_METADATA_BYTES: usize = 72;

pub const DISTANN_METADATA_FORMAT_VERSION_OFFSET: usize = 0;
pub const DISTANN_METADATA_ENTRY_POINT_OFFSET: usize = 2;
pub const DISTANN_METADATA_GRAPH_DEGREE_R_OFFSET: usize = 8;
pub const DISTANN_METADATA_BUILD_LIST_SIZE_L_OFFSET: usize = 10;
pub const DISTANN_METADATA_ALPHA_OFFSET: usize = 12;
pub const DISTANN_METADATA_DIMENSIONS_OFFSET: usize = 16;
pub const DISTANN_METADATA_SEED_OFFSET: usize = 18;
pub const DISTANN_METADATA_NEIGHBOR_CODEC_KIND_OFFSET: usize = 26;
pub const DISTANN_METADATA_FLAGS_OFFSET: usize = 27;
pub const DISTANN_METADATA_HEAD_INDEX_CAP_OFFSET: usize = 28;
pub const DISTANN_METADATA_CLOSURE_EPSILON_OFFSET: usize = 32;
pub const DISTANN_METADATA_NODE_COUNT_OFFSET: usize = 36;
pub const DISTANN_METADATA_HEAD_SAMPLE_HEAD_OFFSET: usize = 44;
pub const DISTANN_METADATA_DELTA_BUFFER_HEAD_OFFSET: usize = 50;
pub const DISTANN_METADATA_CODEC_SUBVECTOR_COUNT_OFFSET: usize = 56;
pub const DISTANN_METADATA_CODEC_SUBVECTOR_DIM_OFFSET: usize = 58;
pub const DISTANN_METADATA_GROUPED_CODEBOOK_HEAD_OFFSET: usize = 60;
pub const DISTANN_METADATA_DIRECTORY_HEAD_OFFSET: usize = 66;

/// Fixed-size metadata record stored on block 0.
#[derive(Debug, Clone, PartialEq)]
pub struct DistannMetadataPage {
    pub format_version: u16,
    /// Graph entry record; INVALID until the monolithic build lands a graph.
    pub entry_point: ItemPointer,
    pub graph_degree_r: u16,
    pub build_list_size_l: u16,
    pub alpha: f32,
    pub dimensions: u16,
    pub seed: u64,
    /// One of `DISTANN_NEIGHBOR_CODEC_*` (D7).
    pub neighbor_codec_kind: u8,
    /// Reserved flag bits (none defined at format v1).
    pub flags: u8,
    /// FR-080 head-index cap C (ADR-085 D3).
    pub head_index_cap: u32,
    /// FR-077 build-shard closure-overlap band; unused by the monolithic
    /// M0 build, persisted so rebuild provenance is complete.
    pub closure_epsilon: f32,
    /// Live graph-node records written at build time.
    pub node_count: u64,
    /// Head of the persisted FR-080 entry-region sample chain; INVALID
    /// until the head-index slice lands.
    pub head_sample_head: ItemPointer,
    /// Head of the FR-083 interim delta-buffer chain; INVALID until the
    /// DML slice lands.
    pub delta_buffer_head: ItemPointer,
    /// Codec shape parameters, mirroring the ec_diskann convention:
    /// GroupedPq -> (group_count, group_size); RaBitQ -> (0, bits);
    /// TurboQuant -> (0, bits).
    pub codec_subvector_count: u16,
    pub codec_subvector_dim: u16,
    /// Head of the persisted GroupedPq codebook chain (INVALID for the
    /// seeded codecs).
    pub grouped_codebook_head: ItemPointer,
    /// Head of the sorted vec_id -> record-TID directory chain (FR-078's
    /// per-node resolution surface; single-node in M0).
    pub directory_head: ItemPointer,
}

impl DistannMetadataPage {
    #[allow(clippy::too_many_arguments)]
    pub fn empty(
        graph_degree_r: u16,
        build_list_size_l: u16,
        alpha: f32,
        dimensions: u16,
        seed: u64,
        neighbor_codec_kind: u8,
        head_index_cap: u32,
        closure_epsilon: f32,
    ) -> Self {
        Self {
            format_version: INDEX_FORMAT_V1_DISTANN,
            entry_point: ItemPointer::INVALID,
            graph_degree_r,
            build_list_size_l,
            alpha,
            dimensions,
            seed,
            neighbor_codec_kind,
            flags: 0,
            head_index_cap,
            closure_epsilon,
            node_count: 0,
            head_sample_head: ItemPointer::INVALID,
            delta_buffer_head: ItemPointer::INVALID,
            codec_subvector_count: 0,
            codec_subvector_dim: 0,
            grouped_codebook_head: ItemPointer::INVALID,
            directory_head: ItemPointer::INVALID,
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(DISTANN_METADATA_BYTES);
        out.extend_from_slice(&self.format_version.to_le_bytes());
        self.entry_point.encode_into(&mut out);
        out.extend_from_slice(&self.graph_degree_r.to_le_bytes());
        out.extend_from_slice(&self.build_list_size_l.to_le_bytes());
        out.extend_from_slice(&self.alpha.to_le_bytes());
        out.extend_from_slice(&self.dimensions.to_le_bytes());
        out.extend_from_slice(&self.seed.to_le_bytes());
        out.push(self.neighbor_codec_kind);
        out.push(self.flags);
        out.extend_from_slice(&self.head_index_cap.to_le_bytes());
        out.extend_from_slice(&self.closure_epsilon.to_le_bytes());
        out.extend_from_slice(&self.node_count.to_le_bytes());
        self.head_sample_head.encode_into(&mut out);
        self.delta_buffer_head.encode_into(&mut out);
        out.extend_from_slice(&self.codec_subvector_count.to_le_bytes());
        out.extend_from_slice(&self.codec_subvector_dim.to_le_bytes());
        self.grouped_codebook_head.encode_into(&mut out);
        self.directory_head.encode_into(&mut out);
        debug_assert_eq!(out.len(), DISTANN_METADATA_BYTES);
        out
    }

    pub fn decode(input: &[u8]) -> Result<Self, String> {
        if input.len() != DISTANN_METADATA_BYTES {
            return Err(format!(
                "distann metadata length mismatch: got {}, expected {DISTANN_METADATA_BYTES}",
                input.len()
            ));
        }

        let format_version =
            u16::from_le_bytes(input[0..2].try_into().expect("format version bytes"));
        if format_version != INDEX_FORMAT_V1_DISTANN {
            return Err(format!(
                "invalid distann metadata format version: got {format_version}, expected {INDEX_FORMAT_V1_DISTANN}"
            ));
        }

        let neighbor_codec_kind = input[DISTANN_METADATA_NEIGHBOR_CODEC_KIND_OFFSET];
        if !matches!(
            neighbor_codec_kind,
            DISTANN_NEIGHBOR_CODEC_GROUPED_PQ
                | DISTANN_NEIGHBOR_CODEC_RABITQ
                | DISTANN_NEIGHBOR_CODEC_TURBOQUANT
        ) {
            return Err(format!(
                "invalid distann metadata neighbor codec kind: got {neighbor_codec_kind}"
            ));
        }

        Ok(Self {
            format_version,
            entry_point: ItemPointer::decode(&input[2..8])?,
            graph_degree_r: u16::from_le_bytes(
                input[8..10].try_into().expect("graph_degree_r bytes"),
            ),
            build_list_size_l: u16::from_le_bytes(
                input[10..12].try_into().expect("build_list_size_l bytes"),
            ),
            alpha: f32::from_le_bytes(input[12..16].try_into().expect("alpha bytes")),
            dimensions: u16::from_le_bytes(input[16..18].try_into().expect("dimensions bytes")),
            seed: u64::from_le_bytes(input[18..26].try_into().expect("seed bytes")),
            neighbor_codec_kind,
            flags: input[DISTANN_METADATA_FLAGS_OFFSET],
            head_index_cap: u32::from_le_bytes(
                input[28..32].try_into().expect("head_index_cap bytes"),
            ),
            closure_epsilon: f32::from_le_bytes(
                input[32..36].try_into().expect("closure_epsilon bytes"),
            ),
            node_count: u64::from_le_bytes(input[36..44].try_into().expect("node_count bytes")),
            head_sample_head: ItemPointer::decode(&input[44..50])?,
            delta_buffer_head: ItemPointer::decode(&input[50..56])?,
            codec_subvector_count: u16::from_le_bytes(
                input[56..58].try_into().expect("codec_subvector_count bytes"),
            ),
            codec_subvector_dim: u16::from_le_bytes(
                input[58..60].try_into().expect("codec_subvector_dim bytes"),
            ),
            grouped_codebook_head: ItemPointer::decode(&input[60..66])?,
            directory_head: ItemPointer::decode(&input[66..72])?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{
        DistannMetadataPage, DISTANN_METADATA_BYTES, DISTANN_NEIGHBOR_CODEC_GROUPED_PQ,
        DISTANN_NEIGHBOR_CODEC_RABITQ,
    };
    use crate::storage::page::ItemPointer;

    fn sample() -> DistannMetadataPage {
        let mut metadata = DistannMetadataPage::empty(
            32,
            100,
            1.2,
            1536,
            42,
            DISTANN_NEIGHBOR_CODEC_GROUPED_PQ,
            4096,
            0.1,
        );
        metadata.entry_point = ItemPointer {
            block_number: 7,
            offset_number: 3,
        };
        metadata.node_count = 12_345;
        metadata.codec_subvector_count = 96;
        metadata.codec_subvector_dim = 16;
        metadata.grouped_codebook_head = ItemPointer {
            block_number: 42,
            offset_number: 1,
        };
        metadata.directory_head = ItemPointer {
            block_number: 43,
            offset_number: 2,
        };
        metadata
    }

    #[test]
    fn distann_metadata_round_trip_preserves_all_fields() {
        let metadata = sample();
        let encoded = metadata.encode();
        assert_eq!(encoded.len(), DISTANN_METADATA_BYTES);
        let decoded = DistannMetadataPage::decode(&encoded).expect("decode should succeed");
        assert_eq!(decoded, metadata);
    }

    #[test]
    fn distann_metadata_rejects_wrong_length() {
        let error = DistannMetadataPage::decode(&[0_u8; 8]).expect_err("short input must fail");
        assert!(error.contains("length mismatch"));
    }

    #[test]
    fn distann_metadata_rejects_unknown_format_version() {
        let mut encoded = sample().encode();
        encoded[0] = 0xFF;
        encoded[1] = 0xFF;
        let error =
            DistannMetadataPage::decode(&encoded).expect_err("unknown version must fail");
        assert!(error.contains("format version"));
    }

    #[test]
    fn distann_metadata_rejects_unknown_codec_kind() {
        let mut encoded = sample().encode();
        encoded[super::DISTANN_METADATA_NEIGHBOR_CODEC_KIND_OFFSET] = 99;
        let error = DistannMetadataPage::decode(&encoded).expect_err("unknown codec must fail");
        assert!(error.contains("neighbor codec kind"));
    }

    #[test]
    fn distann_metadata_codec_kinds_are_distinct() {
        assert_ne!(
            DISTANN_NEIGHBOR_CODEC_GROUPED_PQ,
            DISTANN_NEIGHBOR_CODEC_RABITQ
        );
    }
}
