use super::options::{StorageFormat, TurboQuantScorerGuc, EC_IVF_DEFAULT_RABITQ_RERANK_CLIP};
use super::page;
use crate::am::common::candidate_batch::{
    record_ivf_rabitq_arithmetic_batch_flush_width, score_grouped_pq_batch_for,
    score_rabitq_bits1_batch_for, score_rabitq_bitsn_batch_for,
    score_turboquant_int8_approx_batch_for, score_turboquant_no_qjl_4bit_batch_for,
    score_turboquant_qjl_batch_for, CandidateBatch, CandidateBatchScoringSurface, CandidateMeta,
    CandidatePayload,
};
use crate::am::common::quant_codec::{
    EncodedQuantPayload, QuantCodec, QuantCodecKind, QuantSearchCodecTag,
};
use crate::quant::grouped_pq::{
    build_grouped_pq_lut_f32, grouped_pq_score_f32, GROUPED_PQ_CENTROIDS,
};
use crate::quant::prod::{
    ExactScoreMode, Int8ApproxNoQjl4BitQuery, PreparedLutNoQjl4BitQuery, PreparedQuery,
    PreparedTqCalibratedNoQjl4BitQuery, ProdQuantizer, TqCalibration,
};
use crate::quant::rabitq::{code_len_for, PreparedEstimator, RaBitQQuantizer};
use crate::quant::rotation;
use crate::quant::Quantizer;
use crate::storage::page::ItemPointer;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum IvfQuantizerProfile {
    TurboQuant,
    PqFastScan {
        group_count: usize,
        group_size: usize,
    },
    RaBitQ,
}

