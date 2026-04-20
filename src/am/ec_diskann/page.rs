//! Vamana-specific page-layout primitives for `ec_diskann`.
//!
//! Phase 1C landing: metadata page struct + `INDEX_FORMAT_V3_DISKANN`
//! wire tag only. Element-tuple / neighbor-tuple layouts are phase 1D.
//! Page-aligned persistence and block helpers live in the tqhnsw
//! `page` module and are reused unchanged (shared primitives like
//! `ItemPointer`, `PAGE_HEADER_BYTES`).
//!
//! The Vamana metadata page is laid out on block 0 (same as
//! `tqhnsw`'s `MetadataPage`). The V3 tag distinguishes it from the
//! V1/V2 HNSW metadata at decode time: shared code paths that touch
//! block 0 must inspect the format tag before interpreting the rest
//! of the struct.

use crate::storage::page::ItemPointer;

/// Wire-format tag for Vamana indexes. Distinct from
/// `INDEX_FORMAT_V1_SCALAR` (1) and `INDEX_FORMAT_V2_GROUPED` (2)
/// used by `tqhnsw`.
pub const INDEX_FORMAT_V3_DISKANN: u16 = 3;

/// Vamana metadata page uses SRHT + grouped-PQ search codes, same as
/// the `tqhnsw` V2 layout. These flags are kept compatible so that
/// shared hot/cold page walkers can treat payload-flag bits
/// identically across AMs.
pub const PAYLOAD_FLAG_BINARY_SIDECAR: u8 = 1 << 0;
pub const PAYLOAD_FLAG_GROUPED_SEARCH_CODE: u8 = 1 << 1;
pub const PAYLOAD_FLAG_COLD_RERANK_PAYLOAD: u8 = 1 << 2;

pub const VAMANA_TRANSFORM_KIND_SRHT: u8 = 1;
pub const VAMANA_SEARCH_CODEC_GROUPED_PQ: u8 = 2;

/// Size of the encoded metadata page payload. Locked at 48 bytes for
/// the v0 layout; any field additions must bump the format tag.
pub const VAMANA_METADATA_BYTES: usize = 48;

#[derive(Debug, Clone, PartialEq)]
pub struct VamanaMetadataPage {
    pub format_version: u16,
    pub entry_point: ItemPointer,
    pub graph_degree_r: u16,
    pub build_list_size_l: u16,
    pub alpha: f32,
    pub dimensions: u16,
    pub seed: u64,
    pub inserted_since_rebuild: u64,
    pub needs_medoid_refresh: bool,
    pub transform_kind: u8,
    pub search_codec_kind: u8,
    pub payload_flags: u8,
    pub search_subvector_count: u16,
    pub search_subvector_dim: u16,
    pub grouped_codebook_head: ItemPointer,
}

