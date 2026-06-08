use std::sync::Arc;

use super::assign::SpireLeafAssignmentInput;
use super::storage::{
    SpireLeafAssignmentRow, SPIRE_PAYLOAD_FORMAT_PQ_FASTSCAN, SPIRE_PAYLOAD_FORMAT_RABITQ,
    SPIRE_PAYLOAD_FORMAT_TURBOQUANT,
};
use crate::am::common::candidate_batch::{
    score_turboquant_no_qjl_4bit_batch_for, CandidateBatch, CandidateBatchScoringSurface,
    CandidateMeta, CandidatePayload,
};
use crate::am::common::quant_codec::{
    EncodedQuantPayload, QuantCodec, QuantCodecKind, QuantSearchCodecTag,
};
use crate::quant::prod::{
    payload_len, ExactScoreMode, PreparedLutNoQjl4BitQuery, PreparedQuery, ProdQuantizer,
};
use crate::quant::rabitq::{code_len_for, PreparedEstimator, RaBitQQuantizer};
use crate::quant::Quantizer;
use crate::storage::page::ItemPointer;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SpireAssignmentPayloadFormat {
    TurboQuant,
    PqFastScan,
    RaBitQ,
}

impl SpireAssignmentPayloadFormat {
    pub(super) fn from_tag(payload_format: u8) -> Result<Self, String> {
        match payload_format {
            SPIRE_PAYLOAD_FORMAT_TURBOQUANT => Ok(Self::TurboQuant),
            SPIRE_PAYLOAD_FORMAT_PQ_FASTSCAN => Ok(Self::PqFastScan),
            SPIRE_PAYLOAD_FORMAT_RABITQ => Ok(Self::RaBitQ),
            other => Err(format!(
                "ec_spire assignment payload format {other} is not scoreable"
            )),
        }
    }

    pub(super) fn tag(self) -> u8 {
        match self {
            Self::TurboQuant => SPIRE_PAYLOAD_FORMAT_TURBOQUANT,
            Self::PqFastScan => SPIRE_PAYLOAD_FORMAT_PQ_FASTSCAN,
            Self::RaBitQ => SPIRE_PAYLOAD_FORMAT_RABITQ,
        }
    }
}