pub(super) enum IvfPreparedQuery {
    TurboQuant(PreparedQuery),
    TurboQuantNoQjl4BitLut(PreparedLutNoQjl4BitQuery),
    /// Task 136: factored rank-1 in-register scorer for the no-QJL 4-bit
    /// lane. Query-side alternative to the i16 LUT selected via the
    /// `ec_ivf.turboquant_scorer` session GUC; on-disk codes are unchanged.
    TurboQuantNoQjl4BitInt8Approx(Int8ApproxNoQjl4BitQuery),
    TurboQuantCalibratedNoQjl4Bit(PreparedTqCalibratedNoQjl4BitQuery),
    PqFastScan {
        lut: Vec<f32>,
        group_count: usize,
        suffix_max: Vec<f32>,
    },
    RaBitQ(PreparedEstimator),
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct IvfPqFastScanModel {
    pub(super) group_count: usize,
    pub(super) group_size: usize,
    pub(super) signs: Vec<f32>,
    pub(super) flat_codebooks: Vec<f32>,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct IvfTqCalibrationModel {
    pub(super) calibration: TqCalibration,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct IvfQuantizer {
    profile: IvfQuantizerProfile,
    dimensions: usize,
    /// Per-dim code width for the RaBitQ branch. Ignored for
    /// TurboQuant / PqFastScan profiles. Always one of {1, 2, 4, 8}.
    rabitq_bits: u8,
    /// Integer scalar clip radius for RaBitQ encoders/scorers. Stored as an
    /// integer because reloptions currently expose the Task 111h A/B values
    /// {2,3,4}; default 2 preserves the existing profile.
    rabitq_quant_clip: u8,
    /// Task 115: when set (RaBitQ profile only), postings are encoded as the
    /// residual `o − c` against the assigned IVF centroid via
    /// [`Self::encode_source_residual`], and scan adds the exact per-list
    /// centroid term `⟨q, c⟩` back. Default false (plain RaBitQ). The payload
    /// layout/size is identical either way; only the encoded vector and the
    /// scan-side centroid add differ. The non-residual `encode_source` path
    /// rejects RaBitQ-residual indexes so a centroid is never silently dropped.
    rabitq_residual: bool,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct IvfQuantCodec<'a> {
    quantizer: IvfQuantizer,
    pq_model: Option<&'a IvfPqFastScanModel>,
}

impl<'a> IvfQuantCodec<'a> {
    pub(super) fn new(quantizer: IvfQuantizer) -> Self {
        Self {
            quantizer,
            pq_model: None,
        }
    }

    pub(super) fn with_pq_model(
        quantizer: IvfQuantizer,
        pq_model: &'a IvfPqFastScanModel,
    ) -> Result<Self, String> {
        quantizer.validate_pq_model(pq_model)?;
        Ok(Self {
            quantizer,
            pq_model: Some(pq_model),
        })
    }

    fn pq_model(self) -> Result<&'a IvfPqFastScanModel, String> {
        self.pq_model.ok_or_else(|| {
            "ec_ivf pq_fastscan codec requires persisted grouped codebooks".to_owned()
        })
    }
}

impl IvfQuantizer {
    pub(super) fn resolve(
        storage_format: StorageFormat,
        dimensions: usize,
    ) -> Result<Self, String> {
        Self::resolve_with_pq_group_size_and_bits(storage_format, dimensions, None, None)
    }

    pub(super) fn resolve_with_pq_group_size(
        storage_format: StorageFormat,
        dimensions: usize,
        pq_group_size: Option<usize>,
    ) -> Result<Self, String> {
        Self::resolve_with_pq_group_size_and_bits(storage_format, dimensions, pq_group_size, None)
    }

    pub(super) fn resolve_with_pq_group_size_and_bits(
        storage_format: StorageFormat,
        dimensions: usize,
        pq_group_size: Option<usize>,
        rabitq_bits: Option<u8>,
    ) -> Result<Self, String> {
        Self::resolve_with_pq_group_size_bits_and_residual(
            storage_format,
            dimensions,
            pq_group_size,
            rabitq_bits,
            false,
            None,
        )
    }

    /// Task 115: resolve a quantizer, optionally in RaBitQ residual mode.
    /// `rabitq_residual` is honored only for the RaBitQ profile; it is rejected
    /// for any other storage format so the gate cannot be set on a quantizer
    /// that has no centroid-correction scoring path.
    pub(super) fn resolve_with_pq_group_size_bits_and_residual(
        storage_format: StorageFormat,
        dimensions: usize,
        pq_group_size: Option<usize>,
        rabitq_bits: Option<u8>,
        rabitq_residual: bool,
        rabitq_quant_clip: Option<u8>,
    ) -> Result<Self, String> {
        storage_format.validate_v1_supported()?;
        let profile = match storage_format {
            StorageFormat::Auto | StorageFormat::TurboQuant => IvfQuantizerProfile::TurboQuant,
            StorageFormat::PqFastScan => {
                let transform_dim = rotation::effective_transform_dim(dimensions);
                let group_size = resolve_pq_fastscan_group_size(dimensions, pq_group_size)?;
                IvfQuantizerProfile::PqFastScan {
                    group_count: transform_dim / group_size,
                    group_size,
                }
            }
            StorageFormat::RaBitQ | StorageFormat::CoarseRerank => IvfQuantizerProfile::RaBitQ,
        };
        if rabitq_residual && !matches!(profile, IvfQuantizerProfile::RaBitQ) {
            return Err("ec_ivf rabitq_residual requires storage_format = 'rabitq'".to_owned());
        }
        let bits = match rabitq_bits.unwrap_or(crate::DEFAULT_QUANT_BITS) {
            b @ (1 | 2 | 4 | 8) => b,
            other => {
                return Err(format!(
                    "ec_ivf RaBitQ quant_bits must be one of 1, 2, 4, 8; got {other}"
                ))
            }
        };
        let quant_clip = rabitq_quant_clip.unwrap_or(EC_IVF_DEFAULT_RABITQ_RERANK_CLIP as u8);
        if quant_clip == 0 {
            return Err("ec_ivf RaBitQ quant_clip must be positive".to_owned());
        }
        Ok(Self {
            profile,
            dimensions,
            rabitq_bits: bits,
            rabitq_quant_clip: quant_clip,
            rabitq_residual,
        })
    }

    pub(super) fn encode_source(self, source: &[f32]) -> Result<(u16, f32, Vec<u8>), String> {
        if source.is_empty() {
            return Err("embedding must not be empty".to_owned());
        }
        if source.len() != self.dimensions {
            return Err(format!(
                "embedding dimension mismatch: got {}, expected {}",
                source.len(),
                self.dimensions
            ));
        }
        let dimensions = u16::try_from(source.len())
            .map_err(|_| format!("embedding dimension {} exceeds maximum 65535", source.len()))?;

        match self.profile {
            IvfQuantizerProfile::TurboQuant => {
                let quantizer = ProdQuantizer::cached(
                    self.dimensions,
                    crate::DEFAULT_QUANT_BITS,
                    crate::DEFAULT_QUANT_SEED,
                );
                let encoded = quantizer.encode(source);
                let mut payload = encoded.mse_packed;
                payload.extend_from_slice(&encoded.qjl_packed);
                Ok((dimensions, encoded.gamma, payload))
            }
            IvfQuantizerProfile::RaBitQ => {
                if self.rabitq_residual {
                    return Err(
                        "ec_ivf RaBitQ residual mode requires the assigned centroid; use encode_source_residual".to_owned(),
                    );
                }
                let quantizer = self.rabitq_quantizer()?;
                Ok((dimensions, 0.0, quantizer.encode_code(source).into_vec()))
            }
            IvfQuantizerProfile::PqFastScan { .. } => {
                Err("ec_ivf pq_fastscan encoding requires a trained grouped codebook".to_owned())
            }
        }
    }

    /// Task 115: encode `source` as the RaBitQ residual against its assigned
    /// `centroid`. RaBitQ residual mode only. The returned payload has the
    /// identical length/layout to the plain RaBitQ code; only the encoded vector
    /// (`source − centroid`) differs. `gamma` is 0.0 (RaBitQ does not use it).
    pub(super) fn encode_source_residual(
        self,
        source: &[f32],
        centroid: &[f32],
    ) -> Result<(u16, f32, Vec<u8>), String> {
        if !self.rabitq_residual || !matches!(self.profile, IvfQuantizerProfile::RaBitQ) {
            return Err(
                "ec_ivf encode_source_residual requires a RaBitQ residual-mode quantizer"
                    .to_owned(),
            );
        }
        if source.is_empty() {
            return Err("embedding must not be empty".to_owned());
        }
        if source.len() != self.dimensions {
            return Err(format!(
                "embedding dimension mismatch: got {}, expected {}",
                source.len(),
                self.dimensions
            ));
        }
        if centroid.len() != self.dimensions {
            return Err(format!(
                "centroid dimension mismatch: got {}, expected {}",
                centroid.len(),
                self.dimensions
            ));
        }
        let dimensions = u16::try_from(source.len())
            .map_err(|_| format!("embedding dimension {} exceeds maximum 65535", source.len()))?;
        let quantizer = self.rabitq_quantizer()?;
        Ok((
            dimensions,
            0.0,
            quantizer.encode_code_residual(source, centroid).into_vec(),
        ))
    }

    pub(super) fn encode_source_with_pq_model(
        self,
        source: &[f32],
        model: &IvfPqFastScanModel,
    ) -> Result<(u16, f32, Vec<u8>), String> {
        if source.is_empty() {
            return Err("embedding must not be empty".to_owned());
        }
        if source.len() != self.dimensions {
            return Err(format!(
                "embedding dimension mismatch: got {}, expected {}",
                source.len(),
                self.dimensions
            ));
        }
        self.validate_pq_model(model)?;
        let dimensions = u16::try_from(source.len())
            .map_err(|_| format!("embedding dimension {} exceeds maximum 65535", source.len()))?;
        let rotated = rotation::srht_padded(source, &model.signs);
        let codebook_iter = model
            .flat_codebooks
            .chunks_exact(model.group_size * GROUPED_PQ_CENTROIDS);
        let payload =
            crate::quant::grouped_pq::encode_grouped_pq(&rotated, codebook_iter, model.group_size);
        Ok((dimensions, 0.0, payload))
    }

    pub(super) fn encode_source_with_tq_calibration_model(
        self,
        source: &[f32],
        model: &IvfTqCalibrationModel,
    ) -> Result<(u16, f32, Vec<u8>), String> {
        if source.is_empty() {
            return Err("embedding must not be empty".to_owned());
        }
        if source.len() != self.dimensions {
            return Err(format!(
                "embedding dimension mismatch: got {}, expected {}",
                source.len(),
                self.dimensions
            ));
        }
        if !matches!(self.profile, IvfQuantizerProfile::TurboQuant) {
            return Err(
                "ec_ivf TurboQuant calibration encoding requires a TurboQuant quantizer".to_owned(),
            );
        }
        self.validate_tq_calibration_model(model)?;
        let dimensions = u16::try_from(source.len())
            .map_err(|_| format!("embedding dimension {} exceeds maximum 65535", source.len()))?;
        let quantizer = ProdQuantizer::cached(
            self.dimensions,
            crate::DEFAULT_QUANT_BITS,
            crate::DEFAULT_QUANT_SEED,
        );
        let encoded = quantizer.encode_calibrated_no_qjl_4bit(source, &model.calibration);
        Ok((dimensions, 0.0, encoded.mse_packed))
    }

    pub(super) fn prepare_ip_query(self, query: &[f32]) -> Result<IvfPreparedQuery, String> {
        self.prepare_ip_query_with_turboquant_scorer(
            query,
            super::options::current_session_turboquant_scorer(),
        )
    }

    pub(super) fn prepare_ip_query_with_turboquant_scorer(
        self,
        query: &[f32],
        turboquant_scorer: TurboQuantScorerGuc,
    ) -> Result<IvfPreparedQuery, String> {
        if query.len() != self.dimensions {
            return Err(format!(
                "query dimension mismatch: got {}, expected {}",
                query.len(),
                self.dimensions
            ));
        }
        match self.profile {
            IvfQuantizerProfile::TurboQuant => {
                let quantizer = ProdQuantizer::cached(
                    self.dimensions,
                    crate::DEFAULT_QUANT_BITS,
                    crate::DEFAULT_QUANT_SEED,
                );
                match quantizer.exact_score_mode() {
                    ExactScoreMode::MseNoQjl4Bit => {
                        return Ok(match turboquant_scorer {
                            TurboQuantScorerGuc::Lut => IvfPreparedQuery::TurboQuantNoQjl4BitLut(
                                quantizer.prepare_ip_query_lut_no_qjl_4bit(query),
                            ),
                            TurboQuantScorerGuc::Int8Approx => {
                                IvfPreparedQuery::TurboQuantNoQjl4BitInt8Approx(
                                    quantizer.prepare_ip_query_int8_approx_no_qjl_4bit(query),
                                )
                            }
                        });
                    }
                    ExactScoreMode::MseLutQjl
                    | ExactScoreMode::MseLutOnly
                    | ExactScoreMode::MseQjlOnly
                    | ExactScoreMode::MseScalarOnly => {}
                }
                Ok(IvfPreparedQuery::TurboQuant(
                    quantizer.prepare_ip_query(query),
                ))
            }
            IvfQuantizerProfile::RaBitQ => {
                let quantizer = self.rabitq_quantizer()?;
                Ok(IvfPreparedQuery::RaBitQ(quantizer.prepare_estimator(query)))
            }
            IvfQuantizerProfile::PqFastScan { .. } => {
                Err("ec_ivf pq_fastscan query prep requires persisted grouped codebooks".to_owned())
            }
        }
    }

    pub(super) fn prepare_ip_query_with_pq_model(
        self,
        query: &[f32],
        model: &IvfPqFastScanModel,
    ) -> Result<IvfPreparedQuery, String> {
        if query.len() != self.dimensions {
            return Err(format!(
                "query dimension mismatch: got {}, expected {}",
                query.len(),
                self.dimensions
            ));
        }
        self.validate_pq_model(model)?;
        let prod = ProdQuantizer::cached(
            self.dimensions,
            crate::DEFAULT_QUANT_BITS,
            crate::DEFAULT_QUANT_SEED,
        );
        let rotated = rotation::srht_padded(query, &prod.signs);
        let transform_dim = model.group_count * model.group_size;
        let lut = build_grouped_pq_lut_f32(
            &rotated[..transform_dim],
            &model.flat_codebooks,
            model.group_size,
        );
        let suffix_max = grouped_pq_suffix_max(&lut, model.group_count);
        Ok(IvfPreparedQuery::PqFastScan {
            lut,
            group_count: model.group_count,
            suffix_max,
        })
    }

    pub(super) fn prepare_ip_query_with_tq_calibration_model(
        self,
        query: &[f32],
        model: &IvfTqCalibrationModel,
    ) -> Result<IvfPreparedQuery, String> {
        if query.len() != self.dimensions {
            return Err(format!(
                "query dimension mismatch: got {}, expected {}",
                query.len(),
                self.dimensions
            ));
        }
        if !matches!(self.profile, IvfQuantizerProfile::TurboQuant) {
            return Err(
                "ec_ivf TurboQuant calibration query prep requires a TurboQuant quantizer"
                    .to_owned(),
            );
        }
        self.validate_tq_calibration_model(model)?;
        let quantizer = ProdQuantizer::cached(
            self.dimensions,
            crate::DEFAULT_QUANT_BITS,
            crate::DEFAULT_QUANT_SEED,
        );
        Ok(IvfPreparedQuery::TurboQuantCalibratedNoQjl4Bit(
            quantizer.prepare_ip_query_calibrated_no_qjl_4bit(query, &model.calibration),
        ))
    }

    pub(super) fn score_ip_from_parts(
        self,
        prepared_query: &IvfPreparedQuery,
        gamma: f32,
        payload: &[u8],
    ) -> Result<f32, String> {
        match (self.profile, prepared_query) {
            (IvfQuantizerProfile::TurboQuant, IvfPreparedQuery::TurboQuant(prepared_query)) => {
                let quantizer = ProdQuantizer::cached(
                    self.dimensions,
                    crate::DEFAULT_QUANT_BITS,
                    crate::DEFAULT_QUANT_SEED,
                );
                Ok(quantizer.score_ip_from_parts(prepared_query, gamma, payload))
            }
            (
                IvfQuantizerProfile::TurboQuant,
                IvfPreparedQuery::TurboQuantNoQjl4BitLut(prepared_query),
            ) => {
                let quantizer = ProdQuantizer::cached(
                    self.dimensions,
                    crate::DEFAULT_QUANT_BITS,
                    crate::DEFAULT_QUANT_SEED,
                );
                Ok(quantizer.score_ip_from_parts_lut_no_qjl_4bit(prepared_query, payload))
            }
            (
                IvfQuantizerProfile::TurboQuant,
                IvfPreparedQuery::TurboQuantNoQjl4BitInt8Approx(prepared_query),
            ) => {
                let quantizer = ProdQuantizer::cached(
                    self.dimensions,
                    crate::DEFAULT_QUANT_BITS,
                    crate::DEFAULT_QUANT_SEED,
                );
                Ok(quantizer.score_ip_from_parts_int8_approx_no_qjl_4bit(prepared_query, payload))
            }
            (
                IvfQuantizerProfile::TurboQuant,
                IvfPreparedQuery::TurboQuantCalibratedNoQjl4Bit(prepared_query),
            ) => {
                let _ = gamma;
                let quantizer = ProdQuantizer::cached(
                    self.dimensions,
                    crate::DEFAULT_QUANT_BITS,
                    crate::DEFAULT_QUANT_SEED,
                );
                Ok(quantizer.score_calibrated_no_qjl_4bit(prepared_query, payload))
            }
            (IvfQuantizerProfile::RaBitQ, IvfPreparedQuery::RaBitQ(prepared_query)) => {
                let _ = gamma;
                Ok(prepared_query.estimate_ip_scalar_only(payload))
            }
            (
                IvfQuantizerProfile::PqFastScan { group_count, .. },
                IvfPreparedQuery::PqFastScan {
                    lut,
                    group_count: prepared_group_count,
                    ..
                },
            ) => {
                let _ = gamma;
                if group_count != *prepared_group_count {
                    return Err("ec_ivf pq_fastscan prepared query group count mismatch".to_owned());
                }
                Ok(grouped_pq_score_f32(lut, group_count, payload))
            }
            (IvfQuantizerProfile::TurboQuant, IvfPreparedQuery::RaBitQ(_))
            | (IvfQuantizerProfile::RaBitQ, IvfPreparedQuery::TurboQuant(_))
            | (IvfQuantizerProfile::RaBitQ, IvfPreparedQuery::TurboQuantNoQjl4BitLut(_))
            | (IvfQuantizerProfile::RaBitQ, IvfPreparedQuery::TurboQuantNoQjl4BitInt8Approx(_))
            | (IvfQuantizerProfile::RaBitQ, IvfPreparedQuery::TurboQuantCalibratedNoQjl4Bit(_))
            | (IvfQuantizerProfile::TurboQuant, IvfPreparedQuery::PqFastScan { .. })
            | (IvfQuantizerProfile::RaBitQ, IvfPreparedQuery::PqFastScan { .. })
            | (IvfQuantizerProfile::PqFastScan { .. }, IvfPreparedQuery::TurboQuant(_))
            | (
                IvfQuantizerProfile::PqFastScan { .. },
                IvfPreparedQuery::TurboQuantNoQjl4BitLut(_),
            )
            | (
                IvfQuantizerProfile::PqFastScan { .. },
                IvfPreparedQuery::TurboQuantNoQjl4BitInt8Approx(_),
            )
            | (
                IvfQuantizerProfile::PqFastScan { .. },
                IvfPreparedQuery::TurboQuantCalibratedNoQjl4Bit(_),
            )
            | (IvfQuantizerProfile::PqFastScan { .. }, IvfPreparedQuery::RaBitQ(_)) => {
                Err("ec_ivf prepared query does not match quantizer profile".to_owned())
            }
        }
    }

    pub(super) fn score_ip_dequantized_from_parts(
        self,
        query: &[f32],
        payload: &[u8],
    ) -> Result<f32, String> {
        match self.profile {
            IvfQuantizerProfile::TurboQuant => {
                if query.len() != self.dimensions {
                    return Err(format!(
                        "query dimension mismatch: got {}, expected {}",
                        query.len(),
                        self.dimensions
                    ));
                }
                let quantizer = ProdQuantizer::cached(
                    self.dimensions,
                    crate::DEFAULT_QUANT_BITS,
                    crate::DEFAULT_QUANT_SEED,
                );
                let decoded = quantizer.decode_approximate_from_code(payload);
                Ok(query
                    .iter()
                    .zip(decoded.iter())
                    .map(|(query_i, decoded_i)| query_i * decoded_i)
                    .sum())
            }
            IvfQuantizerProfile::RaBitQ => Err(
                "ec_ivf RaBitQ dequantized scoring is implemented on PreparedEstimator".to_owned(),
            ),
            IvfQuantizerProfile::PqFastScan { .. } => {
                Err("ec_ivf pq_fastscan dequantized scoring is not supported".to_owned())
            }
        }
    }

    pub(super) fn score_ip_from_parts_with_min_bound(
        self,
        prepared_query: &IvfPreparedQuery,
        gamma: f32,
        payload: &[u8],
        min_ip_to_keep: Option<f32>,
    ) -> Result<Option<f32>, String> {
        match (self.profile, prepared_query, min_ip_to_keep) {
            (
                IvfQuantizerProfile::PqFastScan { group_count, .. },
                IvfPreparedQuery::PqFastScan {
                    lut,
                    group_count: prepared_group_count,
                    suffix_max,
                },
                Some(min_ip_to_keep),
            ) => {
                let _ = gamma;
                if group_count != *prepared_group_count {
                    return Err("ec_ivf pq_fastscan prepared query group count mismatch".to_owned());
                }
                Ok(grouped_pq_score_f32_with_min_bound(
                    lut,
                    suffix_max,
                    group_count,
                    payload,
                    min_ip_to_keep,
                ))
            }
            (IvfQuantizerProfile::RaBitQ, IvfPreparedQuery::RaBitQ(prepared), Some(min_ip)) => {
                let _ = gamma;
                Ok(prepared.try_estimate_ip_scalar(payload, min_ip))
            }
            (
                IvfQuantizerProfile::TurboQuant,
                IvfPreparedQuery::TurboQuantNoQjl4BitLut(prepared),
                Some(min_ip),
            ) => {
                let _ = gamma;
                let quantizer = ProdQuantizer::cached(
                    self.dimensions,
                    crate::DEFAULT_QUANT_BITS,
                    crate::DEFAULT_QUANT_SEED,
                );
                Ok(quantizer
                    .score_ip_from_parts_lut_no_qjl_4bit_with_min_bound(prepared, payload, min_ip))
            }
            _ => self
                .score_ip_from_parts(prepared_query, gamma, payload)
                .map(Some),
        }
    }

    pub(super) fn score_ip_bits1_batch_from_payloads(
        self,
        prepared_query: &IvfPreparedQuery,
        payloads: &[u8],
        payload_len: usize,
        out_scores: &mut Vec<f32>,
    ) -> Result<bool, String> {
        match (self.profile, prepared_query) {
            (IvfQuantizerProfile::RaBitQ, IvfPreparedQuery::RaBitQ(prepared_query))
                if self.rabitq_bits == 1 =>
            {
                if payload_len == 0 || payloads.len() % payload_len != 0 {
                    return Err(format!(
                        "ec_ivf RaBitQ bits=1 batch payload slab length {} is not divisible by code length {payload_len}",
                        payloads.len()
                    ));
                }
                let prepared = prepared_query
                    .bits1_block_prepared(payload_len)
                    .ok_or_else(|| {
                        "ec_ivf RaBitQ bits=1 prepared query missing block state".to_owned()
                    })?;
                let mut batch = CandidateBatch::with_capacity(payloads.len() / payload_len);
                for (index, payload) in payloads.chunks_exact(payload_len).enumerate() {
                    batch.push(index, CandidatePayload::new(payload, CandidateMeta::RaBitQ))?;
                }
                out_scores.clear();
                out_scores.resize(batch.len(), 0.0);
                score_rabitq_bits1_batch_for(
                    CandidateBatchScoringSurface::Ivf,
                    prepared,
                    &batch,
                    out_scores,
                )?;
                Ok(true)
            }
            (IvfQuantizerProfile::RaBitQ, IvfPreparedQuery::RaBitQ(prepared_query))
                if self.rabitq_bits == 2 =>
            {
                // bits=2 has no per-candidate SIMD kernel (estimate_ip_batch
                // supports only 1/4/8), so it would otherwise fall to true
                // scalar. The multi-bit block kernel is a measured win here:
                // M5 NEON 30.7µs→11.5µs (2.66×) for a 32-block at dim=1024.
                if payload_len == 0 || payloads.len() % payload_len != 0 {
                    return Err(format!(
                        "ec_ivf RaBitQ bits=2 batch payload slab length {} is not divisible by code length {payload_len}",
                        payloads.len()
                    ));
                }
                let prepared = prepared_query
                    .bitsn_block_prepared(payload_len)
                    .ok_or_else(|| {
                        "ec_ivf RaBitQ multi-bit prepared query missing block state".to_owned()
                    })?;
                let mut batch = CandidateBatch::with_capacity(payloads.len() / payload_len);
                for (index, payload) in payloads.chunks_exact(payload_len).enumerate() {
                    batch.push(index, CandidatePayload::new(payload, CandidateMeta::RaBitQ))?;
                }
                out_scores.clear();
                out_scores.resize(batch.len(), 0.0);
                score_rabitq_bitsn_batch_for(
                    CandidateBatchScoringSurface::Ivf,
                    prepared,
                    &batch,
                    out_scores,
                )?;
                Ok(true)
            }
            (IvfQuantizerProfile::RaBitQ, IvfPreparedQuery::RaBitQ(prepared_query))
                if self.rabitq_bits == 4 || self.rabitq_bits == 8 =>
            {
                // bits=4 and bits=8 keep the arithmetic batch estimator, which
                // dispatches to the per-candidate SIMD kernels (NeonBits4/8,
                // Avx2Bits4/8). The multi-bit *block* kernel measured SLOWER
                // than NeonBits4 on M5 (bits=4: 12.9µs vs 4.6µs for a 32-block
                // at dim=1024), so the block kernel is not used here — an
                // evidence-driven routing choice (Task 106 M5 bench). The AVX2
                // hardware-gather block path may revisit bits=4 on the Intel
                // lane. bits=8 is a full-byte level with no LUT fast-scan shape.
                // No block kernel runs here, but the IVF scan still needs the
                // wrapper-level flush-width histogram for Task 111a evidence.
                if payload_len == 0 || payloads.len() % payload_len != 0 {
                    return Err(format!(
                        "ec_ivf RaBitQ bits={} batch payload slab length {} is not divisible by code length {payload_len}",
                        self.rabitq_bits,
                        payloads.len()
                    ));
                }
                prepared_query.estimate_ip_batch(payloads, payload_len, out_scores)?;
                record_ivf_rabitq_arithmetic_batch_flush_width(payloads.len() / payload_len);
                Ok(true)
            }
            (IvfQuantizerProfile::RaBitQ, IvfPreparedQuery::RaBitQ(_))
            | (IvfQuantizerProfile::TurboQuant, IvfPreparedQuery::TurboQuant(_))
            | (IvfQuantizerProfile::TurboQuant, IvfPreparedQuery::TurboQuantNoQjl4BitLut(_))
            | (
                IvfQuantizerProfile::TurboQuant,
                IvfPreparedQuery::TurboQuantNoQjl4BitInt8Approx(_),
            )
            | (
                IvfQuantizerProfile::TurboQuant,
                IvfPreparedQuery::TurboQuantCalibratedNoQjl4Bit(_),
            )
            | (IvfQuantizerProfile::PqFastScan { .. }, IvfPreparedQuery::PqFastScan { .. }) => {
                Ok(false)
            }
            (IvfQuantizerProfile::TurboQuant, IvfPreparedQuery::RaBitQ(_))
            | (IvfQuantizerProfile::RaBitQ, IvfPreparedQuery::TurboQuant(_))
            | (IvfQuantizerProfile::RaBitQ, IvfPreparedQuery::TurboQuantNoQjl4BitLut(_))
            | (IvfQuantizerProfile::RaBitQ, IvfPreparedQuery::TurboQuantNoQjl4BitInt8Approx(_))
            | (IvfQuantizerProfile::RaBitQ, IvfPreparedQuery::TurboQuantCalibratedNoQjl4Bit(_))
            | (IvfQuantizerProfile::TurboQuant, IvfPreparedQuery::PqFastScan { .. })
            | (IvfQuantizerProfile::RaBitQ, IvfPreparedQuery::PqFastScan { .. })
            | (IvfQuantizerProfile::PqFastScan { .. }, IvfPreparedQuery::TurboQuant(_))
            | (
                IvfQuantizerProfile::PqFastScan { .. },
                IvfPreparedQuery::TurboQuantNoQjl4BitLut(_),
            )
            | (
                IvfQuantizerProfile::PqFastScan { .. },
                IvfPreparedQuery::TurboQuantNoQjl4BitInt8Approx(_),
            )
            | (
                IvfQuantizerProfile::PqFastScan { .. },
                IvfPreparedQuery::TurboQuantCalibratedNoQjl4Bit(_),
            )
            | (IvfQuantizerProfile::PqFastScan { .. }, IvfPreparedQuery::RaBitQ(_)) => {
                Err("ec_ivf prepared query does not match quantizer profile".to_owned())
            }
        }
    }

    pub(super) fn score_turboquant_batch_from_payloads(
        self,
        prepared_query: &IvfPreparedQuery,
        payloads: &[u8],
        payload_len: usize,
        gammas: &[f32],
        out_scores: &mut Vec<f32>,
    ) -> Result<bool, String> {
        let candidate_count = if payload_len == 0 {
            0
        } else {
            payloads.len() / payload_len
        };
        out_scores.clear();
        out_scores.resize(candidate_count, 0.0);
        self.score_turboquant_batch_from_payloads_into(
            prepared_query,
            payloads,
            payload_len,
            gammas,
            out_scores,
            false,
        )
    }

    pub(super) fn score_turboquant_batch_from_payloads_negated_into(
        self,
        prepared_query: &IvfPreparedQuery,
        payloads: &[u8],
        payload_len: usize,
        gammas: &[f32],
        out_scores: &mut [f32],
    ) -> Result<bool, String> {
        self.score_turboquant_batch_from_payloads_into(
            prepared_query,
            payloads,
            payload_len,
            gammas,
            out_scores,
            true,
        )
    }

    fn score_turboquant_batch_from_payloads_into(
        self,
        prepared_query: &IvfPreparedQuery,
        payloads: &[u8],
        payload_len: usize,
        gammas: &[f32],
        out_scores: &mut [f32],
        negate: bool,
    ) -> Result<bool, String> {
        match (self.profile, prepared_query) {
            (
                IvfQuantizerProfile::TurboQuant,
                IvfPreparedQuery::TurboQuantNoQjl4BitLut(prepared_query),
            ) => {
                if payload_len == 0 {
                    return Err("ec_ivf TurboQuant batch payload length must be nonzero".to_owned());
                }
                if payloads.len() != payload_len * out_scores.len() {
                    return Err(format!(
                        "ec_ivf TurboQuant batch payload length mismatch: got {} bytes for {} postings with {} byte payloads",
                        payloads.len(),
                        out_scores.len(),
                        payload_len
                    ));
                }

                let quantizer = ProdQuantizer::cached(
                    self.dimensions,
                    crate::DEFAULT_QUANT_BITS,
                    crate::DEFAULT_QUANT_SEED,
                );
                let mut batch = CandidateBatch::with_capacity(out_scores.len());
                for (index, payload) in payloads.chunks_exact(payload_len).enumerate() {
                    batch.push(
                        index,
                        CandidatePayload {
                            code: payload,
                            meta: CandidateMeta::None,
                        },
                    )?;
                }
                score_turboquant_no_qjl_4bit_batch_for(
                    CandidateBatchScoringSurface::Ivf,
                    quantizer.as_ref(),
                    prepared_query,
                    &batch,
                    out_scores,
                )?;
                if negate {
                    for score in out_scores {
                        *score = -*score;
                    }
                }
                Ok(true)
            }
            (
                IvfQuantizerProfile::TurboQuant,
                IvfPreparedQuery::TurboQuantNoQjl4BitInt8Approx(prepared_query),
            ) => {
                if payload_len == 0 {
                    return Err("ec_ivf TurboQuant batch payload length must be nonzero".to_owned());
                }
                if payloads.len() != payload_len * out_scores.len() {
                    return Err(format!(
                        "ec_ivf TurboQuant batch payload length mismatch: got {} bytes for {} postings with {} byte payloads",
                        payloads.len(),
                        out_scores.len(),
                        payload_len
                    ));
                }

                let quantizer = ProdQuantizer::cached(
                    self.dimensions,
                    crate::DEFAULT_QUANT_BITS,
                    crate::DEFAULT_QUANT_SEED,
                );
                let mut batch = CandidateBatch::with_capacity(out_scores.len());
                for (index, payload) in payloads.chunks_exact(payload_len).enumerate() {
                    batch.push(
                        index,
                        CandidatePayload {
                            code: payload,
                            meta: CandidateMeta::None,
                        },
                    )?;
                }
                score_turboquant_int8_approx_batch_for(
                    CandidateBatchScoringSurface::Ivf,
                    quantizer.as_ref(),
                    prepared_query,
                    &batch,
                    out_scores,
                )?;
                if negate {
                    for score in out_scores {
                        *score = -*score;
                    }
                }
                Ok(true)
            }
            (IvfQuantizerProfile::TurboQuant, IvfPreparedQuery::TurboQuant(prepared_query)) => {
                if payload_len == 0 {
                    return Err(
                        "ec_ivf TurboQuant QJL batch payload length must be nonzero".to_owned()
                    );
                }
                if payloads.len() != payload_len * gammas.len() || gammas.len() != out_scores.len()
                {
                    return Err(format!(
                        "ec_ivf TurboQuant QJL batch payload length mismatch: got {} bytes for {} postings with {} byte payloads",
                        payloads.len(),
                        out_scores.len(),
                        payload_len
                    ));
                }

                let quantizer = ProdQuantizer::cached(
                    self.dimensions,
                    crate::DEFAULT_QUANT_BITS,
                    crate::DEFAULT_QUANT_SEED,
                );
                let mut batch = CandidateBatch::with_capacity(out_scores.len());
                for (index, (payload, gamma)) in payloads
                    .chunks_exact(payload_len)
                    .zip(gammas.iter().copied())
                    .enumerate()
                {
                    batch.push(
                        index,
                        CandidatePayload {
                            code: payload,
                            meta: CandidateMeta::Gamma(gamma),
                        },
                    )?;
                }
                score_turboquant_qjl_batch_for(
                    CandidateBatchScoringSurface::Ivf,
                    quantizer.as_ref(),
                    prepared_query,
                    &batch,
                    out_scores,
                )?;
                if negate {
                    for score in out_scores {
                        *score = -*score;
                    }
                }
                Ok(true)
            }
            (IvfQuantizerProfile::RaBitQ, IvfPreparedQuery::RaBitQ(_))
            | (
                IvfQuantizerProfile::TurboQuant,
                IvfPreparedQuery::TurboQuantCalibratedNoQjl4Bit(_),
            )
            | (IvfQuantizerProfile::PqFastScan { .. }, IvfPreparedQuery::PqFastScan { .. }) => {
                Ok(false)
            }
            (IvfQuantizerProfile::TurboQuant, IvfPreparedQuery::RaBitQ(_))
            | (IvfQuantizerProfile::RaBitQ, IvfPreparedQuery::TurboQuant(_))
            | (IvfQuantizerProfile::RaBitQ, IvfPreparedQuery::TurboQuantNoQjl4BitLut(_))
            | (IvfQuantizerProfile::RaBitQ, IvfPreparedQuery::TurboQuantNoQjl4BitInt8Approx(_))
            | (IvfQuantizerProfile::RaBitQ, IvfPreparedQuery::TurboQuantCalibratedNoQjl4Bit(_))
            | (IvfQuantizerProfile::TurboQuant, IvfPreparedQuery::PqFastScan { .. })
            | (IvfQuantizerProfile::RaBitQ, IvfPreparedQuery::PqFastScan { .. })
            | (IvfQuantizerProfile::PqFastScan { .. }, IvfPreparedQuery::TurboQuant(_))
            | (
                IvfQuantizerProfile::PqFastScan { .. },
                IvfPreparedQuery::TurboQuantNoQjl4BitLut(_),
            )
            | (
                IvfQuantizerProfile::PqFastScan { .. },
                IvfPreparedQuery::TurboQuantNoQjl4BitInt8Approx(_),
            )
            | (
                IvfQuantizerProfile::PqFastScan { .. },
                IvfPreparedQuery::TurboQuantCalibratedNoQjl4Bit(_),
            )
            | (IvfQuantizerProfile::PqFastScan { .. }, IvfPreparedQuery::RaBitQ(_)) => {
                Err("ec_ivf prepared query does not match quantizer profile".to_owned())
            }
        }
    }

    pub(super) fn supports_turboquant_payload_ref_batch(
        self,
        prepared_query: &IvfPreparedQuery,
    ) -> bool {
        matches!(
            (self.profile, prepared_query),
            (
                IvfQuantizerProfile::TurboQuant,
                IvfPreparedQuery::TurboQuantNoQjl4BitLut(_)
            ) | (
                IvfQuantizerProfile::TurboQuant,
                IvfPreparedQuery::TurboQuantNoQjl4BitInt8Approx(_)
            ) | (
                IvfQuantizerProfile::TurboQuant,
                IvfPreparedQuery::TurboQuant(_)
            )
        )
    }

    pub(super) fn score_turboquant_batch_from_payload_refs(
        self,
        prepared_query: &IvfPreparedQuery,
        payloads: &[&[u8]],
        payload_len: usize,
        gammas: &[f32],
        out_scores: &mut Vec<f32>,
    ) -> Result<bool, String> {
        out_scores.clear();
        out_scores.resize(payloads.len(), 0.0);
        self.score_turboquant_batch_from_payload_refs_into(
            prepared_query,
            payloads,
            payload_len,
            gammas,
            out_scores,
            false,
        )
    }

    pub(super) fn score_turboquant_batch_from_payload_refs_negated_into(
        self,
        prepared_query: &IvfPreparedQuery,
        payloads: &[&[u8]],
        payload_len: usize,
        gammas: &[f32],
        out_scores: &mut [f32],
    ) -> Result<bool, String> {
        self.score_turboquant_batch_from_payload_refs_into(
            prepared_query,
            payloads,
            payload_len,
            gammas,
            out_scores,
            true,
        )
    }

    fn score_turboquant_batch_from_payload_refs_into(
        self,
        prepared_query: &IvfPreparedQuery,
        payloads: &[&[u8]],
        payload_len: usize,
        gammas: &[f32],
        out_scores: &mut [f32],
        negate: bool,
    ) -> Result<bool, String> {
        match (self.profile, prepared_query) {
            (
                IvfQuantizerProfile::TurboQuant,
                IvfPreparedQuery::TurboQuantNoQjl4BitLut(prepared_query),
            ) => {
                if payload_len == 0 {
                    return Err("ec_ivf TurboQuant batch payload length must be nonzero".to_owned());
                }
                if payloads.len() != out_scores.len() {
                    return Err(format!(
                        "ec_ivf TurboQuant borrowed batch payload count {} does not match score count {}",
                        payloads.len(),
                        out_scores.len()
                    ));
                }
                if let Some((index, payload)) = payloads
                    .iter()
                    .enumerate()
                    .find(|(_, payload)| payload.len() != payload_len)
                {
                    return Err(format!(
                        "ec_ivf TurboQuant borrowed batch payload {index} has {} bytes, expected {payload_len}",
                        payload.len()
                    ));
                }

                let quantizer = ProdQuantizer::cached(
                    self.dimensions,
                    crate::DEFAULT_QUANT_BITS,
                    crate::DEFAULT_QUANT_SEED,
                );
                let mut batch = CandidateBatch::with_capacity(out_scores.len());
                for (index, payload) in payloads.iter().copied().enumerate() {
                    batch.push(
                        index,
                        CandidatePayload {
                            code: payload,
                            meta: CandidateMeta::None,
                        },
                    )?;
                }
                score_turboquant_no_qjl_4bit_batch_for(
                    CandidateBatchScoringSurface::Ivf,
                    quantizer.as_ref(),
                    prepared_query,
                    &batch,
                    out_scores,
                )?;
                if negate {
                    for score in out_scores {
                        *score = -*score;
                    }
                }
                Ok(true)
            }
            (
                IvfQuantizerProfile::TurboQuant,
                IvfPreparedQuery::TurboQuantNoQjl4BitInt8Approx(prepared_query),
            ) => {
                if payload_len == 0 {
                    return Err("ec_ivf TurboQuant batch payload length must be nonzero".to_owned());
                }
                if payloads.len() != out_scores.len() {
                    return Err(format!(
                        "ec_ivf TurboQuant borrowed batch payload count {} does not match score count {}",
                        payloads.len(),
                        out_scores.len()
                    ));
                }
                if let Some((index, payload)) = payloads
                    .iter()
                    .enumerate()
                    .find(|(_, payload)| payload.len() != payload_len)
                {
                    return Err(format!(
                        "ec_ivf TurboQuant borrowed batch payload {index} has {} bytes, expected {payload_len}",
                        payload.len()
                    ));
                }

                let quantizer = ProdQuantizer::cached(
                    self.dimensions,
                    crate::DEFAULT_QUANT_BITS,
                    crate::DEFAULT_QUANT_SEED,
                );
                let mut batch = CandidateBatch::with_capacity(out_scores.len());
                for (index, payload) in payloads.iter().copied().enumerate() {
                    batch.push(
                        index,
                        CandidatePayload {
                            code: payload,
                            meta: CandidateMeta::None,
                        },
                    )?;
                }
                score_turboquant_int8_approx_batch_for(
                    CandidateBatchScoringSurface::Ivf,
                    quantizer.as_ref(),
                    prepared_query,
                    &batch,
                    out_scores,
                )?;
                if negate {
                    for score in out_scores {
                        *score = -*score;
                    }
                }
                Ok(true)
            }
            (IvfQuantizerProfile::TurboQuant, IvfPreparedQuery::TurboQuant(prepared_query)) => {
                if payload_len == 0 {
                    return Err(
                        "ec_ivf TurboQuant QJL batch payload length must be nonzero".to_owned()
                    );
                }
                if payloads.len() != gammas.len() || gammas.len() != out_scores.len() {
                    return Err(format!(
                        "ec_ivf TurboQuant QJL borrowed batch payload count {} does not match gamma count {}",
                        payloads.len(),
                        out_scores.len()
                    ));
                }
                if let Some((index, payload)) = payloads
                    .iter()
                    .enumerate()
                    .find(|(_, payload)| payload.len() != payload_len)
                {
                    return Err(format!(
                        "ec_ivf TurboQuant QJL borrowed batch payload {index} has {} bytes, expected {payload_len}",
                        payload.len()
                    ));
                }

                let quantizer = ProdQuantizer::cached(
                    self.dimensions,
                    crate::DEFAULT_QUANT_BITS,
                    crate::DEFAULT_QUANT_SEED,
                );
                let mut batch = CandidateBatch::with_capacity(out_scores.len());
                for (index, (payload, gamma)) in payloads
                    .iter()
                    .copied()
                    .zip(gammas.iter().copied())
                    .enumerate()
                {
                    batch.push(
                        index,
                        CandidatePayload {
                            code: payload,
                            meta: CandidateMeta::Gamma(gamma),
                        },
                    )?;
                }
                score_turboquant_qjl_batch_for(
                    CandidateBatchScoringSurface::Ivf,
                    quantizer.as_ref(),
                    prepared_query,
                    &batch,
                    out_scores,
                )?;
                if negate {
                    for score in out_scores {
                        *score = -*score;
                    }
                }
                Ok(true)
            }
            (IvfQuantizerProfile::RaBitQ, IvfPreparedQuery::RaBitQ(_))
            | (
                IvfQuantizerProfile::TurboQuant,
                IvfPreparedQuery::TurboQuantCalibratedNoQjl4Bit(_),
            )
            | (IvfQuantizerProfile::PqFastScan { .. }, IvfPreparedQuery::PqFastScan { .. }) => {
                Ok(false)
            }
            (IvfQuantizerProfile::TurboQuant, IvfPreparedQuery::RaBitQ(_))
            | (IvfQuantizerProfile::RaBitQ, IvfPreparedQuery::TurboQuant(_))
            | (IvfQuantizerProfile::RaBitQ, IvfPreparedQuery::TurboQuantNoQjl4BitLut(_))
            | (IvfQuantizerProfile::RaBitQ, IvfPreparedQuery::TurboQuantNoQjl4BitInt8Approx(_))
            | (IvfQuantizerProfile::RaBitQ, IvfPreparedQuery::TurboQuantCalibratedNoQjl4Bit(_))
            | (IvfQuantizerProfile::TurboQuant, IvfPreparedQuery::PqFastScan { .. })
            | (IvfQuantizerProfile::RaBitQ, IvfPreparedQuery::PqFastScan { .. })
            | (IvfQuantizerProfile::PqFastScan { .. }, IvfPreparedQuery::TurboQuant(_))
            | (
                IvfQuantizerProfile::PqFastScan { .. },
                IvfPreparedQuery::TurboQuantNoQjl4BitLut(_),
            )
            | (
                IvfQuantizerProfile::PqFastScan { .. },
                IvfPreparedQuery::TurboQuantNoQjl4BitInt8Approx(_),
            )
            | (
                IvfQuantizerProfile::PqFastScan { .. },
                IvfPreparedQuery::TurboQuantCalibratedNoQjl4Bit(_),
            )
            | (IvfQuantizerProfile::PqFastScan { .. }, IvfPreparedQuery::RaBitQ(_)) => {
                Err("ec_ivf prepared query does not match quantizer profile".to_owned())
            }
        }
    }

    pub(super) fn score_turboquant_no_qjl_4bit_batch_from_payloads(
        self,
        prepared_query: &IvfPreparedQuery,
        payloads: &[u8],
        payload_len: usize,
        gammas: &[f32],
        out_scores: &mut Vec<f32>,
    ) -> Result<bool, String> {
        self.score_turboquant_batch_from_payloads(
            prepared_query,
            payloads,
            payload_len,
            gammas,
            out_scores,
        )
    }

    pub(super) fn score_grouped_pq_batch_from_payloads(
        self,
        prepared_query: &IvfPreparedQuery,
        payloads: &[u8],
        payload_len: usize,
        out_scores: &mut Vec<f32>,
    ) -> Result<bool, String> {
        match (self.profile, prepared_query) {
            (
                IvfQuantizerProfile::PqFastScan { group_count, .. },
                IvfPreparedQuery::PqFastScan {
                    lut,
                    group_count: prepared_group_count,
                    ..
                },
            ) => {
                if group_count != *prepared_group_count {
                    return Err("ec_ivf pq_fastscan prepared query group count mismatch".to_owned());
                }
                if payload_len == 0 {
                    return Err("ec_ivf PqFastScan batch payload length must be nonzero".to_owned());
                }
                if payload_len != self.payload_len() {
                    return Err(format!(
                        "ec_ivf PqFastScan batch payload length mismatch: got {payload_len}, expected {}",
                        self.payload_len()
                    ));
                }
                if payloads.len() % payload_len != 0 {
                    return Err(format!(
                        "ec_ivf PqFastScan batch payload bytes {} are not divisible by payload length {payload_len}",
                        payloads.len()
                    ));
                }

                let candidate_count = payloads.len() / payload_len;
                let mut batch = CandidateBatch::with_capacity(candidate_count);
                for (index, payload) in payloads.chunks_exact(payload_len).enumerate() {
                    batch.push(
                        index,
                        CandidatePayload {
                            code: payload,
                            meta: CandidateMeta::GroupedPq { group_count },
                        },
                    )?;
                }
                out_scores.clear();
                out_scores.resize(batch.len(), 0.0);
                score_grouped_pq_batch_for(
                    CandidateBatchScoringSurface::Ivf,
                    lut,
                    group_count,
                    &batch,
                    out_scores,
                )?;
                Ok(true)
            }
            (IvfQuantizerProfile::RaBitQ, IvfPreparedQuery::RaBitQ(_))
            | (IvfQuantizerProfile::TurboQuant, IvfPreparedQuery::TurboQuant(_))
            | (IvfQuantizerProfile::TurboQuant, IvfPreparedQuery::TurboQuantNoQjl4BitLut(_))
            | (
                IvfQuantizerProfile::TurboQuant,
                IvfPreparedQuery::TurboQuantNoQjl4BitInt8Approx(_),
            )
            | (
                IvfQuantizerProfile::TurboQuant,
                IvfPreparedQuery::TurboQuantCalibratedNoQjl4Bit(_),
            ) => Ok(false),
            (IvfQuantizerProfile::TurboQuant, IvfPreparedQuery::RaBitQ(_))
            | (IvfQuantizerProfile::RaBitQ, IvfPreparedQuery::TurboQuant(_))
            | (IvfQuantizerProfile::RaBitQ, IvfPreparedQuery::TurboQuantNoQjl4BitLut(_))
            | (IvfQuantizerProfile::RaBitQ, IvfPreparedQuery::TurboQuantNoQjl4BitInt8Approx(_))
            | (IvfQuantizerProfile::RaBitQ, IvfPreparedQuery::TurboQuantCalibratedNoQjl4Bit(_))
            | (IvfQuantizerProfile::TurboQuant, IvfPreparedQuery::PqFastScan { .. })
            | (IvfQuantizerProfile::RaBitQ, IvfPreparedQuery::PqFastScan { .. })
            | (IvfQuantizerProfile::PqFastScan { .. }, IvfPreparedQuery::TurboQuant(_))
            | (
                IvfQuantizerProfile::PqFastScan { .. },
                IvfPreparedQuery::TurboQuantNoQjl4BitLut(_),
            )
            | (
                IvfQuantizerProfile::PqFastScan { .. },
                IvfPreparedQuery::TurboQuantNoQjl4BitInt8Approx(_),
            )
            | (
                IvfQuantizerProfile::PqFastScan { .. },
                IvfPreparedQuery::TurboQuantCalibratedNoQjl4Bit(_),
            )
            | (IvfQuantizerProfile::PqFastScan { .. }, IvfPreparedQuery::RaBitQ(_)) => {
                Err("ec_ivf prepared query does not match quantizer profile".to_owned())
            }
        }
    }

    pub(super) fn payload_len(self) -> usize {
        match self.profile {
            IvfQuantizerProfile::TurboQuant => {
                crate::code_len(self.dimensions, crate::DEFAULT_QUANT_BITS)
            }
            IvfQuantizerProfile::PqFastScan { group_count, .. } => group_count.div_ceil(2),
            IvfQuantizerProfile::RaBitQ => code_len_for(self.dimensions, self.rabitq_bits)
                .expect("RaBitQ quant_bits should be validated at resolve time"),
        }
    }

    pub(super) fn rabitq_bits(self) -> u8 {
        self.rabitq_bits
    }

    /// Task 115: true when this RaBitQ quantizer encodes/decodes posting
    /// residuals against the assigned centroid.
    pub(super) fn rabitq_residual(self) -> bool {
        self.rabitq_residual
    }

    pub(super) fn quant_codec(self) -> IvfQuantCodec<'static> {
        IvfQuantCodec::new(self)
    }

    pub(super) fn quant_codec_with_pq_model<'a>(
        self,
        model: &'a IvfPqFastScanModel,
    ) -> Result<IvfQuantCodec<'a>, String> {
        IvfQuantCodec::with_pq_model(self, model)
    }

    pub(super) fn uses_score_bound_pruning(self) -> bool {
        matches!(
            self.profile,
            IvfQuantizerProfile::TurboQuant
                | IvfQuantizerProfile::PqFastScan { .. }
                | IvfQuantizerProfile::RaBitQ
        )
    }

    fn rabitq_quantizer(self) -> Result<Arc<RaBitQQuantizer>, String> {
        RaBitQQuantizer::cached_seeded_srht_bits_clip(
            self.dimensions,
            crate::DEFAULT_QUANT_SEED,
            self.rabitq_bits,
            f32::from(self.rabitq_quant_clip),
        )
    }

    fn validate_pq_model(self, model: &IvfPqFastScanModel) -> Result<(), String> {
        match self.profile {
            IvfQuantizerProfile::PqFastScan {
                group_count,
                group_size,
            } => {
                if model.group_count != group_count || model.group_size != group_size {
                    return Err(format!(
                        "ec_ivf pq_fastscan model shape mismatch: model {}x{}, expected {}x{}",
                        model.group_count, model.group_size, group_count, group_size
                    ));
                }
                let expected = group_count * GROUPED_PQ_CENTROIDS * group_size;
                if model.flat_codebooks.len() != expected {
                    return Err(format!(
                        "ec_ivf pq_fastscan codebook length mismatch: got {}, expected {expected}",
                        model.flat_codebooks.len()
                    ));
                }
                Ok(())
            }
            _ => Err("ec_ivf pq_fastscan model used with non-pq quantizer".to_owned()),
        }
    }

    fn validate_tq_calibration_model(self, model: &IvfTqCalibrationModel) -> Result<(), String> {
        if !matches!(self.profile, IvfQuantizerProfile::TurboQuant) {
            return Err(
                "ec_ivf TurboQuant calibration model used with non-TurboQuant quantizer".to_owned(),
            );
        }
        if model.calibration.shift.len() != self.dimensions
            || model.calibration.scale.len() != self.dimensions
        {
            return Err(format!(
                "ec_ivf TurboQuant calibration shape mismatch: shift {}, scale {}, expected {}",
                model.calibration.shift.len(),
                model.calibration.scale.len(),
                self.dimensions
            ));
        }
        if model
            .calibration
            .shift
            .iter()
            .chain(model.calibration.scale.iter())
            .any(|value| !value.is_finite())
        {
            return Err("ec_ivf TurboQuant calibration contains non-finite values".to_owned());
        }
        if model.calibration.scale.iter().any(|value| *value == 0.0) {
            return Err("ec_ivf TurboQuant calibration contains zero scale".to_owned());
        }
        let quantizer = ProdQuantizer::cached(
            self.dimensions,
            crate::DEFAULT_QUANT_BITS,
            crate::DEFAULT_QUANT_SEED,
        );
        if quantizer.exact_score_mode() != ExactScoreMode::MseNoQjl4Bit {
            return Err("ec_ivf TurboQuant calibration requires the no-QJL 4-bit lane".to_owned());
        }
        Ok(())
    }
}

