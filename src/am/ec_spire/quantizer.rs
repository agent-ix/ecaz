use std::sync::Arc;

use super::assign::SpireLeafAssignmentInput;
use super::storage::{
    SpireLeafAssignmentRow, SPIRE_PAYLOAD_FORMAT_PQ_FASTSCAN, SPIRE_PAYLOAD_FORMAT_RABITQ,
    SPIRE_PAYLOAD_FORMAT_TURBOQUANT,
};
use crate::quant::prod::{payload_len, PreparedQuery, ProdQuantizer};
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
        quantizer: Arc<ProdQuantizer>,
        prepared: PreparedQuery,
    },
    RaBitQ {
        dimensions: usize,
        quantizer: Arc<RaBitQQuantizer>,
        prepared: PreparedEstimator,
    },
}

impl SpirePreparedAssignmentScorer {
    pub(super) fn prepare(
        payload_format: SpireAssignmentPayloadFormat,
        dimensions: usize,
        query_vector: &[f32],
    ) -> Result<Self, String> {
        validate_vector_shape("query", dimensions, query_vector)?;
        match payload_format {
            SpireAssignmentPayloadFormat::TurboQuant => {
                let quantizer = ProdQuantizer::cached(
                    dimensions,
                    crate::DEFAULT_QUANT_BITS,
                    crate::DEFAULT_QUANT_SEED,
                );
                let prepared = quantizer.prepare_ip_query(query_vector);
                Ok(Self::TurboQuant {
                    dimensions,
                    quantizer,
                    prepared,
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
                    quantizer,
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

        match self {
            Self::TurboQuant {
                dimensions,
                quantizer,
                prepared,
            } => {
                validate_payload_len(*dimensions, assignment_format, &assignment.encoded_payload)?;
                Ok(quantizer.score_ip_from_parts(
                    prepared,
                    assignment.gamma,
                    &assignment.encoded_payload,
                ))
            }
            Self::RaBitQ {
                dimensions,
                quantizer,
                prepared,
            } => {
                validate_payload_len(*dimensions, assignment_format, &assignment.encoded_payload)?;
                if assignment.gamma != 0.0 {
                    return Err("ec_spire RaBitQ assignment gamma must be 0".to_owned());
                }
                Ok(quantizer
                    .estimate_ip(prepared, &assignment.encoded_payload)
                    .estimate)
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
                ..
            } => {
                for ((payload, gamma), out_score) in payloads
                    .chunks_exact(payload_stride)
                    .zip(gammas.iter())
                    .zip(out_scores.iter_mut())
                {
                    *out_score = quantizer.score_ip_from_parts(prepared, *gamma, payload);
                }
            }
            Self::RaBitQ {
                quantizer,
                prepared,
                ..
            } => {
                for ((payload, gamma), out_score) in payloads
                    .chunks_exact(payload_stride)
                    .zip(gammas.iter())
                    .zip(out_scores.iter_mut())
                {
                    if *gamma != 0.0 {
                        return Err("ec_spire RaBitQ assignment gamma must be 0".to_owned());
                    }
                    *out_score = quantizer.estimate_ip(prepared, payload).estimate;
                }
            }
        }
        Ok(())
    }
}

pub(super) fn encode_assignment_payload(
    payload_format: SpireAssignmentPayloadFormat,
    source_vector: &[f32],
) -> Result<(u16, f32, Vec<u8>), String> {
    validate_vector_shape("source", source_vector.len(), source_vector)?;
    let dimensions = u16::try_from(source_vector.len()).map_err(|_| {
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
            Ok((dimensions, encoded.gamma, payload))
        }
        SpireAssignmentPayloadFormat::RaBitQ => {
            let quantizer = RaBitQQuantizer::cached_seeded_srht_bits(
                source_vector.len(),
                crate::DEFAULT_QUANT_SEED,
                crate::DEFAULT_QUANT_BITS,
            )?;
            Ok((
                dimensions,
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
    let (_, gamma, encoded_payload) = encode_assignment_payload(payload_format, source_vector)?;
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

#[cfg(test)]
mod tests {
    use super::{
        encode_assignment_input, encode_assignment_payload, SpireAssignmentPayloadFormat,
        SpirePreparedAssignmentScorer,
    };
    use crate::am::ec_spire::storage::{
        SpireLeafAssignmentRow, SPIRE_ASSIGNMENT_FLAG_PRIMARY, SPIRE_PAYLOAD_FORMAT_NONE,
        SPIRE_PAYLOAD_FORMAT_RABITQ, SPIRE_PAYLOAD_FORMAT_TURBOQUANT,
    };
    use crate::quant::prod::ProdQuantizer;
    use crate::quant::rabitq::RaBitQQuantizer;
    use crate::storage::page::ItemPointer;

    fn tid(block_number: u32, offset_number: u16) -> ItemPointer {
        ItemPointer {
            block_number,
            offset_number,
        }
    }

    fn assignment_row(
        payload_format: SpireAssignmentPayloadFormat,
        gamma: f32,
        encoded_payload: Vec<u8>,
    ) -> SpireLeafAssignmentRow {
        SpireLeafAssignmentRow {
            flags: SPIRE_ASSIGNMENT_FLAG_PRIMARY,
            vec_id: crate::am::ec_spire::storage::SpireVecId::local(1),
            heap_tid: tid(10, 1),
            payload_format: payload_format.tag(),
            gamma,
            encoded_payload,
        }
    }

    #[test]
    fn turboquant_assignment_scorer_matches_direct_quantizer_score() {
        let source = vec![0.25, -0.5, 0.75, 1.0];
        let query = vec![1.0, 0.5, -0.25, 0.125];
        let (dimensions, gamma, payload) =
            encode_assignment_payload(SpireAssignmentPayloadFormat::TurboQuant, &source).unwrap();
        let assignment = assignment_row(
            SpireAssignmentPayloadFormat::TurboQuant,
            gamma,
            payload.clone(),
        );
        let scorer = SpirePreparedAssignmentScorer::prepare(
            SpireAssignmentPayloadFormat::TurboQuant,
            usize::from(dimensions),
            &query,
        )
        .unwrap();
        let quantizer = ProdQuantizer::cached(
            source.len(),
            crate::DEFAULT_QUANT_BITS,
            crate::DEFAULT_QUANT_SEED,
        );
        let prepared = quantizer.prepare_ip_query(&query);
        let expected = quantizer.score_ip_from_parts(&prepared, gamma, &payload);

        let observed = scorer.score_assignment_ip(&assignment).unwrap();

        assert_eq!(scorer.dimensions(), source.len());
        assert_eq!(assignment.payload_format, SPIRE_PAYLOAD_FORMAT_TURBOQUANT);
        assert!((observed - expected).abs() <= f32::EPSILON);
    }

    #[test]
    fn rabitq_assignment_scorer_matches_direct_quantizer_score() {
        let source = vec![0.25, -0.5, 0.75, 1.0];
        let query = vec![1.0, 0.5, -0.25, 0.125];
        let (dimensions, gamma, payload) =
            encode_assignment_payload(SpireAssignmentPayloadFormat::RaBitQ, &source).unwrap();
        let assignment =
            assignment_row(SpireAssignmentPayloadFormat::RaBitQ, gamma, payload.clone());
        let scorer = SpirePreparedAssignmentScorer::prepare(
            SpireAssignmentPayloadFormat::RaBitQ,
            usize::from(dimensions),
            &query,
        )
        .unwrap();
        let quantizer = RaBitQQuantizer::cached_seeded_srht_bits(
            source.len(),
            crate::DEFAULT_QUANT_SEED,
            crate::DEFAULT_QUANT_BITS,
        )
        .unwrap();
        let prepared = quantizer.prepare_estimator(&query);
        let expected = quantizer.estimate_ip(&prepared, &payload).estimate;

        let observed = scorer.score_assignment_ip(&assignment).unwrap();

        assert_eq!(assignment.payload_format, SPIRE_PAYLOAD_FORMAT_RABITQ);
        assert_eq!(gamma, 0.0);
        assert!((observed - expected).abs() <= f32::EPSILON);
    }

    #[test]
    fn assignment_scorer_batch_matches_scalar_scores() {
        let query = vec![1.0, 0.5, -0.25, 0.125];
        let sources = [vec![0.25, -0.5, 0.75, 1.0], vec![-0.125, 0.25, 0.5, -1.0]];

        for payload_format in [
            SpireAssignmentPayloadFormat::TurboQuant,
            SpireAssignmentPayloadFormat::RaBitQ,
        ] {
            let scorer =
                SpirePreparedAssignmentScorer::prepare(payload_format, query.len(), &query)
                    .unwrap();
            let mut payload_stride = None;
            let mut payloads = Vec::new();
            let mut gammas = Vec::new();
            let mut scalar_scores = Vec::new();

            for source in &sources {
                let (_, gamma, payload) =
                    encode_assignment_payload(payload_format, source).unwrap();
                let assignment = assignment_row(payload_format, gamma, payload.clone());
                scalar_scores.push(scorer.score_assignment_ip(&assignment).unwrap());
                payload_stride = Some(payload_stride.unwrap_or(payload.len()));
                assert_eq!(payload_stride, Some(payload.len()));
                gammas.push(gamma);
                payloads.extend_from_slice(&payload);
            }

            let mut batch_scores = vec![0.0; sources.len()];
            scorer
                .score_batch_ip(
                    payload_stride.unwrap(),
                    &payloads,
                    &gammas,
                    &mut batch_scores,
                )
                .unwrap();

            assert_eq!(batch_scores.len(), scalar_scores.len());
            for (batch_score, scalar_score) in batch_scores.iter().zip(scalar_scores.iter()) {
                assert!((batch_score - scalar_score).abs() <= f32::EPSILON);
            }
        }
    }

    #[test]
    fn assignment_scorer_rejects_mismatched_format_and_bad_lengths() {
        let source = vec![0.25, -0.5, 0.75, 1.0];
        let query = vec![1.0, 0.5, -0.25, 0.125];
        let (dimensions, gamma, mut payload) =
            encode_assignment_payload(SpireAssignmentPayloadFormat::TurboQuant, &source).unwrap();
        let scorer = SpirePreparedAssignmentScorer::prepare(
            SpireAssignmentPayloadFormat::TurboQuant,
            usize::from(dimensions),
            &query,
        )
        .unwrap();
        let mut assignment = assignment_row(
            SpireAssignmentPayloadFormat::TurboQuant,
            gamma,
            payload.clone(),
        );

        assignment.payload_format = SPIRE_PAYLOAD_FORMAT_RABITQ;
        assert!(scorer.score_assignment_ip(&assignment).is_err());

        assignment.payload_format = SPIRE_PAYLOAD_FORMAT_TURBOQUANT;
        payload.pop();
        assignment.encoded_payload = payload;
        assert!(scorer.score_assignment_ip(&assignment).is_err());
    }

    #[test]
    fn assignment_scorer_batch_rejects_bad_shapes() {
        let source = vec![0.25, -0.5, 0.75, 1.0];
        let query = vec![1.0, 0.5, -0.25, 0.125];
        let (_, gamma, payload) =
            encode_assignment_payload(SpireAssignmentPayloadFormat::TurboQuant, &source).unwrap();
        let scorer = SpirePreparedAssignmentScorer::prepare(
            SpireAssignmentPayloadFormat::TurboQuant,
            source.len(),
            &query,
        )
        .unwrap();
        let mut out = [0.0];

        assert!(scorer
            .score_batch_ip(payload.len() + 1, &payload, &[gamma], &mut out)
            .is_err());
        assert!(scorer
            .score_batch_ip(payload.len(), &payload, &[], &mut out)
            .is_err());

        let (_, _, rabitq_payload) =
            encode_assignment_payload(SpireAssignmentPayloadFormat::RaBitQ, &source).unwrap();
        let rabitq_scorer = SpirePreparedAssignmentScorer::prepare(
            SpireAssignmentPayloadFormat::RaBitQ,
            source.len(),
            &query,
        )
        .unwrap();
        assert!(rabitq_scorer
            .score_batch_ip(rabitq_payload.len(), &rabitq_payload, &[1.0], &mut out)
            .is_err());
    }

    #[test]
    fn assignment_scorer_rejects_unscoreable_and_deferred_formats() {
        assert!(SpireAssignmentPayloadFormat::from_tag(SPIRE_PAYLOAD_FORMAT_NONE).is_err());
        assert!(SpirePreparedAssignmentScorer::prepare(
            SpireAssignmentPayloadFormat::PqFastScan,
            4,
            &[1.0, 0.5, -0.25, 0.125],
        )
        .is_err());
        assert!(encode_assignment_payload(
            SpireAssignmentPayloadFormat::PqFastScan,
            &[0.25, -0.5, 0.75, 1.0],
        )
        .is_err());
    }

    #[test]
    fn assignment_scorer_validates_query_and_source_shape() {
        assert!(encode_assignment_payload(SpireAssignmentPayloadFormat::TurboQuant, &[]).is_err());
        assert!(encode_assignment_payload(
            SpireAssignmentPayloadFormat::TurboQuant,
            &[1.0, f32::NAN]
        )
        .is_err());
        assert!(SpirePreparedAssignmentScorer::prepare(
            SpireAssignmentPayloadFormat::TurboQuant,
            2,
            &[1.0],
        )
        .is_err());
        assert!(SpirePreparedAssignmentScorer::prepare(
            SpireAssignmentPayloadFormat::TurboQuant,
            2,
            &[1.0, f32::INFINITY],
        )
        .is_err());
    }

    #[test]
    fn encode_assignment_input_builds_leaf_assignment_input() {
        let source = vec![0.25, -0.5, 0.75, 1.0];
        let (_, gamma, payload) =
            encode_assignment_payload(SpireAssignmentPayloadFormat::TurboQuant, &source).unwrap();

        let input = encode_assignment_input(
            SpireAssignmentPayloadFormat::TurboQuant,
            tid(10, 2),
            &source,
        )
        .unwrap();

        assert_eq!(input.heap_tid, tid(10, 2));
        assert_eq!(input.payload_format, SPIRE_PAYLOAD_FORMAT_TURBOQUANT);
        assert_eq!(input.gamma, gamma);
        assert_eq!(input.encoded_payload, payload);
    }

    #[test]
    fn encode_assignment_input_rejects_invalid_locator() {
        assert!(encode_assignment_input(
            SpireAssignmentPayloadFormat::TurboQuant,
            ItemPointer::INVALID,
            &[0.25, -0.5, 0.75, 1.0],
        )
        .is_err());
    }
}
