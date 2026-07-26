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
    generation_descriptor::DistannCodecArtifact,
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

    /// Freeze the complete generation codec state. Seeded codecs need only
    /// shape/seed; GroupedPQ carries transform signs and every trained
    /// codebook so a participant never retrains on an owner-local corpus.
    pub(crate) fn to_artifact(
        &self,
        dimensions: u16,
        seed: u64,
    ) -> Result<DistannCodecArtifact, String> {
        let artifact = match self {
            Self::GroupedPq { model } => DistannCodecArtifact::GroupedPq4 {
                dimensions,
                seed,
                model: model.clone(),
            },
            Self::RaBitQ { .. } => DistannCodecArtifact::RaBitQ {
                dimensions,
                seed,
                bits: DISTANN_RABITQ_BITS,
            },
            Self::TurboQuant { .. } => DistannCodecArtifact::TurboQuant {
                dimensions,
                seed,
                bits: DISTANN_TURBOQUANT_BITS,
            },
        };
        artifact.validate()?;
        Ok(artifact)
    }

    pub(crate) fn from_artifact(artifact: &DistannCodecArtifact) -> Result<Self, String> {
        artifact.validate()?;
        let dimensions = usize::from(artifact.dimensions());
        let binding = match artifact {
            DistannCodecArtifact::GroupedPq4 { model, .. } => Self::GroupedPq {
                model: model.clone(),
            },
            DistannCodecArtifact::RaBitQ { seed, bits, .. } => Self::RaBitQ {
                quantizer: RaBitQQuantizer::cached_seeded_srht_bits(dimensions, *seed, *bits)?,
            },
            DistannCodecArtifact::TurboQuant { seed, bits, .. } => {
                let quantizer = ProdQuantizer::cached(dimensions, *bits, *seed);
                if !quantizer.int8_approx_no_qjl_4bit_supported() {
                    return Err(
                        "EC_GENERATION_DESCRIPTOR: restored TurboQuant artifact lacks supported no-QJL lane"
                            .to_owned(),
                    );
                }
                Self::TurboQuant { quantizer }
            }
        };
        Ok(binding)
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

    /// Prepare directly from the immutable generation artifact. This is the
    /// physical-owner path: no index-page codebook lookup and no retraining.
    pub(crate) fn prepare_artifact(
        artifact: &DistannCodecArtifact,
        raw_query: &[f32],
    ) -> Result<Self, String> {
        artifact.validate()?;
        let dimensions = usize::from(artifact.dimensions());
        if raw_query.len() != dimensions {
            return Err(format!(
                "ec_distann query dimension mismatch: artifact dim {dimensions}, query dim {}",
                raw_query.len()
            ));
        }
        match artifact {
            DistannCodecArtifact::GroupedPq4 { model, .. } => {
                let rotated = crate::quant::rotation::srht_padded(raw_query, &model.signs);
                let flat_codebooks = model
                    .codebooks
                    .iter()
                    .flat_map(|codebook| codebook.iter().copied())
                    .collect::<Vec<_>>();
                Ok(Self::GroupedPq {
                    query_lut: build_grouped_pq_lut_f32(
                        &rotated,
                        &flat_codebooks,
                        model.group_size,
                    ),
                    group_count: model.group_count,
                })
            }
            DistannCodecArtifact::RaBitQ { seed, bits, .. } => {
                let quantizer =
                    RaBitQQuantizer::cached_seeded_srht_bits(dimensions, *seed, *bits)?;
                Ok(Self::RaBitQ {
                    prepared: quantizer.prepare_estimator(raw_query),
                })
            }
            DistannCodecArtifact::TurboQuant { seed, bits, .. } => {
                let quantizer = ProdQuantizer::cached(dimensions, *bits, *seed);
                let prepared = quantizer.prepare_ip_query_lut_no_qjl_4bit(raw_query);
                Ok(Self::TurboQuant {
                    quantizer,
                    prepared,
                })
            }
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

#[cfg(test)]
mod tests {
    use super::*;

    const ARTIFACT_TEST_DIMENSIONS: usize = 1536;
    const SIMD_DIFF_WIDTHS: [usize; 9] = [1, 7, 8, 9, 16, 17, 31, 32, 33];

    fn corpus() -> Vec<Vec<f32>> {
        (0..32)
            .map(|row| {
                let mut vector = (0..ARTIFACT_TEST_DIMENSIONS)
                    .map(|dimension| ((row * 17 + dimension * 11) as f32).sin())
                    .collect::<Vec<_>>();
                let norm = vector.iter().map(|value| value * value).sum::<f32>().sqrt();
                for value in &mut vector {
                    *value /= norm;
                }
                vector
            })
            .collect()
    }

    fn deterministic_vector(dimensions: usize, row: usize) -> Vec<f32> {
        let mut vector = (0..dimensions)
            .map(|dimension| {
                let phase = (row * 37 + dimension * 13) as f32;
                phase.sin() + 0.25 * (phase * 0.03125).cos()
            })
            .collect::<Vec<_>>();
        let norm = vector.iter().map(|value| value * value).sum::<f32>().sqrt();
        for value in &mut vector {
            *value /= norm.max(f32::MIN_POSITIVE);
        }
        vector
    }

    fn grouped_pq_fixture() -> DistannCodecBinding {
        let dimensions = 64;
        let group_size = 8;
        let group_count = dimensions / group_size;
        let codebooks = (0..group_count)
            .map(|group| {
                (0..GROUPED_PQ_CENTROIDS * group_size)
                    .map(|index| {
                        let centroid = index / group_size;
                        let lane = index % group_size;
                        ((group * 101 + centroid * 17 + lane * 7) as f32 * 0.03125).sin()
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        DistannCodecBinding::GroupedPq {
            model: GroupedPq4Model {
                codebooks,
                group_count,
                group_size,
                transform_dim: dimensions,
                signs: crate::quant::rotation::sign_vector(dimensions, 42),
            },
        }
    }

    fn assert_distann_score(format: NeighborCodeFormat, actual: f32, expected: f32) {
        if format == NeighborCodeFormat::RaBitQ {
            let tolerance = 1.0e-5_f32 * actual.abs().max(expected.abs()).max(1.0);
            assert!(
                (actual - expected).abs() <= tolerance,
                "format={} actual={actual} expected={expected} tolerance={tolerance}",
                format.as_str()
            );
        } else {
            assert_eq!(
                actual.to_bits(),
                expected.to_bits(),
                "format={}",
                format.as_str()
            );
        }
    }

    #[test]
    fn simd_diff_distann_codec_batches_match_direct_scalar_scores_across_widths() {
        for format in [
            NeighborCodeFormat::GroupedPq,
            NeighborCodeFormat::RaBitQ,
            NeighborCodeFormat::TurboQuant,
        ] {
            let (binding, dimensions) = match format {
                NeighborCodeFormat::GroupedPq => (grouped_pq_fixture(), 64),
                NeighborCodeFormat::RaBitQ | NeighborCodeFormat::TurboQuant => (
                    DistannCodecBinding::prepare(format, &[], ARTIFACT_TEST_DIMENSIONS, 42)
                        .unwrap(),
                    ARTIFACT_TEST_DIMENSIONS,
                ),
            };
            let artifact = binding.to_artifact(dimensions as u16, 42).unwrap();
            let query = deterministic_vector(dimensions, 10_000);
            let prepared = DistannPreparedQuery::prepare_artifact(&artifact, &query).unwrap();
            let code_len = binding.code_len(dimensions).unwrap();
            let codes = (0..SIMD_DIFF_WIDTHS[SIMD_DIFF_WIDTHS.len() - 1])
                .map(|row| binding.encode(&deterministic_vector(dimensions, row + 1)))
                .collect::<Vec<_>>();
            assert!(
                codes.iter().all(|code| code.len() == code_len),
                "{} payload stride",
                format.as_str()
            );

            let first_raw_ip = match &prepared {
                DistannPreparedQuery::GroupedPq {
                    query_lut,
                    group_count,
                } => grouped_pq_score_f32(query_lut, *group_count, &codes[0]),
                DistannPreparedQuery::RaBitQ { prepared } => {
                    prepared.estimate_ip_scalar_only(&codes[0])
                }
                DistannPreparedQuery::TurboQuant {
                    quantizer,
                    prepared,
                } => quantizer.score_ip_from_parts_lut_no_qjl_4bit(prepared, &codes[0]),
            };
            assert_eq!(
                prepared.score_dist(&codes[0]).to_bits(),
                (-first_raw_ip).to_bits(),
                "{} direct IP-to-distance negation",
                format.as_str()
            );

            for width in SIMD_DIFF_WIDTHS {
                let mut slab = codes[..width]
                    .iter()
                    .flat_map(|code| code.iter().copied())
                    .collect::<Vec<_>>();
                // A complete poison payload after the requested candidates
                // proves the binding uses count × persisted stride and does
                // not consume a neighboring record.
                slab.extend(std::iter::repeat_n(0xA5, code_len));
                let mut batch_scores = vec![f32::NAN; width];
                prepared
                    .score_dists_batch(&slab, code_len, width, &mut batch_scores)
                    .unwrap();

                for (slot, actual) in batch_scores.iter().copied().enumerate() {
                    let expected = prepared.score_dist(&codes[slot]);
                    assert_distann_score(format, actual, expected);
                }
            }

            eprintln!(
                "task36_distann format={} dimensions={} widths={:?} host_isa={}",
                format.as_str(),
                dimensions,
                SIMD_DIFF_WIDTHS,
                crate::quant::isa::current_isa().label()
            );
        }
    }

    #[test]
    fn codec_artifact_restores_codes_and_prepared_scores_without_retraining() {
        let corpus = corpus();
        let refs = corpus.iter().map(Vec::as_slice).collect::<Vec<_>>();
        for format in [
            NeighborCodeFormat::RaBitQ,
            NeighborCodeFormat::TurboQuant,
            NeighborCodeFormat::GroupedPq,
        ] {
            let binding =
                DistannCodecBinding::prepare(format, &refs, ARTIFACT_TEST_DIMENSIONS, 42).unwrap();
            let artifact = binding
                .to_artifact(ARTIFACT_TEST_DIMENSIONS as u16, 42)
                .unwrap();
            let canonical = artifact.encode().unwrap();
            let decoded = DistannCodecArtifact::decode(&canonical).unwrap();
            let restored = DistannCodecBinding::from_artifact(&decoded).unwrap();

            let original_code = binding.encode(&corpus[1]);
            assert_eq!(restored.encode(&corpus[1]), original_code);

            let artifact_prepared =
                DistannPreparedQuery::prepare_artifact(&decoded, &corpus[0]).unwrap();
            let mut metadata = DistannMetadataPage::empty(
                4,
                16,
                1.2,
                ARTIFACT_TEST_DIMENSIONS as u16,
                42,
                binding.metadata_kind(),
                16,
                0.3,
            );
            metadata.codec_subvector_count = binding.metadata_subvector_count();
            metadata.codec_subvector_dim = binding.metadata_subvector_dim();
            let flat_codebooks = binding.grouped_model().map(|model| {
                model
                    .codebooks
                    .iter()
                    .flat_map(|codebook| codebook.iter().copied())
                    .collect::<Vec<_>>()
            });
            let legacy_prepared =
                DistannPreparedQuery::prepare(&metadata, flat_codebooks.as_deref(), &corpus[0])
                    .unwrap();
            assert_eq!(
                artifact_prepared.score_dist(&original_code).to_bits(),
                legacy_prepared.score_dist(&original_code).to_bits(),
                "format {}",
                format.as_str()
            );
        }
    }

    #[test]
    fn seeded_codec_v1_golden_score_vectors() {
        use sha2::{Digest, Sha256};

        let source_pattern = [0.5, -0.25, 0.75, -1.0, 0.125, 0.625, -0.875, 0.375];
        let query_pattern = [-0.75, 0.5, 0.25, 0.875, -0.125, 1.0, -0.5, 0.625];
        let source = (0..ARTIFACT_TEST_DIMENSIONS)
            .map(|index| source_pattern[index % source_pattern.len()])
            .collect::<Vec<_>>();
        let query = (0..ARTIFACT_TEST_DIMENSIONS)
            .map(|index| query_pattern[index % query_pattern.len()])
            .collect::<Vec<_>>();
        for (format, expected_len, expected_digest, expected_score_bits) in [
            (
                NeighborCodeFormat::RaBitQ,
                204,
                "808e6d09cf3495d0366956f20f4b7c50b54049c93f26fad0324c2437c42b4fce",
                0xc17c_0de7,
            ),
            (
                NeighborCodeFormat::TurboQuant,
                768,
                "2ea3509f51fe2414ad68dc28d97a7eca20f302a793c652c4f17e3f085100cf0b",
                0xbfba_042d,
            ),
        ] {
            let binding = DistannCodecBinding::prepare(format, &[], source.len(), 42).unwrap();
            let artifact = binding.to_artifact(source.len() as u16, 42).unwrap();
            let code = binding.encode(&source);
            let score = DistannPreparedQuery::prepare_artifact(&artifact, &query)
                .unwrap()
                .score_dist(&code);
            assert_eq!(code.len(), expected_len, "{} code length", format.as_str());
            assert_eq!(
                hex::encode(Sha256::digest(&code)),
                expected_digest,
                "{} canonical code bytes changed; bump codec artifact version before updating this vector",
                format.as_str()
            );
            assert_eq!(
                score.to_bits(),
                expected_score_bits,
                "{} canonical score changed; bump codec artifact version before updating this vector",
                format.as_str()
            );
        }
    }
}