impl QuantCodec for IvfQuantizer {
    type PreparedQuery = IvfPreparedQuery;

    fn codec_kind(&self) -> QuantCodecKind {
        self.quant_codec().codec_kind()
    }

    fn search_codec_tag(&self) -> QuantSearchCodecTag {
        self.quant_codec().search_codec_tag()
    }

    fn payload_len(&self) -> usize {
        self.quant_codec().payload_len()
    }

    fn encode_source(&self, source: &[f32]) -> Result<EncodedQuantPayload, String> {
        self.quant_codec().encode_source(source)
    }

    fn prepare_ip_query(&self, query: &[f32]) -> Result<Self::PreparedQuery, String> {
        self.quant_codec().prepare_ip_query(query)
    }

    fn score_ip_candidate(
        &self,
        prepared_query: &Self::PreparedQuery,
        payload: CandidatePayload<'_>,
    ) -> Result<f32, String> {
        self.quant_codec()
            .score_ip_candidate(prepared_query, payload)
    }

    fn try_score_ip_candidate(
        &self,
        prepared_query: &Self::PreparedQuery,
        payload: CandidatePayload<'_>,
        min_ip_to_keep: Option<f32>,
    ) -> Result<Option<f32>, String> {
        self.quant_codec()
            .try_score_ip_candidate(prepared_query, payload, min_ip_to_keep)
    }

