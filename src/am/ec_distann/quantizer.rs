//! Neighbor-code codec binding for `ec_distann` (ADR-085 D7).
//!
//! One codec serves both FR-076 code roles: the record's own `search_code`
//! and the embedded per-neighbor codes are the same format and stride, so a
//! single prepared query scores both. The three formats mirror the
//! `ec_diskann` codec family: GroupedPq (trained, codebooks persisted in
//! the index chain), RaBitQ and TurboQuant (seeded, nothing persisted).

use std::sync::Arc;

use crate::am::common::training::{self, GroupedPq4Model};
use crate::quant::{
    prod::{mse_code_len, ProdQuantizer},
    rabitq::{code_len_for, RaBitQQuantizer},
    Quantizer,
};

use super::{
    options::NeighborCodeFormat,
    page::{
        DistannMetadataPage, DISTANN_NEIGHBOR_CODEC_GROUPED_PQ, DISTANN_NEIGHBOR_CODEC_RABITQ,
        DISTANN_NEIGHBOR_CODEC_TURBOQUANT,
    },
};

/// Same widths as the benchmarked ec_diskann codecs, so the M0 parity A/B
/// compares equal-fidelity codes.
pub(super) const DISTANN_RABITQ_BITS: u8 = 1;
pub(super) const DISTANN_TURBOQUANT_BITS: u8 = 4;
pub(super) const DISTANN_PQ_DEFAULT_MAX_TRAIN_SIZE: usize = 1024;
pub(super) const DISTANN_PQ_DEFAULT_KMEANS_ITERS: usize = 8;

pub(crate) enum DistannCodecBinding {
    GroupedPq { model: GroupedPq4Model },
    RaBitQ { quantizer: Arc<RaBitQQuantizer> },
    TurboQuant { quantizer: Arc<ProdQuantizer> },
}

impl DistannCodecBinding {
    pub(crate) fn prepare(
        format: NeighborCodeFormat,
        source_refs: &[&[f32]],
        dimensions: usize,
        seed: u64,
    ) -> Result<Self, String> {
        match format {
            NeighborCodeFormat::GroupedPq => {
                let group_size = crate::am::ec_diskann::default_group_size(
                    u16::try_from(dimensions)
                        .map_err(|_| format!("ec_distann dimensions {dimensions} exceed u16"))?,
                );
                let model = training::train_grouped_pq4_model(
                    source_refs,
                    dimensions,
                    seed,
                    group_size,
                    DISTANN_PQ_DEFAULT_MAX_TRAIN_SIZE,
                    DISTANN_PQ_DEFAULT_KMEANS_ITERS,
                )?;
                Ok(Self::GroupedPq { model })
            }
            NeighborCodeFormat::RaBitQ => {
                let quantizer = RaBitQQuantizer::cached_seeded_srht_bits(
                    dimensions,
                    seed,
                    DISTANN_RABITQ_BITS,
                )?;
                Ok(Self::RaBitQ { quantizer })
            }
            NeighborCodeFormat::TurboQuant => {
                let quantizer = ProdQuantizer::cached(dimensions, DISTANN_TURBOQUANT_BITS, seed);
                if !quantizer.int8_approx_no_qjl_4bit_supported() {
                    return Err(
                        "ec_distann TurboQuant neighbor_code_format requires a no-QJL 4-bit dimension lane"
                            .to_owned(),
                    );
                }
                Ok(Self::TurboQuant { quantizer })
            }
        }
    }

    pub(crate) fn encode(&self, source_vector: &[f32]) -> Vec<u8> {
        match self {
            Self::GroupedPq { model } => training::derive_grouped_pq4_code(source_vector, model),
            Self::RaBitQ { quantizer } => quantizer.encode_code(source_vector).into_vec(),
            Self::TurboQuant { quantizer } => quantizer.encode(source_vector).mse_packed,
        }
    }

    pub(crate) fn code_len(&self, dimensions: usize) -> Result<usize, String> {
        match self {
            Self::GroupedPq { model } => Ok(model.group_count.div_ceil(2)),
            Self::RaBitQ { .. } => code_len_for(dimensions, DISTANN_RABITQ_BITS),
            Self::TurboQuant { .. } => Ok(mse_code_len(dimensions, DISTANN_TURBOQUANT_BITS)),
        }
    }

    pub(super) fn metadata_kind(&self) -> u8 {
        match self {
            Self::GroupedPq { .. } => DISTANN_NEIGHBOR_CODEC_GROUPED_PQ,
            Self::RaBitQ { .. } => DISTANN_NEIGHBOR_CODEC_RABITQ,
            Self::TurboQuant { .. } => DISTANN_NEIGHBOR_CODEC_TURBOQUANT,
        }
    }

    pub(super) fn metadata_subvector_count(&self) -> u16 {
        match self {
            Self::GroupedPq { model } => u16::try_from(model.group_count)
                .expect("ec_distann grouped-PQ group count should fit in u16"),
            Self::RaBitQ { .. } | Self::TurboQuant { .. } => 0,
        }
    }

    pub(super) fn metadata_subvector_dim(&self) -> u16 {
        match self {
            Self::GroupedPq { model } => u16::try_from(model.group_size)
                .expect("ec_distann grouped-PQ group size should fit in u16"),
            Self::RaBitQ { .. } => u16::from(DISTANN_RABITQ_BITS),
            Self::TurboQuant { .. } => u16::from(DISTANN_TURBOQUANT_BITS),
        }
    }

    pub(super) fn grouped_model(&self) -> Option<&GroupedPq4Model> {
        match self {
            Self::GroupedPq { model } => Some(model),
            Self::RaBitQ { .. } | Self::TurboQuant { .. } => None,
        }
    }
}

/// Code stride implied by persisted metadata, for readers that have no
/// binding (mirrors `ec_diskann::quantizer::metadata_search_code_len`).
pub(crate) fn metadata_code_len(metadata: &DistannMetadataPage) -> Result<usize, String> {
    match metadata.neighbor_codec_kind {
        DISTANN_NEIGHBOR_CODEC_GROUPED_PQ => {
            Ok(usize::from(metadata.codec_subvector_count).div_ceil(2))
        }
        DISTANN_NEIGHBOR_CODEC_RABITQ => {
            let bits = u8::try_from(metadata.codec_subvector_dim).map_err(|_| {
                format!(
                    "ec_distann RaBitQ bit width {} exceeds u8",
                    metadata.codec_subvector_dim
                )
            })?;
            code_len_for(usize::from(metadata.dimensions), bits)
        }
        DISTANN_NEIGHBOR_CODEC_TURBOQUANT => Ok(mse_code_len(
            usize::from(metadata.dimensions),
            DISTANN_TURBOQUANT_BITS,
        )),
        other => Err(format!("ec_distann unsupported neighbor codec kind {other}")),
    }
}