pub(super) enum SpirePreparedAssignmentScorer {
    TurboQuant {
        dimensions: usize,
        query_l2_norm: f32,
        quantizer: Arc<ProdQuantizer>,
        prepared: PreparedQuery,
        no_qjl_4bit_lut: Option<PreparedLutNoQjl4BitQuery>,
    },
    RaBitQ {
        dimensions: usize,
        query_l2_norm: f32,
        prepared: PreparedEstimator,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct SpireAssignmentQuantCodec {
    payload_format: SpireAssignmentPayloadFormat,
    dimensions: usize,
}

impl SpireAssignmentQuantCodec {
    pub(super) fn new(payload_format: SpireAssignmentPayloadFormat, dimensions: usize) -> Self {
        Self {
            payload_format,
            dimensions,
        }
    }
}

impl SpirePreparedAssignmentScorer {
    pub(super) fn prepare(
        payload_format: SpireAssignmentPayloadFormat,
        dimensions: usize,
        query_vector: &[f32],
    ) -> Result<Self, String> {
        validate_vector_shape("query", dimensions, query_vector)?;
        let query_l2_norm = query_vector
            .iter()
            .map(|value| value * value)
            .sum::<f32>()
            .sqrt();
        match payload_format {
            SpireAssignmentPayloadFormat::TurboQuant => {
                let quantizer = ProdQuantizer::cached(
                    dimensions,
                    crate::DEFAULT_QUANT_BITS,
                    crate::DEFAULT_QUANT_SEED,
                );
                let prepared = quantizer.prepare_ip_query(query_vector);
                let no_qjl_4bit_lut = (quantizer.exact_score_mode()
                    == ExactScoreMode::MseNoQjl4Bit)
                    .then(|| quantizer.prepare_ip_query_lut_no_qjl_4bit(query_vector));
                Ok(Self::TurboQuant {
                    dimensions,
                    query_l2_norm,
                    quantizer,
                    prepared,
                    no_qjl_4bit_lut,
                })
            }
            SpireAssignmentPayloadFormat::RaBitQ => {
                let quantizer = RaBitQQuantizer::cached_seeded_srht_bits(
                    dimensions,
                    crate::DEFAULT_QUANT_SEED,
                    crate::DEFAULT_QUANT_BITS,
                )?;
                let prepared = quantizer.prepare_estimator(query_vector);
                Ok(Self::RaBitQ {
                    dimensions,
                    query_l2_norm,
                    prepared,
                })
            }
            SpireAssignmentPayloadFormat::PqFastScan => {
                Err("ec_spire PQ-FastScan scoring requires a persisted grouped-PQ model".to_owned())
            }
        }
    }

    pub(super) fn payload_format(&self) -> SpireAssignmentPayloadFormat {
        match self {
            Self::TurboQuant { .. } => SpireAssignmentPayloadFormat::TurboQuant,
            Self::RaBitQ { .. } => SpireAssignmentPayloadFormat::RaBitQ,
        }
    }

    pub(super) fn dimensions(&self) -> usize {
        match self {
            Self::TurboQuant { dimensions, .. } | Self::RaBitQ { dimensions, .. } => *dimensions,
        }
    }

    pub(super) fn query_l2_norm(&self) -> f32 {
        match self {
            Self::TurboQuant { query_l2_norm, .. } | Self::RaBitQ { query_l2_norm, .. } => {
                *query_l2_norm
            }
        }
    }

    pub(super) fn payload_stride(&self) -> Result<usize, String> {
        expected_payload_len(self.dimensions(), self.payload_format())
    }

    pub(super) fn score_assignment_ip(
        &self,
        assignment: &SpireLeafAssignmentRow,
    ) -> Result<f32, String> {
        let assignment_format = SpireAssignmentPayloadFormat::from_tag(assignment.payload_format)?;
        if assignment_format != self.payload_format() {
            return Err(format!(
                "ec_spire assignment payload format {:?} does not match prepared scorer {:?}",
                assignment_format,
                self.payload_format()
            ));
        }

        self.score_payload_ip(
            assignment_format,
            assignment.gamma,
            &assignment.encoded_payload,
        )
    }

    pub(super) fn try_score_assignment_ip(
        &self,
        assignment: &SpireLeafAssignmentRow,
        min_ip_to_keep: f32,
    ) -> Result<Option<f32>, String> {
        let assignment_format = SpireAssignmentPayloadFormat::from_tag(assignment.payload_format)?;
        if assignment_format != self.payload_format() {
            return Err(format!(
                "ec_spire assignment payload format {:?} does not match prepared scorer {:?}",
                assignment_format,
                self.payload_format()
            ));
        }

        self.try_score_payload_ip(
            assignment_format,
            assignment.gamma,
            &assignment.encoded_payload,
            min_ip_to_keep,
        )
    }

    pub(super) fn score_payload_ip(
        &self,
        payload_format: SpireAssignmentPayloadFormat,
        gamma: f32,
        encoded_payload: &[u8],
    ) -> Result<f32, String> {
        match self {
            Self::TurboQuant {
                dimensions,
                quantizer,
                prepared,
                no_qjl_4bit_lut,
                ..
            } => {
                validate_payload_len(*dimensions, payload_format, encoded_payload)?;
                if let Some(prepared_lut) = no_qjl_4bit_lut {
                    // The no-QJL 4-bit lane has no residual sign payload, so gamma is not
                    // part of the exact score. Keep the generic scorer as the fallback for
                    // modes that still carry a QJL residual term.
                    Ok(
                        quantizer
                            .score_ip_from_parts_lut_no_qjl_4bit(prepared_lut, encoded_payload),
                    )
                } else {
                    Ok(quantizer.score_ip_from_parts(prepared, gamma, encoded_payload))
                }
            }
            Self::RaBitQ {
                dimensions,
                prepared,
                ..
            } => {
                validate_payload_len(*dimensions, payload_format, encoded_payload)?;
                if gamma != 0.0 {
                    return Err("ec_spire RaBitQ assignment gamma must be 0".to_owned());
                }
                Ok(prepared.estimate_ip_scalar_only(encoded_payload))
            }
        }
    }

    pub(super) fn score_zero_gamma_payload_chunks_max_prevalidated(
        &self,
        payload_stride: usize,
        encoded_payload: &[u8],
    ) -> f32 {
        debug_assert!(payload_stride > 0);
        debug_assert!(!encoded_payload.is_empty());
        debug_assert_eq!(encoded_payload.len() % payload_stride, 0);

        if encoded_payload.len() == payload_stride {
            return self.score_zero_gamma_payload_prevalidated(encoded_payload);
        }

        match self {
            Self::TurboQuant {
                quantizer,
                prepared,
                no_qjl_4bit_lut,
                ..
            } => encoded_payload
                .chunks_exact(payload_stride)
                .map(|payload| {
                    if let Some(prepared_lut) = no_qjl_4bit_lut {
                        // The no-QJL 4-bit lane ignores gamma by construction.
                        quantizer.score_ip_from_parts_lut_no_qjl_4bit(prepared_lut, payload)
                    } else {
                        quantizer.score_ip_from_parts(prepared, 0.0, payload)
                    }
                })
                .fold(f32::NEG_INFINITY, f32::max),
            Self::RaBitQ { prepared, .. } => {
                if matches!(prepared.bits_per_dim(), 1 | 4 | 8) {
                    prepared.estimate_ip_batch_max_prevalidated(encoded_payload, payload_stride)
                } else {
                    encoded_payload
                        .chunks_exact(payload_stride)
                        .map(|payload| prepared.estimate_ip_scalar_only(payload))
                        .fold(f32::NEG_INFINITY, f32::max)
                }
            }
        }
    }

    pub(super) fn score_zero_gamma_payload_prevalidated(&self, encoded_payload: &[u8]) -> f32 {
        debug_assert!(!encoded_payload.is_empty());

        match self {
            Self::TurboQuant {
                quantizer,
                prepared,
                ..
            } => quantizer.score_ip_from_parts(prepared, 0.0, encoded_payload),
            Self::RaBitQ { prepared, .. } => prepared.estimate_ip_scalar_only(encoded_payload),
        }
    }

    pub(super) fn try_score_payload_ip(
        &self,
        payload_format: SpireAssignmentPayloadFormat,
        gamma: f32,
        encoded_payload: &[u8],
        min_ip_to_keep: f32,
    ) -> Result<Option<f32>, String> {
        match self {
            Self::TurboQuant { .. } => self
                .score_payload_ip(payload_format, gamma, encoded_payload)
                .map(Some),
            Self::RaBitQ {
                dimensions,
                prepared,
                ..
            } => {
                validate_payload_len(*dimensions, payload_format, encoded_payload)?;
                if gamma != 0.0 {
                    return Err("ec_spire RaBitQ assignment gamma must be 0".to_owned());
                }
                Ok(prepared.try_estimate_ip_scalar(encoded_payload, min_ip_to_keep))
            }
        }
    }

    pub(super) fn score_batch_ip(
        &self,
        payload_stride: usize,
        payloads: &[u8],
        gammas: &[f32],
        out_scores: &mut [f32],
    ) -> Result<(), String> {
        if gammas.len() != out_scores.len() {
            return Err(format!(
                "ec_spire batch scorer gamma count {} does not match output count {}",
                gammas.len(),
                out_scores.len()
            ));
        }
        let payload_count = out_scores.len();
        let expected_payload_bytes = payload_stride
            .checked_mul(payload_count)
            .ok_or_else(|| "ec_spire batch scorer payload byte count overflow".to_owned())?;
        if payloads.len() != expected_payload_bytes {
            return Err(format!(
                "ec_spire batch scorer payload bytes mismatch: got {}, expected {expected_payload_bytes}",
                payloads.len()
            ));
        }

        let payload_format = self.payload_format();
        validate_payload_stride(self.dimensions(), payload_format, payload_stride)?;
        match self {
            Self::TurboQuant {
                quantizer,
                prepared,
                no_qjl_4bit_lut,
                ..
            } => {
                if super::options::candidate_batch_scoring_enabled() {
                    if let Some(prepared_lut) = no_qjl_4bit_lut {
                        let mut batch = CandidateBatch::with_capacity(payload_count);
                        for (candidate_index, (payload, gamma)) in payloads
                            .chunks_exact(payload_stride)
                            .zip(gammas.iter())
                            .enumerate()
                        {
                            batch.push(
                                candidate_index,
                                CandidatePayload::new(payload, CandidateMeta::Gamma(*gamma)),
                            )?;
                        }
                        return score_turboquant_no_qjl_4bit_batch_for(
                            CandidateBatchScoringSurface::Spire,
                            quantizer,
                            prepared_lut,
                            &batch,
                            out_scores,
                        );
                    }
                }

                for ((payload, gamma), out_score) in payloads
                    .chunks_exact(payload_stride)
                    .zip(gammas.iter())
                    .zip(out_scores.iter_mut())
                {
                    *out_score = quantizer.score_ip_from_parts(prepared, *gamma, payload);
                }
            }
            Self::RaBitQ { prepared, .. } => {
                for ((payload, gamma), out_score) in payloads
                    .chunks_exact(payload_stride)
                    .zip(gammas.iter())
                    .zip(out_scores.iter_mut())
                {
                    if *gamma != 0.0 {
                        return Err("ec_spire RaBitQ assignment gamma must be 0".to_owned());
                    }
                    *out_score = prepared.estimate_ip_scalar_only(payload);
                }
            }
        }
        Ok(())
    }

    pub(super) fn score_candidate_batch_ip<Id>(
        &self,
        batch: &CandidateBatch<'_, Id>,
        out_scores: &mut [f32],
    ) -> Result<(), String> {
        if batch.len() != out_scores.len() {
            return Err(format!(
                "ec_spire candidate batch scorer output count {} does not match candidate count {}",
                out_scores.len(),
                batch.len()
            ));
        }

        match self {
            Self::TurboQuant {
                dimensions,
                quantizer,
                prepared,
                no_qjl_4bit_lut,
                ..
            } => {
                for payload in batch.payloads() {
                    validate_payload_stride(
                        *dimensions,
                        self.payload_format(),
                        payload.code.len(),
                    )?;
                }
                if super::options::candidate_batch_scoring_enabled() {
                    if let Some(prepared_lut) = no_qjl_4bit_lut {
                        return score_turboquant_no_qjl_4bit_batch_for(
                            CandidateBatchScoringSurface::Spire,
                            quantizer,
                            prepared_lut,
                            batch,
                            out_scores,
                        );
                    }
                }

                for (payload, out_score) in batch.payloads().iter().zip(out_scores.iter_mut()) {
                    let gamma = match payload.meta {
                        CandidateMeta::None
                        | CandidateMeta::Binary
                        | CandidateMeta::RaBitQ
                        | CandidateMeta::GroupedPq { .. } => 0.0,
                        CandidateMeta::Gamma(gamma) => gamma,
                        CandidateMeta::GammaAndResidualSigns { gamma, .. } => gamma,
                    };
                    *out_score = quantizer.score_ip_from_parts(prepared, gamma, payload.code);
                }
            }
            Self::RaBitQ { .. } => {
                for (payload, out_score) in batch.payloads().iter().zip(out_scores.iter_mut()) {
                    let gamma = match payload.meta {
                        CandidateMeta::None
                        | CandidateMeta::Binary
                        | CandidateMeta::RaBitQ
                        | CandidateMeta::GroupedPq { .. } => 0.0,
                        CandidateMeta::Gamma(gamma) => gamma,
                        CandidateMeta::GammaAndResidualSigns { gamma, .. } => gamma,
                    };
                    *out_score =
                        self.score_payload_ip(self.payload_format(), gamma, payload.code)?;
                }
            }
        }
        Ok(())
    }
}

impl QuantCodec for SpireAssignmentQuantCodec {
    type PreparedQuery = SpirePreparedAssignmentScorer;

    fn codec_kind(&self) -> QuantCodecKind {
        match self.payload_format {
            SpireAssignmentPayloadFormat::TurboQuant => QuantCodecKind::TurboQuant,
            SpireAssignmentPayloadFormat::PqFastScan => QuantCodecKind::GroupedPq,
            SpireAssignmentPayloadFormat::RaBitQ => QuantCodecKind::RaBitQ,
        }
    }

    fn search_codec_tag(&self) -> QuantSearchCodecTag {
        match self.payload_format {
            SpireAssignmentPayloadFormat::TurboQuant => QuantSearchCodecTag::TurboQuant,
            SpireAssignmentPayloadFormat::PqFastScan => QuantSearchCodecTag::GroupedPq {
                group_count: 0,
                group_size: 0,
            },
            SpireAssignmentPayloadFormat::RaBitQ => QuantSearchCodecTag::RaBitQ {
                bits: crate::DEFAULT_QUANT_BITS,
            },
        }
    }

    fn payload_len(&self) -> usize {
        expected_payload_len(self.dimensions, self.payload_format).unwrap_or(0)
    }

    fn encode_source(&self, source: &[f32]) -> Result<EncodedQuantPayload, String> {
        validate_vector_shape("source", self.dimensions, source)?;
        let dimensions = u16::try_from(source.len()).map_err(|_| {
            format!(
                "ec_spire source vector dimension {} exceeds maximum 65535",
                source.len()
            )
        })?;
        let (gamma, code) = encode_assignment_payload(self.payload_format, source)?;
        Ok(EncodedQuantPayload {
            dimensions,
            gamma,
            code,
        })
    }

    fn prepare_ip_query(&self, query: &[f32]) -> Result<Self::PreparedQuery, String> {
        SpirePreparedAssignmentScorer::prepare(self.payload_format, self.dimensions, query)
    }

    fn score_ip_candidate(
        &self,
        prepared_query: &Self::PreparedQuery,
        payload: CandidatePayload<'_>,
    ) -> Result<f32, String> {
        let gamma = match payload.meta {
            CandidateMeta::None
            | CandidateMeta::Binary
            | CandidateMeta::RaBitQ
            | CandidateMeta::GroupedPq { .. } => 0.0,
            CandidateMeta::Gamma(gamma) => gamma,
            CandidateMeta::GammaAndResidualSigns { gamma, .. } => gamma,
        };
        prepared_query.score_payload_ip(self.payload_format, gamma, payload.code)
    }

    fn score_ip_batch<Id>(
        &self,
        prepared_query: &Self::PreparedQuery,
        batch: &CandidateBatch<'_, Id>,
        out_scores: &mut [f32],
    ) -> Result<(), String> {
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

pub(super) fn encode_assignment_payload(
    payload_format: SpireAssignmentPayloadFormat,
    source_vector: &[f32],
) -> Result<(f32, Vec<u8>), String> {
    validate_vector_shape("source", source_vector.len(), source_vector)?;
    u16::try_from(source_vector.len()).map_err(|_| {
        format!(
            "ec_spire source vector dimension {} exceeds maximum 65535",
            source_vector.len()
        )
    })?;

    match payload_format {
        SpireAssignmentPayloadFormat::TurboQuant => {
            let quantizer = ProdQuantizer::cached(
                source_vector.len(),
                crate::DEFAULT_QUANT_BITS,
                crate::DEFAULT_QUANT_SEED,
            );
            let encoded = quantizer.encode(source_vector);
            let mut payload = encoded.mse_packed;
            payload.extend_from_slice(&encoded.qjl_packed);
            Ok((encoded.gamma, payload))
        }
        SpireAssignmentPayloadFormat::RaBitQ => {
            let quantizer = RaBitQQuantizer::cached_seeded_srht_bits(
                source_vector.len(),
                crate::DEFAULT_QUANT_SEED,
                crate::DEFAULT_QUANT_BITS,
            )?;
            Ok((
                0.0,
                Quantizer::encode_code(&*quantizer, source_vector).into_vec(),
            ))
        }
        SpireAssignmentPayloadFormat::PqFastScan => {
            Err("ec_spire PQ-FastScan encoding requires a persisted grouped-PQ model".to_owned())
        }
    }
}

pub(super) fn encode_assignment_input(
    payload_format: SpireAssignmentPayloadFormat,
    heap_tid: ItemPointer,
    source_vector: &[f32],
) -> Result<SpireLeafAssignmentInput, String> {
    if heap_tid == ItemPointer::INVALID {
        return Err("ec_spire assignment input heap_tid must be valid".to_owned());
    }
    let (gamma, encoded_payload) = encode_assignment_payload(payload_format, source_vector)?;
    Ok(SpireLeafAssignmentInput {
        heap_tid,
        payload_format: payload_format.tag(),
        gamma,
        encoded_payload,
    })
}

fn validate_vector_shape(label: &str, dimensions: usize, vector: &[f32]) -> Result<(), String> {
    if dimensions == 0 {
        return Err(format!("ec_spire {label} vector dimensions must be > 0"));
    }
    if vector.len() != dimensions {
        return Err(format!(
            "ec_spire {label} vector dimension mismatch: got {}, expected {dimensions}",
            vector.len()
        ));
    }
    if vector.iter().any(|value| !value.is_finite()) {
        return Err(format!(
            "ec_spire {label} vector contains a non-finite value"
        ));
    }
    Ok(())
}

fn validate_payload_len(
    dimensions: usize,
    payload_format: SpireAssignmentPayloadFormat,
    payload: &[u8],
) -> Result<(), String> {
    let expected_len = expected_payload_len(dimensions, payload_format)?;
    if payload.len() != expected_len {
        return Err(format!(
            "ec_spire {:?} assignment payload length mismatch: got {}, expected {expected_len}",
            payload_format,
            payload.len()
        ));
    }
    Ok(())
}

fn validate_payload_stride(
    dimensions: usize,
    payload_format: SpireAssignmentPayloadFormat,
    payload_stride: usize,
) -> Result<(), String> {
    let expected_len = expected_payload_len(dimensions, payload_format)?;
    if payload_stride != expected_len {
        return Err(format!(
            "ec_spire {:?} assignment payload stride mismatch: got {payload_stride}, expected {expected_len}",
            payload_format
        ));
    }
    Ok(())
}

fn expected_payload_len(
    dimensions: usize,
    payload_format: SpireAssignmentPayloadFormat,
) -> Result<usize, String> {
    Ok(match payload_format {
        SpireAssignmentPayloadFormat::TurboQuant => {
            payload_len(dimensions, crate::DEFAULT_QUANT_BITS) - size_of::<f32>()
        }
        SpireAssignmentPayloadFormat::RaBitQ => code_len_for(dimensions, crate::DEFAULT_QUANT_BITS)
            .expect("default RaBitQ configuration should be valid"),
        SpireAssignmentPayloadFormat::PqFastScan => {
            return Err(
                "ec_spire PQ-FastScan payload length requires a persisted grouped-PQ model"
                    .to_owned(),
            );
        }
    })
}

include!("tests.rs");