    fn score_ip_batch<Id>(
        &self,
        prepared_query: &Self::PreparedQuery,
        batch: &CandidateBatch<'_, Id>,
        out_scores: &mut [f32],
    ) -> Result<(), String> {
        self.quant_codec()
            .score_ip_batch(prepared_query, batch, out_scores)
    }
}

impl QuantCodec for IvfQuantCodec<'_> {
    type PreparedQuery = IvfPreparedQuery;

    fn codec_kind(&self) -> QuantCodecKind {
        match self.quantizer.profile {
            IvfQuantizerProfile::TurboQuant => QuantCodecKind::TurboQuant,
            IvfQuantizerProfile::RaBitQ => QuantCodecKind::RaBitQ,
            IvfQuantizerProfile::PqFastScan { .. } => QuantCodecKind::GroupedPq,
        }
    }

    fn search_codec_tag(&self) -> QuantSearchCodecTag {
        match self.quantizer.profile {
            IvfQuantizerProfile::TurboQuant => QuantSearchCodecTag::TurboQuant,
            IvfQuantizerProfile::RaBitQ => QuantSearchCodecTag::RaBitQ {
                bits: self.quantizer.rabitq_bits,
            },
            IvfQuantizerProfile::PqFastScan {
                group_count,
                group_size,
            } => QuantSearchCodecTag::GroupedPq {
                group_count,
                group_size,
            },
        }
    }

    fn payload_len(&self) -> usize {
        IvfQuantizer::payload_len(self.quantizer)
    }

    fn encode_source(&self, source: &[f32]) -> Result<EncodedQuantPayload, String> {
        let (dimensions, gamma, code) = match self.quantizer.profile {
            IvfQuantizerProfile::PqFastScan { .. } => self
                .quantizer
                .encode_source_with_pq_model(source, self.pq_model()?)?,
            _ => IvfQuantizer::encode_source(self.quantizer, source)?,
        };
        Ok(EncodedQuantPayload {
            dimensions,
            gamma,
            code,
        })
    }

    fn prepare_ip_query(&self, query: &[f32]) -> Result<Self::PreparedQuery, String> {
        match self.quantizer.profile {
            IvfQuantizerProfile::PqFastScan { .. } => self
                .quantizer
                .prepare_ip_query_with_pq_model(query, self.pq_model()?),
            _ => IvfQuantizer::prepare_ip_query(self.quantizer, query),
        }
    }

    fn score_ip_candidate(
        &self,
        prepared_query: &Self::PreparedQuery,
        payload: CandidatePayload<'_>,
    ) -> Result<f32, String> {
        validate_candidate_meta(self.quantizer.profile, prepared_query, payload.meta)?;
        let gamma = match payload.meta {
            CandidateMeta::None
            | CandidateMeta::Binary
            | CandidateMeta::RaBitQ
            | CandidateMeta::GroupedPq { .. } => 0.0,
            CandidateMeta::Gamma(gamma) => gamma,
            CandidateMeta::GammaAndResidualSigns { gamma, .. } => gamma,
        };
        IvfQuantizer::score_ip_from_parts(self.quantizer, prepared_query, gamma, payload.code)
    }

    fn try_score_ip_candidate(
        &self,
        prepared_query: &Self::PreparedQuery,
        payload: CandidatePayload<'_>,
        min_ip_to_keep: Option<f32>,
    ) -> Result<Option<f32>, String> {
        validate_candidate_meta(self.quantizer.profile, prepared_query, payload.meta)?;
        let gamma = match payload.meta {
            CandidateMeta::None
            | CandidateMeta::Binary
            | CandidateMeta::RaBitQ
            | CandidateMeta::GroupedPq { .. } => 0.0,
            CandidateMeta::Gamma(gamma) => gamma,
            CandidateMeta::GammaAndResidualSigns { gamma, .. } => gamma,
        };
        IvfQuantizer::score_ip_from_parts_with_min_bound(
            self.quantizer,
            prepared_query,
            gamma,
            payload.code,
            min_ip_to_keep,
        )
    }

    fn score_ip_batch<Id>(
        &self,
        prepared_query: &Self::PreparedQuery,
        batch: &CandidateBatch<'_, Id>,
        out_scores: &mut [f32],
    ) -> Result<(), String> {
        match (self.quantizer.profile, prepared_query) {
            (
                IvfQuantizerProfile::TurboQuant,
                IvfPreparedQuery::TurboQuantNoQjl4BitLut(prepared_query),
            ) => {
                let quantizer = ProdQuantizer::cached(
                    self.quantizer.dimensions,
                    crate::DEFAULT_QUANT_BITS,
                    crate::DEFAULT_QUANT_SEED,
                );
                score_turboquant_no_qjl_4bit_batch_for(
                    CandidateBatchScoringSurface::Ivf,
                    quantizer.as_ref(),
                    prepared_query,
                    batch,
                    out_scores,
                )
            }
            (
                IvfQuantizerProfile::TurboQuant,
                IvfPreparedQuery::TurboQuantNoQjl4BitInt8Approx(prepared_query),
            ) => {
                let quantizer = ProdQuantizer::cached(
                    self.quantizer.dimensions,
                    crate::DEFAULT_QUANT_BITS,
                    crate::DEFAULT_QUANT_SEED,
                );
                score_turboquant_int8_approx_batch_for(
                    CandidateBatchScoringSurface::Ivf,
                    quantizer.as_ref(),
                    prepared_query,
                    batch,
                    out_scores,
                )
            }
            (IvfQuantizerProfile::TurboQuant, IvfPreparedQuery::TurboQuant(prepared_query)) => {
                let quantizer = ProdQuantizer::cached(
                    self.quantizer.dimensions,
                    crate::DEFAULT_QUANT_BITS,
                    crate::DEFAULT_QUANT_SEED,
                );
                score_turboquant_qjl_batch_for(
                    CandidateBatchScoringSurface::Ivf,
                    quantizer.as_ref(),
                    prepared_query,
                    batch,
                    out_scores,
                )
            }
            (
                IvfQuantizerProfile::PqFastScan { group_count, .. },
                IvfPreparedQuery::PqFastScan {
                    lut,
                    group_count: prepared_group_count,
                    ..
                },
            ) => {
                if group_count != *prepared_group_count {
                    return Err("ec_ivf pq_fastscan prepared query group count mismatch".to_owned());
                }
                score_grouped_pq_batch_for(
                    CandidateBatchScoringSurface::Ivf,
                    lut,
                    group_count,
                    batch,
                    out_scores,
                )
            }
            (IvfQuantizerProfile::RaBitQ, IvfPreparedQuery::RaBitQ(prepared_query))
                if self.quantizer.rabitq_bits == 1 =>
            {
                let prepared = prepared_query
                    .bits1_block_prepared(self.quantizer.payload_len())
                    .ok_or_else(|| {
                        "ec_ivf RaBitQ bits=1 prepared query missing block state".to_owned()
                    })?;
                score_rabitq_bits1_batch_for(
                    CandidateBatchScoringSurface::Ivf,
                    prepared,
                    batch,
                    out_scores,
                )
            }
            _ => {
                if batch.len() != out_scores.len() {
                    return Err(format!(
                        "quant codec batch output count {} does not match candidate count {}",
                        out_scores.len(),
                        batch.len()
                    ));
                }
                for (payload, out_score) in batch.payloads().iter().zip(out_scores.iter_mut()) {
                    *out_score = self.score_ip_candidate(prepared_query, *payload)?;
                }
                Ok(())
            }
        }
    }
}

