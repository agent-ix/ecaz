//! Neighbor-code codec binding for `ec_distann` (ADR-085 D7).
//!
//! One codec serves both FR-076 code roles: the record's own `search_code`
//! and the embedded per-neighbor codes are the same format and stride, so a
//! single prepared query scores both. The three formats mirror the
//! `ec_diskann` codec family: GroupedPq (trained, codebooks persisted in
//! the index chain), RaBitQ and TurboQuant (seeded, nothing persisted).

use std::sync::Arc;

use crate::am::common::candidate_batch::{
    score_grouped_pq_batch_for, score_rabitq_bits1_batch_for,
    score_turboquant_no_qjl_4bit_batch_for, CandidateBatch, CandidateBatchScoringSurface,
    CandidateMeta, CandidatePayload,
};
use crate::am::common::training::{self, GroupedPq4Model};
use crate::quant::{
    grouped_pq::{build_grouped_pq_lut_f32, grouped_pq_score_f32, GROUPED_PQ_CENTROIDS},
    prod::{mse_code_len, PreparedLutNoQjl4BitQuery, ProdQuantizer},
    rabitq::{code_len_for, PreparedEstimator, RaBitQQuantizer},
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

/// Prepared per-query scoring state; one instance scores both search codes
/// and embedded neighbor codes (same codec, same stride). `score_dist`
/// returns the negated estimated inner product, matching the `-ip` distance
/// convention of the ec_diskann scan path.
pub(crate) enum DistannPreparedQuery {
    GroupedPq {
        query_lut: Vec<f32>,
        group_count: usize,
    },
    RaBitQ {
        prepared: PreparedEstimator,
    },
    TurboQuant {
        quantizer: Arc<ProdQuantizer>,
        prepared: PreparedLutNoQjl4BitQuery,
    },
}

impl DistannPreparedQuery {
    /// `flat_codebooks` is required for (and only for) the GroupedPq codec:
    /// the flat centroid array read from the persisted codebook chain.
    pub(crate) fn prepare(
        metadata: &DistannMetadataPage,
        flat_codebooks: Option<&[f32]>,
        raw_query: &[f32],
    ) -> Result<Self, String> {
        let dimensions = usize::from(metadata.dimensions);
        if raw_query.len() != dimensions {
            return Err(format!(
                "ec_distann query dimension mismatch: index dim {dimensions}, query dim {}",
                raw_query.len()
            ));
        }
        match metadata.neighbor_codec_kind {
            DISTANN_NEIGHBOR_CODEC_GROUPED_PQ => {
                let flat_codebooks = flat_codebooks.ok_or_else(|| {
                    "ec_distann grouped_pq query preparation requires the codebook chain"
                        .to_owned()
                })?;
                let group_count = usize::from(metadata.codec_subvector_count);
                let group_size = usize::from(metadata.codec_subvector_dim);
                if group_count == 0 || group_size == 0 {
                    return Err(
                        "ec_distann grouped_pq metadata is missing subvector parameters".to_owned()
                    );
                }
                let rotated = crate::am::ec_diskann::scan_query::encode_query_srht(
                    raw_query,
                    dimensions,
                    metadata.seed,
                );
                let query_lut = build_grouped_pq_lut_f32(&rotated, flat_codebooks, group_size);
                Ok(Self::GroupedPq {
                    query_lut,
                    group_count,
                })
            }
            DISTANN_NEIGHBOR_CODEC_RABITQ => {
                let bits = u8::try_from(metadata.codec_subvector_dim)
                    .map_err(|_| "ec_distann RaBitQ bit width exceeds u8".to_owned())?;
                let quantizer =
                    RaBitQQuantizer::cached_seeded_srht_bits(dimensions, metadata.seed, bits)?;
                Ok(Self::RaBitQ {
                    prepared: quantizer.prepare_estimator(raw_query),
                })
            }
            DISTANN_NEIGHBOR_CODEC_TURBOQUANT => {
                let quantizer =
                    ProdQuantizer::cached(dimensions, DISTANN_TURBOQUANT_BITS, metadata.seed);
                let prepared = quantizer.prepare_ip_query_lut_no_qjl_4bit(raw_query);
                Ok(Self::TurboQuant {
                    quantizer,
                    prepared,
                })
            }
            other => Err(format!(
                "ec_distann unsupported neighbor codec kind {other}"
            )),
        }
    }

    /// Distance-ordered code score (`-estimated_ip`; smaller is better).
    pub(crate) fn score_dist(&self, code: &[u8]) -> f32 {
        match self {
            Self::GroupedPq {
                query_lut,
                group_count,
            } => -grouped_pq_score_f32(query_lut, *group_count, code),
            Self::RaBitQ { prepared } => -prepared.estimate_ip_scalar_only(code),
            Self::TurboQuant {
                quantizer,
                prepared,
            } => -quantizer.score_ip_from_parts_lut_no_qjl_4bit(prepared, code),
        }
    }
}

impl DistannPreparedQuery {
    /// Batched variant of [`Self::score_dist`] over a fixed-stride block of
    /// `count` codes (the FR-076 embedded neighbor-code block): dispatches
    /// into the 32-wide block kernels instead of per-code scalar scoring.
    pub(crate) fn score_dists_batch(
        &self,
        codes: &[u8],
        code_len: usize,
        count: usize,
        out_scores: &mut [f32],
    ) -> Result<(), String> {
        if out_scores.len() != count || codes.len() < count * code_len {
            return Err("ec_distann batch scoring shape mismatch".to_owned());
        }
        if count == 0 {
            return Ok(());
        }
        match self {
            Self::GroupedPq {
                query_lut,
                group_count,
            } => {
                let mut batch = CandidateBatch::with_capacity(count);
                for slot in 0..count {
                    batch.push(
                        slot,
                        CandidatePayload::new(
                            &codes[slot * code_len..(slot + 1) * code_len],
                            CandidateMeta::GroupedPq {
                                group_count: *group_count,
                            },
                        ),
                    )?;
                }
                score_grouped_pq_batch_for(
                    CandidateBatchScoringSurface::Distann,
                    query_lut,
                    *group_count,
                    &batch,
                    out_scores,
                )?;
            }
            Self::RaBitQ { prepared } => {
                match prepared.bits1_block_prepared(code_len) {
                    Some(block_prepared) => {
                        let mut batch = CandidateBatch::with_capacity(count);
                        for slot in 0..count {
                            batch.push(
                                slot,
                                CandidatePayload::new(
                                    &codes[slot * code_len..(slot + 1) * code_len],
                                    CandidateMeta::RaBitQ,
                                ),
                            )?;
                        }
                        score_rabitq_bits1_batch_for(
                            CandidateBatchScoringSurface::Distann,
                            block_prepared,
                            &batch,
                            out_scores,
                        )?;
                    }
                    None => {
                        for slot in 0..count {
                            out_scores[slot] = prepared.estimate_ip_scalar_only(
                                &codes[slot * code_len..(slot + 1) * code_len],
                            );
                        }
                    }
                }
            }
            Self::TurboQuant {
                quantizer,
                prepared,
            } => {
                let mut batch = CandidateBatch::with_capacity(count);
                for slot in 0..count {
                    batch.push(
                        slot,
                        CandidatePayload::new(
                            &codes[slot * code_len..(slot + 1) * code_len],
                            CandidateMeta::None,
                        ),
                    )?;
                }
                score_turboquant_no_qjl_4bit_batch_for(
                    CandidateBatchScoringSurface::Distann,
                    quantizer,
                    prepared,
                    &batch,
                    out_scores,
                )?;
            }
        }
        // Codec batch entry points return ip estimates; the scan distance
        // convention is -ip (matching score_dist).
        for score in out_scores.iter_mut() {
            *score = -*score;
        }
        Ok(())
    }
}

/// Centroid count per codebook tuple for the persisted GroupedPq chain.
pub(crate) fn grouped_centroid_count(metadata: &DistannMetadataPage) -> usize {
    usize::from(metadata.codec_subvector_dim) * GROUPED_PQ_CENTROIDS
}