impl VamanaMetadataPage {
    pub fn empty(
        graph_degree_r: u16,
        build_list_size_l: u16,
        alpha: f32,
        dimensions: u16,
        seed: u64,
    ) -> Self {
        Self {
            format_version: INDEX_FORMAT_V3_DISKANN,
            entry_point: ItemPointer::INVALID,
            graph_degree_r,
            build_list_size_l,
            alpha,
            dimensions,
            seed,
            inserted_since_rebuild: 0,
            needs_medoid_refresh: false,
            transform_kind: VAMANA_TRANSFORM_KIND_SRHT,
            search_codec_kind: VAMANA_SEARCH_CODEC_GROUPED_PQ,
            payload_flags: PAYLOAD_FLAG_GROUPED_SEARCH_CODE | PAYLOAD_FLAG_COLD_RERANK_PAYLOAD,
            search_subvector_count: 0,
            search_subvector_dim: 0,
            grouped_codebook_head: ItemPointer::INVALID,
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(VAMANA_METADATA_BYTES);
        out.extend_from_slice(&self.format_version.to_le_bytes());
        self.entry_point.encode_into(&mut out);
        out.extend_from_slice(&self.graph_degree_r.to_le_bytes());
        out.extend_from_slice(&self.build_list_size_l.to_le_bytes());
        out.extend_from_slice(&self.alpha.to_le_bytes());
        out.extend_from_slice(&self.dimensions.to_le_bytes());
        out.extend_from_slice(&self.seed.to_le_bytes());
        out.extend_from_slice(&self.inserted_since_rebuild.to_le_bytes());
        out.push(self.needs_medoid_refresh as u8);
        out.push(self.transform_kind);
        out.push(self.search_codec_kind);
        out.push(self.payload_flags);
        out.extend_from_slice(&self.search_subvector_count.to_le_bytes());
        out.extend_from_slice(&self.search_subvector_dim.to_le_bytes());
        self.grouped_codebook_head.encode_into(&mut out);
        debug_assert_eq!(out.len(), VAMANA_METADATA_BYTES);
        out
    }

    pub fn decode(input: &[u8]) -> Result<Self, String> {
        if input.len() != VAMANA_METADATA_BYTES {
            return Err(format!(
                "vamana metadata length mismatch: got {}, expected {VAMANA_METADATA_BYTES}",
                input.len()
            ));
        }

        let format_version =
            u16::from_le_bytes(input[0..2].try_into().expect("format version bytes"));
        if format_version != INDEX_FORMAT_V3_DISKANN {
            return Err(format!(
                "invalid vamana metadata format version: got {format_version}, expected {INDEX_FORMAT_V3_DISKANN}"
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
            inserted_since_rebuild: u64::from_le_bytes(
                input[26..34]
                    .try_into()
                    .expect("inserted-since-rebuild bytes"),
            ),
            needs_medoid_refresh: input[34] != 0,
            transform_kind: input[35],
            search_codec_kind: input[36],
            payload_flags: input[37],
            search_subvector_count: u16::from_le_bytes(
                input[38..40]
                    .try_into()
                    .expect("search_subvector_count bytes"),
            ),
            search_subvector_dim: u16::from_le_bytes(
                input[40..42].try_into().expect("search_subvector_dim bytes"),
            ),
            grouped_codebook_head: ItemPointer::decode(&input[42..48])?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // LA-001: fresh metadata round-trips losslessly.
    #[test]
    fn la_001_fresh_metadata_roundtrip() {
        let metadata = VamanaMetadataPage::empty(32, 100, 1.2, 1536, 0xdeadbeef);
        let encoded = metadata.encode();
        let decoded = VamanaMetadataPage::decode(&encoded).expect("decode");
        assert_eq!(metadata, decoded);
    }

    // LA-002: the wire-format tag is exactly 3 (distinct from V1=1, V2=2).
    #[test]
    fn la_002_format_version_tag_is_three() {
        assert_eq!(INDEX_FORMAT_V3_DISKANN, 3);
        let metadata = VamanaMetadataPage::empty(32, 100, 1.2, 1536, 0);
        assert_eq!(metadata.format_version, INDEX_FORMAT_V3_DISKANN);
    }

    // LA-003: encoded payload is exactly VAMANA_METADATA_BYTES long.
    #[test]
    fn la_003_encoded_length_matches_constant() {
        let metadata = VamanaMetadataPage::empty(32, 100, 1.2, 1536, 0);
        assert_eq!(metadata.encode().len(), VAMANA_METADATA_BYTES);
    }

    // LA-004: a byte blob carrying a foreign format tag is rejected.
    #[test]
    fn la_004_foreign_format_tag_rejected() {
        let mut metadata = VamanaMetadataPage::empty(32, 100, 1.2, 1536, 0);
        metadata.format_version = 2; // INDEX_FORMAT_V2_GROUPED
        let encoded = metadata.encode();
        let err = VamanaMetadataPage::decode(&encoded).expect_err("decode should fail");
        assert!(
            err.contains("invalid vamana metadata format version"),
            "unexpected error: {err}"
        );
    }

    // LA-005: alpha survives round-trip with bit-exact f32 representation.
    #[test]
    fn la_005_alpha_f32_bit_exact() {
        let metadata = VamanaMetadataPage::empty(32, 100, 1.2, 1536, 0);
        let encoded = metadata.encode();
        let decoded = VamanaMetadataPage::decode(&encoded).expect("decode");
        assert_eq!(metadata.alpha.to_bits(), decoded.alpha.to_bits());
    }

    // LA-006: needs_medoid_refresh flag toggles round-trip correctly.
    #[test]
    fn la_006_needs_medoid_refresh_flag_roundtrips() {
        let mut metadata = VamanaMetadataPage::empty(32, 100, 1.2, 1536, 0);
        metadata.needs_medoid_refresh = true;
        let encoded = metadata.encode();
        let decoded = VamanaMetadataPage::decode(&encoded).expect("decode");
        assert!(decoded.needs_medoid_refresh);
    }
}