fn validate_candidate_meta(
    profile: IvfQuantizerProfile,
    prepared_query: &IvfPreparedQuery,
    meta: CandidateMeta<'_>,
) -> Result<(), String> {
    match (profile, prepared_query, meta) {
        (
            IvfQuantizerProfile::PqFastScan { group_count, .. },
            IvfPreparedQuery::PqFastScan {
                group_count: prepared_group_count,
                ..
            },
            CandidateMeta::GroupedPq {
                group_count: meta_group_count,
            },
        ) if group_count == *prepared_group_count && group_count == meta_group_count => Ok(()),
        (
            IvfQuantizerProfile::PqFastScan { .. },
            IvfPreparedQuery::PqFastScan { .. },
            CandidateMeta::GroupedPq { .. },
        ) => Err("ec_ivf pq_fastscan candidate group count mismatch".to_owned()),
        (IvfQuantizerProfile::PqFastScan { .. }, IvfPreparedQuery::PqFastScan { .. }, _) => {
            Err("ec_ivf pq_fastscan candidate requires grouped-PQ metadata".to_owned())
        }
        _ => Ok(()),
    }
}

pub(super) fn default_pq_fastscan_group_size(dimensions: usize) -> usize {
    rotation::effective_transform_dim(dimensions).min(16)
}

pub(super) fn resolve_pq_fastscan_group_size(
    dimensions: usize,
    requested_group_size: Option<usize>,
) -> Result<usize, String> {
    let transform_dim = rotation::effective_transform_dim(dimensions);
    let group_size =
        requested_group_size.unwrap_or_else(|| default_pq_fastscan_group_size(dimensions));
    if group_size == 0 {
        return Err("ec_ivf pq_fastscan pq_group_size must be greater than zero".to_owned());
    }
    if !matches!(group_size, 8 | 16 | 32) && group_size != transform_dim {
        return Err(format!(
            "ec_ivf pq_fastscan pq_group_size must be 8, 16, 32, or the full transformed dimension {transform_dim}; got {group_size}"
        ));
    }
    if group_size > transform_dim || transform_dim % group_size != 0 {
        return Err(format!(
            "ec_ivf pq_fastscan pq_group_size {group_size} must divide transformed dimension {transform_dim}"
        ));
    }
    Ok(group_size)
}

pub(super) unsafe fn load_pq_fastscan_model(
    index_relation: pgrx::pg_sys::Relation,
    metadata: &page::MetadataPage,
) -> Result<IvfPqFastScanModel, String> {
    if metadata.storage_format != StorageFormat::PqFastScan {
        return Err("ec_ivf pq_fastscan model load requires a pq_fastscan index".to_owned());
    }
    if metadata.pq_codebook_head == ItemPointer::INVALID {
        return Err("ec_ivf pq_fastscan metadata is missing a codebook head".to_owned());
    }
    if metadata.pq_group_size == 0 {
        return Err("ec_ivf pq_fastscan metadata has zero group size".to_owned());
    }
    let group_size = usize::from(metadata.pq_group_size);
    let transform_dim = rotation::effective_transform_dim(metadata.dimensions as usize);
    if transform_dim % group_size != 0 {
        return Err(format!(
            "ec_ivf pq_fastscan transform dim {transform_dim} is not divisible by group size {group_size}"
        ));
    }
    let group_count = transform_dim / group_size;
    let centroid_count = group_size * GROUPED_PQ_CENTROIDS;
    let mut flat_codebooks = Vec::with_capacity(group_count * centroid_count);
    let mut next_tid = metadata.pq_codebook_head;

    for expected_group_index in 0..group_count {
        if next_tid == ItemPointer::INVALID {
            return Err(format!(
                "ec_ivf pq_fastscan codebook chain ended at group {expected_group_index}"
            ));
        }
        let tuple = page::read_ivf_pq_codebook(index_relation, next_tid, centroid_count)?;
        if usize::from(tuple.group_index) != expected_group_index {
            return Err(format!(
                "ec_ivf pq_fastscan codebook order mismatch: got {}, expected {expected_group_index}",
                tuple.group_index
            ));
        }
        flat_codebooks.extend(tuple.centroids);
        next_tid = tuple.next_tid;
    }

    if next_tid != ItemPointer::INVALID {
        return Err("ec_ivf pq_fastscan codebook chain has trailing tuples".to_owned());
    }

    Ok(IvfPqFastScanModel {
        group_count,
        group_size,
        signs: rotation::sign_vector(transform_dim, metadata.seed),
        flat_codebooks,
    })
}

pub(super) unsafe fn load_tq_calibration_model(
    index_relation: pgrx::pg_sys::Relation,
    metadata: &page::MetadataPage,
) -> Result<IvfTqCalibrationModel, String> {
    if metadata.turboquant_profile != super::options::TurboQuantProfile::TqPlus {
        return Err(
            "ec_ivf TurboQuant calibration model load requires turboquant_profile = 'tqplus'"
                .to_owned(),
        );
    }
    if metadata.turboquant_calibration_head == ItemPointer::INVALID {
        return Err(
            "ec_ivf TurboQuant calibration metadata is missing a calibration head".to_owned(),
        );
    }
    let shift = unsafe {
        page::read_ivf_tq_calibration(index_relation, metadata.turboquant_calibration_head)
    }?;
    if shift.array_kind != page::IvfTqCalibrationArrayKind::Shift {
        return Err("ec_ivf TurboQuant calibration chain must start with shift tuple".to_owned());
    }
    if shift.next_tid == ItemPointer::INVALID {
        return Err("ec_ivf TurboQuant calibration chain is missing scale tuple".to_owned());
    }
    let scale = unsafe { page::read_ivf_tq_calibration(index_relation, shift.next_tid) }?;
    if scale.array_kind != page::IvfTqCalibrationArrayKind::Scale {
        return Err("ec_ivf TurboQuant calibration chain second tuple must be scale".to_owned());
    }
    if scale.next_tid != ItemPointer::INVALID {
        return Err("ec_ivf TurboQuant calibration chain has trailing tuples".to_owned());
    }
    let model = IvfTqCalibrationModel {
        calibration: TqCalibration {
            shift: shift.values,
            scale: scale.values,
        },
    };
    IvfQuantizer::resolve(StorageFormat::TurboQuant, metadata.dimensions as usize)?
        .validate_tq_calibration_model(&model)?;
    Ok(model)
}

impl IvfPreparedQuery {
    #[cfg(any(test, feature = "pg_test"))]
    pub(super) fn lut_len(&self) -> usize {
        match self {
            Self::TurboQuant(prepared) => prepared.lut.len(),
            Self::TurboQuantNoQjl4BitLut(prepared) => prepared.lut.len(),
            Self::TurboQuantNoQjl4BitInt8Approx(_) => 0,
            Self::PqFastScan { lut, .. } => lut.len(),
            Self::RaBitQ(_) => 0,
        }
    }

    #[cfg(any(test, feature = "pg_test"))]
    pub(super) fn sq_len(&self) -> usize {
        match self {
            Self::TurboQuant(prepared) => prepared.sq.len(),
            Self::TurboQuantNoQjl4BitLut(_) => 0,
            Self::TurboQuantNoQjl4BitInt8Approx(_) => 0,
            Self::PqFastScan { .. } => 0,
            Self::RaBitQ(_) => 0,
        }
    }
}

fn grouped_pq_suffix_max(lut: &[f32], group_count: usize) -> Vec<f32> {
    let mut suffix_max = vec![0.0_f32; group_count + 1];
    for group_index in (0..group_count).rev() {
        let row_start = group_index * GROUPED_PQ_CENTROIDS;
        let row_max = lut[row_start..row_start + GROUPED_PQ_CENTROIDS]
            .iter()
            .copied()
            .fold(f32::NEG_INFINITY, f32::max);
        suffix_max[group_index] = suffix_max[group_index + 1] + row_max;
    }
    suffix_max
}

