use super::{options, page};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HnswStorageCodec {
    TurboQuant,
    PqFastScan,
}

impl HnswStorageCodec {
    pub(crate) fn from_storage_format(storage_format: options::StorageFormat) -> Self {
        match storage_format {
            options::StorageFormat::TurboQuant => Self::TurboQuant,
            options::StorageFormat::PqFastScan => Self::PqFastScan,
        }
    }

    pub(crate) fn from_metadata(metadata: &page::MetadataPage) -> Result<Self, String> {
        match metadata.graph_storage_format()? {
            page::GraphStorageFormat::TurboQuant => Ok(Self::TurboQuant),
            page::GraphStorageFormat::PqFastScan => Ok(Self::PqFastScan),
        }
    }

    pub(crate) fn storage_format(self) -> options::StorageFormat {
        match self {
            Self::TurboQuant => options::StorageFormat::TurboQuant,
            Self::PqFastScan => options::StorageFormat::PqFastScan,
        }
    }

    pub(crate) fn storage_format_name(self) -> &'static str {
        self.storage_format().as_str()
    }

    pub(crate) fn matches_storage_format(self, storage_format: options::StorageFormat) -> bool {
        self.storage_format() == storage_format
    }

    pub(crate) fn initial_metadata(self, m: u16, ef_construction: u16) -> page::MetadataPage {
        match self {
            Self::TurboQuant => {
                page::MetadataPage::current_v3_turbo_hot_cold(page::CurrentFormatMetadata {
                    m,
                    ef_construction,
                    entry_point: page::ItemPointer::INVALID,
                    dimensions: 0,
                    bits: 0,
                    max_level: 0,
                    seed: 0,
                    inserted_since_rebuild: 0,
                    persisted_binary_sidecar: false,
                })
            }
            Self::PqFastScan => page::MetadataPage {
                m,
                ef_construction,
                entry_point: page::ItemPointer::INVALID,
                dimensions: 0,
                bits: 0,
                max_level: 0,
                seed: 0,
                inserted_since_rebuild: 0,
                format_version: page::INDEX_FORMAT_V2_GROUPED,
                transform_kind: page::TransformKind::Srht,
                search_codec_kind: page::SearchCodecKind::GroupedPq,
                payload_flags: page::PAYLOAD_FLAG_GROUPED_SEARCH_CODE
                    | page::PAYLOAD_FLAG_COLD_RERANK_PAYLOAD,
                search_bits: 4,
                rerank_codec_kind: page::RerankCodecKind::ScalarQuantized,
                search_subvector_count: 0,
                search_subvector_dim: 0,
                grouped_codebook_head: page::ItemPointer::INVALID,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn storage_codec_maps_reloptions_to_names() {
        assert_eq!(
            HnswStorageCodec::from_storage_format(options::StorageFormat::TurboQuant)
                .storage_format_name(),
            "turboquant"
        );
        assert_eq!(
            HnswStorageCodec::from_storage_format(options::StorageFormat::PqFastScan)
                .storage_format_name(),
            "pq_fastscan"
        );
    }

    #[test]
    fn initial_metadata_preserves_existing_format_identities() {
        let turbo = HnswStorageCodec::TurboQuant.initial_metadata(8, 64);
        assert_eq!(turbo.format_version, page::INDEX_FORMAT_V3_TURBO_HOT_COLD);
        assert_eq!(turbo.search_codec_kind, page::SearchCodecKind::Unknown);
        assert_eq!(turbo.rerank_codec_kind, page::RerankCodecKind::None);

        let grouped = HnswStorageCodec::PqFastScan.initial_metadata(8, 64);
        assert_eq!(grouped.format_version, page::INDEX_FORMAT_V2_GROUPED);
        assert_eq!(grouped.search_codec_kind, page::SearchCodecKind::GroupedPq);
        assert_eq!(
            grouped.rerank_codec_kind,
            page::RerankCodecKind::ScalarQuantized
        );
    }

    #[test]
    fn metadata_maps_back_to_codec() {
        let turbo = HnswStorageCodec::TurboQuant.initial_metadata(8, 64);
        assert_eq!(
            HnswStorageCodec::from_metadata(&turbo).unwrap(),
            HnswStorageCodec::TurboQuant
        );

        let grouped = HnswStorageCodec::PqFastScan.initial_metadata(8, 64);
        assert_eq!(
            HnswStorageCodec::from_metadata(&grouped).unwrap(),
            HnswStorageCodec::PqFastScan
        );
    }
}
