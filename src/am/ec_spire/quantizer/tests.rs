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
        let (gamma, payload) =
            encode_assignment_payload(SpireAssignmentPayloadFormat::TurboQuant, &source).unwrap();
        let assignment = assignment_row(
            SpireAssignmentPayloadFormat::TurboQuant,
            gamma,
            payload.clone(),
        );
        let scorer = SpirePreparedAssignmentScorer::prepare(
            SpireAssignmentPayloadFormat::TurboQuant,
            source.len(),
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
        let (gamma, payload) =
            encode_assignment_payload(SpireAssignmentPayloadFormat::RaBitQ, &source).unwrap();
        let assignment =
            assignment_row(SpireAssignmentPayloadFormat::RaBitQ, gamma, payload.clone());
        let scorer = SpirePreparedAssignmentScorer::prepare(
            SpireAssignmentPayloadFormat::RaBitQ,
            source.len(),
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
                let (gamma, payload) = encode_assignment_payload(payload_format, source).unwrap();
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
        let (gamma, mut payload) =
            encode_assignment_payload(SpireAssignmentPayloadFormat::TurboQuant, &source).unwrap();
        let scorer = SpirePreparedAssignmentScorer::prepare(
            SpireAssignmentPayloadFormat::TurboQuant,
            source.len(),
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
        let (gamma, payload) =
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

        let (_, rabitq_payload) =
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
        let (gamma, payload) =
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