fn grouped_pq_score_f32_with_min_bound(
    lut: &[f32],
    suffix_max: &[f32],
    group_count: usize,
    packed_nibbles: &[u8],
    min_ip_to_keep: f32,
) -> Option<f32> {
    debug_assert_eq!(suffix_max.len(), group_count + 1);
    let mut score = 0.0_f32;
    for group_index in 0..group_count {
        let centroid_index =
            crate::quant::grouped_pq::grouped_pq_nibble(packed_nibbles, group_index);
        score += lut[group_index * GROUPED_PQ_CENTROIDS + centroid_index];
        if score + suffix_max[group_index + 1] < min_ip_to_keep {
            return None;
        }
    }
    Some(score)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unit_vector(dimensions: usize) -> Vec<f32> {
        let mut values = (0..dimensions)
            .map(|index| (index as f32 + 1.0) / dimensions as f32)
            .collect::<Vec<_>>();
        let norm = values.iter().map(|value| value * value).sum::<f32>().sqrt();
        values.iter_mut().for_each(|value| *value /= norm);
        values
    }

    fn pq_fastscan_test_model(dimensions: usize) -> IvfPqFastScanModel {
        let training_rows = [
            unit_vector(dimensions),
            unit_vector(dimensions),
            (0..dimensions)
                .map(|index| if index % 2 == 0 { 0.25 } else { -0.25 })
                .collect::<Vec<_>>(),
            (0..dimensions)
                .map(|index| if index % 2 == 0 { -0.25 } else { 0.25 })
                .collect::<Vec<_>>(),
        ];
        let training_refs = training_rows.iter().map(Vec::as_slice).collect::<Vec<_>>();
        let trained = crate::am::common::training::train_grouped_pq4_model(
            &training_refs,
            dimensions,
            crate::DEFAULT_QUANT_SEED,
            default_pq_fastscan_group_size(dimensions),
            training_refs.len(),
            3,
        )
        .unwrap();
        IvfPqFastScanModel {
            group_count: trained.group_count,
            group_size: trained.group_size,
            signs: trained.signs,
            flat_codebooks: trained.codebooks.into_iter().flatten().collect(),
        }
    }

    #[test]
    fn supported_v1_formats_resolve_to_turboquant() {
        let auto = IvfQuantizer::resolve(StorageFormat::Auto, 16).unwrap();
        let explicit = IvfQuantizer::resolve(StorageFormat::TurboQuant, 16).unwrap();

        assert_eq!(auto.profile, IvfQuantizerProfile::TurboQuant);
        assert_eq!(explicit.profile, IvfQuantizerProfile::TurboQuant);
    }

    #[test]
    fn rabitq_v1_format_resolves_to_rabitq() {
        let explicit = IvfQuantizer::resolve(StorageFormat::RaBitQ, 16).unwrap();

        assert_eq!(explicit.profile, IvfQuantizerProfile::RaBitQ);
    }

    #[test]
    fn coarse_rerank_v1_format_resolves_to_rabitq() {
        let explicit = IvfQuantizer::resolve(StorageFormat::CoarseRerank, 16).unwrap();

        assert_eq!(explicit.profile, IvfQuantizerProfile::RaBitQ);
    }

    #[test]
    fn pq_fastscan_v1_format_resolves_to_grouped_profile() {
        let explicit = IvfQuantizer::resolve(StorageFormat::PqFastScan, 16).unwrap();

        assert_eq!(
            explicit.profile,
            IvfQuantizerProfile::PqFastScan {
                group_count: 1,
                group_size: 16
            }
        );
    }

    #[test]
    fn pq_fastscan_accepts_metadata_group_size_override() {
        let explicit =
            IvfQuantizer::resolve_with_pq_group_size(StorageFormat::PqFastScan, 64, Some(8))
                .unwrap();

        assert_eq!(
            explicit.profile,
            IvfQuantizerProfile::PqFastScan {
                group_count: 8,
                group_size: 8
            }
        );
        assert_eq!(explicit.payload_len(), 4);
    }

    #[test]
    fn pq_fastscan_rejects_group_size_that_does_not_divide_transform() {
        let err = IvfQuantizer::resolve_with_pq_group_size(StorageFormat::PqFastScan, 64, Some(12))
            .unwrap_err();

        assert!(err.contains("pq_group_size"));
        assert!(err.contains("must be 8, 16, 32"));
    }

    #[test]
    fn turboquant_dispatch_matches_direct_prod_score() {
        let dimensions = 32;
        let source = unit_vector(dimensions);
        let query = unit_vector(dimensions);
        let dispatch = IvfQuantizer::resolve(StorageFormat::TurboQuant, dimensions).unwrap();
        let (_, gamma, payload) = dispatch.encode_source(&source).unwrap();
        let prepared = dispatch.prepare_ip_query(&query).unwrap();

        let direct = ProdQuantizer::cached(
            dimensions,
            crate::DEFAULT_QUANT_BITS,
            crate::DEFAULT_QUANT_SEED,
        );
        let direct_prepared = direct.prepare_ip_query(&query);

        assert_eq!(
            dispatch
                .score_ip_from_parts(&prepared, gamma, &payload)
                .unwrap(),
            direct.score_ip_from_parts(&direct_prepared, gamma, &payload)
        );
    }

    #[test]
    fn turboquant_dispatch_uses_lut_for_no_qjl_4bit_lane() {
        let dimensions = 1536;
        let source = unit_vector(dimensions);
        let query = unit_vector(dimensions);
        let dispatch = IvfQuantizer::resolve(StorageFormat::TurboQuant, dimensions).unwrap();
        let (_, gamma, payload) = dispatch.encode_source(&source).unwrap();
        let prepared = dispatch.prepare_ip_query(&query).unwrap();

        assert_eq!(prepared.lut_len(), dimensions * 16);
        assert_eq!(prepared.sq_len(), 0);

        let direct = ProdQuantizer::cached(
            dimensions,
            crate::DEFAULT_QUANT_BITS,
            crate::DEFAULT_QUANT_SEED,
        );
        let direct_prepared = direct.prepare_ip_query_lut_no_qjl_4bit(&query);

        assert_eq!(
            dispatch
                .score_ip_from_parts(&prepared, gamma, &payload)
                .unwrap(),
            direct.score_ip_from_parts_lut_no_qjl_4bit(&direct_prepared, &payload)
        );
    }

    #[test]
    fn turboquant_no_qjl_4bit_batch_scores_match_scalar_scores() {
        let dimensions = 1536;
        let query = unit_vector(dimensions);
        let dispatch = IvfQuantizer::resolve(StorageFormat::TurboQuant, dimensions).unwrap();
        let prepared = dispatch.prepare_ip_query(&query).unwrap();
        let sources = [
            unit_vector(dimensions),
            (0..dimensions)
                .map(|index| ((index % 17) as f32 - 8.0) / dimensions as f32)
                .collect::<Vec<_>>(),
        ];
        let encoded = sources
            .iter()
            .map(|source| dispatch.encode_source(source).unwrap())
            .collect::<Vec<_>>();
        let gammas = encoded
            .iter()
            .map(|(_, gamma, _)| *gamma)
            .collect::<Vec<_>>();
        let payloads = encoded
            .iter()
            .flat_map(|(_, _, payload)| payload.iter().copied())
            .collect::<Vec<_>>();
        let mut batch_scores = Vec::new();

        let used_batch = dispatch
            .score_turboquant_no_qjl_4bit_batch_from_payloads(
                &prepared,
                &payloads,
                dispatch.payload_len(),
                &gammas,
                &mut batch_scores,
            )
            .unwrap();

        assert!(used_batch);
        assert_eq!(batch_scores.len(), sources.len());
        for (index, (_, gamma, payload)) in encoded.iter().enumerate() {
            let scalar = dispatch
                .score_ip_from_parts(&prepared, *gamma, payload)
                .unwrap();
            assert!(
                (batch_scores[index] - scalar).abs() < 1e-6,
                "index={index} batch={} scalar={scalar}",
                batch_scores[index]
            );
        }
    }

    #[test]
    fn turboquant_no_qjl_4bit_batch_ignores_gamma_side_input() {
        let dimensions = 1536;
        let query = unit_vector(dimensions);
        let dispatch = IvfQuantizer::resolve(StorageFormat::TurboQuant, dimensions).unwrap();
        let prepared = dispatch.prepare_ip_query(&query).unwrap();
        let sources = [
            unit_vector(dimensions),
            (0..dimensions)
                .map(|index| ((index % 19) as f32 - 9.0) / dimensions as f32)
                .collect::<Vec<_>>(),
        ];
        let encoded = sources
            .iter()
            .map(|source| dispatch.encode_source(source).unwrap())
            .collect::<Vec<_>>();
        let payloads = encoded
            .iter()
            .flat_map(|(_, _, payload)| payload.iter().copied())
            .collect::<Vec<_>>();
        let mut batch_scores = Vec::new();

        let used_batch = dispatch
            .score_turboquant_no_qjl_4bit_batch_from_payloads(
                &prepared,
                &payloads,
                dispatch.payload_len(),
                &[],
                &mut batch_scores,
            )
            .unwrap();

        assert!(used_batch);
        assert_eq!(batch_scores.len(), sources.len());
        for (index, (_, gamma, payload)) in encoded.iter().enumerate() {
            let scalar = dispatch
                .score_ip_from_parts(&prepared, *gamma, payload)
                .unwrap();
            assert!(
                (batch_scores[index] - scalar).abs() < 1e-6,
                "index={index} batch={} scalar={scalar}",
                batch_scores[index]
            );
        }
    }

    #[test]
    fn turboquant_no_qjl_4bit_negated_batch_writes_caller_slice() {
        let dimensions = 1536;
        let query = unit_vector(dimensions);
        let dispatch = IvfQuantizer::resolve(StorageFormat::TurboQuant, dimensions).unwrap();
        let prepared = dispatch.prepare_ip_query(&query).unwrap();
        let sources = [
            unit_vector(dimensions),
            (0..dimensions)
                .map(|index| ((index % 29) as f32 - 14.0) / dimensions as f32)
                .collect::<Vec<_>>(),
        ];
        let encoded = sources
            .iter()
            .map(|source| dispatch.encode_source(source).unwrap())
            .collect::<Vec<_>>();
        let payloads = encoded
            .iter()
            .flat_map(|(_, _, payload)| payload.iter().copied())
            .collect::<Vec<_>>();
        let mut negated_scores = vec![123.0; sources.len()];

        let used_batch = dispatch
            .score_turboquant_batch_from_payloads_negated_into(
                &prepared,
                &payloads,
                dispatch.payload_len(),
                &[],
                &mut negated_scores,
            )
            .unwrap();

        assert!(used_batch);
        for (index, (_, gamma, payload)) in encoded.iter().enumerate() {
            let scalar = dispatch
                .score_ip_from_parts(&prepared, *gamma, payload)
                .unwrap();
            assert!(
                (negated_scores[index] + scalar).abs() < 1e-6,
                "index={index} negated={} scalar={scalar}",
                negated_scores[index]
            );
        }
    }

    #[test]
    fn turboquant_int8_approx_scorer_prepares_factored_variant() {
        let dimensions = 1536;
        let source = unit_vector(dimensions);
        let query = unit_vector(dimensions);
        let dispatch = IvfQuantizer::resolve(StorageFormat::TurboQuant, dimensions).unwrap();
        let (_, gamma, payload) = dispatch.encode_source(&source).unwrap();
        let prepared = dispatch
            .prepare_ip_query_with_turboquant_scorer(&query, TurboQuantScorerGuc::Int8Approx)
            .unwrap();

        assert!(matches!(
            prepared,
            IvfPreparedQuery::TurboQuantNoQjl4BitInt8Approx(_)
        ));
        assert_eq!(prepared.lut_len(), 0);
        assert_eq!(prepared.sq_len(), 0);

        let direct = ProdQuantizer::cached(
            dimensions,
            crate::DEFAULT_QUANT_BITS,
            crate::DEFAULT_QUANT_SEED,
        );
        let direct_prepared = direct.prepare_ip_query_int8_approx_no_qjl_4bit(&query);

        assert_eq!(
            dispatch
                .score_ip_from_parts(&prepared, gamma, &payload)
                .unwrap()
                .to_bits(),
            direct
                .score_ip_from_parts_int8_approx_no_qjl_4bit(&direct_prepared, &payload)
                .to_bits()
        );
    }

    #[test]
    fn turboquant_int8_approx_default_session_scorer_stays_lut() {
        let dimensions = 1536;
        let query = unit_vector(dimensions);
        let dispatch = IvfQuantizer::resolve(StorageFormat::TurboQuant, dimensions).unwrap();
        let prepared = dispatch.prepare_ip_query(&query).unwrap();

        assert!(matches!(
            prepared,
            IvfPreparedQuery::TurboQuantNoQjl4BitLut(_)
        ));
    }

    #[test]
    fn turboquant_int8_approx_batch_scores_match_scalar_scores() {
        let dimensions = 1536;
        let query = unit_vector(dimensions);
        let dispatch = IvfQuantizer::resolve(StorageFormat::TurboQuant, dimensions).unwrap();
        let prepared = dispatch
            .prepare_ip_query_with_turboquant_scorer(&query, TurboQuantScorerGuc::Int8Approx)
            .unwrap();
        let sources = [
            unit_vector(dimensions),
            (0..dimensions)
                .map(|index| ((index % 17) as f32 - 8.0) / dimensions as f32)
                .collect::<Vec<_>>(),
        ];
        let encoded = sources
            .iter()
            .map(|source| dispatch.encode_source(source).unwrap())
            .collect::<Vec<_>>();
        let payloads = encoded
            .iter()
            .flat_map(|(_, _, payload)| payload.iter().copied())
            .collect::<Vec<_>>();
        let mut batch_scores = Vec::new();

        let used_batch = dispatch
            .score_turboquant_batch_from_payloads(
                &prepared,
                &payloads,
                dispatch.payload_len(),
                &[],
                &mut batch_scores,
            )
            .unwrap();

        assert!(used_batch);
        assert_eq!(batch_scores.len(), sources.len());
        for (index, (_, gamma, payload)) in encoded.iter().enumerate() {
            let scalar = dispatch
                .score_ip_from_parts(&prepared, *gamma, payload)
                .unwrap();
            assert_eq!(
                batch_scores[index].to_bits(),
                scalar.to_bits(),
                "index={index} batch={} scalar={scalar}",
                batch_scores[index]
            );
        }
    }

    #[test]
    fn turboquant_int8_approx_payload_ref_batch_matches_scalar_scores() {
        let dimensions = 1536;
        let query = unit_vector(dimensions);
        let dispatch = IvfQuantizer::resolve(StorageFormat::TurboQuant, dimensions).unwrap();
        let prepared = dispatch
            .prepare_ip_query_with_turboquant_scorer(&query, TurboQuantScorerGuc::Int8Approx)
            .unwrap();

        assert!(dispatch.supports_turboquant_payload_ref_batch(&prepared));

        let sources = [
            unit_vector(dimensions),
            (0..dimensions)
                .map(|index| ((index % 19) as f32 - 9.0) / dimensions as f32)
                .collect::<Vec<_>>(),
        ];
        let encoded = sources
            .iter()
            .map(|source| dispatch.encode_source(source).unwrap())
            .collect::<Vec<_>>();
        let payload_refs = encoded
            .iter()
            .map(|(_, _, payload)| payload.as_slice())
            .collect::<Vec<_>>();
        let mut negated_scores = vec![123.0; sources.len()];

        let used_batch = dispatch
            .score_turboquant_batch_from_payload_refs_negated_into(
                &prepared,
                &payload_refs,
                dispatch.payload_len(),
                &[],
                &mut negated_scores,
            )
            .unwrap();

        assert!(used_batch);
        for (index, (_, gamma, payload)) in encoded.iter().enumerate() {
            let scalar = dispatch
                .score_ip_from_parts(&prepared, *gamma, payload)
                .unwrap();
            assert_eq!(
                negated_scores[index].to_bits(),
                (-scalar).to_bits(),
                "index={index} negated={} scalar={scalar}",
                negated_scores[index]
            );
        }
    }

    #[test]
    fn turboquant_int8_approx_min_bound_dispatch_scores_without_pruning() {
        let dimensions = 1536;
        let source = unit_vector(dimensions);
        let query = unit_vector(dimensions);
        let dispatch = IvfQuantizer::resolve(StorageFormat::TurboQuant, dimensions).unwrap();
        let (_, gamma, payload) = dispatch.encode_source(&source).unwrap();
        let prepared = dispatch
            .prepare_ip_query_with_turboquant_scorer(&query, TurboQuantScorerGuc::Int8Approx)
            .unwrap();

        // The factored query carries no suffix-max table, so the min-bound
        // dispatch must fall through to full scoring instead of pruning.
        let scored = dispatch
            .score_ip_from_parts_with_min_bound(&prepared, gamma, &payload, Some(f32::MAX))
            .unwrap();
        let unbounded = dispatch
            .score_ip_from_parts(&prepared, gamma, &payload)
            .unwrap();

        assert_eq!(scored.map(f32::to_bits), Some(unbounded.to_bits()));
    }

    #[test]
    fn common_quant_codec_scores_turboquant_batch() {
        let dimensions = 1536;
        let source = unit_vector(dimensions);
        let query = unit_vector(dimensions);
        let codec = IvfQuantizer::resolve(StorageFormat::TurboQuant, dimensions).unwrap();
        let encoded = QuantCodec::encode_source(&codec, &source).unwrap();
        let prepared = QuantCodec::prepare_ip_query(&codec, &query).unwrap();
        let mut batch = CandidateBatch::with_capacity(1);
        batch
            .push(
                7_u32,
                CandidatePayload::new(&encoded.code, CandidateMeta::Gamma(encoded.gamma)),
            )
            .unwrap();
        let mut batch_scores = vec![0.0];

        QuantCodec::score_ip_batch(&codec, &prepared, &batch, &mut batch_scores).unwrap();

        assert_eq!(QuantCodec::codec_kind(&codec), QuantCodecKind::TurboQuant);
        assert_eq!(
            QuantCodec::search_codec_tag(&codec),
            QuantSearchCodecTag::TurboQuant
        );
        assert_eq!(encoded.dimensions, dimensions as u16);
        assert_eq!(encoded.code.len(), QuantCodec::payload_len(&codec));
        assert_eq!(
            batch_scores[0],
            QuantCodec::score_ip_candidate(
                &codec,
                &prepared,
                CandidatePayload::new(&encoded.code, CandidateMeta::Gamma(encoded.gamma)),
            )
            .unwrap()
        );
    }

    #[test]
    fn common_quant_codec_turboquant_batch_is_bit_exact_with_scalar() {
        let dimensions = 32;
        let sources = [
            unit_vector(dimensions),
            (0..dimensions)
                .map(|index| ((index % 7) as f32 - 3.0) / dimensions as f32)
                .collect::<Vec<_>>(),
        ];
        let query = unit_vector(dimensions);
        let codec = IvfQuantizer::resolve(StorageFormat::TurboQuant, dimensions).unwrap();
        let prepared = QuantCodec::prepare_ip_query(&codec, &query).unwrap();
        let encoded = sources
            .iter()
            .map(|source| QuantCodec::encode_source(&codec, source).unwrap())
            .collect::<Vec<_>>();
        let mut batch = CandidateBatch::with_capacity(encoded.len());
        for (index, payload) in encoded.iter().enumerate() {
            batch
                .push(
                    index,
                    CandidatePayload::new(&payload.code, CandidateMeta::Gamma(payload.gamma)),
                )
                .unwrap();
        }
        let mut batch_scores = vec![0.0; batch.len()];

        QuantCodec::score_ip_batch(&codec, &prepared, &batch, &mut batch_scores).unwrap();

        for (index, payload) in encoded.iter().enumerate() {
            let scalar = codec
                .score_ip_from_parts(&prepared, payload.gamma, &payload.code)
                .unwrap();
            assert_eq!(
                batch_scores[index].to_bits(),
                scalar.to_bits(),
                "index={index} batch={} scalar={scalar}",
                batch_scores[index]
            );
        }
    }

    #[test]
    fn common_quant_codec_turboquant_no_qjl_lut32_batch_is_bit_exact_with_scalar() {
        let dimensions = 1536;
        let query = unit_vector(dimensions);
        let codec = IvfQuantizer::resolve(StorageFormat::TurboQuant, dimensions).unwrap();
        let prepared = QuantCodec::prepare_ip_query(&codec, &query).unwrap();
        let encoded = (0..crate::quant::lut32::BLOCK_WIDTH + 1)
            .map(|index| {
                let source = (0..dimensions)
                    .map(|col| ((index + col) % 23) as f32 / dimensions as f32)
                    .collect::<Vec<_>>();
                QuantCodec::encode_source(&codec, &source).unwrap()
            })
            .collect::<Vec<_>>();
        let mut batch = CandidateBatch::with_capacity(crate::quant::lut32::BLOCK_WIDTH + 1);
        for (index, payload) in encoded.iter().enumerate() {
            batch
                .push(
                    index,
                    CandidatePayload::new(&payload.code, CandidateMeta::Gamma(payload.gamma)),
                )
                .unwrap();
        }
        let mut batch_scores = vec![0.0; batch.len()];

        QuantCodec::score_ip_batch(&codec, &prepared, &batch, &mut batch_scores).unwrap();

        for (index, payload) in encoded.iter().enumerate() {
            let scalar = codec
                .score_ip_from_parts(&prepared, payload.gamma, &payload.code)
                .unwrap();
            assert_eq!(
                batch_scores[index].to_bits(),
                scalar.to_bits(),
                "index={index} batch={} scalar={scalar}",
                batch_scores[index]
            );
        }
    }

    #[test]
    fn rabitq_dispatch_matches_direct_quantizer_score() {
        let dimensions = 32;
        let source = unit_vector(dimensions);
        let query = unit_vector(dimensions);
        let dispatch = IvfQuantizer::resolve(StorageFormat::RaBitQ, dimensions).unwrap();
        let (_, gamma, payload) = dispatch.encode_source(&source).unwrap();
        let prepared = dispatch.prepare_ip_query(&query).unwrap();

        let direct = RaBitQQuantizer::with_seeded_srht_bits(
            dimensions,
            crate::DEFAULT_QUANT_SEED,
            crate::DEFAULT_QUANT_BITS,
        )
        .unwrap();
        let direct_prepared = direct.prepare_estimator(&query);

        assert_eq!(gamma, 0.0);
        assert_eq!(payload.len(), direct.code_len());
        assert_eq!(
            dispatch
                .score_ip_from_parts(&prepared, gamma, &payload)
                .unwrap(),
            direct.estimate_ip(&direct_prepared, &payload).estimate
        );
    }

    #[test]
    fn common_quant_codec_scores_rabitq_batch() {
        // bits=2/4 now record block-kernel counters through the unified driver,
        // so this test serializes on the shared counter lock to avoid polluting
        // the count-asserting tests running in parallel.
        let _guard = crate::am::common::candidate_batch::CANDIDATE_BATCH_COUNTER_TEST_LOCK
            .lock()
            .unwrap();
        let dimensions = 40;
        let query = unit_vector(dimensions);
        let codec = IvfQuantizer::resolve_with_pq_group_size_and_bits(
            StorageFormat::RaBitQ,
            dimensions,
            None,
            Some(4),
        )
        .unwrap();
        let prepared = QuantCodec::prepare_ip_query(&codec, &query).unwrap();
        let encoded = (0..2)
            .map(|row| {
                let source = (0..dimensions)
                    .map(|col| (row as f32 - col as f32) * 0.03125)
                    .collect::<Vec<_>>();
                QuantCodec::encode_source(&codec, &source).unwrap()
            })
            .collect::<Vec<_>>();
        let mut batch = CandidateBatch::with_capacity(encoded.len());
        for (index, encoded) in encoded.iter().enumerate() {
            batch
                .push(
                    index,
                    CandidatePayload::new(&encoded.code, CandidateMeta::RaBitQ),
                )
                .unwrap();
        }
        let mut batch_scores = vec![0.0; batch.len()];

        QuantCodec::score_ip_batch(&codec, &prepared, &batch, &mut batch_scores).unwrap();

        assert_eq!(QuantCodec::codec_kind(&codec), QuantCodecKind::RaBitQ);
        assert_eq!(
            QuantCodec::search_codec_tag(&codec),
            QuantSearchCodecTag::RaBitQ { bits: 4 }
        );
        for (index, encoded) in encoded.iter().enumerate() {
            let scalar = codec
                .score_ip_from_parts(&prepared, encoded.gamma, &encoded.code)
                .unwrap();
            assert!(
                (batch_scores[index] - scalar).abs() < 1e-6,
                "index={index} batch={} scalar={scalar}",
                batch_scores[index]
            );
        }
    }

    #[test]
    fn common_quant_codec_rabitq_batch_is_bit_exact_with_scalar() {
        // Records block-kernel counters at bits=4; serialize on the shared lock.
        let _guard = crate::am::common::candidate_batch::CANDIDATE_BATCH_COUNTER_TEST_LOCK
            .lock()
            .unwrap();
        let dimensions = 40;
        let query = unit_vector(dimensions);
        let codec = IvfQuantizer::resolve_with_pq_group_size_and_bits(
            StorageFormat::RaBitQ,
            dimensions,
            None,
            Some(4),
        )
        .unwrap();
        let prepared = QuantCodec::prepare_ip_query(&codec, &query).unwrap();
        let encoded = (0..3)
            .map(|index| {
                let source = (0..dimensions)
                    .map(|col| (index as f32 - col as f32) * 0.03125)
                    .collect::<Vec<_>>();
                QuantCodec::encode_source(&codec, &source).unwrap()
            })
            .collect::<Vec<_>>();
        let mut batch = CandidateBatch::with_capacity(3);
        for (index, payload) in encoded.iter().enumerate() {
            batch
                .push(
                    index,
                    CandidatePayload::new(&payload.code, CandidateMeta::RaBitQ),
                )
                .unwrap();
        }
        let mut batch_scores = vec![0.0; batch.len()];

        QuantCodec::score_ip_batch(&codec, &prepared, &batch, &mut batch_scores).unwrap();

        for (index, payload) in encoded.iter().enumerate() {
            let scalar = codec
                .score_ip_from_parts(&prepared, payload.gamma, &payload.code)
                .unwrap();
            assert_eq!(
                batch_scores[index].to_bits(),
                scalar.to_bits(),
                "index={index} batch={} scalar={scalar}",
                batch_scores[index]
            );
        }
    }

    #[test]
    fn rabitq_dispatch_does_not_rebuild_quantizer_while_scoring() {
        let dimensions = 40;
        let source = unit_vector(dimensions);
        let query = unit_vector(dimensions);
        let dispatch = IvfQuantizer::resolve(StorageFormat::RaBitQ, dimensions).unwrap();

        crate::quant::rabitq::clear_seeded_srht_cache_for_test();
        crate::quant::rabitq::reset_seeded_srht_construction_count_for_test(dimensions);
        let (_, gamma, payload) = dispatch.encode_source(&source).unwrap();
        let prepared = dispatch.prepare_ip_query(&query).unwrap();
        let after_prepare = crate::quant::rabitq::seeded_srht_construction_count_for_test();

        assert_eq!(after_prepare, 1);
        for _ in 0..8 {
            let _ = dispatch
                .score_ip_from_parts(&prepared, gamma, &payload)
                .unwrap();
        }
        assert_eq!(
            crate::quant::rabitq::seeded_srht_construction_count_for_test(),
            after_prepare
        );
    }

    #[test]
    fn rabitq_bits1_batch_dispatch_matches_scalar_scores() {
        // Scoring through the batch dispatch mutates the global counters this
        // lock guards; without it, concurrent counter-asserting tests see
        // double-counted ivf/rabitq rows.
        let _guard = crate::am::common::candidate_batch::CANDIDATE_BATCH_COUNTER_TEST_LOCK
            .lock()
            .unwrap();
        let dimensions = 40;
        let query = unit_vector(dimensions);
        let dispatch = IvfQuantizer::resolve_with_pq_group_size_and_bits(
            StorageFormat::RaBitQ,
            dimensions,
            None,
            Some(1),
        )
        .unwrap();
        let prepared = dispatch.prepare_ip_query(&query).unwrap();
        let payloads = (0..4)
            .flat_map(|row| {
                let source = (0..dimensions)
                    .map(|col| (row as f32 - col as f32) * 0.03125)
                    .collect::<Vec<_>>();
                let (_, _, payload) = dispatch.encode_source(&source).unwrap();
                payload
            })
            .collect::<Vec<_>>();
        let mut batch_scores = Vec::new();

        let used_batch = dispatch
            .score_ip_bits1_batch_from_payloads(
                &prepared,
                &payloads,
                dispatch.payload_len(),
                &mut batch_scores,
            )
            .unwrap();

        assert!(used_batch);
        assert_eq!(batch_scores.len(), 4);
        for (index, payload) in payloads.chunks_exact(dispatch.payload_len()).enumerate() {
            let scalar = dispatch
                .score_ip_from_parts(&prepared, 0.0, payload)
                .unwrap();
            assert!(
                (batch_scores[index] - scalar).abs() < 1e-6,
                "index={index} batch={} scalar={scalar}",
                batch_scores[index]
            );
        }
    }

    #[test]
    fn rabitq_bits1_batch_dispatch_routes_through_block_kernel() {
        // Same counter-lock requirement as the dispatch test above.
        let _guard = crate::am::common::candidate_batch::CANDIDATE_BATCH_COUNTER_TEST_LOCK
            .lock()
            .unwrap();
        let dimensions = 40;
        let query = unit_vector(dimensions);
        let dispatch = IvfQuantizer::resolve_with_pq_group_size_and_bits(
            StorageFormat::RaBitQ,
            dimensions,
            None,
            Some(1),
        )
        .unwrap();
        let prepared = dispatch.prepare_ip_query(&query).unwrap();
        let payloads = (0..35)
            .flat_map(|row| {
                let source = (0..dimensions)
                    .map(|col| ((row * 3 + col) as f32).sin())
                    .collect::<Vec<_>>();
                let (_, _, payload) = dispatch.encode_source(&source).unwrap();
                payload
            })
            .collect::<Vec<_>>();
        let mut batch_scores = Vec::new();

        let used_batch = dispatch
            .score_ip_bits1_batch_from_payloads(
                &prepared,
                &payloads,
                dispatch.payload_len(),
                &mut batch_scores,
            )
            .unwrap();

        assert!(used_batch);
        assert_eq!(batch_scores.len(), 35);
        let IvfPreparedQuery::RaBitQ(estimator) = &prepared else {
            panic!("RaBitQ profile should prepare a RaBitQ query");
        };
        let block_prepared = estimator
            .bits1_block_prepared(dispatch.payload_len())
            .expect("bits=1 block prepared should exist");

        // First 32 payloads route through the dispatched block kernel;
        // reproduce that call directly so the expectation matches whichever
        // ISA backend this host selects. The 3-payload tail is scalar.
        let code_chunks: Vec<&[u8]> = payloads.chunks_exact(dispatch.payload_len()).collect();
        let block_codes: [&[u8]; 32] = code_chunks[..32].to_vec().try_into().unwrap();
        let mut expected_block_scores = vec![0.0; 32];
        crate::quant::rabitq32::score_rabitq_bits1_block32(
            block_prepared,
            block_codes,
            &mut expected_block_scores,
        );
        for (index, expected) in expected_block_scores.iter().enumerate() {
            assert_eq!(
                batch_scores[index].to_bits(),
                expected.to_bits(),
                "index={index} batch={} kernel={expected}",
                batch_scores[index]
            );
        }
        let mut expected_tail_scores = vec![0.0; code_chunks.len() - 32];
        crate::quant::rabitq32::score_rabitq_bits1_partial(
            block_prepared,
            &code_chunks[32..],
            &mut expected_tail_scores,
        );
        for (index, expected) in expected_tail_scores.iter().enumerate() {
            assert_eq!(
                batch_scores[32 + index].to_bits(),
                expected.to_bits(),
                "tail index={index} batch={} kernel={expected}",
                batch_scores[32 + index]
            );
        }
    }

    #[test]
    fn rabitq_bits2_batch_dispatch_routes_through_block_kernel() {
        // bits=2 has no per-candidate SIMD kernel, so it engages the multi-bit
        // block kernel (a measured win over scalar; Task 106 M5 bench). Holds
        // the counter lock because the dispatch records block-kernel rows.
        let _guard = crate::am::common::candidate_batch::CANDIDATE_BATCH_COUNTER_TEST_LOCK
            .lock()
            .unwrap();
        let dimensions = 48;
        let query = unit_vector(dimensions);
        let dispatch = IvfQuantizer::resolve_with_pq_group_size_and_bits(
            StorageFormat::RaBitQ,
            dimensions,
            None,
            Some(2),
        )
        .unwrap();
        let prepared = dispatch.prepare_ip_query(&query).unwrap();
        let payloads = (0..35)
            .flat_map(|row| {
                let source = (0..dimensions)
                    .map(|col| ((row * 3 + col) as f32).sin())
                    .collect::<Vec<_>>();
                let (_, _, payload) = dispatch.encode_source(&source).unwrap();
                payload
            })
            .collect::<Vec<_>>();
        let mut batch_scores = Vec::new();

        let used_batch = dispatch
            .score_ip_bits1_batch_from_payloads(
                &prepared,
                &payloads,
                dispatch.payload_len(),
                &mut batch_scores,
            )
            .unwrap();

        assert!(used_batch, "bits=2 multi-bit block batch should engage");
        assert_eq!(batch_scores.len(), 35);

        let IvfPreparedQuery::RaBitQ(estimator) = &prepared else {
            panic!("RaBitQ profile should prepare a RaBitQ query");
        };
        let block_prepared = estimator
            .bitsn_block_prepared(dispatch.payload_len())
            .expect("bits=2 block prepared should exist");
        let code_chunks: Vec<&[u8]> = payloads.chunks_exact(dispatch.payload_len()).collect();

        // First 32 route through the dispatched block kernel; reproduce the
        // same call so the expectation matches the host ISA backend.
        let block_codes: [&[u8]; 32] = code_chunks[..32].to_vec().try_into().unwrap();
        let mut expected_block = vec![0.0; 32];
        crate::quant::rabitq32::score_rabitq_bitsn_block32(
            block_prepared,
            block_codes,
            &mut expected_block,
        );
        for (index, expected) in expected_block.iter().enumerate() {
            assert_eq!(
                batch_scores[index].to_bits(),
                expected.to_bits(),
                "index={index}"
            );
        }
        let mut expected_tail = vec![0.0; code_chunks.len() - 32];
        crate::quant::rabitq32::score_rabitq_bitsn_partial(
            block_prepared,
            &code_chunks[32..],
            &mut expected_tail,
        );
        for (index, expected) in expected_tail.iter().enumerate() {
            assert_eq!(
                batch_scores[32 + index].to_bits(),
                expected.to_bits(),
                "tail index={index}"
            );
        }
    }

    #[test]
    fn rabitq_bits4_and_bits8_batch_dispatch_use_arithmetic_estimator_with_width_probe() {
        // bits=4/8 route to the per-candidate arithmetic estimator
        // (estimate_ip_batch -> NeonBits4/8 or Avx2Bits4/8), which the M5
        // bench measured faster than the multi-bit block kernel for bits=4.
        // They still record a width-only IVF/RaBitQ row so bench suites can
        // audit scan flush widths without misattributing block-kernel work.
        let _guard = crate::am::common::candidate_batch::CANDIDATE_BATCH_COUNTER_TEST_LOCK
            .lock()
            .unwrap();
        let dimensions = 48;
        let query = unit_vector(dimensions);

        for bits in [4, 8] {
            crate::am::common::candidate_batch::reset_candidate_batch_scoring_counters();
            let dispatch = IvfQuantizer::resolve_with_pq_group_size_and_bits(
                StorageFormat::RaBitQ,
                dimensions,
                None,
                Some(bits),
            )
            .unwrap();
            let prepared = dispatch.prepare_ip_query(&query).unwrap();
            let payloads = (0..35)
                .flat_map(|row| {
                    let source = (0..dimensions)
                        .map(|col| ((row * 3 + col) as f32).sin())
                        .collect::<Vec<_>>();
                    let (_, _, payload) = dispatch.encode_source(&source).unwrap();
                    payload
                })
                .collect::<Vec<_>>();
            let mut batch_scores = Vec::new();

            let used_batch = dispatch
                .score_ip_bits1_batch_from_payloads(
                    &prepared,
                    &payloads,
                    dispatch.payload_len(),
                    &mut batch_scores,
                )
                .unwrap();

            assert!(used_batch, "bits={bits} batch should engage the estimator");
            assert_eq!(batch_scores.len(), 35);

            let IvfPreparedQuery::RaBitQ(estimator) = &prepared else {
                panic!("RaBitQ profile should prepare a RaBitQ query");
            };
            for (index, payload) in payloads.chunks_exact(dispatch.payload_len()).enumerate() {
                let expected = estimator.estimate_ip_scalar_only(payload);
                let bound = 1e-5_f32 * batch_scores[index].abs().max(expected.abs()).max(1.0);
                assert!(
                    (batch_scores[index] - expected).abs() <= bound,
                    "bits={bits} index={index} batch={} estimate={expected}",
                    batch_scores[index]
                );
            }

            let block_snapshots =
                crate::am::common::candidate_batch::block_kernel_scoring_snapshots();
            let row = block_snapshots
                .iter()
                .find(|snapshot| {
                    snapshot.surface == "ivf"
                        && snapshot.quant_kind == "rabitq"
                        && snapshot.isa == "scalar"
                })
                .expect("arithmetic RaBitQ batch should record a width-only row");
            assert_eq!(row.width_ge32_flushes, 1);
            assert_eq!(row.width_lt8_flushes, 0);
            assert_eq!(row.width_8_15_flushes, 0);
            assert_eq!(row.width_16_31_flushes, 0);
            assert_eq!(row.flushes, 0);
            assert_eq!(row.candidates, 0);
            assert_eq!(row.kernel_flushes, 0);
            assert_eq!(row.scalar_flushes, 0);
        }

        crate::am::common::candidate_batch::reset_candidate_batch_scoring_counters();
    }

    #[test]
    fn pq_fastscan_dispatch_scores_grouped_code_with_persisted_model() {
        let dimensions = 16;
        let source = unit_vector(dimensions);
        let query = unit_vector(dimensions);
        let model = pq_fastscan_test_model(dimensions);
        let dispatch = IvfQuantizer::resolve(StorageFormat::PqFastScan, dimensions).unwrap();
        let (_, gamma, payload) = dispatch
            .encode_source_with_pq_model(&source, &model)
            .unwrap();
        let prepared = dispatch
            .prepare_ip_query_with_pq_model(&query, &model)
            .unwrap();
        let score = dispatch
            .score_ip_from_parts(&prepared, gamma, &payload)
            .unwrap();
        let low_bound_score = dispatch
            .score_ip_from_parts_with_min_bound(&prepared, gamma, &payload, Some(score - 1.0))
            .unwrap();
        let high_bound_score = dispatch
            .score_ip_from_parts_with_min_bound(&prepared, gamma, &payload, Some(score + 1.0))
            .unwrap();

        let IvfPreparedQuery::PqFastScan {
            lut,
            group_count,
            suffix_max,
        } = prepared
        else {
            panic!("expected pq_fastscan prepared query");
        };
        assert_eq!(gamma, 0.0);
        assert_eq!(payload.len(), model.group_count.div_ceil(2));
        assert_eq!(suffix_max.len(), model.group_count + 1);
        assert_eq!(score, grouped_pq_score_f32(&lut, group_count, &payload));
        assert_eq!(low_bound_score, Some(score));
        assert_eq!(high_bound_score, None);
    }

    #[test]
    fn turboquant_no_qjl_lut_dispatch_prunes_with_min_bound() {
        let dimensions = 1536;
        let source = unit_vector(dimensions);
        let query = unit_vector(dimensions);
        let dispatch = IvfQuantizer::resolve(StorageFormat::TurboQuant, dimensions).unwrap();
        let (_, gamma, payload) = dispatch.encode_source(&source).unwrap();
        let prepared = dispatch.prepare_ip_query(&query).unwrap();
        let score = dispatch
            .score_ip_from_parts(&prepared, gamma, &payload)
            .unwrap();

        let retained = dispatch
            .score_ip_from_parts_with_min_bound(&prepared, gamma, &payload, Some(score - 1.0))
            .unwrap();
        let pruned = dispatch
            .score_ip_from_parts_with_min_bound(&prepared, gamma, &payload, Some(score + 1.0))
            .unwrap();

        assert_eq!(retained, Some(score));
        assert_eq!(pruned, None);
    }

    #[test]
    fn turboquant_dispatch_uses_score_bound_pruning() {
        let dispatch = IvfQuantizer::resolve(StorageFormat::TurboQuant, 1536).unwrap();

        assert!(dispatch.uses_score_bound_pruning());
    }

    #[test]
    fn common_quant_codec_scores_grouped_pq_batch_with_prepared_model() {
        let _guard = crate::am::common::candidate_batch::CANDIDATE_BATCH_COUNTER_TEST_LOCK
            .lock()
            .unwrap();
        let dimensions = 16;
        let source = unit_vector(dimensions);
        let query = unit_vector(dimensions);
        let model = pq_fastscan_test_model(dimensions);
        let dispatch = IvfQuantizer::resolve(StorageFormat::PqFastScan, dimensions).unwrap();
        let codec = dispatch.quant_codec_with_pq_model(&model).unwrap();
        let encoded = QuantCodec::encode_source(&codec, &source).unwrap();
        let prepared = QuantCodec::prepare_ip_query(&codec, &query).unwrap();
        let mut batch = CandidateBatch::with_capacity(1);
        batch
            .push(
                0_u32,
                CandidatePayload::new(
                    &encoded.code,
                    CandidateMeta::GroupedPq {
                        group_count: model.group_count,
                    },
                ),
            )
            .unwrap();
        let mut batch_scores = vec![0.0];

        QuantCodec::score_ip_batch(&codec, &prepared, &batch, &mut batch_scores).unwrap();

        assert_eq!(encoded.gamma, 0.0);
        assert_eq!(QuantCodec::codec_kind(&codec), QuantCodecKind::GroupedPq);
        assert_eq!(
            QuantCodec::search_codec_tag(&codec),
            QuantSearchCodecTag::GroupedPq {
                group_count: model.group_count,
                group_size: model.group_size
            }
        );
        assert_eq!(
            batch_scores[0],
            dispatch
                .score_ip_from_parts(&prepared, encoded.gamma, &encoded.code)
                .unwrap()
        );
    }

    #[test]
    fn common_quant_codec_grouped_pq_requires_model_binding() {
        let dimensions = 16;
        let query = unit_vector(dimensions);
        let dispatch = IvfQuantizer::resolve(StorageFormat::PqFastScan, dimensions).unwrap();
        let model = pq_fastscan_test_model(dimensions);

        let err = match QuantCodec::prepare_ip_query(&dispatch, &query) {
            Ok(_) => panic!("unbound grouped-PQ codec should reject query preparation"),
            Err(err) => err,
        };
        assert!(err.contains("persisted grouped codebooks"));

        let codec = dispatch.quant_codec_with_pq_model(&model).unwrap();
        QuantCodec::prepare_ip_query(&codec, &query).unwrap();
    }

    #[test]
    fn common_quant_codec_grouped_pq_batch_is_bit_exact_with_scalar() {
        let _guard = crate::am::common::candidate_batch::CANDIDATE_BATCH_COUNTER_TEST_LOCK
            .lock()
            .unwrap();
        let dimensions = 16;
        let query = unit_vector(dimensions);
        let model = pq_fastscan_test_model(dimensions);
        let dispatch = IvfQuantizer::resolve(StorageFormat::PqFastScan, dimensions).unwrap();
        let codec = dispatch.quant_codec_with_pq_model(&model).unwrap();
        let prepared = QuantCodec::prepare_ip_query(&codec, &query).unwrap();
        let encoded = (0..39)
            .map(|index| {
                let source = (0..dimensions)
                    .map(|col| if (index + col) % 2 == 0 { 0.25 } else { -0.25 })
                    .collect::<Vec<_>>();
                QuantCodec::encode_source(&codec, &source).unwrap()
            })
            .collect::<Vec<_>>();
        let mut batch = CandidateBatch::with_capacity(encoded.len());
        for (index, payload) in encoded.iter().enumerate() {
            batch
                .push(
                    index,
                    CandidatePayload::new(
                        &payload.code,
                        CandidateMeta::GroupedPq {
                            group_count: model.group_count,
                        },
                    ),
                )
                .unwrap();
        }
        let mut batch_scores = vec![0.0; batch.len()];

        QuantCodec::score_ip_batch(&codec, &prepared, &batch, &mut batch_scores).unwrap();

        for (index, payload) in encoded.iter().enumerate() {
            let scalar = dispatch
                .score_ip_from_parts(&prepared, payload.gamma, &payload.code)
                .unwrap();
            assert_eq!(
                batch_scores[index].to_bits(),
                scalar.to_bits(),
                "index={index} batch={} scalar={scalar}",
                batch_scores[index]
            );
        }
    }

    #[test]
    fn pq_fastscan_payload_batch_scores_match_scalar_and_records_counters() {
        let _guard = crate::am::common::candidate_batch::CANDIDATE_BATCH_COUNTER_TEST_LOCK
            .lock()
            .unwrap();
        crate::am::common::candidate_batch::reset_candidate_batch_scoring_counters();

        let dimensions = 16;
        let query = unit_vector(dimensions);
        let model = pq_fastscan_test_model(dimensions);
        let dispatch = IvfQuantizer::resolve(StorageFormat::PqFastScan, dimensions).unwrap();
        let prepared = dispatch
            .prepare_ip_query_with_pq_model(&query, &model)
            .unwrap();
        let encoded = (0..39)
            .map(|index| {
                let source = (0..dimensions)
                    .map(|col| if (index + col) % 2 == 0 { 0.25 } else { -0.25 })
                    .collect::<Vec<_>>();
                dispatch
                    .encode_source_with_pq_model(&source, &model)
                    .unwrap()
            })
            .collect::<Vec<_>>();
        let payloads = encoded
            .iter()
            .flat_map(|(_, _, payload)| payload.iter().copied())
            .collect::<Vec<_>>();
        let mut batch_scores = Vec::new();

        let used_batch = dispatch
            .score_grouped_pq_batch_from_payloads(
                &prepared,
                &payloads,
                dispatch.payload_len(),
                &mut batch_scores,
            )
            .unwrap();

        assert!(used_batch);
        assert_eq!(batch_scores.len(), encoded.len());
        for (index, (_, gamma, payload)) in encoded.iter().enumerate() {
            let scalar = dispatch
                .score_ip_from_parts(&prepared, *gamma, payload)
                .unwrap();
            assert_eq!(
                batch_scores[index].to_bits(),
                scalar.to_bits(),
                "index={index} batch={} scalar={scalar}",
                batch_scores[index]
            );
        }

        let snapshots = crate::am::common::candidate_batch::block_kernel_scoring_snapshots();
        let grouped_pq = snapshots
            .iter()
            .filter(|snapshot| snapshot.surface == "ivf" && snapshot.quant_kind == "grouped_pq")
            .collect::<Vec<_>>();
        let kernel_candidates = grouped_pq
            .iter()
            .map(|snapshot| snapshot.kernel_candidates)
            .sum::<u64>();
        let scalar_candidates = grouped_pq
            .iter()
            .map(|snapshot| snapshot.scalar_candidates)
            .sum::<u64>();
        assert_eq!(kernel_candidates + scalar_candidates, 39);
        assert!(kernel_candidates >= 32);
        if grouped_pq.iter().any(|snapshot| snapshot.isa != "scalar") {
            assert_eq!(kernel_candidates, 39);
            assert_eq!(scalar_candidates, 0);
        } else {
            assert_eq!(kernel_candidates, 32);
            assert_eq!(scalar_candidates, 7);
        }

        crate::am::common::candidate_batch::reset_candidate_batch_scoring_counters();
    }

    #[test]
    fn common_quant_codec_grouped_pq_cutoff_prunes_through_trait() {
        let dimensions = 16;
        let source = unit_vector(dimensions);
        let query = unit_vector(dimensions);
        let model = pq_fastscan_test_model(dimensions);
        let dispatch = IvfQuantizer::resolve(StorageFormat::PqFastScan, dimensions).unwrap();
        let codec = dispatch.quant_codec_with_pq_model(&model).unwrap();
        let encoded = QuantCodec::encode_source(&codec, &source).unwrap();
        let prepared = QuantCodec::prepare_ip_query(&codec, &query).unwrap();
        let payload = CandidatePayload::new(
            &encoded.code,
            CandidateMeta::GroupedPq {
                group_count: model.group_count,
            },
        );
        let expected = QuantCodec::score_ip_candidate(&codec, &prepared, payload).unwrap();

        let kept = QuantCodec::try_score_ip_candidate(
            &codec,
            &prepared,
            CandidatePayload::new(
                &encoded.code,
                CandidateMeta::GroupedPq {
                    group_count: model.group_count,
                },
            ),
            Some(expected - 1.0),
        )
        .unwrap();
        let pruned = QuantCodec::try_score_ip_candidate(
            &codec,
            &prepared,
            CandidatePayload::new(
                &encoded.code,
                CandidateMeta::GroupedPq {
                    group_count: model.group_count,
                },
            ),
            Some(expected + 1.0),
        )
        .unwrap();

        assert_eq!(kept, Some(expected));
        assert_eq!(pruned, None);
    }

    #[test]
    fn common_quant_codec_pq_fastscan_batch_is_bit_exact_with_direct_path() {
        let _guard = crate::am::common::candidate_batch::CANDIDATE_BATCH_COUNTER_TEST_LOCK
            .lock()
            .unwrap();
        let dimensions = 16;
        let query = unit_vector(dimensions);
        let model = pq_fastscan_test_model(dimensions);
        let dispatch = IvfQuantizer::resolve(StorageFormat::PqFastScan, dimensions).unwrap();
        let codec = dispatch.quant_codec_with_pq_model(&model).unwrap();
        let trait_prepared = QuantCodec::prepare_ip_query(&codec, &query).unwrap();
        let direct_prepared = dispatch
            .prepare_ip_query_with_pq_model(&query, &model)
            .unwrap();
        let sources = (0..3)
            .map(|index| {
                (0..dimensions)
                    .map(|col| if (index + col) % 2 == 0 { 0.25 } else { -0.25 })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        let trait_encoded = sources
            .iter()
            .map(|source| QuantCodec::encode_source(&codec, source).unwrap())
            .collect::<Vec<_>>();
        let direct_encoded = sources
            .iter()
            .map(|source| {
                dispatch
                    .encode_source_with_pq_model(source, &model)
                    .unwrap()
            })
            .collect::<Vec<_>>();
        let mut batch = CandidateBatch::with_capacity(sources.len());
        for (index, payload) in trait_encoded.iter().enumerate() {
            batch
                .push(
                    index,
                    CandidatePayload::new(
                        &payload.code,
                        CandidateMeta::GroupedPq {
                            group_count: model.group_count,
                        },
                    ),
                )
                .unwrap();
        }
        let mut batch_scores = vec![0.0; batch.len()];

        QuantCodec::score_ip_batch(&codec, &trait_prepared, &batch, &mut batch_scores).unwrap();

        for (index, payload) in trait_encoded.iter().enumerate() {
            let (direct_dimensions, direct_gamma, direct_code) = &direct_encoded[index];
            assert_eq!(payload.dimensions, *direct_dimensions);
            assert_eq!(payload.gamma.to_bits(), direct_gamma.to_bits());
            assert_eq!(payload.code.as_slice(), direct_code.as_slice());
            let direct_score = dispatch
                .score_ip_from_parts(&direct_prepared, *direct_gamma, direct_code)
                .unwrap();
            assert_eq!(
                batch_scores[index].to_bits(),
                direct_score.to_bits(),
                "index={index} batch={} direct={direct_score}",
                batch_scores[index]
            );
        }
    }

    #[test]
    fn common_quant_codec_grouped_pq_rejects_mismatched_candidate_meta() {
        let _guard = crate::am::common::candidate_batch::CANDIDATE_BATCH_COUNTER_TEST_LOCK
            .lock()
            .unwrap();
        let dimensions = 16;
        let source = unit_vector(dimensions);
        let query = unit_vector(dimensions);
        let model = pq_fastscan_test_model(dimensions);
        let dispatch = IvfQuantizer::resolve(StorageFormat::PqFastScan, dimensions).unwrap();
        let codec = dispatch.quant_codec_with_pq_model(&model).unwrap();
        let encoded = QuantCodec::encode_source(&codec, &source).unwrap();
        let prepared = QuantCodec::prepare_ip_query(&codec, &query).unwrap();
        let mut batch = CandidateBatch::with_capacity(1);
        batch
            .push(
                0_u32,
                CandidatePayload::new(
                    &encoded.code,
                    CandidateMeta::GroupedPq {
                        group_count: model.group_count + 1,
                    },
                ),
            )
            .unwrap();
        let mut batch_scores = vec![0.0];

        let err =
            QuantCodec::score_ip_batch(&codec, &prepared, &batch, &mut batch_scores).unwrap_err();

        assert!(err.contains("candidate group count mismatch"));
    }

    #[test]
    fn grouped_pq_score_bound_prunes_when_suffix_cannot_reach_minimum() {
        let group_count = 2;
        let mut lut = vec![0.0_f32; group_count * GROUPED_PQ_CENTROIDS];
        lut[1] = 0.25;
        lut[GROUPED_PQ_CENTROIDS + 2] = 0.5;
        lut[GROUPED_PQ_CENTROIDS + 3] = 2.0;
        let suffix_max = grouped_pq_suffix_max(&lut, group_count);
        let payload = crate::quant::grouped_pq::pack_grouped_pq_nibbles(&[1, 2]);

        assert_eq!(
            grouped_pq_score_f32_with_min_bound(&lut, &suffix_max, group_count, &payload, 0.7),
            Some(0.75)
        );
        assert_eq!(
            grouped_pq_score_f32_with_min_bound(&lut, &suffix_max, group_count, &payload, 0.8),
            None
        );
    }
}
